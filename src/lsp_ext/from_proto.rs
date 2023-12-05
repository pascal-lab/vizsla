use utils::paths::AbsPathBuf;
use vfs::vfs_path::VfsPath;

pub(crate) fn vfs_path(url: &lsp_types::Url) -> anyhow::Result<vfs::vfs::VfsPath> {
    let path = url.to_file_path().map_err(|()| anyhow::format_err!("url is not a file"))?;
    Ok(VfsPath::from(AbsPathBuf::try_from(path).unwrap()))
}
