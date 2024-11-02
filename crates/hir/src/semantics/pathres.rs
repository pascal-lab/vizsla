use syntax::SyntaxTokenWithParent;

use super::SemanticsImpl;
use crate::{
    container::{ContainerId, ContainerParent, InBlock, InContainer, InFile, InModule},
    hir_def::{
        block::BlockId,
        expr::declarator::DeclId,
        lower_ident_opt,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    scope::{self, BlockEntry, ModuleEntry, UnitEntry},
};

impl SemanticsImpl<'_> {
    pub fn resolve_ident(
        &self,
        SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
    ) -> Option<PathResolution> {
        self.with_ctx(|ctx| {
            let db = self.db;
            let file_id = self.find_file(parent);
            let container = ctx.find_container(InFile::new(file_id, parent))?;
            let ident = lower_ident_opt(Some(tok))?;

            ContainerParent::start_from(db, container).find_map(|id| match id {
                ContainerId::HirFileId(_) => {
                    let scope = db.unit_scope();
                    let entry = scope.get(&ident)?;
                    Some(entry.into())
                }
                ContainerId::ModuleId(module_id) => {
                    let scope = db.module_scope(module_id);
                    let entry = scope.get(&ident)?;
                    Some(InModule::new(module_id, entry).into())
                }
                ContainerId::BlockId(block_id) => {
                    let scope = db.block_scope(block_id);
                    let entry = scope.get(&ident)?;
                    Some(InBlock::new(block_id, entry).into())
                }
            })
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PathResolution {
    Module(ModuleId),
    Decl(InContainer<DeclId>),
    Port {
        label: Option<NonAnsiPortId>,
        port_decl: Option<DeclId>,
        data_decl: Option<DeclId>,
        module: ModuleId,
    },
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
    Block(BlockId),
}

impl From<UnitEntry> for PathResolution {
    fn from(entry: UnitEntry) -> Self {
        use UnitEntry::*;
        match entry {
            ModuleId(idx) => Self::Module(idx),
            FiledDeclId(idx) => Self::Decl(idx.into()),
        }
    }
}

impl From<InModule<ModuleEntry>> for PathResolution {
    fn from(entry: InModule<ModuleEntry>) -> Self {
        use ModuleEntry::*;
        match entry.value {
            DeclId(decl_id) => Self::Decl(entry.with_value(decl_id).into()),
            InstanceId(idx) => Self::Instance(entry.with_value(idx)),
            StmtId(idx) => Self::Stmt(entry.with_value(idx).into()),
            NonAnsiPortEntry(scope::NonAnsiPortEntry { label, port_decl, data_decl }) => {
                Self::Port { label, port_decl, data_decl, module: entry.cont_id }
            }
            BlockId(block_id) => Self::Block(block_id),
        }
    }
}

impl From<InBlock<BlockEntry>> for PathResolution {
    fn from(entry: InBlock<BlockEntry>) -> Self {
        use BlockEntry::*;
        match entry.value {
            DeclId(idx) => Self::Decl(entry.with_value(idx).into()),
            StmtId(idx) => Self::Stmt(entry.with_value(idx).into()),
            BlockId(block_id) => Self::Block(block_id),
        }
    }
}
