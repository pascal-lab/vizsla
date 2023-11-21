use vfs::AbsPathBuf;

use crate::project_manifest::ProjectManifest;

pub struct ProjectWorkspace {
    workspace_root: AbsPathBuf,
}

impl ProjectWorkspace {
    pub fn load(manifest: ProjectManifest) -> anyhow::Result<ProjectWorkspace> {
        todo!()
    }
}
