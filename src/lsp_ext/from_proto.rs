use ide::code_lens::CodeLensKind;
use span::{FilePosition, FileRange};
use utils::{
    line_index::{LineCol, TextRange, TextSize, WideLineCol},
    lines::{LineInfo, PositionEncoding},
    paths::AbsPathBuf,
};
use vfs::{FileId, VfsPath};

use super::ext;
use crate::global_state::snapshot::GlobalStateSnapshot;

pub(crate) fn vfs_path(url: &lspt::Uri) -> anyhow::Result<vfs::VfsPath> {
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    let path = AbsPathBuf::try_from(path)
        .map_err(|path| anyhow::format_err!("file url path is not absolute UTF-8: {path:?}"))?;
    Ok(VfsPath::from(path))
}

pub(crate) fn abs_path(url: &lspt::Uri) -> anyhow::Result<AbsPathBuf> {
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    AbsPathBuf::try_from(path)
        .map_err(|path| anyhow::format_err!("file url path is not absolute UTF-8: {path:?}"))
}

// convert position (line, col) to Offset
pub(crate) fn offset(
    LineInfo { index, encoding, .. }: &LineInfo,
    pos: lspt::Position,
) -> anyhow::Result<TextSize> {
    let line_col = match *encoding {
        PositionEncoding::Utf8 => LineCol { line: pos.line, col: pos.character },
        PositionEncoding::Wide(enc) => {
            let line_col = WideLineCol { line: pos.line, col: pos.character };
            index
                .to_utf8(enc, line_col)
                .ok_or_else(|| anyhow::format_err!("Invalid wide col offset"))?
        }
    };
    let text_size = index.offset(line_col).ok_or_else(|| anyhow::format_err!("Invalid offset"))?;
    Ok(text_size)
}

pub(crate) fn text_range(
    line_info: &LineInfo,
    lspt::Range { start, end }: lspt::Range,
) -> anyhow::Result<TextRange> {
    let start = offset(line_info, start)?;
    let end = offset(line_info, end)?;

    if end < start {
        return Err(anyhow::format_err!("Invalid Range"));
    }

    Ok(TextRange::new(start, end))
}

pub(crate) fn file_position(
    snap: &GlobalStateSnapshot,
    text_document: lspt::TextDocumentIdentifier,
    position: lspt::Position,
) -> anyhow::Result<FilePosition> {
    let file_id = snap.file_id(&text_document.uri)?;
    let line_index = snap.line_info(file_id)?;
    let offset = offset(&line_index, position)?;
    Ok(FilePosition { file_id, offset })
}

pub(crate) fn file_range(
    snap: &GlobalStateSnapshot,
    url: &lspt::Uri,
    range: lspt::Range,
) -> anyhow::Result<FileRange> {
    let file_id = snap.file_id(url)?;
    let line_index = snap.line_info(file_id)?;
    let range = text_range(&line_index, range)?;
    Ok(FileRange { file_id, range })
}

pub(crate) fn file_id(snap: &GlobalStateSnapshot, url: &lspt::Uri) -> anyhow::Result<FileId> {
    snap.file_id(url)
}

pub(crate) fn code_lens(
    snap: &GlobalStateSnapshot,
    data: serde_json::Value,
) -> anyhow::Result<(FileId, CodeLensKind)> {
    let data = serde_json::from_value::<ext::CodeLensData>(data)?;
    let (file_id, kind) = match data.kind {
        ext::CodeLensDataKind::Instantiation(pos_params) => {
            let pos = self::file_position(snap, pos_params.text_document, pos_params.position)?;
            let file_id = pos.file_id;
            (file_id, CodeLensKind::ModuleInstance { pos, data: None })
        }
    };

    Ok((file_id, kind))
}

pub(crate) fn code_action_data(data: serde_json::Value) -> anyhow::Result<ext::CodeActionData> {
    let data = serde_json::from_value::<ext::CodeActionData>(data)?;
    Ok(data)
}
