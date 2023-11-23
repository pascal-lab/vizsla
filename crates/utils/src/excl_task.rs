// Exclusive task, make sure only one long-running operation is being executed

#[derive(Debug, Default)]
pub struct ExclTask<Args, Output, Cause = String> {
    requested: Option<(Cause, Args)>,
    in_process: bool,
    last_result: Output,
}

impl<Args, Output, Cause> ExclTask<Args, Output, Cause> {
    pub fn request(&mut self, reason: Cause, args: Args) {
        self.requested = Some((reason, args));
    }

    pub fn can_start(&mut self) -> Option<(Cause, Args)> {
        if self.in_process {
            return None;
        }
        self.in_process = self.requested.is_some();
        self.requested.take()
    }

    pub fn complete(&mut self, result: Output) {
        assert!(self.in_process);
        self.in_process = false;
        self.last_result = result;
    }

    pub fn last_op_result(&self) -> &Output {
        &self.last_result
    }

    pub fn in_process(&self) -> bool {
        self.in_process
    }

    pub fn has_op_requested(&self) -> bool {
        self.requested.is_some()
    }
}
