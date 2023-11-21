use vfs::AbsPathBuf;

use crate::project_manifest::ProjectManifest;

pub struct Workspace {
    workspace_root: AbsPathBuf,
}

impl Workspace {
    pub fn load(manifest: ProjectManifest) -> anyhow::Result<Workspace> {
        todo!()
    }
}
