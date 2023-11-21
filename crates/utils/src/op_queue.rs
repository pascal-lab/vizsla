pub type Cause = String;

#[derive(Debug, Default)]
pub struct OpQueue<Args = (), Output = ()> {
    requested: Option<(Cause, Args)>,
    in_process: bool,
    last_op_result: Output,
}

impl<Args, Output> OpQueue<Args, Output> {
    pub fn request(&mut self, reason: Cause, args: Args) {
        self.requested = Some((reason, args));
    }

    pub fn should_start(&mut self) -> Option<(Cause, Args)> {
        if self.in_process {
            return None;
        }
        self.in_process = self.requested.is_some();
        self.requested.take()
    }

    pub fn complete(&mut self, result: Output) {
        assert!(self.in_process);
        self.in_process = false;
        self.last_op_result = result;
    }

    pub fn last_op_result(&self) -> &Output {
        &self.last_op_result
    }

    pub fn in_process(&self) -> bool {
        self.in_process
    }

    pub fn has_op_requested(&self) -> bool {
        self.requested.is_some()
    }
}
