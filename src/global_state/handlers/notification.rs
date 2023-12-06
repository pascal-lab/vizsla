use std::ops::Range;

use line_index::LineIndex;
use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWorkspaceFoldersParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
};
use triomphe::Arc;
use utils::{
    lines::{LineEndings, LineIndexEnding, PositionEncoding},
    paths::AbsPathBuf,
    try_,
};
use vfs::vfs::ChangeKind;

use crate::{
    global_state::{reload, GlobalState},
    lsp_ext::from_proto,
    mem_docs::DocumentData,
    DEFAULT_PROCESS_NAME,
};

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

pub(crate) fn handle_did_close_text_document(
    state: &mut GlobalState,
    params: DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        if state.mem_docs.remove(&path).is_none() {
            tracing::error!("orphan DidCloseTextDocument: {}", path);
        }

        if let Some(path) = path.as_abs_path() {
            state.vfs_loader.handle.invalidate(path.to_path_buf());
        }
    }
    Ok(())
}

pub(crate) fn handle_did_save_text_document(
    state: &mut GlobalState,
    params: DidSaveTextDocumentParams,
) -> anyhow::Result<()> {
    // TODO: check on save
    if let Ok(vfs_path) = from_proto::vfs_path(&params.text_document.uri) {
        // Re-fetch workspaces if a workspace related file has changed
        if let Some(abs_path) = vfs_path.as_abs_path() {
            if reload::should_refresh_for_change(abs_path, ChangeKind::Modify) {
                state.fetch_workspaces_task.request(format!("DidSaveTextDocument {abs_path}"), ());
            }
        }
    }

    Ok(())
}

pub(crate) fn handle_did_change_configuration(
    state: &mut GlobalState,
    // As stated in https://github.com/microsoft/language-server-protocol/issues/676,
    // this notification's parameters should be ignored and the actual config queried separately.
    _params: DidChangeConfigurationParams,
) -> anyhow::Result<()> {
    state.send_request::<lsp_types::request::WorkspaceConfiguration>(
        lsp_types::ConfigurationParams {
            items: vec![lsp_types::ConfigurationItem {
                scope_uri: None,
                section: Some(DEFAULT_PROCESS_NAME.into()),
            }],
        },
        |this, resp| {
            tracing::debug!("config update response: '{:?}", resp);
            let lsp_server::Response { result, error, .. } = resp;

            match (result, error) {
                (_, Some(err)) => {
                    tracing::error!("failed to fetch the server settings: {:?}", err)
                }
                (Some(mut configs), None) => {
                    if let Some(json) = configs.get_mut(0) {
                        // Note that json can be null according to the spec if the client can't
                        // provide a configuration. This is handled in Config::update below.
                        let mut config = (*this.config).clone();
                        this.config_errors = config.update(json.take()).err();
                        this.update_configuration(config);
                    }
                }
                (None, None) => {
                    tracing::error!("received empty server settings response from the client")
                }
            }
        },
    );

    Ok(())
}

pub(crate) fn handle_did_change_workspace_folders(
    state: &mut GlobalState,
    params: DidChangeWorkspaceFoldersParams,
) -> anyhow::Result<()> {
    let config = Arc::make_mut(&mut state.config);

    for workspace in params.event.removed {
        if let Ok(path) = AbsPathBuf::try_from(workspace.uri) {
            config.remove_workspace(&path);
        }
    }

    let added = params.event.added.into_iter().filter_map(|it| AbsPathBuf::try_from(it.uri).ok());
    config.add_workspaces(added);

    if config.detached_files.is_empty() {
        config.rediscover_manifest();
        state.fetch_workspaces_task.request("client workspaces changed".to_string(), ())
    }

    Ok(())
}
