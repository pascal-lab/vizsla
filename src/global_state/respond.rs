use lspt::{notification, request};
use serde::Serialize;

use super::DEFAULT_REQ_HANDLER;
use crate::global_state::{GlobalState, ReqHandler};

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

fn progress_value<T: Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).expect("work-done progress payload should serialize")
}

impl GlobalState {
    pub(crate) fn send(&self, message: lsp_server::Message) {
        if self.sender.send(message).is_err() {
            tracing::debug!("LSP message dropped because client connection is closed");
        }
    }

    pub(crate) fn send_notification<N: notification::Notification>(&self, params: N::Params) {
        let notif = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(notif.into());
    }

    pub(crate) fn send_request<R: request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self.req_queue.outgoing.register(R::METHOD.to_string(), params, handler);
        self.send(request.into());
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(&response.id) {
            if let Some(err) = &response.error
                && err.message.starts_with("server panicked")
            {
                tracing::error!("{:?}", err);
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
    ) {
        if !self.config.cli_work_done_progress() {
            return;
        }

        let percentage = fraction.map(|f| {
            assert!((0.0..=1.0).contains(&f));
            (f * 100.0) as u32
        });

        let cancellable = Some(cancel_token.is_some());

        let token = lspt::Union2::B(
            cancel_token.unwrap_or_else(|| format!("{}/{title}", self.config.opt.process_name)),
        );

        let work_done_progress = match state {
            Progress::Begin => {
                self.send_request::<request::WorkDoneProgressCreateRequest>(
                    lspt::WorkDoneProgressCreateParams { token: token.clone() },
                    DEFAULT_REQ_HANDLER,
                );

                progress_value(lspt::WorkDoneProgressBegin {
                    kind: "begin".to_owned(),
                    title: title.to_owned(),
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::Report => progress_value(lspt::WorkDoneProgressReport {
                kind: "report".to_owned(),
                cancellable,
                message,
                percentage,
            }),
            Progress::End => {
                progress_value(lspt::WorkDoneProgressEnd { kind: "end".to_owned(), message })
            }
        };

        self.send_notification::<lspt::notification::ProgressNotification>(lspt::ProgressParams {
            token,
            value: work_done_progress,
        });
    }
}
