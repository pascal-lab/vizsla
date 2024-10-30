use ide::{
    Cancellable, SymbolKind,
    document_highlight::{DocumentHighlight, ReferenceCategory},
    navigation_target::NavTarget,
};
use itertools::Itertools;
use line_index::{TextRange, TextSize};
use span::FileRange;
use utils::{
    lines::{LineInfo, PositionEncoding},
    paths::{
        AbsPath,
        camino::{Utf8Component, Utf8Prefix},
    },
};
use vfs::FileId;

use crate::global_state::snapshot::GlobalStateSnapshot;

pub(crate) fn goto_definition_response(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    targets: Vec<NavTarget>,
) -> Cancellable<lsp_types::GotoDefinitionResponse> {
    let res = if snap.config.location_link() {
        let links = targets
            .into_iter()
            .unique_by(|nav| (nav.file_id, nav.full_range, nav.focus_range))
            .map(|nav| location_link(snap, src, nav))
            .collect::<Cancellable<Vec<_>>>()?;
        links.into()
    } else {
        let locations = targets
            .into_iter()
            .map(|nav| FileRange { file_id: nav.file_id, range: nav.focus_or_full_range() })
            .unique()
            .map(|range| location(snap, range))
            .collect::<Cancellable<Vec<_>>>()?;
        locations.into()
    };
    Ok(res)
}

pub(crate) fn document_symbol(
    line_info: &LineInfo,
    symbol: ide::document_symbols::DocumentSymbol,
) -> lsp_types::DocumentSymbol {
    let children = symbol
        .children
        .map(|it| it.into_iter().map(|child| document_symbol(line_info, child)).collect());

    lsp_types::DocumentSymbol {
        name: symbol.name,
        detail: symbol.detail,
        kind: symbol_kind(symbol.kind),
        tags: None,
        deprecated: None,
        range: lsp_range(line_info, symbol.full_range),
        selection_range: lsp_range(line_info, symbol.focus_range),
        children,
    }
}

pub(crate) fn document_symbol_information(
    symbol: ide::document_symbols::DocumentSymbol,
    url: lsp_types::Url,
    line_info: &LineInfo,
    res: &mut Vec<lsp_types::SymbolInformation>,
) {
    res.push(lsp_types::SymbolInformation {
        name: symbol.name,
        kind: symbol_kind(symbol.kind),
        tags: None,
        deprecated: None,
        location: lsp_types::Location::new(url.clone(), lsp_range(line_info, symbol.focus_range)),
        container_name: symbol.container_name,
    });

    if let Some(children) = symbol.children {
        for child in children {
            document_symbol_information(child, url.clone(), line_info, res);
        }
    }
}

pub(crate) fn document_highlight(
    line_info: &LineInfo,
    DocumentHighlight { range, category }: DocumentHighlight,
) -> lsp_types::DocumentHighlight {
    let kind = if category.contains(ReferenceCategory::READ) {
        Some(lsp_types::DocumentHighlightKind::READ)
    } else if category.contains(ReferenceCategory::WRITE) {
        Some(lsp_types::DocumentHighlightKind::WRITE)
    } else {
        None
    };

    lsp_types::DocumentHighlight { range: lsp_range(line_info, range), kind }
}

fn symbol_kind(symbol_kind: SymbolKind) -> lsp_types::SymbolKind {
    use lsp_types::SymbolKind as LspSymbolKind;
    match symbol_kind {
        SymbolKind::Module => LspSymbolKind::MODULE,
        SymbolKind::PortLabel => LspSymbolKind::FIELD,
        SymbolKind::Decl => LspSymbolKind::VARIABLE,
        SymbolKind::Instance => LspSymbolKind::OBJECT,
        SymbolKind::Block => LspSymbolKind::NAMESPACE,
        SymbolKind::Stmt => LspSymbolKind::NAMESPACE,
        SymbolKind::Fn => LspSymbolKind::FUNCTION,
        SymbolKind::Generate => LspSymbolKind::NAMESPACE,
        SymbolKind::Interface => LspSymbolKind::INTERFACE,
    }
}

fn location(
    snap: &GlobalStateSnapshot,
    FileRange { file_id, range }: FileRange,
) -> Cancellable<lsp_types::Location> {
    let url = url(snap, file_id);
    let line_info = snap.line_info(file_id)?;
    let range = lsp_range(&line_info, range);
    let loc = lsp_types::Location::new(url, range);
    Ok(loc)
}

fn location_link(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    target: NavTarget,
) -> Cancellable<lsp_types::LocationLink> {
    let origin_selection_range = src.and_then(|FileRange { file_id, range }| {
        Some(lsp_range(&snap.line_info(file_id).ok()?, range))
    });
    let (target_uri, target_range, target_selection_range) = location_info(snap, target)?;
    let res = lsp_types::LocationLink {
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
) -> Cancellable<(lsp_types::Url, lsp_types::Range, lsp_types::Range)> {
    let line_info = snap.line_info(file_id)?;

    let target_uri = url(snap, file_id);
    let target_range = lsp_range(&line_info, full_range);
    let target_selection_range =
        focus_range.map(|it| lsp_range(&line_info, it)).unwrap_or(target_range);
    Ok((target_uri, target_range, target_selection_range))
}

pub(crate) fn url(snap: &GlobalStateSnapshot, file_id: FileId) -> lsp_types::Url {
    snap.url(file_id)
}

// Returns a `Url` object from a given path, will lowercase drive letters if
// present.
//
// This will only happen when processing windows paths.
// When processing non-windows path, this is the same as `Url::from_file_path`.
pub(crate) fn url_from_abs_path(path: &AbsPath) -> lsp_types::Url {
    let url = lsp_types::Url::from_file_path(path).unwrap();

    let Some(Utf8Component::Prefix(prefix)) = path.components().next() else {
        return url;
    };

    if !matches!(prefix.kind(), Utf8Prefix::Disk(_) | Utf8Prefix::VerbatimDisk(_)) {
        return url;
    }

    let Some((scheme, drive_letter, _)) = url.as_str().splitn(3, ':').collect_tuple() else {
        return url;
    };

    let start = scheme.len() + ':'.len_utf8();
    let driver_letter_range = start..(start + drive_letter.len());

    // lowercasing the `path` itself doesn't help, the `Url::parse` machinery
    // also canonicalizes the drive letter.
    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    lsp_types::Url::parse(&url).unwrap()
}

pub(crate) fn lsp_range(line_info: &LineInfo, range: TextRange) -> lsp_types::Range {
    let start = position(line_info, range.start());
    let end = position(line_info, range.end());
    lsp_types::Range::new(start, end)
}

pub(crate) fn position(
    LineInfo { index, encoding, .. }: &LineInfo,
    offset: TextSize,
) -> lsp_types::Position {
    let line_col = index.line_col(offset);
    match *encoding {
        PositionEncoding::Utf8 => lsp_types::Position::new(line_col.line, line_col.col),
        PositionEncoding::Wide(enc) => {
            let line_col = index.to_wide(enc, line_col).unwrap();
            lsp_types::Position::new(line_col.line, line_col.col)
        }
    }
}
