use base_db::source_db::SourceDb;
use hir::{
    db::HirDb,
    file::HirFileId,
    hir_def::{Ident, block::BlockItem, file::FileItem, lower_ident_opt, module::ModuleItem},
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use syntax::{
    SyntaxNode, SyntaxNodePreorder, SyntaxToken, WalkEvent,
    ast::{self, AstNode},
    match_ast,
};
use utils::text_edit::SourceRangeExt;
use vfs::FileId;

use crate::SymbolKind;

#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub label: String,
    pub focus_range: TextRange,
    pub full_range: TextRange,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub container_name: Option<String>,
    pub children: Option<Vec<DocumentSymbol>>,
}

pub(crate) fn document_symbols(db: &RootDb, file_id: FileId) -> Vec<DocumentSymbol> {
    let mut res = Vec::default();
    collect_file_items(db, HirFileId(file_id), &mut res);
    res
}

fn collect_file_items(db: &RootDb, file_id: HirFileId, parent: &mut Vec<DocumentSymbol>) {
    let file = db.hir_file(file_id);
    for item in file.items.iter() {
        match item {
            FileItem::LocalModuleId(idx) => todo!(),
            FileItem::ProcId(idx) => todo!(),
            FileItem::DeclarationId(idx) => todo!()
        }
    }
}

// fn collect_module_items(db: &RootDb, module_id: ModuleId, parent: &mut Vec<DocumentSymbol>) {
//     let module = db.module(module_id);
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
// fn collect_block_items(db: &RootDb, block_id: LocalBlockId, parent: &mut Vec<DocumentSymbol>) {
//     let block = db.block(block_id);
//     for item in block.items.iter() {
//         match item {
//             BlockItem::DeclarationId(idx) => todo!(),
//             BlockItem::StmtId(idx) => todo!(),
//         }
//     }
// }
