use lsp_types::notification::Notification;

pub enum ReloadWorkspace {}

impl Notification for ReloadWorkspace {
    type Params = ();
    const METHOD: &'static str = "rust-analyzer/workspace/reloadWorkspace";
}
