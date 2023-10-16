use lsp_types::{
    CallHierarchyServerCapability, ClientCapabilities, CodeActionKind, CodeActionOptions,
    CodeActionProviderCapability, CodeLensOptions, CompletionOptions,
    CompletionOptionsCompletionItem, DeclarationCapability, DocumentOnTypeFormattingOptions,
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, FoldingRangeProviderCapability, HoverProviderCapability,
    ImplementationProviderCapability, InlayHintOptions, InlayHintServerCapabilities, OneOf,
    PositionEncodingKind, RenameOptions, SaveOptions, SelectionRangeProviderCapability,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities,
    SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TypeDefinitionProviderCapability, WorkDoneProgressOptions,
    WorkspaceFileOperationsServerCapabilities, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities,
};
use serde::de::DeserializeOwned;
use serde_json::{Error};
use utils::paths::AbsPathBuf;
use std::{iter, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    workspace_roots: Vec<AbsPathBuf>,
    client_caps: lsp_types::ClientCapabilities,
    root_path: AbsPathBuf,
    user_config: UserConfig,
    detached_files: Vec<AbsPathBuf>,
}

#[derive(Debug, Clone)]
pub struct Snippet {

}

impl Config {
    pub fn new(
        root_path: AbsPathBuf,
        client_caps: ClientCapabilities,
        workspace_roots: Vec<AbsPathBuf>,
        user_config: UserConfig,
        detached_files: Vec<AbsPathBuf>,
        snippets: Vec<Snippet>,
    ) -> Self {
        Config {
            workspace_roots,
            client_caps,
            root_path,
            user_config,
            detached_files,
        }
    }

    pub fn get_server_capabilities(&self) -> ServerCapabilities {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct UserConfig {}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {}
    }
}

pub fn get_field<T: DeserializeOwned>(
    json: &mut serde_json::Value,
    error_sink: &mut Vec<(String, serde_json::Error)>,
    field: &'static str,
    alias: Option<&'static str>,
    default: &str,
) -> T {
    // XXX: check alias first, to work around the VS Code where it pre-fills the
    // defaults instead of sending an empty object.
    alias
        .into_iter()
        .chain(iter::once(field))
        .filter_map(move |field| {
            let mut pointer = field.replace('_', "/");
            pointer.insert(0, '/');
            json.pointer_mut(&pointer)
                .map(|it| serde_json::from_value(it.take()).map_err(|e| (e, pointer)))
        })
        .find(Result::is_ok)
        .and_then(|res| match res {
            Ok(it) => Some(it),
            Err((e, pointer)) => {
                tracing::warn!("Failed to deserialize config field at {}: {:?}", pointer, e);
                error_sink.push((pointer, e));
                None
            }
        })
        .unwrap_or_else(|| {
            serde_json::from_str(default).unwrap_or_else(|e| panic!("{e} on: `{default}`"))
        })
}

pub fn parse_initialization_options(
    mut options: serde_json::Value
) -> (UserConfig, Vec<AbsPathBuf>, Vec<Snippet>, Vec<(String, Error)>){
    tracing::info!("Server initialized with options: {:#}", options);
    if options.is_null() || options.as_object().map_or(false, |obj| obj.is_empty()) {
        return Default::default();
    }

    let mut errors = Vec::new();

    // TODO: user configuration in package.json
    let user_config: UserConfig = UserConfig {};

    let detached_files = get_field::<Vec<PathBuf>>(&mut options, &mut errors, "detachedFiles", None, "[]")
        .into_iter()
        .map(AbsPathBuf::assert)
        .collect::<Vec<AbsPathBuf>>();

    // TODO: user-defined snippets
    let snippets: Vec<Snippet> = Vec::new();

    (user_config, detached_files, snippets, errors)
}
