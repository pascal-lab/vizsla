use hir::{
    container::{InContainer, InModule},
    hir_def::{
        block::BlockId,
        expr::declarator::DeclId,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    semantics::pathres::PathResolution,
};
use smallvec::{SmallVec, smallvec};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Definition {
    ModuleId(ModuleId),
    BlockId(BlockId),

    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl Definition {
    pub(crate) fn from_pathres(res: PathResolution) -> SmallVec<[Definition; 3]> {
        match res {
            PathResolution::Module(module_id) => smallvec![Definition::ModuleId(module_id)],
            PathResolution::Decl(decl_id) => smallvec![Definition::Decl(decl_id)],
            PathResolution::Port { label, port_decl, data_decl, module } => {
                let mut defs = SmallVec::new();
                let container = module.into();
                if let Some(label) = label {
                    defs.push(Definition::NonAnsiPort(InModule::new(module, label)));
                }
                if let Some(port_decl) = port_decl {
                    defs.push(Definition::Decl(InContainer::new(container, port_decl)));
                }
                if let Some(decl) = data_decl {
                    defs.push(Definition::Decl(InContainer::new(container, decl)));
                }
                defs
            }
            PathResolution::Instance(instance_id) => smallvec![Definition::Instance(instance_id)],
            PathResolution::Stmt(stmt_id) => smallvec![Definition::Stmt(stmt_id)],
            PathResolution::Block(blk_id) => smallvec![Definition::BlockId(blk_id)],
        }
    }
}
