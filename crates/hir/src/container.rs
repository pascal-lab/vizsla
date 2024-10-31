use base_db::intern::Lookup;
use utils::define_enum_deriving_from;
use vfs::FileId;

use crate::{
    db::InternDb,
    file::HirFileId,
    hir_def::{block::BlockId, module::ModuleId},
};

define_enum_deriving_from! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub enum ContainerId {
        HirFileId,
        ModuleId,
        BlockId,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InContainer<T, C = ContainerId> {
    pub value: T,
    pub cont_id: C,
}

impl<T, C> InContainer<T, C> {
    pub fn new(cont_id: C, value: T) -> InContainer<T, C> {
        InContainer { value, cont_id }
    }

    pub fn with_value<U>(self, value: U) -> InContainer<U, C> {
        InContainer::<U, C>::new(self.cont_id, value)
    }
}

macro_rules! impl_container_id {
    ($($name:ident<$id:ty>),*) => {
        $(
            pub type $name<T> = InContainer<T, $id>;

            impl<T> From<$name<T>> for InContainer<T, ContainerId> {
                fn from(item: $name<T>) -> InContainer<T, ContainerId> {
                    InContainer::new(item.cont_id.into(), item.value)
                }
            }
        )*

        impl ContainerId {
            pub fn file_id(self, db: &dyn InternDb) -> FileId {
                match self {
                    ContainerId::HirFileId(file_id) => file_id.file_id(),
                    ContainerId::ModuleId(module_id) => module_id.file_id(),
                    ContainerId::BlockId(block_id) => block_id.file_id(db),
                }
            }
        }
    };
}

impl_container_id! {
    InFile<HirFileId>,
    InModule<ModuleId>,
    InBlock<BlockId>
}

impl HirFileId {
    pub fn file_id(self) -> FileId {
        self.0
    }
}

impl ModuleId {
    pub fn file_id(self) -> FileId {
        self.cont_id.0
    }
}

impl BlockId {
    pub fn file_id(self, db: &dyn InternDb) -> FileId {
        self.lookup(db).src.cont_id.0
    }
}

/// Parents of a scope.
pub struct ContainerParent<'db> {
    db: &'db dyn InternDb,
    cont_id: Option<ContainerId>,
}

impl ContainerParent<'_> {
    pub fn start_from(db: &dyn InternDb, cont_id: ContainerId) -> ContainerParent {
        ContainerParent { db, cont_id: Some(cont_id) }
    }
}

impl Iterator for ContainerParent<'_> {
    type Item = ContainerId;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.cont_id;
        self.cont_id = match self.cont_id? {
            ContainerId::HirFileId(_) => None,
            ContainerId::ModuleId(module_id) => Some(module_id.cont_id.into()),
            ContainerId::BlockId(block_id) => Some(block_id.lookup(self.db).cont_id),
        };
        next
    }
}
