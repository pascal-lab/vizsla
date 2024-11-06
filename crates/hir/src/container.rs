use base_db::intern::Lookup;
use proc_macro_utils::impl_container;
use smol_str::SmolStr;
use triomphe::Arc;
use utils::define_enum_deriving_from;
use vfs::FileId;

use crate::{
    db::{HirDb, InternDb},
    file::HirFileId,
    hir_def::{
        block::{Block, BlockId, BlockInfo, BlockSourceMap, BlockSrc, LocalBlockId},
        declaration::{Declaration, DeclarationId, DeclarationSrc},
        expr::{
            Expr, ExprId, ExprSrc,
            declarator::{DeclId, Declarator, DeclaratorSrc},
            timing_control::{EventExpr, EventExprId, EventExprSrc},
        },
        file::{FileSourceMap, HirFile},
        module::{Module, ModuleId, ModuleSourceMap},
        stmt::{Stmt, StmtId, StmtSrc},
    },
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

macro_rules! define_container_id {
    ($($name:ident[$id:ident : $ty:ty]),* $(,)?) => {
        $(
            #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
            pub struct $name<T> {
                pub value: T,
                pub $id: $ty,
            }

            impl<T> $name<T> {
                pub fn new($id: $ty, value: T) -> Self {
                    Self { value, $id }
                }

                pub fn with_value<U>(self, value: U) -> $name<U> {
                    $name::<U>::new(self.$id, value)
                }
            }

            impl<T> From<$name<T>> for InContainer<T, ContainerId> {
                fn from(item: $name<T>) -> InContainer<T, ContainerId> {
                    InContainer::new(item.$id.into(), item.value)
                }
            }
        )*
    };
}

define_container_id! {
    InFile[file_id: HirFileId],
    InModule[module_id: ModuleId],
    InBlock[block_id: BlockId],
}

impl ContainerId {
    pub fn file_id(self, db: &dyn InternDb) -> FileId {
        match self {
            ContainerId::HirFileId(file_id) => file_id.file_id(),
            ContainerId::ModuleId(module_id) => module_id.file_id(),
            ContainerId::BlockId(block_id) => block_id.file_id(db),
        }
    }

    pub fn to_container(self, db: &dyn HirDb) -> Container {
        match self {
            ContainerId::HirFileId(file_id) => file_id.to_container(db).into(),
            ContainerId::ModuleId(module_id) => module_id.to_container(db).into(),
            ContainerId::BlockId(block_id) => block_id.to_container(db).into(),
        }
    }

    pub fn to_container_src_map(self, db: &dyn HirDb) -> ContainerSrcMap {
        match self {
            ContainerId::HirFileId(file_id) => file_id.to_container_src_map(db).into(),
            ContainerId::ModuleId(module_id) => module_id.to_container_src_map(db).into(),
            ContainerId::BlockId(block_id) => block_id.to_container_src_map(db).into(),
        }
    }
}

impl HirFileId {
    pub fn file_id(&self) -> FileId {
        self.0
    }

    #[inline]
    pub fn to_container(&self, db: &dyn HirDb) -> Arc<HirFile> {
        db.hir_file(*self)
    }

    #[inline]
    pub fn to_container_src_map(&self, db: &dyn HirDb) -> Arc<FileSourceMap> {
        db.hir_file_with_source_map(*self).1
    }
}

impl ModuleId {
    pub fn file_id(&self) -> FileId {
        self.file_id.0
    }

    #[inline]
    pub fn to_container(&self, db: &dyn HirDb) -> Arc<Module> {
        db.module(*self)
    }

    #[inline]
    pub fn to_container_src_map(&self, db: &dyn HirDb) -> Arc<ModuleSourceMap> {
        db.module_with_source_map(*self).1
    }
}

impl BlockId {
    pub fn file_id(&self, db: &dyn InternDb) -> FileId {
        self.lookup(db).src.file_id.0
    }

    #[inline]
    pub fn to_container(&self, db: &dyn HirDb) -> Arc<Block> {
        db.block(*self)
    }

    #[inline]
    pub fn to_container_src_map(&self, db: &dyn HirDb) -> Arc<BlockSourceMap> {
        db.block_with_source_map(*self).1
    }
}

impl_container! {
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum {
        HirFile | FileSourceMap,
        Module | ModuleSourceMap,
        Block | BlockSourceMap,
    } => {
        Declaration[DeclarationId | DeclarationSrc],
        Expr[ExprId | ExprSrc],
        EventExpr[EventExprId | EventExprSrc],
        Declarator[DeclId | DeclaratorSrc],
        Stmt[StmtId | StmtSrc],
        BlockInfo[LocalBlockId | BlockSrc],
    }
}

impl Container {
    #[inline]
    pub fn name(&self) -> Option<&SmolStr> {
        match self {
            Container::HirFile(_) => None,
            Container::Module(module) => module.name.as_ref(),
            Container::Block(block) => block.name.as_ref(),
        }
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
            ContainerId::ModuleId(module_id) => Some(module_id.file_id.into()),
            ContainerId::BlockId(block_id) => Some(block_id.lookup(self.db).cont_id),
        };
        next
    }
}
