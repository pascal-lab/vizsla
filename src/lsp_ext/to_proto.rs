use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Error};
use hir::container::InFile;
use ide::{
    SymbolKind,
    code_action::{CodeAction, CodeActionKind},
    code_lens::{CodeLens, CodeLensKind},
    diagnostics as ide_diagnostics,
    document_highlight::DocumentHighlight,
    folding_ranges::{Fold, FoldingConfig},
    hover::HoverFormat,
    inlay_hint::{InlayHint, InlayKind},
    markup::Markup,
    navigation_target::NavTarget,
    references::ReferenceCategory,
    rename::RenameError,
    semantic_tokens::{SemaToken, SemaTokenModifier, SemaTokenPort, SemaTokenTag},
    signature_help::SignatureHelp,
    source_change::SourceChange,
};
use itertools::Itertools;
use span::{FilePosition, FileRange};
use syntax::DiagnosticSeverity as SlangDiagnosticSeverity;
use utils::{
    line_index::{LineCol, LineIndex, TextRange, TextSize},
    lines::{LineEnding, LineInfo, PositionEncoding},
    paths::{
        AbsPath,
        camino::{Utf8Component, Utf8Prefix},
    },
    text_edit::{TextEdit, TextEditItem},
};
use vfs::FileId;

use super::{
    ext::{self, CodeActionResolveError, CodeLensData, CodeLensDataKind},
    lsp_error::LspError,
};
use crate::{
    global_state::snapshot::GlobalStateSnapshot,
    i18n::{I18n, keys},
    lsp_ext::ext::{
        SEMA_TOKENS_TYPES, SemaTokenModifierSet, sema_token_modifiers, sema_token_types,
    },
};

pub(crate) fn goto_definition_response(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    targets: Vec<NavTarget>,
) -> anyhow::Result<<lspt::request::DefinitionRequest as lspt::request::Request>::Result> {
    let res = if snap.config.location_link() {
        let links = targets
            .into_iter()
            .unique_by(|NavTarget { file_id, full_range, focus_range, .. }| {
                (*file_id, *full_range, *focus_range)
            })
            .map(|nav| location_link(snap, src, nav))
            .collect::<anyhow::Result<Vec<_>>>()?;
        lspt::Union3::B(links)
    } else {
        let locations = targets
            .into_iter()
            .map(|nav| FileRange { file_id: nav.file_id, range: nav.focus_or_full_range() })
            .unique()
            .map(|file_range| location(snap, file_range))
            .collect::<anyhow::Result<Vec<_>>>()?;
        lspt::Union3::A(lspt::Union2::B(locations))
    };
    Ok(Some(res))
}

#[allow(deprecated)]
pub(crate) fn document_symbol(
    line_info: &LineInfo,
    symbol: ide::document_symbols::DocumentSymbol,
) -> lspt::DocumentSymbol {
    let children =
        symbol.children.into_iter().map(|child| document_symbol(line_info, child)).collect_vec();

    lspt::DocumentSymbol {
        name: symbol.name,
        detail: symbol.detail,
        kind: symbol_kind(symbol.kind),
        tags: None,
        range: self::range(line_info, symbol.full_range),
        selection_range: self::range(line_info, symbol.focus_range),
        children: if children.is_empty() { None } else { Some(children) },
    }
}

#[allow(deprecated)]
pub(crate) fn document_symbol_information(
    symbol: ide::document_symbols::DocumentSymbol,
    url: lspt::Uri,
    line_info: &LineInfo,
    res: &mut Vec<lspt::SymbolInformation>,
) {
    res.push(lspt::SymbolInformation {
        name: symbol.name,
        kind: symbol_kind(symbol.kind),
        tags: None,
        location: lspt::Location {
            uri: url.clone(),
            range: self::range(line_info, symbol.focus_range),
        },
        container_name: symbol.container_name,
    });

    for child in symbol.children {
        document_symbol_information(child, url.clone(), line_info, res);
    }
}

