use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams};
use utils::lines::PositionEncoding;

use crate::{global_state::GlobalState, lsp_ext::from_proto, mem_docs::DocumentData};

pub(crate) fn handle_cancel(
    state: &mut GlobalState,
    params: lsp_types::CancelParams,
) -> anyhow::Result<()> {
    let id: lsp_server::RequestId = match params.id {
        lsp_types::NumberOrString::Number(id) => id.into(),
        lsp_types::NumberOrString::String(id) => id.into(),
    };
    state.cancel(id);
    Ok(())
}

pub(crate) fn handle_did_open_text_document(
    state: &mut GlobalState,
    params: DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        if state
            .mem_docs
            .insert(path.clone(), DocumentData::new(params.text_document.version))
            .is_some()
        {
            tracing::error!("duplicate DidOpenTextDocument: {}", path);
        }
        state.vfs.write().0.set_file_contents(path, Some(params.text_document.text.into_bytes()));
    }
    Ok(())
}
