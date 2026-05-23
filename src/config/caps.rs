use ide::hover::HoverFormat;
use lspt::{
    CodeActionKind, CodeActionOptions, CodeLensOptions, DiagnosticOptions,
    DocumentOnTypeFormattingOptions, FileOperationFilter, FileOperationOptions,
    FileOperationPattern, FileOperationPatternKind, FileOperationRegistrationOptions,
    InlayHintOptions, PositionEncodingKind, RenameOptions, SaveOptions, SemanticTokensFullDelta,
    SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities, SignatureHelpOptions,
    TextDocumentSyncKind, TextDocumentSyncOptions, Union2, Union3,
    WorkspaceFoldersServerCapabilities, WorkspaceOptions,
};
use utils::{line_index::WideEncoding, lines::PositionEncoding, try_, try_or_default};

use crate::{
    config::Config,
    lsp_ext::ext::{self, RELOAD_WORKSPACE_COMMAND, RUN_QIHE_ANALYSIS_COMMAND},
};

impl Config {
    #[allow(dead_code)]
    pub fn cli_completion_label_details_support(&self) -> bool {
        try_!(
            self.client_caps
                .text_document
                .as_ref()?
                .completion
                .as_ref()?
                .completion_item
                .as_ref()?
                .label_details_support
                .as_ref()?
        )
        .is_some()
    }

    #[allow(dead_code)]
    pub fn cli_completion_item_edit_resolve(&self) -> bool {
        try_!(
            self.client_caps
                .text_document
                .as_ref()?
                .completion
                .as_ref()?
                .completion_item
                .as_ref()?
                .resolve_support
                .as_ref()?
                .properties
                .iter()
                .any(|cap_string| cap_string.as_str() == "additionalTextEdits")
        ) == Some(true)
    }

    pub fn cli_completion_snippet_support(&self) -> bool {
        try_or_default!(
            self.client_caps
                .text_document
                .as_ref()?
                .completion
                .as_ref()?
                .completion_item
                .as_ref()?
                .snippet_support?
        )
    }

    pub fn hierarchical_symbols(&self) -> bool {
        try_!(
            self.client_caps
                .text_document
                .as_ref()?
                .document_symbol
                .as_ref()?
                .hierarchical_document_symbol_support?
        )
        .unwrap_or_default()
    }

    pub fn location_link(&self) -> bool {
        try_or_default!(self.client_caps.text_document.as_ref()?.definition.as_ref()?.link_support?)
    }

    pub fn cli_did_save_dyn_reg(&self) -> bool {
        let caps =
            try_or_default!(self.client_caps.text_document.as_ref()?.synchronization.clone()?);
        caps.did_save == Some(true) && caps.dynamic_registration == Some(true)
    }

    pub fn cli_did_change_watched_files_dyn_reg(&self) -> bool {
        try_or_default!(
            self.client_caps
                .workspace
                .as_ref()?
                .did_change_watched_files
                .as_ref()?
                .dynamic_registration?
        )
    }

    pub fn cli_work_done_progress(&self) -> bool {
        try_or_default!(self.client_caps.window.as_ref()?.work_done_progress?)
    }

    pub fn cli_line_folding_only(&self) -> bool {
        try_or_default! {
            self.client_caps
            .text_document.as_ref()?
            .folding_range.as_ref()?
            .line_folding_only?
        }
    }

    pub fn cli_hover_markdown_support(&self) -> HoverFormat {
        let support_markdown = try_or_default! {
            self.client_caps
            .text_document.as_ref()?
            .hover.as_ref()?
            .content_format.as_ref()?
            .contains(&lspt::MarkupKind::Markdown)
        };

        if support_markdown { HoverFormat::Markdown } else { HoverFormat::PlainText }
    }

    pub fn cli_inlay_hint_refresh_support(&self) -> bool {
        try_or_default! {
            self.client_caps
            .workspace.as_ref()?
            .inlay_hint.as_ref()?
            .refresh_support?
        }
    }

    pub fn cli_code_lens_refresh_support(&self) -> bool {
        try_or_default! {
            self.client_caps
            .workspace.as_ref()?
            .code_lens.as_ref()?
            .refresh_support?
        }
    }

    pub fn cli_workspace_diagnostic_refresh_support(&self) -> bool {
        try_or_default! {
            self.client_caps
            .workspace.as_ref()?
            .diagnostics.as_ref()?
            .refresh_support?
        }
    }

    pub fn cli_pull_diagnostics_support(&self) -> bool {
        try_!(self.client_caps.text_document.as_ref()?.diagnostic.as_ref()).is_some()
    }

    pub fn cli_signature_help_label_offsets_support(&self) -> bool {
        try_or_default! {
            self.client_caps
                .text_document
                .as_ref()?
                .signature_help
                .as_ref()?
                .signature_information
                .as_ref()?
                .parameter_information
                .as_ref()?
                .label_offset_support?
        }
    }

    pub fn cli_code_action_literals(&self) -> bool {
        try_or_default! {
            self.client_caps
            .text_document
            .as_ref()?
            .code_action
            .as_ref()?
            .code_action_literal_support
            .as_ref()
        }
        .is_some()
    }

    pub fn cli_code_action_resolve(&self) -> bool {
        try_or_default! {
            self.client_caps
            .text_document
            .as_ref()?
            .code_action
            .as_ref()?
            .resolve_support
            .as_ref()?
            .properties
            .iter()
            .any(|it| it.as_str() == "edit")
        }
    }

