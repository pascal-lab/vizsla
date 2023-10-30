use std::time::Instant;

use crossbeam_channel::{select, Receiver};
use lsp_server::{Connection, Notification, Request};
use lsp_types::{notification::Notification as _};

use crate::{
    Config,
    global_state::GlobalState, dispatcher::{ReqDispatcher, NotifDispatcher},
};

#[derive(Debug)]
enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
    Vfs(vfs::loader::Message),
}

#[derive(Debug)]
pub(crate) enum Task {
    Response(lsp_server::Response),
    Retry(lsp_server::Request),
    // Diagnostics(Vec<(FileId, Vec<lsp_types::Diagnostic>)>)
}

pub fn main_loop(config: Config, connection: Connection) -> anyhow::Result<()> {
    tracing::info!("initial config: {:#?}", config);

    // TODO: hack for windwos`

    GlobalState::new(connection.sender, config).run(connection.receiver)
}

impl GlobalState {
    pub(crate) fn run(&mut self, cli_inbox: Receiver<lsp_server::Message>) -> anyhow::Result<()> {
        // TODO: check for status
        // TODO: fetch workspace

        while let Some(event) = self.next_event(&cli_inbox) {
            match &event {
                Event::Lsp(lsp_server::Message::Notification(Notification { method, .. }))
                    if method == lsp_types::notification::Exit::METHOD => {
                        return Ok(());
                    }
                _ => self.handle_event(event)?,
            }
        }
        anyhow::bail!("Server {} exited without proper shutdown sequence", &self.config.opt.process_name);
    }

    fn next_event(&self, cli_inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        select! {
            recv(cli_inbox) -> cli_msg => cli_msg.ok().map(Event::Lsp),
            recv(self.task_pool.receiver) -> task => Some(Event::Task(task.unwrap())),
            recv(self.vfs_loader.receiver) -> vfs_task => Some(Event::Vfs(vfs_task.unwrap())),
        }
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        let loop_start = Instant::now();

        let loop_start_dbg_msg = format!("{loop_start:?}");
        let event_dbg_msg = format!("{event:?}");
        tracing::debug!("{} [handle_event]: {}", loop_start_dbg_msg, event_dbg_msg);

        // check if is quiescent when loading workspace
        match event {
            Event::Lsp(msg) => match msg {
                lsp_server::Message::Request(req) => self.handle_lsp_request(loop_start, req),
                lsp_server::Message::Notification(notif) => self.handle_lsp_notification(notif)?,
                lsp_server::Message::Response(_) => todo!(),
            }
            Event::Task(_) => todo!(),
            Event::Vfs(_) => todo!(),
        }

        todo!()
    }

    fn handle_lsp_request(&mut self, req_received: Instant, req: Request) {
        self.register_request(req_received, &req);
        self.dispatch_request(req);
    }

    fn dispatch_request(&mut self, req: Request) {
        let mut dispatcher = ReqDispatcher { req: Some(req), global_state: self };

        // Handle shutdown req first
        dispatcher.on_sync_mut::<lsp_types::request::Shutdown>(|this, ()| {
            this.shutdown_requested = true;
            Ok(())
        });

        match &mut dispatcher {
            ReqDispatcher { req: Some(req), global_state: this } if this.shutdown_requested => {
                this.respond(lsp_server::Response::new_err(
                    req.id.clone(),
                    lsp_server::ErrorCode::InvalidRequest as i32,
                    "Shutdown already requested.".to_owned(),
                ));
                return;
            }
            _ => (),
        }

        // TODO: Add handlers
        dispatcher.finish();
    }

    fn handle_lsp_notification(&mut self, notif: Notification) -> anyhow::Result<()> {
        NotifDispatcher { notif: Some(notif), global_state: self }
        .finish();

        Ok(())
    }
}
