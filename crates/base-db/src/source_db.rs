use rustc_hash::FxHashSet;
use syntax::parse::SyntaxTree;
use triomphe::Arc;
use vfs::{
    anchored_path::AnchoredPath,
    vfs::{ChangeKind, ChangedFile, FileId},
};

use crate::{
    package_graph::{PackageGraph, PackageId},
    source_root::{SourceRoot, SourceRootId},
};

pub trait FileLoader {
    fn file_text(&self, file_id: FileId) -> Arc<str>;
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId>;
    fn relevant_packages(&self, file_id: FileId) -> Arc<FxHashSet<PackageId>>;
}

// Source code, syntax tree and project model.
// Everything else is derived from these queries.
#[salsa::query_group(SourceDbStorage)]
pub trait SourceDb: FileLoader + std::fmt::Debug {
    #[salsa::input]
    fn syntax_tree(&self, file_id: FileId) -> Option<SyntaxTree>;

    #[salsa::input]
    fn package_graph(&self) -> Arc<PackageGraph>;
}

// `edits` = None => old syntax tree should be removed
pub fn edit_syntax_tree(db: &mut dyn SourceDb, changed_file: ChangedFile) {
    let file_id = changed_file.file_id;
    match changed_file.change_kind {
        ChangeKind::Modify(Some(edits)) => {
            let syntax_tree = db.syntax_tree(file_id).expect("Initial parse expected");
            let mut tree = syntax_tree.tree().clone();
            edits.iter().for_each(|edit| tree.edit(edit));
            db.set_syntax_tree(file_id, Some(SyntaxTree::new(tree)));
        }
        _ => db.set_syntax_tree(file_id, None),
    }
}

pub fn parse_source(db: &mut dyn SourceDb, file_id: FileId) {
    // TODO: make the parser static?
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_verilog::language()).unwrap();

    let old_syntax_tree = db.syntax_tree(file_id);
    let old_tree = old_syntax_tree.as_ref().map(|it| it.tree());

    let new_text = db.file_text(file_id);
    let new_tree = parser.parse(new_text.as_bytes(), old_tree).expect("tree-sitter parse error");
    let new_syntax_tree = SyntaxTree::new(new_tree);
    db.set_syntax_tree(file_id, Some(new_syntax_tree));
}

// Don't expose source roots to HIR, so extract them in a separate DB.
#[salsa::query_group(SourceRootDbStorage)]
pub trait SourceRootDb: SourceDb {
    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> Arc<str>;

    #[salsa::input]
    fn source_root_id(&self, file_id: FileId) -> SourceRootId;

    #[salsa::input]
    fn source_root(&self, id: SourceRootId) -> Arc<SourceRoot>;

    fn package_id(&self, id: SourceRootId) -> Arc<FxHashSet<PackageId>>;
}

fn package_id(db: &dyn SourceRootDb, id: SourceRootId) -> Arc<FxHashSet<PackageId>> {
    let graph = db.package_graph();
    let res = graph
        .iter()
        .filter(|&pack_id| {
            let root_file = graph[pack_id].root_file_id;
            db.source_root_id(root_file) == id
        })
        .collect();
    Arc::new(res)
}