    pub(crate) fn negotiated_encoding(&self) -> PositionEncoding {
        let client_encodings = match &self.client_caps.general {
            Some(general) => general.position_encodings.as_deref().unwrap_or_default(),
            None => &[],
        };

        for enc in client_encodings {
            if enc == &PositionEncodingKind::Utf8 {
                return PositionEncoding::Utf8;
            } else if enc == &PositionEncodingKind::Utf32 {
                return PositionEncoding::Wide(WideEncoding::Utf32);
            }
            // NB: intentionally prefer just about anything else to utf-16.
        }

        PositionEncoding::Wide(WideEncoding::Utf16)
    }
}

impl Config {
    pub fn server_caps(&self) -> ServerCapabilities {
        ServerCapabilities {
            position_encoding: match self.negotiated_encoding() {
                PositionEncoding::Utf8 => Some(PositionEncodingKind::Utf8),
                PositionEncoding::Wide(wide) => match wide {
                    WideEncoding::Utf16 => Some(PositionEncodingKind::Utf16),
                    WideEncoding::Utf32 => Some(PositionEncodingKind::Utf32),
                    _ => None,
                },
            },
            text_document_sync: Some(Union2::A(TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::Incremental),
                will_save: None,
                will_save_wait_until: None,
                save: Some(Union2::B(SaveOptions::default())),
            })),
            notebook_document_sync: None,
            selection_range_provider: Some(Union3::A(true)),
            hover_provider: Some(Union2::A(true)),
            completion_provider: Some(lspt::CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![
                    ".".into(),
                    "(".into(),
                    ",".into(),
                    "@".into(),
                    "#".into(),
                    "$".into(),
                    "`".into(),
                    "'".into(),
                    "\n".into(),
                ]),
                ..Default::default()
            }),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(["(", ",", "."].map(String::from).into()),
                retrigger_characters: None,
                work_done_progress: Default::default(),
            }),
            declaration_provider: Some(Union3::A(true)),
            definition_provider: Some(Union2::A(true)),
            type_definition_provider: Some(Union3::A(true)),
            implementation_provider: Some(Union3::A(false)),
            references_provider: Some(Union2::A(true)),
            document_highlight_provider: Some(Union2::A(true)),
            document_symbol_provider: Some(Union2::A(true)),
            workspace_symbol_provider: Some(Union2::A(true)),
            code_action_provider: Some(Union2::B(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::Empty,
                    CodeActionKind::QuickFix,
                    CodeActionKind::Refactor,
                    CodeActionKind::RefactorExtract,
                    CodeActionKind::RefactorInline,
                    CodeActionKind::RefactorRewrite,
                ]),
                work_done_progress: Default::default(),
                resolve_provider: Some(true),
            })),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
                work_done_progress: None,
            }),
            document_formatting_provider: Some(Union2::A(true)),
            document_range_formatting_provider: Some(Union2::A(true)),
            document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                first_trigger_character: "\n".to_owned(),
                more_trigger_character: None,
            }),
            rename_provider: Some(Union2::B(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress: Default::default(),
            })),
            document_link_provider: None,
            color_provider: None,
            folding_range_provider: Some(Union3::A(true)),
            execute_command_provider: Some(lspt::ExecuteCommandOptions {
                commands: vec![
                    RUN_QIHE_ANALYSIS_COMMAND.to_string(),
                    RELOAD_WORKSPACE_COMMAND.to_string(),
                ],
                work_done_progress: Default::default(),
            }),
            workspace: Some(WorkspaceOptions {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(Union2::B(true)),
                }),
                file_operations: Some(FileOperationOptions {
                    did_create: None,
                    will_create: None,
                    did_rename: None,
                    will_rename: Some(FileOperationRegistrationOptions {
                        filters: vec![
                            FileOperationFilter {
                                scheme: Some(String::from("file")),
                                pattern: FileOperationPattern {
                                    glob: String::from("**/*.{v,sv}"),
                                    matches: Some(FileOperationPatternKind::File),
                                    options: None,
                                },
                            },
                            FileOperationFilter {
                                scheme: Some(String::from("file")),
                                pattern: FileOperationPattern {
                                    glob: String::from("**"),
                                    matches: Some(FileOperationPatternKind::Folder),
                                    options: None,
                                },
                            },
                        ],
                    }),
                    did_delete: None,
                    will_delete: None,
                }),
            }),
            call_hierarchy_provider: Some(Union3::A(true)),
            semantic_tokens_provider: Some(Union2::A(SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: ext::SEMA_TOKENS_TYPES.iter().map(|it| it.to_string()).collect(),
                    token_modifiers: ext::SEMA_TOKENS_MODIFIERS
                        .iter()
                        .map(|it| it.to_string())
                        .collect(),
                },

                full: Some(Union2::B(SemanticTokensFullDelta { delta: Some(true) })),
                range: Some(Union2::A(true)),
                work_done_progress: Default::default(),
            })),
            moniker_provider: None,
            linked_editing_range_provider: None,
            type_hierarchy_provider: None,
            inline_value_provider: None,
            inlay_hint_provider: Some(Union3::B(InlayHintOptions {
                work_done_progress: Default::default(),
                resolve_provider: Some(false),
            })),
            diagnostic_provider: Some(Union2::A(DiagnosticOptions {
                identifier: None,
                inter_file_dependencies: true,
                workspace_diagnostics: true,
                work_done_progress: Default::default(),
            })),
            experimental: None,
        }
    }
}
