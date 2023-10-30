use lsp_types::{
    ClientCapabilities, CodeActionKind, CodeActionOptions, CodeLensOptions, CompletionOptions,
    CompletionOptionsCompletionItem, DocumentOnTypeFormattingOptions, FileOperationFilter,
    FileOperationPattern, FileOperationPatternKind, FileOperationRegistrationOptions,
    InlayHintOptions, PositionEncodingKind, RenameOptions, SaveOptions, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities, SignatureHelpOptions,
    TextDocumentSyncKind, TextDocumentSyncOptions, WorkDoneProgressOptions,
    WorkspaceFileOperationsServerCapabilities, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities, OneOf, DeclarationCapability, InlayHintServerCapabilities,
};
use serde::de::DeserializeOwned;
use serde_json::Error;
use utils::paths::AbsPathBuf;
use std::{iter, path::PathBuf};
use crate::{
    line_idx::{PositionEncoding, WideEncoding},
    Opt,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) opt: Opt,
    pub(crate) workspace_roots: Vec<AbsPathBuf>,
    pub(crate) client_caps: lsp_types::ClientCapabilities,
    pub(crate) root_path: AbsPathBuf,
    pub(crate) user_config: UserConfig,
    pub(crate) detached_files: Vec<AbsPathBuf>,
}

#[derive(Debug, Clone)]
pub struct Snippet {

}

#[derive(Debug, Clone)]
pub struct UserConfig {}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {}
    }
}

macro_rules! try_ {
    ($expr:expr) => {
        || -> _ { Some($expr) }()
    };
}

impl Config {
    pub fn new(
        opt: Opt,
        root_path: AbsPathBuf,
        client_caps: ClientCapabilities,
        workspace_roots: Vec<AbsPathBuf>,
        user_config: UserConfig,
        detached_files: Vec<AbsPathBuf>,
        snippets: Vec<Snippet>,
    ) -> Self {
        Config {
            opt,
            workspace_roots,
            client_caps,
            root_path,
            user_config,
            detached_files,
        }
    }

    pub fn cli_completion_label_details_support(&self) -> bool {
        try_!(self.client_caps
              .text_document.as_ref()?
              .completion.as_ref()?
              .completion_item.as_ref()?
              .label_details_support.as_ref()?
        ).is_some()
    }

    pub fn cli_completion_item_edit_resolve(&self) -> bool {
        try_!(self.client_caps.text_document.as_ref()?
              .completion.as_ref()?
              .completion_item.as_ref()?
              .resolve_support.as_ref()?
              .properties.iter()
              .any(|cap_string| cap_string.as_str() == "additionalTextEdits")
        ) == Some(true)
    }

    pub fn negotiated_encoding(&self) -> PositionEncoding {
        let client_encodings = match &self.client_caps.general {
            Some(general) => general.position_encodings.as_deref().unwrap_or_default(),
            None => &[],
        };

        for enc in client_encodings {
            if enc == &PositionEncodingKind::UTF8 {
                return PositionEncoding::Utf8;
            } else if enc == &PositionEncodingKind::UTF32 {
                return PositionEncoding::Wide(WideEncoding::Utf32);
            }
            // NB: intentionally prefer just about anything else to utf-16.
        }

        PositionEncoding::Wide(WideEncoding::Utf16)
    }

    pub fn main_loop_threads_num(&self) -> usize {
        num_cpus::get_physical().try_into().unwrap_or(1)
    }

