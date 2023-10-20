use lsp_server::Connection;

use crate::{
    Config,
    global_state::GlobalState,
};

pub(crate) enum Task {
    Response(lsp_server::Response),
    Retry(lsp_server::Request),
    // Diagnostics(Vec<(FileId, Vec<lsp_types::Diagnostic>)>)
}

pub fn main_loop(config: Config, connection: Connection) {
    tracing::info!("initial config: {:#?}", config);

    // Windows scheduler implements priority boosts: if thread waits for an
    // event (like a condvar), and event fires, priority of the thread is
    // temporary bumped. This optimization backfires in our case: each time the
    // `main_loop` schedules a task to run on a threadpool, the worker threads
    // gets a higher priority, and (on a machine with fewer cores) displaces the
    // main loop! We work around this by marking the main loop as a
    // higher-priority thread.
    #[cfg(windows)]
    unsafe {
        use winapi::um::processthreadsapi::*;
        let thread = GetCurrentThread();
        let thread_priority_above_normal = 1;
        SetThreadPriority(thread, thread_priority_above_normal);
    }

    GlobalState::new(connection.sender, config).run(connection.receiver)
}
