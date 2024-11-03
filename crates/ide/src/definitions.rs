use hir::{
    container::{InContainer, InModule},
    hir_def::{
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
use utils::{define_enum_deriving_from, impl_from};

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

// Definition may have multiple sources, e.g. non-ansi port
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Definition(SmallVec<[DefinitionSource; 3]>);

impl Definition {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
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

        let res = match_ast! { parent in
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
}