pub(crate) fn document_highlight(
    line_info: &LineInfo,
    DocumentHighlight { range, category }: DocumentHighlight,
) -> lspt::DocumentHighlight {
    let kind = if category.contains(ReferenceCategory::READ) {
        Some(lspt::DocumentHighlightKind::Read)
    } else if category.contains(ReferenceCategory::WRITE) {
        Some(lspt::DocumentHighlightKind::Write)
    } else {
        None
    };

    lspt::DocumentHighlight { range: self::range(line_info, range), kind }
}

const SLANG_DIAGNOSTIC_SOURCE: &str = "slang";
const VIZSLA_DIAGNOSTIC_SOURCE: &str = "vizsla";
pub(crate) fn diagnostic(
    i18n: I18n,
    line_info: &LineInfo,
    diag: ide_diagnostics::Diagnostic,
) -> lspt::Diagnostic {
    let data = diagnostic_data(&diag);
    let message = diagnostic_message(i18n, &diag);
    lspt::Diagnostic {
        range: self::range(line_info, diag.range),
        severity: diagnostic_severity(diag.severity),
        code: Some(lspt::Union2::B(format!("{}:{}", diag.subsystem, diag.code))),
        code_description: None,
        source: Some(
            match diag.source {
                ide_diagnostics::DiagnosticSource::SlangParse
                | ide_diagnostics::DiagnosticSource::SlangSemantic => SLANG_DIAGNOSTIC_SOURCE,
                ide_diagnostics::DiagnosticSource::Vizsla => VIZSLA_DIAGNOSTIC_SOURCE,
            }
            .to_string(),
        ),
        message,
        related_information: None,
        tags: None,
        data: Some(data),
    }
}

fn diagnostic_message(i18n: I18n, diag: &ide_diagnostics::Diagnostic) -> String {
    match diag.message_key {
        Some(key) if diag.source == ide_diagnostics::DiagnosticSource::Vizsla => {
            i18n.format(key, diag.message_args.iter().map(|(name, value)| (*name, value.clone())))
        }
        _ => diag.message.clone(),
    }
}

fn diagnostic_data(diag: &ide_diagnostics::Diagnostic) -> serde_json::Value {
    serde_json::json!({
        "source": match diag.source {
            ide_diagnostics::DiagnosticSource::SlangParse => "parse",
            ide_diagnostics::DiagnosticSource::SlangSemantic => "semantic",
            ide_diagnostics::DiagnosticSource::Vizsla => VIZSLA_DIAGNOSTIC_SOURCE,
        },
        "subsystem": diag.subsystem,
        "code": diag.code,
        "name": diag.name,
        "option": diag.option_name,
        "groups": diag.groups,
        "selectorHints": diagnostic_selector_hints(diag),
    })
}

fn diagnostic_selector_hints(diag: &ide_diagnostics::Diagnostic) -> Vec<String> {
    let mut selectors = vec![format!("code:{}:{}", diag.subsystem, diag.code)];

    selectors.push(format!("name:{}", diag.name));

    if let Some(option) = &diag.option_name {
        selectors.push(format!("option:{option}"));
    }

    selectors.extend(diag.groups.iter().map(|group| format!("group:{group}")));
    selectors.push(match diag.source {
        ide_diagnostics::DiagnosticSource::SlangParse => "source:parse".to_owned(),
        ide_diagnostics::DiagnosticSource::SlangSemantic => "source:semantic".to_owned(),
        ide_diagnostics::DiagnosticSource::Vizsla => {
            format!("source:{VIZSLA_DIAGNOSTIC_SOURCE}")
        }
    });

    selectors
}

fn diagnostic_severity(severity: SlangDiagnosticSeverity) -> Option<lspt::DiagnosticSeverity> {
    use lspt::DiagnosticSeverity as LspSeverity;
    match severity {
        SlangDiagnosticSeverity::Ignored => None,
        SlangDiagnosticSeverity::Note => Some(LspSeverity::Information),
        SlangDiagnosticSeverity::Warning => Some(LspSeverity::Warning),
        SlangDiagnosticSeverity::Error | SlangDiagnosticSeverity::Fatal => Some(LspSeverity::Error),
    }
}

