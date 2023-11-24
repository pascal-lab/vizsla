use std::{collections::BTreeSet, fs};

use const_format::formatcp;
use itertools::Itertools;
use vfs::AbsPathBuf;

pub const MANIFEST_FILE_NAME: &str = formatcp!("vizsla_config.toml");

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ProjectManifest(AbsPathBuf);

impl TryFrom<AbsPathBuf> for ProjectManifest {
    type Error = String;

    fn try_from(path: AbsPathBuf) -> Result<Self, Self::Error> {
        ProjectManifest::validate_manifest(&path).and_then(|_| Ok(ProjectManifest(path)))
    }
}

impl TryFrom<&AbsPathBuf> for ProjectManifest {
    type Error = String;

    fn try_from(path: &AbsPathBuf) -> Result<Self, Self::Error> {
        ProjectManifest::validate_manifest(path).and_then(|_| Ok(ProjectManifest(path.clone())))
    }
}

impl ProjectManifest {
    fn validate_manifest(path: &AbsPathBuf) -> Result<(), String> {
        if path.parent() == None {
            return Err(String::from("Bad manifest path: {path}"));
        }

        if path.file_name().unwrap_or_default() != MANIFEST_FILE_NAME {
            return Err(String::from(
                "Project root must point to {MANIFEST_FILE_NAME}.toml: {path}",
            ));
        }

        Ok(())
    }

    pub fn discover(path: &AbsPathBuf) -> anyhow::Result<Vec<ProjectManifest>> {
        if let Ok(manifest) = ProjectManifest::try_from(path) {
            return Ok(vec![manifest]);
        }

        // find in parent dirs
        let mut cur = Some(path.as_path());
        while let Some(path) = cur {
            let candidate = path.join(MANIFEST_FILE_NAME);

            if fs::metadata(&candidate).is_ok() {
                if let Ok(manifest) = ProjectManifest::try_from(candidate) {
                    return Ok(vec![manifest]);
                }
            }

            cur = path.parent();
        }

        // Only one level down to avoid cycles
        let entities = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|it| it.path().join(MANIFEST_FILE_NAME))
            .filter(|x| x.exists())
            .filter_map(|x| ProjectManifest::try_from(AbsPathBuf::assert(x)).ok())
            .collect_vec();

        Ok(entities)
    }

    pub fn discover_all(paths: &[AbsPathBuf]) -> Vec<ProjectManifest> {
        paths
            .iter()
            .filter_map(|path| ProjectManifest::discover(path).ok())
            .flatten()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect_vec()
    }
}
