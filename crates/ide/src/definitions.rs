use base_db::intern::Lookup;
use hir::{
    container::{ContainerId, InContainer, InModule},
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
use syntax::{SyntaxTokenWithParent, TokenKind, ast, match_ast};
use utils::{define_enum_deriving_from, get::GetRef, impl_from};

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
    #[inline]
    pub fn name(&self, db: &dyn HirDb) -> Option<Ident> {
        match *self {
            DefinitionSource::ModuleId(module_id) => db.module(module_id).name.clone(),
            DefinitionSource::BlockId(block_id) => db.block(block_id).name.clone(),
            DefinitionSource::NonAnsiPort(InModule { value, cont_id: module_id }) => {
                db.module(module_id).get(value).label.clone()
            }
            DefinitionSource::Decl(InContainer { value, cont_id }) => match cont_id {
                ContainerId::HirFileId(file_id) => db.hir_file(file_id).get(value).name.clone(),
                ContainerId::ModuleId(module_id) => db.module(module_id).get(value).name.clone(),
                ContainerId::BlockId(block_id) => db.block(block_id).get(value).name.clone(),
            },
            DefinitionSource::Instance(InModule { value, cont_id: module_id }) => {
                db.module(module_id).get(value).name.clone()
            }
            DefinitionSource::Stmt(InContainer { value, cont_id }) => match cont_id {
                ContainerId::HirFileId(file_id) => db.hir_file(file_id).get(value).label.clone(),
                ContainerId::ModuleId(module_id) => db.module(module_id).get(value).label.clone(),
                ContainerId::BlockId(block_id) => db.block(block_id).get(value).label.clone(),
            },
        }
    }

    #[inline]
    pub fn container(&self, db: &dyn HirDb) -> ContainerId {
        match *self {
            DefinitionSource::ModuleId(InContainer { cont_id, .. }) => cont_id.into(),
            DefinitionSource::BlockId(block_id) => block_id.lookup(db).cont_id,
            DefinitionSource::NonAnsiPort(InModule { cont_id, .. }) => cont_id.into(),
            DefinitionSource::Decl(InContainer { cont_id, .. }) => cont_id,
            DefinitionSource::Instance(InModule { cont_id, .. }) => cont_id.into(),
            DefinitionSource::Stmt(InContainer { cont_id, .. }) => cont_id,
        }
    }
}

// Definition may have multiple sources, e.g. non-ansi port
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Definition(SmallVec<[DefinitionSource; 3]>);

impl Definition {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DefinitionSource> {
        self.0.iter()
    }
}

impl IntoIterator for Definition {
    type IntoIter = smallvec::IntoIter<[DefinitionSource; 3]>;
    type Item = DefinitionSource;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<PathResolution> for Definition {
    fn from(path_res: PathResolution) -> Self {
        let mut res = smallvec![];
        let mut add_source = |source| res.push(source);

        match path_res {
            PathResolution::Module(module_id) => add_source(module_id.into()),
            PathResolution::Decl(decl_id) => add_source(decl_id.into()),
            PathResolution::AnsiPort(decl_id) => add_source(DefinitionSource::Decl(decl_id.into())),
            PathResolution::NonAnsiPort { label, port_decl, data_decl, module } => {
                let container = module.into();
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
            PathResolution::Instance(instance_id) => add_source(instance_id.into()),
            PathResolution::Stmt(stmt_id) => add_source(stmt_id.into()),
            PathResolution::Block(blk_id) => add_source(blk_id.into()),
        };

        Self(res)
    }
}

define_enum_deriving_from! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DefinitionClass {
        Definition,
        PortConnShorthand,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortConnShorthand {
    pub port: Definition,
    pub data: Definition,
}

impl DefinitionClass {
    pub(crate) fn resolve(
        sema: &Semantics<'_, RootDb>,
        tp @ SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
    ) -> Option<Self> {
        if !matches!(tok.kind(), TokenKind::IDENTIFIER | TokenKind::SYSTEM_IDENTIFIER) {
            return None;
        }

        let res = match_ast! { parent,
            ast::MemberAccessExpression => unimplemented!(),
            ast::ScopedName => unimplemented!(),
            ast::NamedPortConnection[it] if it.name() == Some(tok) => {
                let port = sema.resolve_port_conn_name(it).map(Definition::from).unwrap_or_default();

                let data = if it.open_paren().is_none() && it.close_paren().is_none() {
                    sema.resolve_ident_in_cont(tp).map(Definition::from).unwrap_or_default()
                } else {
                    Definition::default()
                };

                if port.is_empty() && data.is_empty() {
                    return None;
                }

                PortConnShorthand { port, data }.into()
            },
            _ => Definition::from(sema.resolve_ident_in_cont(tp)?).into(),
        };

        Some(res)
    }

    pub(crate) fn sources(self) -> SmallVec<[DefinitionSource; 6]> {
        match self {
            DefinitionClass::Definition(definition) => definition.into_iter().collect(),
            DefinitionClass::PortConnShorthand(PortConnShorthand { port, data }) => {
                port.into_iter().chain(data).collect()
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            DefinitionClass::Definition(definition) => definition.is_empty(),
            DefinitionClass::PortConnShorthand(port_conn) => {
                port_conn.port.is_empty() && port_conn.data.is_empty()
            }
        }
    }
}

impl IntoIterator for DefinitionClass {
    type IntoIter = smallvec::IntoIter<[Definition; 2]>;
    type Item = Definition;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DefinitionClass::Definition(definition) => smallvec![definition].into_iter(),
            DefinitionClass::PortConnShorthand(port_conn) => {
                smallvec![port_conn.port, port_conn.data].into_iter()
            }
        }
    }
}