    pub fn get_server_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            position_encoding: match self.negotiated_encoding() {
                PositionEncoding::Utf8 => Some(PositionEncodingKind::UTF8),
                PositionEncoding::Wide(wide) => match wide {
                    WideEncoding::Utf16 => Some(PositionEncodingKind::UTF16),
                    WideEncoding::Utf32 => Some(PositionEncodingKind::UTF32),
                    _ => None,
                },
            },
            text_document_sync: Some(TextDocumentSyncOptions {
                open_close: true.into(),
                change: TextDocumentSyncKind::INCREMENTAL.into(),
                will_save: None,
                will_save_wait_until: None,
                save: Some(SaveOptions::default().into()),
            }.into()),
            selection_range_provider: Some(true.into()),
            hover_provider: Some(true.into()),
            completion_provider: CompletionOptions {
                resolve_provider: self.cli_completion_item_edit_resolve().into(),
                trigger_characters: Some([":", ",", "'", "(", "`"].map(String::from).into()),
                all_commit_characters: None,
                completion_item: CompletionOptionsCompletionItem{
                    label_details_support: self.cli_completion_label_details_support().into(),
                }.into(),
                work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
            }.into(),
            signature_help_provider: SignatureHelpOptions {
                trigger_characters: Some(["(", ",", "`"].map(String::from).into()),
                retrigger_characters: None,
                work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
            }.into(),
            declaration_provider: Some(DeclarationCapability::Simple(true)),
            definition_provider: OneOf::Left(true).into(),
            type_definition_provider: Some(true.into()),
            implementation_provider: Some(true.into()),
            references_provider: OneOf::Left(true).into(),
            document_highlight_provider: OneOf::Left(true).into(),
            document_symbol_provider: OneOf::Left(true).into(),
            workspace_symbol_provider: OneOf::Left(true).into(),
            code_action_provider: Some({
                try_!(self.client_caps
                      .text_document.as_ref()?
                      .code_action.as_ref()?
                      .code_action_literal_support.as_ref()?
                ).map_or(true.into(), |_| {
                    CodeActionOptions {
                        code_action_kinds: vec![
                            CodeActionKind::EMPTY,
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR,
                            CodeActionKind::REFACTOR_EXTRACT,
                            CodeActionKind::REFACTOR_INLINE,
                            CodeActionKind::REFACTOR_REWRITE,
                        ].into(),
                        resolve_provider: true.into(),
                        work_done_progress_options: Default::default(),
                    }.into()
                })
            }),
            code_lens_provider: CodeLensOptions { resolve_provider: true.into() }.into(),
            document_formatting_provider: OneOf::Left(true).into(),
            document_range_formatting_provider: OneOf::Left(true).into(),
            document_on_type_formatting_provider: DocumentOnTypeFormattingOptions {
                first_trigger_character: "=".to_string(),
                more_trigger_character: Some([".", ">", "{", "(", "<"].map(String::from).into()),
            }.into(),
            rename_provider: OneOf::Right(RenameOptions {
                prepare_provider: true.into(),
                work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
            }).into(),
            document_link_provider: None,
            color_provider: None,
            folding_range_provider: Some(true.into()),
            execute_command_provider: None,
            workspace: WorkspaceServerCapabilities {
                workspace_folders: WorkspaceFoldersServerCapabilities {
                    supported: true.into(),
                    change_notifications: OneOf::Left(true).into(),
                }.into(),
                file_operations: WorkspaceFileOperationsServerCapabilities {
                    did_create: None,
                    will_create: None,
                    did_rename: None,
                    will_rename: FileOperationRegistrationOptions {
                        filters: vec![
                            FileOperationFilter {
                                scheme: String::from("file").into(),
                                pattern: FileOperationPattern {
                                    glob: String::from("**/*.rs"),
                                    matches: FileOperationPatternKind::File.into(),
                                    options: None,
                                },
                            },
                            FileOperationFilter {
                                scheme: String::from("file").into(),
                                pattern: FileOperationPattern {
                                    glob: String::from("**"),
                                    matches: FileOperationPatternKind::Folder.into(),
                                    options: None,
                                },
                            },
                        ],
                    }.into(),
                    did_delete: None,
                    will_delete: None,
                }.into(),
            }.into(),
            // TODO:
            call_hierarchy_provider: Some(true.into()),
            semantic_tokens_provider: Some(SemanticTokensOptions {
                // TODO:
                legend: SemanticTokensLegend::default(),
                // {
                //     token_types: SemanticTokensLegend::
                //     token_modifiers: semantic_tokens::SUPPORTED_MODIFIERS.to_vec(),
                // },
                full: SemanticTokensFullOptions::Delta { delta: true.into() }.into(),
                range: true.into(),
                work_done_progress_options: Default::default(),
            }.into()),
            moniker_provider: None,
            linked_editing_range_provider: None,
            inline_value_provider: None,
            inlay_hint_provider: OneOf::Right(InlayHintServerCapabilities::Options(
                InlayHintOptions {
                work_done_progress_options: Default::default(),
                resolve_provider: true.into(),
            })).into(),
            diagnostic_provider: None,
            experimental: None
        }
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