fn symbol_kind(symbol_kind: SymbolKind) -> lspt::SymbolKind {
    use lspt::SymbolKind as LspSymbolKind;
    match symbol_kind {
        SymbolKind::Module => LspSymbolKind::Module,
        SymbolKind::Config => LspSymbolKind::Namespace,
        SymbolKind::Primitive => LspSymbolKind::Object,
        SymbolKind::NonAnsiPortLabel => LspSymbolKind::Field,
        SymbolKind::PortDecl => LspSymbolKind::Field,
        SymbolKind::ParamDecl => LspSymbolKind::TypeParameter,
        SymbolKind::NetDecl => LspSymbolKind::Property,
        SymbolKind::DataDecl => LspSymbolKind::Variable,
        SymbolKind::Genvar => LspSymbolKind::Variable,
        SymbolKind::Specparam => LspSymbolKind::TypeParameter,
        SymbolKind::Typedef => LspSymbolKind::TypeParameter,
        SymbolKind::Instance => LspSymbolKind::Object,
        SymbolKind::Block => LspSymbolKind::Namespace,
        SymbolKind::Stmt => LspSymbolKind::Namespace,
        SymbolKind::Fn => LspSymbolKind::Function,
        SymbolKind::Generate => LspSymbolKind::Namespace,
        SymbolKind::Specify => LspSymbolKind::Namespace,
        SymbolKind::Interface => LspSymbolKind::Interface,
        SymbolKind::Library => LspSymbolKind::Namespace,
        SymbolKind::Region => LspSymbolKind::Namespace,
        SymbolKind::Unknown => LspSymbolKind::Namespace,
    }
}

fn location_link(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    target: NavTarget,
) -> anyhow::Result<lspt::LocationLink> {
    let origin_selection_range = try {
        let FileRange { file_id, range } = src?;
        let line_info = snap.line_info(file_id).ok()?;
        self::range(&line_info, range)
    };

    let (target_uri, target_range, target_selection_range) = location_info(snap, target)?;
    let res = lspt::LocationLink {
        origin_selection_range,
        target_uri,
        target_range,
        target_selection_range,
    };
    Ok(res)
}

fn location_info(
    snap: &GlobalStateSnapshot,
    NavTarget { file_id, full_range, focus_range, .. }: NavTarget,
) -> anyhow::Result<(lspt::Uri, lspt::Range, lspt::Range)> {
    let line_info = snap.line_info(file_id)?;

    let target_uri = url(snap, file_id)?;
    let target_range = self::range(&line_info, full_range);
    let target_selection_range =
        focus_range.map(|it| self::range(&line_info, it)).unwrap_or(target_range);
    Ok((target_uri, target_range, target_selection_range))
}

pub(crate) fn url(snap: &GlobalStateSnapshot, file_id: FileId) -> anyhow::Result<lspt::Uri> {
    snap.url(file_id)
}

pub(crate) fn url_from_abs_path(path: &AbsPath) -> anyhow::Result<lspt::Uri> {
    let url = lspt::Uri::from_file_path(path)
        .map_err(|()| anyhow::format_err!("failed to convert file path to URL: {path}"))?;

    let Some(Utf8Component::Prefix(prefix)) = path.components().next() else {
        return Ok(url);
    };

    if !matches!(prefix.kind(), Utf8Prefix::Disk(_) | Utf8Prefix::VerbatimDisk(_)) {
        return Ok(url);
    }

    let Some((scheme, drive_letter, _)) = url.as_str().splitn(3, ':').collect_tuple() else {
        return Ok(url);
    };

    let start = scheme.len() + ':'.len_utf8();
    let driver_letter_range = start..(start + drive_letter.len());

    // lowercasing the `path` itself doesn't help, the `Url::parse` machinery
    // also canonicalizes the drive letter.
    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    lspt::Uri::parse(&url).with_context(|| format!("failed to parse file URL: {url}"))
}

pub(crate) fn range(line_info: &LineInfo, range: TextRange) -> lspt::Range {
    let start = position(line_info, range.start());
    let end = position(line_info, range.end());
    lspt::Range { start, end }
}

pub(crate) fn location(
    snap: &GlobalStateSnapshot,
    FileRange { file_id, range }: FileRange,
) -> anyhow::Result<lspt::Location> {
    let url = url(snap, file_id)?;
    let line_info = snap.line_info(file_id)?;
    let range = self::range(&line_info, range);
    Ok(lspt::Location { uri: url, range })
}

