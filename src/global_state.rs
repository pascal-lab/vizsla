use std::{time::Instant, sync::{Arc, RwLock}};
use crossbeam_channel::{Sender, unbounded, Receiver};
use lsp_server::{Message, ReqQueue, Request};
use lsp_types::notification;
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

    pub(crate) fn send_notification<N: notification::Notification>(&self, params: N::Params) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
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
}
