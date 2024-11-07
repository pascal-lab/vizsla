use base_db::{intern::Lookup, source_db::SourceDb};
use hir::{
    container::{InContainer, InFile, InModule},
    db::HirDb,
    hir_def::{
        block::{BlockId, BlockLoc},
        expr::declarator::DeclId,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    source_map::ToAstNode,
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use smol_str::SmolStr;
use syntax::{SyntaxTokenWithParent, has_name::HasName, has_text_range::HasTextRange};
use utils::get::{Get, GetRef};
use vfs::FileId;

use crate::{SymbolKind, definitions::DefinitionOrigins};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NavTarget {
    pub file_id: FileId,
    pub full_range: TextRange,
    pub focus_range: Option<TextRange>,

    pub name: Option<SmolStr>,
    pub kind: Option<SymbolKind>,
    pub container_name: Option<SmolStr>,
    // TODO: how to represent this?
    pub description: Option<String>,
}

impl NavTarget {
    pub fn focus_or_full_range(&self) -> TextRange {
        self.focus_range.unwrap_or(self.full_range)
    }
}

pub(crate) trait ToNav {
    fn to_nav(&self, db: &RootDb) -> NavTarget;
}

impl ToNav for DefinitionOrigins {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        match self {
            DefinitionOrigins::ModuleId(module_id) => module_id.to_nav(db),
            DefinitionOrigins::BlockId(block_id) => block_id.to_nav(db),
            DefinitionOrigins::NonAnsiPort(nonansi_port_id) => nonansi_port_id.to_nav(db),
            DefinitionOrigins::Decl(decl_id) => decl_id.to_nav(db),
            DefinitionOrigins::Instance(instance_id) => instance_id.to_nav(db),
            DefinitionOrigins::Stmt(stmt_id) => stmt_id.to_nav(db),
        }
    }
}

impl ToNav for ModuleId {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InFile { value: local_module_id, file_id } = *self;
        let tree = db.parse(file_id);
        let decl_node = file_id.to_container_src_map(db).get(local_module_id).to_node(&tree).unwrap();
        let name = self.to_container(db).name.clone();

        build_nav_target(file_id.file_id(), decl_node, name, None)
    }
}

impl ToNav for BlockId {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let BlockLoc { cont_id, src: InFile { value: src, file_id } } = self.lookup(db);
        let tree = db.parse(file_id);
        let block_node = src.to_node(&tree).unwrap();

        let name = self.to_container(db).name.clone();
        let container_name = cont_id.to_container(db).name().cloned();

        build_nav_target(file_id.file_id(), block_node, name, container_name)
    }
}

impl ToNav for InModule<NonAnsiPortId> {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InModule { value: port_id, module_id } = *self;

        let file_id = module_id.file_id;
        let tree = db.parse(file_id);
        let port_node = module_id.to_container_src_map(db).get(port_id).to_node(&tree).unwrap();

        let module = db.module(module_id);
        let name = module.get(port_id).label.clone();
        let container_name = module.name.clone();
        build_nav_target(file_id.file_id(), port_node, name, container_name)
    }
}

impl ToNav for InContainer<DeclId> {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InContainer { value: decl_id, cont_id } = *self;
        let file_id = cont_id.file_id(db);
        let tree = db.parse_src(cont_id.file_id(db));

        let decl_node = cont_id.to_container_src_map(db).get(decl_id).to_node(&tree).unwrap();
        let cont = cont_id.to_container(db);
        let name = cont.get(decl_id).name.clone();
        let container_name = cont.name().cloned();

        build_nav_target(file_id, decl_node, name, container_name)
    }
}

impl ToNav for InModule<InstanceId> {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InModule { value: instance_id, module_id } = *self;
        let file_id = module_id.file_id();
        let tree = db.parse_src(file_id);

        let instance_node = module_id.to_container_src_map(db).get(instance_id).to_node(&tree).unwrap();

        let module = module_id.to_container(db);
        let name = module.get(instance_id).name.clone();
        let container_name = module.name.clone();
        build_nav_target(file_id, instance_node, name, container_name)
    }
}

impl ToNav for InContainer<StmtId> {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InContainer { value: stmt_id, cont_id } = *self;
        let file_id = cont_id.file_id(db);
        let tree = db.parse_src(file_id);

        let stmt_node = cont_id.to_container_src_map(db).get(stmt_id).to_node(&tree).unwrap();

        let cont = cont_id.to_container(db);
        let name = cont.get(stmt_id).label.clone();
        let container_name = cont.name().cloned();

        build_nav_target(file_id, stmt_node, name, container_name)
    }
}

impl ToNav for InFile<SyntaxTokenWithParent<'_>> {
    fn to_nav(&self, db: &RootDb) -> NavTarget {
        let InFile { value: SyntaxTokenWithParent { parent, tok }, file_id } = *self;
        NavTarget {
            file_id: file_id.file_id(),
            full_range: parent.text_range().unwrap(),
            focus_range: tok.text_range(),
            name: None,
            kind: Some(SymbolKind::from_node(parent)),
            container_name: None,
            description: None,
        }
    }
}

#[inline]
fn build_nav_target<'a>(
    file_id: FileId,
    node: impl HasName<'a>,
    name: Option<SmolStr>,
    container_name: Option<SmolStr>,
) -> NavTarget {
    NavTarget {
        file_id,
        full_range: node.syntax().text_range().unwrap(),
        focus_range: node.name().and_then(|name| name.text_range()),
        name,
        kind: Some(SymbolKind::from_node(node.syntax())),
        container_name,
        description: None,
    }
}