pub(crate) fn position(
    LineInfo { index, encoding, .. }: &LineInfo,
    offset: TextSize,
) -> lspt::Position {
    let line_col = line_col_for_position(index, offset);
    match *encoding {
        PositionEncoding::Utf8 => lspt::Position { line: line_col.line, character: line_col.col },
        PositionEncoding::Wide(enc) => match index.to_wide(enc, line_col) {
            Some(line_col) => lspt::Position { line: line_col.line, character: line_col.col },
            None => {
                tracing::debug!(?line_col, "failed to convert UTF-8 position to wide encoding");
                lspt::Position { line: line_col.line, character: line_col.col }
            }
        },
    }
}

fn line_col_for_position(index: &LineIndex, offset: TextSize) -> LineCol {
    let mut offset = u32::from(offset.min(index.text_len()));
    loop {
        let text_size = TextSize::from(offset);
        if let Some(line_col) = index.try_line_col(text_size) {
            return line_col;
        }

        if offset == 0 {
            tracing::debug!("failed to map offset to a valid line/column; using start of file");
            return LineCol { line: 0, col: 0 };
        }
        offset -= 1;
    }
}

pub(crate) fn rename_error(i18n: I18n, err: RenameError) -> LspError {
    let key = match err {
        RenameError::NoRefFound => keys::RENAME_NO_REF_FOUND,
        RenameError::NoDefFound => keys::RENAME_NO_DEF_FOUND,
        RenameError::OverlappingEdits => keys::RENAME_OVERLAPPING_EDITS,
    };
    LspError::new(lsp_server::ErrorCode::InvalidParams as i32, i18n.text(key).to_owned())
}

pub(crate) fn format_error(err: Error) -> LspError {
    LspError::new(lsp_server::ErrorCode::RequestFailed as i32, err.to_string())
}

pub(crate) fn code_action_resolve_error(i18n: I18n, err: CodeActionResolveError) -> LspError {
    let message = match err {
        CodeActionResolveError::NoData => i18n.text(keys::CODE_ACTION_RESOLVE_NO_DATA).to_owned(),
        CodeActionResolveError::Stable => i18n.text(keys::CODE_ACTION_RESOLVE_STALE).to_owned(),
        CodeActionResolveError::InvalidId(id) => {
            i18n.format(keys::CODE_ACTION_RESOLVE_INVALID_ID, [("id", id)])
        }
    };
    LspError::new(lsp_server::ErrorCode::InvalidParams as i32, message)
}

pub(crate) fn workspace_edit(
    snap: &GlobalStateSnapshot,
    source_change: SourceChange,
) -> anyhow::Result<lspt::WorkspaceEdit> {
    let mut document_changes = Vec::with_capacity(source_change.text_edits.len());

    for (file_id, edit) in source_change.text_edits {
        let text_document = optional_versioned_text_document_identifier(snap, file_id)?;
        let line_info = snap.line_info(file_id)?;
        let edits = edit.into_iter().map(|it| lspt::Union2::A(text_edit(&line_info, it))).collect();
        document_changes.push(lspt::TextDocumentEdit { text_document, edits });
    }

    let workspace_edit = lspt::WorkspaceEdit {
        changes: None,
        document_changes: Some(document_changes.into_iter().map(lspt::Union4::A).collect()),
        change_annotations: None,
    };

    Ok(workspace_edit)
}

pub(crate) fn optional_versioned_text_document_identifier(
    snap: &GlobalStateSnapshot,
    file_id: FileId,
) -> anyhow::Result<lspt::OptionalVersionedTextDocumentIdentifier> {
    let url = url(snap, file_id)?;
    let version = snap.url_file_version(&url);
    Ok(lspt::OptionalVersionedTextDocumentIdentifier { uri: url, version })
}

pub(crate) fn text_edit(line_info: &LineInfo, item: TextEditItem) -> lspt::TextEdit {
    let range = self::range(line_info, item.del);
    let new_text = match line_info.ending {
        LineEnding::Unix => item.ins,
        LineEnding::Dos => item.ins.replace('\n', "\r\n"),
    };
    lspt::TextEdit { range, new_text }
}

