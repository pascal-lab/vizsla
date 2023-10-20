use std::time::Instant;
use crossbeam_channel::Sender;
use lsp_server::{Message, ReqQueue};

type ReqHandler = fn(&mut GlobalState, lsp_server::Response);

struct Handle<H, C> {
    handle: H,
    receiver: C,
}

pub(crate) struct GlobalState {
    sender: Sender<Message>,
    req_queue: ReqQueue<(String, Instant), ReqHandler>,
}
