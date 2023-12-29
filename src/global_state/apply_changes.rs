use std::{borrow::BorrowMut, collections::hash_map::Entry::Occupied, ops::DerefMut};

use base_db::change::Change;
use itertools::Itertools;
use nohash_hasher::IntMap;
use parking_lot::{RwLockUpgradableReadGuard, RwLockWriteGuard};
use rustc_hash::FxHashMap;
use triomphe::Arc;
use utils::lines::LineEndings;
use vfs::vfs::{ChangeKind, ChangedFile, FileId, Vfs};

use super::{reload::should_refresh_for_change, GlobalState};

// Apply changes
impl GlobalState {
    pub(crate) fn process_changes(&mut self) -> bool {
        let mut write_guard = self.vfs.write();
        let changed_files = write_guard.0.take_changes();
        // downgrade earlier to allow more reader
        let read_guard = RwLockWriteGuard::downgrade_to_upgradable(write_guard);
        let vfs = &read_guard.0;

        // collect changes
        let Some(changed_files) = Self::colease_modifications(changed_files) else {
            return false;
        };

        let mut workspace_structure_change = None;
        // A file was added or deleted
        let mut has_structure_changes = false;
        let mut bytes = vec![];
        for changed_file in changed_files {
            let path = vfs.file_path(changed_file.file_id);
            if let Some(path) = path.as_abs_path().map(|apath| apath.to_path_buf()) {
                if changed_file.is_created_or_deleted() {
                    has_structure_changes = true;
                    workspace_structure_change = Some(path);
                } else if should_refresh_for_change(&path, &changed_file.change_kind) {
                    workspace_structure_change = Some(path);
                }
            }

            // Collect changes
            let new_text = if changed_file.exists() {
                let contents = vfs.file_contents(changed_file.file_id).unwrap().to_vec();

                String::from_utf8(contents).ok().map(|text| {
                    // TODO: Consider doing normalization in the `vfs` instead to get rid of some locking?
                    let (text, line_endings) = LineEndings::normalize(text);
                    (Arc::<str>::from(text), line_endings)
                })
            } else {
                None
            };

            bytes.push((changed_file, new_text))
        }

        let mut write_guard = RwLockUpgradableReadGuard::upgrade(read_guard);
        let (vfs, line_endings_map) = &mut *write_guard;
        let change = self.collect_changes(bytes, line_endings_map, vfs, has_structure_changes);

        std::mem::drop(write_guard);

        self.analysis_host.apply_change(change);

        if let Some(path) = workspace_structure_change {
            self.fetch_workspaces_task.request(format!("workspace vfs change: {:?}", path), ());
        }

        true
    }

    fn collect_changes(
        &self,
        bytes: Vec<(ChangedFile, Option<(Arc<str>, LineEndings)>)>,
        line_ending_map: &mut IntMap<FileId, LineEndings>,
        vfs: &mut Vfs,
        has_structure_changes: bool,
    ) -> Change {
        let mut change = Change::new();
        for (changed_file, text_endings) in bytes {
            let file_id = changed_file.file_id;
            match text_endings {
                None => change.add_changed_file(changed_file, None),
                Some((text, line_endings)) => {
                    line_ending_map.insert(file_id, line_endings);
                    change.add_changed_file(changed_file, Some(text));
                }
            }
        }
        if has_structure_changes {
            let roots = self.source_root_config.partition(vfs);
            change.set_roots(roots);
        }
        change
    }

    fn colease_modifications(vfs_changes: Vec<ChangedFile>) -> Option<Vec<ChangedFile>> {
        if vfs_changes.is_empty() {
            return None;
        }

        // collapse modifications
        use vfs::vfs::ChangeKind::*;

        let mut file_changes = FxHashMap::default();
        for changed_file in vfs_changes {
            if let Occupied(mut entry) = file_changes.entry(changed_file.file_id) {
                let (change, just_created) = entry.get_mut();

                match (change, *just_created, changed_file.change_kind) {
                    (change @ Delete, _, Create) => {
                        *change = Modify(None);
                        *just_created = true;
                    }
                    (change @ _, _, Delete) => {
                        // *change = Delete;
                    }
                    (Create, _, Create) | (Create, _, Modify(_)) => {}
                    (Modify(a), _, Modify(b)) => {
                        if let (Some(a), Some(b)) = (a.as_mut(), b.as_ref()) {
                            a.extend(b.into_iter());
                        } else {
                            *a = None;
                        }
                    }
                    (Delete, _, Modify(_)) | (Modify(_), _, Create) => unreachable!(),
                }
            } else {
                let just_created = matches!(changed_file.change_kind, Create);
                file_changes.insert(changed_file.file_id, (changed_file.change_kind, just_created));
            }
        }

        let changed_file = file_changes
            .into_iter()
            .filter(|(_, (kind, just_created))| !(*kind == Delete && *just_created))
            .map(|(file_id, (change_kind, _))| ChangedFile { file_id, change_kind })
            .collect_vec();

        Some(changed_file)
    }
}
