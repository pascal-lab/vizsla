use base_db::intern::Lookup;
use utils::impl_from;

use crate::{
    db::HirDb,
    file::HirFileId,
    hir_def::{
        block::{BlockId, BlockLoc},
        ModuleId,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ContainerId {
    // [`HirFileId`] is a special container.
    HirFileId(HirFileId),
    ModuleId(ModuleId),
    BlockId(BlockId),
}

impl_from!(HirFileId, ModuleId, BlockId for ContainerId);

impl ContainerId {
    pub fn file_id(&self, db: &dyn HirDb) -> HirFileId {
        match self {
            ContainerId::HirFileId(file_id) => *file_id,
            ContainerId::ModuleId(module_id) => module_id.container_id,
            ContainerId::BlockId(block_id) => block_id.lookup(db).block_src.container_id,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InContainer<T, C = ContainerId> {
    pub value: T,
    pub container_id: C,
}

impl<T, C> InContainer<T, C> {
    pub fn new(container_id: C, value: T) -> InContainer<T, C> {
        InContainer { value, container_id }
    }

    pub fn with_value<U>(self, value: U) -> InContainer<U, C> {
        InContainer::<U, C>::new(self.container_id, value)
    }
}

impl<T> From<InFile<T>> for InContainer<T, ContainerId> {
    fn from(file: InFile<T>) -> InContainer<T, ContainerId> {
        InContainer::new(file.container_id.into(), file.value)
    }
}

impl<T> From<InModule<T>> for InContainer<T, ContainerId> {
    fn from(module: InModule<T>) -> InContainer<T, ContainerId> {
        InContainer::new(module.container_id.into(), module.value)
    }
}

impl<T> From<InBlock<T>> for InContainer<T, ContainerId> {
    fn from(block: InBlock<T>) -> InContainer<T, ContainerId> {
        InContainer::new(block.container_id.into(), block.value)
    }
}

pub type InFile<T> = InContainer<T, HirFileId>;
pub type InModule<T> = InContainer<T, ModuleId>;
pub type InBlock<T> = InContainer<T, BlockId>;

/// Parents of a scope.
pub struct ContainerParent<'db> {
    db: &'db dyn HirDb,
    container_id: Option<ContainerId>,
}

impl<'db> ContainerParent<'db> {
    pub fn new(db: &'db dyn HirDb, container_id: ContainerId) -> ContainerParent {
        ContainerParent { db, container_id: Some(container_id) }
    }
}

impl<'db> Iterator for ContainerParent<'db> {
    type Item = ContainerId;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.container_id;
        match self.container_id {
            Some(ContainerId::ModuleId(module_id)) => {
                self.container_id = Some(ContainerId::HirFileId(module_id.container_id));
            }
            Some(ContainerId::BlockId(block_id)) => {
                let BlockLoc { container_id, .. } = block_id.lookup(self.db);
                self.container_id = Some(container_id);
            }
            _ => {
                self.container_id = None;
            }
        }
        next
    }
}