pub(crate) fn text_edits(line_info: &LineInfo, edit: TextEdit) -> Vec<lspt::TextEdit> {
    edit.into_iter().map(|it| self::text_edit(line_info, it)).collect()
}

pub(crate) fn selection_ranges(
    line_info: &LineInfo,
    ranges: Vec<TextRange>,
) -> Option<lspt::SelectionRange> {
    ranges.into_iter().rfold(None, |parent, range| {
        Some(lspt::SelectionRange {
            range: self::range(line_info, range),
            parent: parent.map(Box::new),
        })
    })
}

pub(crate) fn folding_range(
    text: &str,
    line_info: &LineInfo,
    FoldingConfig { line_fold_only }: &FoldingConfig,
    Fold { range, kind, collapsed_text }: Fold,
) -> lspt::FoldingRange {
    use ide::folding_ranges::FoldKind;
    let kind = match kind {
        FoldKind::Comment => Some(lspt::FoldingRangeKind::Comment),
        FoldKind::Imports => Some(lspt::FoldingRangeKind::Imports),
        FoldKind::Region => Some(lspt::FoldingRangeKind::Region),
        _ => None,
    };

    let lspt::Range { start, end } = self::range(line_info, range);

    if *line_fold_only {
        // Clients with `line_folding_only` will fold the whole end line even if
        // it contains text not in the folding range. So we should exclude the end
        // line if there is more text after the end character on the same line.
        let range_end = range.end().into();
        let end_line = if range_end < text.len() && has_more_text_in_line(&text[range_end..]) {
            end.line
        } else {
            end.line.saturating_sub(1)
        };

        lspt::FoldingRange {
            start_line: start.line,
            start_character: None,
            end_line,
            end_character: None,
            kind,
            collapsed_text,
        }
    } else {
        lspt::FoldingRange {
            start_line: start.line,
            start_character: Some(start.character),
            end_line: end.line,
            end_character: Some(end.character),
            kind,
            collapsed_text,
        }
    }
}

fn has_more_text_in_line(text: &str) -> bool {
    let mut iter = text.chars().peekable();

    let mut met_first_punct = false;
    while let Some(c) = iter.next() {
        match c {
            ',' | ';' => {
                if met_first_punct {
                    return false;
                } else {
                    met_first_punct = true;
                }
            }
            '\n' => return true,
            '/' if iter.next_if_eq(&'/').is_some() => return true,
            '/' if iter.next_if_eq(&'*').is_some() => {
                while let Some(c) = iter.next() {
                    if c == '*' && iter.next_if_eq(&'/').is_some() {
                        break;
                    } else if c == '\n' {
                        return false;
                    }
                }
            }
            _ if c.is_whitespace() => {}
            _ => return false,
        }
    }

    true
}

pub(crate) fn hover_contents(
    markup: Markup,
    format: HoverFormat,
) -> lspt::Union3<lspt::MarkupContent, lspt::MarkedString, Vec<lspt::MarkedString>> {
    let kind = match format {
        HoverFormat::Markdown => lspt::MarkupKind::Markdown,
        HoverFormat::PlainText => lspt::MarkupKind::PlainText,
    };

    let value = markup.into();
    lspt::Union3::A(lspt::MarkupContent { kind, value })
}

pub(crate) fn inlay_hint(
    snap: &GlobalStateSnapshot,
    line_info: &LineInfo,
    hint: InlayHint,
) -> lspt::InlayHint {
    let InlayHint {
        label,
        tooltip,
        target_location,
        position,
        kind,
        text_edit,
        padding_left,
        padding_right,
    } = hint;

    let label = lspt::InlayHintLabelPart {
        value: label,
        tooltip: tooltip.map(|tooltip| {
            lspt::Union2::B(lspt::MarkupContent {
                kind: lspt::MarkupKind::Markdown,
                value: tooltip.into(),
            })
        }),
        location: target_location.and_then(|InFile { value, file_id }| {
            let file_range = FileRange { file_id: file_id.file_id(), range: value };
            self::location(snap, file_range).ok()
        }),
        command: None,
    };

    let position = self::position(line_info, position);
    let kind = match kind {
        InlayKind::ParamAssign | InlayKind::Port => Some(lspt::InlayHintKind::Parameter),
        InlayKind::EndStructure => None,
    };

    let text_edits = text_edit.map(|it| self::text_edits(line_info, it));

    lspt::InlayHint {
        position,
        label: lspt::Union2::B(vec![label]),
        kind,
        text_edits,
        tooltip: None,
        padding_left: Some(padding_left),
        padding_right: Some(padding_right),
        data: None,
    }
}

