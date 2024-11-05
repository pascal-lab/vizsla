use base_db::intern::Lookup;
use hir::{
    container::{ContainerId, InContainer, InFile, InModule},
    db::HirDb,
    hir_def::{
        Ident,
        block::BlockId,
        expr::declarator::DeclId,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    semantics::{Semantics, pathres::PathResolution},
};
use ide_db::root_db::RootDb;
use smallvec::{SmallVec, smallvec};
use syntax::{SyntaxTokenWithParent, ast, match_ast, token::TokenKindExt};
use utils::{get::GetRef, impl_from};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionSource {
    ModuleId(ModuleId),
    BlockId(BlockId),

    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl_from! { DefinitionSource =>
    ModuleId,
    BlockId,
    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl DefinitionSource {
    pub fn name(&self, db: &dyn HirDb) -> Option<Ident> {
        match *self {
            DefinitionSource::ModuleId(module_id) => db.module(module_id).name.clone(),
            DefinitionSource::BlockId(block_id) => db.block(block_id).name.clone(),
            DefinitionSource::NonAnsiPort(InModule { value, module_id }) => {
                db.module(module_id).get(value).label.clone()
            }
            DefinitionSource::Decl(InContainer { value, cont_id }) => match cont_id {
                ContainerId::HirFileId(file_id) => db.hir_file(file_id).get(value).name.clone(),
                ContainerId::ModuleId(module_id) => db.module(module_id).get(value).name.clone(),
                ContainerId::BlockId(block_id) => db.block(block_id).get(value).name.clone(),
            },
            DefinitionSource::Instance(InModule { value, module_id }) => {
                db.module(module_id).get(value).name.clone()
            }
            DefinitionSource::Stmt(InContainer { value, cont_id }) => match cont_id {
                ContainerId::HirFileId(file_id) => db.hir_file(file_id).get(value).label.clone(),
                ContainerId::ModuleId(module_id) => db.module(module_id).get(value).label.clone(),
                ContainerId::BlockId(block_id) => db.block(block_id).get(value).label.clone(),
            },
        }
    }

    pub fn container(&self, db: &dyn HirDb) -> ContainerId {
        match *self {
            DefinitionSource::ModuleId(InFile { file_id, .. }) => file_id.into(),
            DefinitionSource::BlockId(block_id) => block_id.lookup(db).cont_id,
            DefinitionSource::NonAnsiPort(InModule { module_id, .. }) => module_id.into(),
            DefinitionSource::Decl(InContainer { cont_id, .. }) => cont_id,
            DefinitionSource::Instance(InModule { module_id, .. }) => module_id.into(),
            DefinitionSource::Stmt(InContainer { cont_id, .. }) => cont_id,
        }
    }
}

// Definition may have multiple sources, e.g. non-ansi port
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition(PathResolution);

impl From<PathResolution> for Definition {
    fn from(res: PathResolution) -> Self {
        Self(res)
    }
}

impl Definition {
    pub fn sources(&self) -> SmallVec<[DefinitionSource; 3]> {
        let mut res = smallvec![];
        let mut add_source = |source| res.push(source);

        match self.0 {
            PathResolution::NonAnsiPort { label, port_decl, data_decl, module } => {
                let container: ContainerId = module.into();
                if let Some(label) = label {
                    add_source(InModule::new(module, label).into());
                }
                if let Some(port_decl) = port_decl {
                    add_source(InContainer::new(container, port_decl).into());
                }
                if let Some(decl) = data_decl {
                    add_source(InContainer::new(container, decl).into());
                }
            }
            _ => add_source(self.pick()),
        };

        res
    }

    pub fn is_port(&self) -> bool {
        matches!(self.0, PathResolution::AnsiPort(_) | PathResolution::NonAnsiPort { .. })
    }

    pub fn name(&self, db: &dyn HirDb) -> Option<Ident> {
        self.pick().name(db)
    }

    pub fn container(&self, db: &dyn HirDb) -> ContainerId {
        self.pick().container(db)
    }

    fn pick(&self) -> DefinitionSource {
        match self.0 {
            PathResolution::Module(module_id) => module_id.into(),
            PathResolution::Decl(decl_id) => decl_id.into(),
            PathResolution::Instance(instance_id) => instance_id.into(),
            PathResolution::Stmt(stmt_id) => stmt_id.into(),
            PathResolution::Block(blk_id) => blk_id.into(),
            PathResolution::AnsiPort(decl_id) => {
                InContainer::new(decl_id.module_id.into(), decl_id.value).into()
            }
            PathResolution::NonAnsiPort { label, port_decl, data_decl, module } => {
                let container: ContainerId = module.into();
                if let Some(label) = label {
                    InModule::new(module, label).into()
                } else if let Some(port_decl) = port_decl {
                    InContainer::new(container, port_decl).into()
                } else if let Some(decl) = data_decl {
                    InContainer::new(container, decl).into()
                } else {
                    unreachable!(
                        "NonAnsiPort should have at least one of label, port_decl, data_decl"
                    )
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionClass {
    Definition(Definition),
    PortConnShorthand { port: Definition, data: Definition },
}

impl_from! { DefinitionClass =>
    Definition,
}

impl DefinitionClass {
    pub(crate) fn resolve(
        sema: &Semantics<'_, RootDb>,
        tp @ SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
    ) -> Option<Self> {
        if !tok.kind().name_like() {
            return None;
        }

        let res = match_ast! { parent,
            ast::MemberAccessExpression => unimplemented!(),
            ast::ScopedName => unimplemented!(),
            ast::NamedPortConnection[it] if it.name() == Some(tok) => {
                let port = sema.resolve_port_conn_name(it).map(Definition::from);

                if it.open_paren().is_none() && it.close_paren().is_none() {
                    let data = sema.resolve_ident_in_cont(tp).map(Definition::from);

                    match (port, data) {
                        (Some(port), Some(data)) => Self::PortConnShorthand { port, data },
                        (Some(it), None) | (None, Some(it)) => it.into(),
                        (None, None) => return None,
                    }
                } else {
                    port?.into()
                }
            },
            _ => Definition::from(sema.resolve_ident_in_cont(tp)?).into(),
        };

        Some(res)
    }

    pub(crate) fn sources(self) -> SmallVec<[DefinitionSource; 6]> {
        match self {
            DefinitionClass::Definition(definition) => definition.sources().into_iter().collect(),
            DefinitionClass::PortConnShorthand { port, data } => {
                port.sources().into_iter().chain(data.sources()).collect()
            }
        }
    }
}
