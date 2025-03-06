use span::{FilePosition, FileRange};
use utils::{
    line_index::{LineCol, TextRange, TextSize, WideLineCol},
    lines::{LineInfo, PositionEncoding},
    paths::AbsPathBuf,
};
use vfs::{FileId, VfsPath};

use crate::global_state::snapshot::GlobalStateSnapshot;

pub(crate) fn vfs_path(url: &lsp_types::Url) -> anyhow::Result<vfs::VfsPath> {
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    Ok(VfsPath::from(AbsPathBuf::try_from(path).unwrap()))
}

pub(crate) fn abs_path(url: &lsp_types::Url) -> anyhow::Result<AbsPathBuf> {
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    Ok(AbsPathBuf::try_from(path).unwrap())
}

// convert position (line, col) to Offset
pub(crate) fn offset(
    LineInfo { index, encoding, .. }: &LineInfo,
    pos: lsp_types::Position,
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
    lsp_types::Range { start, end }: lsp_types::Range,
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
    pos_params: lsp_types::TextDocumentPositionParams,
) -> anyhow::Result<FilePosition> {
    let file_id = snap.file_id(&pos_params.text_document.uri)?;
    let line_index = snap.line_info(file_id)?;
    let offset = offset(&line_index, pos_params.position)?;
    Ok(FilePosition { file_id, offset })
}

pub(crate) fn file_range(
    snap: &GlobalStateSnapshot,
    url: &lsp_types::Url,
    range: lsp_types::Range,
) -> anyhow::Result<FileRange> {
    let file_id = snap.file_id(url)?;
    let line_index = snap.line_info(file_id)?;
    let range = text_range(&line_index, range)?;
    Ok(FileRange { file_id, range })
}

pub(crate) fn file_id(snap: &GlobalStateSnapshot, url: &lsp_types::Url) -> anyhow::Result<FileId> {
    snap.file_id(url)
}