pub(crate) fn code_lens(
    snap: &GlobalStateSnapshot,
    line_info: &LineInfo,
    file_id: FileId,
    CodeLens { range, kind }: CodeLens,
) -> Option<lspt::CodeLens> {
    let range = self::range(line_info, range);
    let (command, data) = self::code_lens_kind(snap, file_id, line_info, kind).ok()?;
    Some(lspt::CodeLens { range, command, data })
}

pub(crate) fn code_lens_kind(
    snap: &GlobalStateSnapshot,
    file_id: FileId,
    line_info: &LineInfo,
    kind: CodeLensKind,
) -> anyhow::Result<(Option<lspt::Command>, Option<serde_json::Value>)> {
    let url = self::url(snap, file_id)?;

    let command = match kind {
        CodeLensKind::ModuleInstance { data, .. } => data.map(|ranges| {
            let count = ranges.len();
            let key = if count == 1 {
                keys::CODE_LENS_INSTANCES_ONE
            } else {
                keys::CODE_LENS_INSTANCES_MANY
            };
            lspt::Command {
                title: snap.config.i18n.format(key, [("count", count.to_string())]),
                command: String::new(),
                arguments: None,
            }
        }),
    };

    let data = match kind {
        CodeLensKind::ModuleInstance { pos: FilePosition { offset, .. }, .. } => {
            snap.url_file_version(&url).and_then(|version| {
                let file_pos = ext::TextDocumentPositionParams {
                    text_document: lspt::TextDocumentIdentifier { uri: url.clone() },
                    position: self::position(line_info, offset),
                };
                let data =
                    CodeLensData { version, kind: CodeLensDataKind::Instantiation(file_pos) };
                serde_json::to_value(data).ok()
            })
        }
    };

    Ok((command, data))
}

pub(crate) struct SemanticTokensBuilder {
    id: String,
    prev_line: u32,
    prev_char: u32,
    data: Vec<u32>,
}

impl SemanticTokensBuilder {
    pub(crate) fn new(id: String) -> Self {
        SemanticTokensBuilder { id, prev_line: 0, prev_char: 0, data: Default::default() }
    }

