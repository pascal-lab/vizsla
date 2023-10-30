use std::{time::Instant, sync::{Arc, RwLock}};
use crossbeam_channel::{Sender, unbounded, Receiver};
use lsp_server::{Message, ReqQueue, Request};
use lsp_types::{request, notification};
use nohash_hasher::IntMap;
use utils::thread::{Pool, ThreadIntent};

use crate::{config::Config, main_loop::Task, mem_docs::MemDocs, line_idx::LineEndings};
use vfs::{self, FileId};
use ide::{self, analysis_host::{AnalysisHost, Analysis}};

type ReqHandler = fn(&mut GlobalState, lsp_server::Response);

pub(crate) struct TaskPool<T> {
    pub(crate) sender: Sender<T>,
    pub(crate) pool: Pool,
}

impl<T> TaskPool<T> {
    pub(crate) fn new_with_threads_num(sender: Sender<T>, threads_num: usize) -> TaskPool<T> {
        TaskPool { sender, pool: Pool::new(threads_num) }
    }

    pub(crate) fn spawn_and_send<F>(&mut self, intent: ThreadIntent, task: F)
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.pool.spawn(intent, {
            let sender = self.sender.clone();
            move || sender.send(task()).unwrap()
        })
    }

    pub(crate) fn spawn_and_send_cps<F>(&mut self, intent: ThreadIntent, task: F)
    where
        F: FnOnce(Sender<T>) + Send + 'static,
        T: Send + 'static,
    {
        self.pool.spawn(intent, {
            let sender = self.sender.clone();
            move || task(sender)
        })
    }
}

pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

pub(crate) struct GlobalState {
    pub(crate) sender: Sender<Message>,
    pub(crate) req_queue: ReqQueue<(String, Instant), ReqHandler>,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,

    pub(crate) config: Arc<Config>,

    pub(crate) analysis_host: AnalysisHost,
    pub(crate) mem_docs: MemDocs,

    pub(crate) shutdown_requested: bool,

    pub(crate) vfs_loader: Handle<Box<dyn vfs::loader::Handle>, Receiver<vfs::loader::Message>>,
    pub(crate) vfs: Arc<RwLock<(vfs::Vfs, IntMap<FileId, LineEndings>)>>,
    pub(crate) vfs_config_version: u32,
    pub(crate) vfs_progress_config_version: u32,
    pub(crate) vfs_progress_n_total: usize,
    pub(crate) vfs_progress_n_done: usize,
}

// immutable
// TODO:
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) analysis: Analysis,
    // pub(crate) check_fixes: CheckFixes,
    mem_docs: MemDocs,
    vfs: Arc<RwLock<(vfs::Vfs, IntMap<FileId, LineEndings>)>>,
    // pub(crate) workspaces: Arc<Vec<ProjectWorkspace>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalState {
    pub(crate) fn new(sender: Sender<lsp_server::Message>, config: Config) -> GlobalState {
        let vfs_loader = {
            let (sender, receiver) = unbounded::<vfs::loader::Message>();
            let handle: vfs_notify::NotifyHandle = vfs::loader::Handle::spawn(Box::new(move |msg| sender.send(msg).unwrap()));
            let handle = Box::new(handle) as Box<dyn vfs::loader::Handle>;
            Handle { handle, receiver }
        };

        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads_num(sender, config.main_loop_threads_num());
            Handle { handle, receiver }
        };

        let mut analysis_host = AnalysisHost::new();

        GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            task_pool,
            config: Arc::new(config.clone()),
            analysis_host,
            mem_docs: MemDocs::default(),
            shutdown_requested: false,

            vfs_loader: vfs_loader,
            vfs: Arc::new(RwLock::new((vfs::Vfs::default(), IntMap::default()))),
            vfs_config_version: 0,
            vfs_progress_config_version: 0,
            vfs_progress_n_total: 0,
            vfs_progress_n_done: 0,
        }
    }

    pub(crate) fn register_request(&mut self, req_received: Instant, req: &Request) {
        self.req_queue.incoming.register(req.id.clone(), (req.method.clone(), req_received));
    }

    pub(crate) fn make_snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: Arc::clone(&self.config),
            // workspaces: Arc::clone(&self.workspaces),
            analysis: self.analysis_host.make_analysis(),
            vfs: Arc::clone(&self.vfs),
            mem_docs: self.mem_docs.clone(),
        }
    }

    pub(crate) fn is_completed(&self, req: &Request) -> bool {
        self.req_queue.incoming.is_completed(&req.id)
    }
}


// Send and Respond stuff
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Progress {
    Begin,
    Report,
    End,
}

impl Progress {
    pub(crate) fn fraction(done: usize, total: usize) -> f64 {
        assert!(done <= total);
        done as f64 / total.max(1) as f64
    }
}

impl GlobalState {
    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
    }

    pub(crate) fn send_notification<N: notification::Notification>(&self, params: N::Params) {
        let notif = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(notif.into());
    }

    pub(crate) fn send_request<R: request::Request>(&mut self, params: R::Params, handler: ReqHandler) {
        let request = self.req_queue.outgoing.register(R::METHOD.to_string(), params, handler);
        self.send(request.into());
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            if let Some(err) = &response.error {
                // TODO: less msg to be more `resilient'?
                if err.message.starts_with("server panicked") {
                    tracing::error!("{:?}", err);
                }
            }

            let duration = start.elapsed();
            tracing::debug!("handled {} {}) in {:0.2?}", method, response.id, duration);
            self.send(response.into());
        }
    }

    pub(crate) fn report_progress(
        &mut self,
        title: &str,
        state: Progress,
        message: Option<String>,
        fraction: Option<f64>,
        cancel_token: Option<String>,
    ){
        // TODO: check if work_down_progress enabled in config
        // if !self.config.work_done_progress() {
        //     return;
        // }

        let percentage = fraction.map(|f| {
            assert!((0.0..=1.0).contains(&f));
            (f * 100.0) as u32
        });

        let cancellable = Some(cancel_token.is_some());

        let token = lsp_types::ProgressToken::String(
            cancel_token.unwrap_or_else(|| format!("{}/{title}", &self.config.opt.process_name)),
        );

        let work_done_progress = match state {
            Progress::Begin => {
                self.send_request::<request::WorkDoneProgressCreate>(
                    lsp_types::WorkDoneProgressCreateParams { token: token.clone() },
                    |_, _| (),
                );

                lsp_types::WorkDoneProgress::Begin(lsp_types::WorkDoneProgressBegin {
                    title: title.into(),
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::Report => {
                lsp_types::WorkDoneProgress::Report(lsp_types::WorkDoneProgressReport {
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::End => {
                lsp_types::WorkDoneProgress::End(lsp_types::WorkDoneProgressEnd { message })
            }
        };

        self.send_notification::<lsp_types::notification::Progress>(lsp_types::ProgressParams {
            token,
            value: lsp_types::ProgressParamsValue::WorkDone(work_done_progress),
        });
    }
}
