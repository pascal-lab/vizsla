use hir::{
    container::{InContainer, InModule},
    hir_def::{
        block::BlockId, data::SubDecl, module::module_item::HierarchicalInst, stmt::Stmt, ModuleId,
    },
    semantics::{pathres::PathResolution, Semantics},
};
use ide_db::root_db::RootDb;
use la_arena::Idx;
use smallvec::{smallvec, SmallVec};
use syntax::{
    ast::{self, AstNode},
    syntax_kind, SyntaxNode,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Definition {
    ModuleId(ModuleId),
    BlockId(BlockId),
    HierarchyInst(InModule<Idx<HierarchicalInst>>),
    SubDecl(InContainer<Idx<SubDecl>>),
    Stmt(InContainer<Idx<Stmt>>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdentClass {
    ModuleId(ModuleId),
    BlockId(BlockId),
    Port { port: Idx<SubDecl>, data: Option<Idx<SubDecl>>, module_id: ModuleId },
    HierarchyInst(InModule<Idx<HierarchicalInst>>),
    SubDecl(InContainer<Idx<SubDecl>>),
    Stmt(InContainer<Idx<Stmt>>),
}

impl IdentClass {
    pub fn classify(sema: &Semantics<'_, RootDb>, node: SyntaxNode) -> Option<IdentClass> {
        let res = match node.kind_id() {
            syntax_kind::SIMPLE_IDENTIFIER => {
                let ident = node.parent()?;
                let parent = ident.parent();
                match parent.map(|it| it.kind_id()) {
                    Some(syntax_kind::NET_LVALUE) => {
                        let parent = ast::NetLvalue::cast(parent.unwrap()).unwrap();
                        sema.resolve_path(parent.identifiers())
                    }
                    Some(syntax_kind::VARIABLE_LVALUE) => {
                        let parent = ast::VariableLvalue::cast(parent.unwrap()).unwrap();
                        sema.resolve_path(parent.identifiers())
                    }
                    Some(syntax_kind::TF_CALL) => {
                        let parent = ast::TfCall::cast(parent.unwrap()).unwrap();
                        sema.resolve_path(parent.identifiers())
                    }
                    Some(syntax_kind::PRIMARY) => {
                        let parent = ast::Primary::cast(parent.unwrap()).unwrap();
                        sema.resolve_path(parent.identifiers())
                    }
                    _ => sema.resolve_ident(&ast::Identifier::cast(ident).unwrap()),
                }
            }
            _ => return None,
        }?;
        Some(res.into())
    }

    pub fn definitions(self) -> SmallVec<[Definition; 2]> {
        match self {
            Self::ModuleId(module) => smallvec![Definition::ModuleId(module)],
            Self::BlockId(block) => smallvec![Definition::BlockId(block)],
            Self::Port { port, data, module_id } => {
                let container_id = module_id.into();
                let mut res =
                    smallvec![Definition::SubDecl(InContainer { value: port, container_id })];
                if let Some(data) = data {
                    res.push(Definition::SubDecl(InContainer { value: data, container_id }));
                }
                res
            }
            Self::HierarchyInst(inst) => smallvec![Definition::HierarchyInst(inst)],
            Self::SubDecl(sub_decl) => smallvec![Definition::SubDecl(sub_decl)],
            Self::Stmt(stmt) => smallvec![Definition::Stmt(stmt)],
        }
    }
}

impl From<PathResolution> for IdentClass {
    fn from(res: PathResolution) -> Self {
        match res {
            PathResolution::ModuleId(module) => Self::ModuleId(module),
            PathResolution::BlockId(block) => Self::BlockId(block),
            PathResolution::PortDecl { port, data, module_id } => {
                Self::Port { port, data, module_id }
            }
            PathResolution::HierarchyInst(inst) => Self::HierarchyInst(inst),
            PathResolution::SubDecl(sub_decl) => Self::SubDecl(sub_decl),
            PathResolution::Stmt(stmt) => Self::Stmt(stmt),
        }
    }
}