    /// Push a new token onto the builder
    pub(crate) fn push(
        &mut self,
        range: lspt::Range,
        token_type: u32,
        token_modifiers_bitset: u32,
    ) {
        let lspt::Position { line: mut push_line, character: mut push_char } = range.start;

        if !self.data.is_empty() {
            if push_line < self.prev_line
                || (push_line == self.prev_line && push_char < self.prev_char)
            {
                tracing::debug!(
                    ?range,
                    prev_line = self.prev_line,
                    prev_char = self.prev_char,
                    "skipping out-of-order semantic token"
                );
                return;
            }
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        self.data.extend([
            push_line,
            push_char,
            range.end.character.saturating_sub(range.start.character),
            token_type,
            token_modifiers_bitset,
        ]);

        self.prev_line = range.start.line;
        self.prev_char = range.start.character;
    }

    pub(crate) fn build(self) -> lspt::SemanticTokens {
        lspt::SemanticTokens { result_id: Some(self.id), data: self.data }
    }
}

pub(crate) fn semantic_tokens(
    text: &str,
    line_info: &LineInfo,
    sema_tokens: Vec<SemaToken>,
) -> lspt::SemanticTokens {
    static TOKEN_RESULT_COUNTER: AtomicU32 = AtomicU32::new(1);

    let id = TOKEN_RESULT_COUNTER.fetch_add(1, Ordering::SeqCst).to_string();
    let mut builder = SemanticTokensBuilder::new(id);

    for SemaToken { range, tag, mods } in sema_tokens.into_iter().filter(|tok| !tok.is_empty()) {
        let ty = match tag {
            SemaTokenTag::Port(SemaTokenPort::Clk) => sema_token_types::CLK_PORT,
            SemaTokenTag::Port(SemaTokenPort::Rst) => sema_token_types::RST_PORT,
            SemaTokenTag::Port(SemaTokenPort::Others) => sema_token_types::OTHERS_PORT,
            SemaTokenTag::Instance => sema_token_types::INSTANCE,
            SemaTokenTag::Type => sema_token_types::TYPE_ALIAS,
            SemaTokenTag::None => sema_token_types::GENERIC,
        };
        // Prefer standard tokens where we have an explicit fallback, otherwise
        // keep the token from the advertised legend.
        let legend_ty = sema_token_types::fallback(ty).unwrap_or(ty);
        let Some(ty) =
            SEMA_TOKENS_TYPES.iter().position(|it| it == &legend_ty).map(|idx| idx as u32)
        else {
            tracing::debug!(?legend_ty, "skipping semantic token with unknown type");
            continue;
        };

        let mut mods_set = SemaTokenModifierSet::default();
        for modifier in mods {
            let modifier = match modifier {
                SemaTokenModifier::DECL => sema_token_modifiers::DECLARATION,
                SemaTokenModifier::DEF => sema_token_modifiers::DEF,
                SemaTokenModifier::READ => sema_token_modifiers::READ,
                SemaTokenModifier::WRITE => sema_token_modifiers::WRITE,
                SemaTokenModifier::REF => sema_token_modifiers::REF,
                modifier => {
                    tracing::debug!(?modifier, "skipping unknown semantic token modifier");
                    continue;
                }
            };
            // Prefer standard modifiers where we have an explicit fallback,
            // otherwise keep the modifier from the advertised legend.
            mods_set |= sema_token_modifiers::fallback(modifier).unwrap_or(modifier);
        }
        let mods = mods_set.finish();

        for mut range in line_info.index.lines(range) {
            if text[range].ends_with('\n') {
                range = TextRange::new(range.start(), range.end() - TextSize::of('\n'));
            }
            let range = self::range(line_info, range);
            builder.push(range, ty, mods);
        }
    }

    builder.build()
}

pub(crate) fn semantic_token_delta(
    lspt::SemanticTokens { data: old, .. }: &lspt::SemanticTokens,
    lspt::SemanticTokens { data: new, result_id }: &lspt::SemanticTokens,
) -> lspt::SemanticTokensDelta {
    let old = old.as_slice();
    let new = new.as_slice();

    let offset = new.iter().zip(old.iter()).take_while(|&(n, p)| n == p).count();
    let (_, old) = old.split_at(offset);
    let (_, new) = new.split_at(offset);

    let offset_from_end =
        new.iter().rev().zip(old.iter().rev()).take_while(|&(n, p)| n == p).count();

    let (old, _) = old.split_at(old.len() - offset_from_end);
    let (new, _) = new.split_at(new.len() - offset_from_end);

    let edits = if old.is_empty() && new.is_empty() {
        vec![]
    } else {
        vec![lspt::SemanticTokensEdit {
            start: offset as u32,
            delete_count: old.len() as u32,
            data: Some(new.to_vec()),
        }]
    };

    lspt::SemanticTokensDelta { result_id: result_id.clone().clone(), edits }
}

pub(crate) fn signature_help(
    sig_help: SignatureHelp,
    support_label_offsets: bool,
) -> lspt::SignatureHelp {
    let parameters = if support_label_offsets {
        sig_help
            .param_ranges
            .iter()
            .map(|it| {
                let start = sig_help.label[..it.start().into()].chars().count() as u32;
                let end = sig_help.label[..it.end().into()].chars().count() as u32;
                [start, end]
            })
            .map(|range| lspt::ParameterInformation {
                label: lspt::Union2::B((range[0], range[1])),
                documentation: None,
            })
            .collect()
    } else {
        sig_help
            .param_ranges
            .iter()
            .map(|range| lspt::ParameterInformation {
                label: lspt::Union2::A(sig_help.label[*range].to_owned()),
                documentation: None,
            })
            .collect()
    };

    let active_parameter = sig_help.active_parameter.map(|it| it as u32);

    let signature = lspt::SignatureInformation {
        label: sig_help.label,
        documentation: None,
        parameters: Some(parameters),
        active_parameter,
    };

    lspt::SignatureHelp { signatures: vec![signature], active_signature: Some(0), active_parameter }
}

pub(crate) fn code_action_kind(kind: CodeActionKind) -> lspt::CodeActionKind {
    match kind {
        CodeActionKind::Generate => lspt::CodeActionKind::Empty,
        CodeActionKind::QuickFix => lspt::CodeActionKind::QuickFix,
        CodeActionKind::Refactor => lspt::CodeActionKind::Refactor,
        CodeActionKind::RefactorExtract => lspt::CodeActionKind::RefactorExtract,
        CodeActionKind::RefactorInline => lspt::CodeActionKind::RefactorInline,
        CodeActionKind::RefactorRewrite => lspt::CodeActionKind::RefactorRewrite,
    }
}

pub(crate) fn code_action(
    snap: &GlobalStateSnapshot,
    CodeAction { id, label, source_change, .. }: CodeAction,
    resolve_data: Option<(usize, lspt::CodeActionParams, Option<i32>)>,
    diagnostics: Option<Vec<lspt::Diagnostic>>,
) -> anyhow::Result<lspt::CodeAction> {
    let title = code_action_title(snap.config.i18n, id.name, &label);
    let mut res = lspt::CodeAction {
        title,
        kind: Some(self::code_action_kind(id.kind)),
        edit: None,
        is_preferred: None,
        command: None, // TODO: fill commands
        diagnostics,
        disabled: None,
        data: None,
        tags: None,
    };

    match (source_change, resolve_data) {
        (Some(it), _) => res.edit = Some(workspace_edit(snap, it)?),
        (None, Some((idx, code_action_params, version))) => {
            let data = ext::CodeActionData {
                id: format!("{}:{idx}", id.name),
                code_action_params,
                version,
            };
            res.data = Some(serde_json::to_value(data)?);
        }
        (None, None) => {
            return Err(anyhow::format_err!(
                "code action '{}' has no edit and no resolve data",
                id.name
            ));
        }
    };
    Ok(res)
}

fn code_action_title(i18n: I18n, id: &str, label: &str) -> String {
    code_action_title_key(id, label)
        .map(|key| i18n.text(key).to_owned())
        .unwrap_or_else(|| label.to_owned())
}

fn code_action_title_key(id: &str, label: &str) -> Option<&'static str> {
    Some(match id {
        "add_missing_connections" => keys::CODE_ACTION_ADD_MISSING_CONNECTIONS,
        "add_missing_parameters" => keys::CODE_ACTION_ADD_MISSING_PARAMETERS,
        "convert_ordered_ports" => keys::CODE_ACTION_CONVERT_ORDERED_PORTS,
        "convert_ordered_params" => keys::CODE_ACTION_CONVERT_ORDERED_PARAMS,
        "remove_empty_port_connections" => keys::CODE_ACTION_REMOVE_EMPTY_PORT_CONNECTIONS,
        "add_implicit_named_port_parens" => keys::CODE_ACTION_ADD_IMPLICIT_NAMED_PORT_PARENS,
        "add_instance_parens" => keys::CODE_ACTION_ADD_INSTANCE_PARENS,
        "convert_literal_base" => match label {
            "Convert literal to binary" => keys::CODE_ACTION_CONVERT_LITERAL_TO_BINARY,
            "Convert literal to octal" => keys::CODE_ACTION_CONVERT_LITERAL_TO_OCTAL,
            "Convert literal to decimal" => keys::CODE_ACTION_CONVERT_LITERAL_TO_DECIMAL,
            "Convert literal to hexadecimal" => keys::CODE_ACTION_CONVERT_LITERAL_TO_HEXADECIMAL,
            _ => return None,
        },
        _ => return None,
    })
}
