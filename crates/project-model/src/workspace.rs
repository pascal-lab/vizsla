use anyhow::Context;
use triomphe::Arc;
use vfs::AbsPathBuf;

use crate::{
    macro_def::MacroDef, project_manifest::ProjectManifest, toml_workspace::TomlWorkspace,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Workspace {
    Project(TomlWorkspace),
    DetachedFiles(Arc<Vec<AbsPathBuf>>),
}

impl Workspace {
    pub fn load(manifest: &ProjectManifest) -> anyhow::Result<Workspace> {
        Self::load_helper(&manifest)
            .with_context(|| format!("failed to load workspace {:?}", &manifest))
    }

    fn load_helper(manifest: &ProjectManifest) -> anyhow::Result<Workspace> {
        match manifest {
            ProjectManifest::Toml(toml) => {
                assert_eq!(toml.extension().unwrap(), "toml");

                Ok(Workspace::Project(TomlWorkspace::load_from_file(toml)?))
            }
            ProjectManifest::Discover(path) => {
                Ok(Workspace::Project(TomlWorkspace::default_from_path(path)))
            }
        }
    }

    pub fn load_detached_files(files: Arc<Vec<AbsPathBuf>>) -> anyhow::Result<Workspace> {
        Ok(Workspace::DetachedFiles(files))
    }
}
