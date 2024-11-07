use base_db::intern::Lookup;
use hir::{
    container::{ContainerId, InContainer, InFile, InModule},
    db::HirDb,
    hir_def::{
        Ident,
        block::{BlockId, BlockLoc},
        expr::declarator::DeclId,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    semantics::{Semantics, pathres::PathResolution},
    source_map::IsSrc,
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use smallvec::{SmallVec, smallvec};
use smol_str::SmolStr;
use syntax::{SyntaxTokenWithParent, ast, match_ast, token::TokenKindExt};
use utils::{
    get::{Get, GetRef},
    impl_from,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionOrigins {
    ModuleId(ModuleId),
    BlockId(BlockId),

    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl_from! { DefinitionOrigins =>
    ModuleId,
    BlockId,
    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl DefinitionOrigins {
    #[inline]
    pub fn container_id(&self, db: &dyn HirDb) -> ContainerId {
        match *self {
            DefinitionOrigins::ModuleId(InFile { file_id, .. }) => file_id.into(),
            DefinitionOrigins::BlockId(block_id) => block_id.lookup(db).cont_id,
            DefinitionOrigins::NonAnsiPort(InModule { module_id, .. }) => module_id.into(),
            DefinitionOrigins::Decl(InContainer { cont_id, .. }) => cont_id,
            DefinitionOrigins::Instance(InModule { module_id, .. }) => module_id.into(),
            DefinitionOrigins::Stmt(InContainer { cont_id, .. }) => cont_id,
        }
    }

    #[inline]
    pub fn name(&self, db: &dyn HirDb) -> InFile<(SmolStr, TextRange)> {
        match *self {
            DefinitionOrigins::ModuleId(InFile { value, file_id }) => {
                let name = file_id.to_container(db).get(value).name.clone().unwrap();
                let range = file_id.to_container_src_map(db).get(value).range();
                InFile { value: (name, range), file_id }
            }
            DefinitionOrigins::BlockId(block_id) => {
                let BlockLoc { cont_id, src: InFile { value, file_id } } = block_id.lookup(db);
                let cont = cont_id.to_container(db);
                let cont_src_map = cont_id.to_container_src_map(db);
                let name = value.hir(&cont, &cont_src_map).name.clone().unwrap();
                InFile::new(file_id, (name, value.range()))
            }
            DefinitionOrigins::NonAnsiPort(InModule { value, module_id }) => {
                let cont = module_id.to_container(db);
                let name = cont.get(value).label.clone().unwrap();

                let cont_src_map = module_id.to_container_src_map(db);
                let src = cont_src_map.get(value);

                InFile::new(module_id.file_id, (name, src.range()))
            }
            DefinitionOrigins::Decl(InContainer { value, cont_id }) => {
                let cont = cont_id.to_container(db);
                let name = cont.get(value).name.clone().unwrap();

                let cont_src_map = cont_id.to_container_src_map(db);
                let src = cont_src_map.get(value);

                InFile::new(cont_id.file_id(db).into(), (name, src.range()))
            }
            DefinitionOrigins::Instance(InModule { value, module_id }) => {
                let cont = module_id.to_container(db);
                let name = cont.get(value).name.clone().unwrap();

                let cont_src_map = module_id.to_container_src_map(db);
                let src = cont_src_map.get(value);

                InFile::new(module_id.file_id, (name, src.range()))
            }
            DefinitionOrigins::Stmt(InContainer { value, cont_id }) => {
                let cont = cont_id.to_container(db);
                let name = cont.get(value).label.clone().unwrap();

                let cont_src_map = cont_id.to_container_src_map(db);
                let src = cont_src_map.get(value);

                InFile::new(cont_id.file_id(db).into(), (name, src.range()))
            }
        }
    }
}

// Definition may have multiple origins, e.g. non-ansi port
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition(PathResolution);

impl From<PathResolution> for Definition {
    fn from(res: PathResolution) -> Self {
        Self(res)
    }
}

impl Definition {
    pub fn origins(&self) -> SmallVec<[DefinitionOrigins; 3]> {
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

    pub fn container_id(&self, db: &dyn HirDb) -> ContainerId {
        let container_id = self.pick().container_id(db);
        debug_assert! {
            self.origins().into_iter().all(|source| source.container_id(db) == container_id)
        };
        container_id
    }

    #[inline]
    fn pick(&self) -> DefinitionOrigins {
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

    pub(crate) fn origins(self) -> SmallVec<[DefinitionOrigins; 6]> {
        match self {
            DefinitionClass::Definition(definition) => definition.origins().into_iter().collect(),
            DefinitionClass::PortConnShorthand { port, data } => {
                port.origins().into_iter().chain(data.origins()).collect()
            }
        }
    }
}
