use anyhow::Context;
use vfs::AbsPathBuf;

use crate::project_manifest::ProjectManifest;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Workspace {
    Project { workspace_root: AbsPathBuf },
    DetachedFiles { files: Vec<AbsPathBuf> },
}

impl Workspace {
    pub fn load(manifest: &ProjectManifest) -> anyhow::Result<Workspace> {
        Self::load_helper(&manifest)
            .with_context(|| format!("failed to load workspace {:?}", &manifest))
    }

    fn load_helper(manifest: &ProjectManifest) -> anyhow::Result<Workspace> {
        todo!()
    }

    pub fn load_detached_files(files: &Vec<AbsPathBuf>) -> anyhow::Result<Workspace> {
        todo!()
    }
}
