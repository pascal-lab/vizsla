use anyhow::Context;
use vfs::AbsPathBuf;

use crate::{macro_def::MacroDef, project_manifest::ProjectManifest};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Workspace {
    Project { workspace_root: AbsPathBuf, macro_defs: MacroDef },
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
