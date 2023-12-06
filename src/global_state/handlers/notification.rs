use std::ops::Range;

use line_index::LineIndex;
use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams};
use triomphe::Arc;
use utils::lines::{LineEndings, LineIndexEnding, PositionEncoding};

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

fn apply_document_changes(
    encoding: PositionEncoding,
    get_vfs_file_contents: impl FnOnce() -> String,
    mut content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) -> String {
    // Skip to the last full document change
    let mut start = content_changes
        .iter()
        .rev()
        .position(|change| change.range.is_none())
        .map(|idx| content_changes.len() - idx - 1)
        .unwrap_or(0);

    // TODO: more tricks to optimize ranges?
    let mut text: String = match content_changes.get_mut(start) {
        // peek at the first content change as an optimization
        Some(lsp_types::TextDocumentContentChangeEvent { range: None, text, .. }) => {
            let text = std::mem::take(text);
            start += 1;
            if start == content_changes.len() {
                return text;
            }
            text
        }
        Some(_) => get_vfs_file_contents(),
        // we received no content changes
        None => return get_vfs_file_contents(),
    };

    let mut line_index = LineIndexEnding {
        index: Arc::new(LineIndex::new(&text)),
        // We don't care about line endings here.
        endings: LineEndings::Unix,
        encoding,
    };

    // The changes can cross lines so we have to keep our line index updated.
    // Here's an optimization: we only rebuild the index if we have to, iff
    // the change's start line is greater than the last valid line.
    // The VFS will normalize the end of lines to `\n`.
    let mut index_valid_until = !0u32; // set to infinity at first
    for change in content_changes {
        // The None case can't happen
        let range = change.range.unwrap();
        if index_valid_until <= range.end.line {
            *Arc::make_mut(&mut line_index.index) = LineIndex::new(&text);
        }
        index_valid_until = range.start.line;
        // TODO: Use rope for better performance?
        if let Ok(range) = from_proto::text_range(&line_index, range) {
            text.replace_range(Range::<usize>::from(range), &change.text);
        }
    }
    text
}

pub(crate) fn handle_did_change_text_document(
    state: &mut GlobalState,
    params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        match state.mem_docs.get_mut(&path) {
            Some(doc) => {
                // The version in DidChangeTextDocument is the one after all edits
                // so we should apply it before the vfs is notified.
                doc.version = params.text_document.version;
            }
            None => {
                tracing::error!("unexpected DidChangeTextDocument: {}", path);
                return Ok(());
            }
        };

        let text = apply_document_changes(
            state.config.position_encoding(),
            || {
                let vfs = &state.vfs.read().0;
                let file_id = vfs.file_id(&path).unwrap();
                let contents = vfs.file_contents(file_id).unwrap();
                std::str::from_utf8(contents).unwrap().into()
            },
            params.content_changes,
        );
        state.vfs.write().0.set_file_contents(path, Some(text.into_bytes()));
    }
    Ok(())
}
