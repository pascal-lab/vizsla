use std::iter::Peekable;

use base_db::source_db::SourceDb;
use hir::{
    db::HirDb,
    file::HirFileId,
    hir_def::{
        expr::declarator::{Declarator, DeclaratorSrc},
        module::{ModuleItem, ModuleSrc},
    },
    source_map::get_by_src,
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use smol_str::SmolStr;
use syntax::{
    ast::{self, AstNode, ModuleDeclaration},
    has_name::HasName,
    has_text_range::HasTextRange,
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

    let tree = db.parse_src(file_id);
    let Some(root) = tree.root().and_then(ast::CompilationUnit::cast) else {
        return res;
    };

    let (file, file_source_map) = db.hir_file_with_source_map(HirFileId(file_id));

    // We iterate over the syntax tree, to avoid converting SyntaxNodePtr to AST
    // node, which is expensive.
    for member in root.members().children() {
        use ast::Member::*;
        match member {
            ModuleDeclaration(decl) => {
                let src = ModuleSrc::from(decl);
                let hir = get_by_src(&file, &file_source_map, src);
                // collect_module_items(db, decl, &mut res)
                todo!()
            }
            ProceduralBlock(proc) => todo!(),
            DataDeclaration(data_decl) => {
                for decl in data_decl.declarators().children() {
                    let src = DeclaratorSrc::from(decl);
                    let hir = get_by_src(&file, &file_source_map, src);
                    build_decl(hir, decl, None, &mut res);
                }
            }
            NetDeclaration(net_decl) => {
                for decl in net_decl.declarators().children() {
                    let src = DeclaratorSrc::from(decl);
                    let hir = get_by_src(&file, &file_source_map, src);
                    build_decl(hir, decl, None, &mut res);
                }
            }
            _ => unimplemented!(),
        };
    }

    res
}

// fn collect_module_items(db: &RootDb, decl: ast::ModuleDeclaration, res: &mut
// Vec<DocumentSymbol>) {
//
//     for item in module.items.values() {
//         match item {
//             ModuleItem::DeclarationId(idx) => todo!(),
//             ModuleItem::InstantiationId(idx) => todo!(),
//             ModuleItem::ProcId(idx) => todo!(),
//             ModuleItem::ContinuousAssignId(_) => {}
//         }
//     }
// }

fn build_decl(
    hir: &Declarator,
    node: ast::Declarator,
    container_name: Option<String>,
    res: &mut Vec<DocumentSymbol>,
) {
    let Some(name) = hir.name.clone() else {
        return;
    };
    if let Some(sym) = build_sym(name, node, SymbolKind::Decl, container_name) {
        res.push(sym);
    }
}

#[inline]
fn build_sym<'a>(
    name: SmolStr,
    node: impl HasName<'a>,
    kind: SymbolKind,
    container_name: Option<String>,
) -> Option<DocumentSymbol> {
    build_sym_with_detail(name, node, kind, container_name, None)
}

#[inline]
fn build_sym_with_detail<'a>(
    name: SmolStr,
    node: impl HasName<'a>,
    kind: SymbolKind,
    container_name: Option<String>,
    detail: Option<String>,
) -> Option<DocumentSymbol> {
    let sym = DocumentSymbol {
        name: name.to_string(),
        focus_range: node.name()?.text_range()?,
        full_range: node.syntax().text_range()?,
        kind,
        detail,
        container_name,
        children: None,
    };
    Some(sym)
}
