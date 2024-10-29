use std::iter::Peekable;

use base_db::source_db::SourceDb;
use hir::{
    db::HirDb,
    file::HirFileId,
    hir_def::{
        Ident, block::BlockItem, declaration::DeclarationSrc, file::FileItem, lower_ident_opt,
        module::ModuleItem,
    },
    source_map::get_by_src,
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use syntax::{
    SyntaxNode, SyntaxNodePreorder, SyntaxToken, WalkEvent,
    ast::{self, AstNode},
    has_name::HasName,
    has_text_range::HasTextRange,
    match_ast,
};
use utils::get::{Get, GetRef};
use vfs::FileId;

use crate::SymbolKind;

#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub focus_range: TextRange,
    pub full_range: TextRange,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub container_name: Option<String>,
    pub children: Option<Vec<DocumentSymbol>>,
}

pub(crate) fn document_symbols(db: &RootDb, file_id: FileId) -> Vec<DocumentSymbol> {
    let mut res = Vec::default();

    // let tree = db.parse_src(file_id);
    // let root = ast::CompilationUnit::cast(tree.root()?)?;
    //
    // let (file, file_source_map) =
    // db.hir_file_with_source_map(HirFileId(file_id));

    // We iterate over the syntax tree, to avoid converting SyntaxNodePtr to AST
    // node, which is expensive.
    // for member in root.members().children() {
    //     use ast::Member::*;
    //     match member {
    //         ModuleDeclaration(decl) => {
    //             todo!()
    //         }
    //         ProceduralBlock(proc) => self.proc_ctx().lower_proc(proc).into(),
    //         DataDeclaration(data_decl) => {
    //             let src = DeclarationSrc::from(data_decl);
    //             let decls = get_by_src(&file, &file_source_map, src).decls();
    //             for decl in decls {
    //                 let name = decl.name().map(|name|
    // name.text().to_string()).unwrap_or_default();                 let
    // focus_range =                     decl.name().map(|name|
    // name.syntax().range()).unwrap_or_default();                 let
    // full_range = decl.syntax().range();                 let kind =
    // SymbolKind::Variable;                 let detail = None;
    //                 let container_name = None;
    //                 let children = None;
    //                 res.push(DocumentSymbol {
    //                     name,
    //                     focus_range,
    //                     full_range,
    //                     kind,
    //                     detail,
    //                     container_name,
    //                     children,
    //                 });
    //             }
    //         }
    //         NetDeclaration(_) => {}
    //         _ => unimplemented!(),
    //     };
    // }

    res
}

// fn collect_module_items(db: &RootDb, module_id: ModuleId, parent: &mut
// Vec<DocumentSymbol>) {     let module = db.module(module_id);
//     for item in module.items.values() {
//         match item {
//             ModuleItem::DeclarationId(idx) => todo!(),
//             ModuleItem::InstantiationId(idx) => todo!(),
//             ModuleItem::ProcId(idx) => todo!(),
//             ModuleItem::ContinuousAssignId(_) => {}
//         }
//     }
// }
//
// fn collect_block_items(db: &RootDb, block_id: LocalBlockId, parent: &mut
// Vec<DocumentSymbol>) {     let block = db.block(block_id);
//     for item in block.items.iter() {
//         match item {
//             BlockItem::DeclarationId(idx) => todo!(),
//             BlockItem::StmtId(idx) => todo!(),
//         }
//     }
// }

// fn build<'a>(
//     name: String,
//     node: impl HasName<'a>,
//     kind: SymbolKind,
//     container_name: Option<String>,
// ) -> Option<DocumentSymbol> {
//     DocumentSymbol {
//         name,
//         focus_range: node.name()?.text_range(),
//         full_range: node.syntax().text_range()?,
//         kind,
//         detail: None,
//         container_name,
//         children: None,
//     }
// }
