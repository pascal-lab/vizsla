pub mod bit;
pub mod block;
pub mod data;
pub mod expr;
pub mod generate;
pub mod module;
pub mod stmt;
pub mod tf;

use la_arena::{Arena, ArenaMap, Idx};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{ops::Index, sync::Arc};
use syntax::ast::ptr;
use syntax::{
    ast::{self, AstNode},
    SyntaxNodePtr,
};

use crate::InFile;

macro_rules! impl_index {
    ($datas:ident for $($tpy:ident, $fld:ident),+ $(,)? ) => {
        $(
            impl Index<Idx<$tpy>> for $datas {
                type Output = $tpy;
                fn index(&self, index: Idx<$tpy>) -> &Self::Output {
                    &self.$fld[index]
                }
            }
        )+
    };
}

pub type Ident = SmolStr;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IdentSelect {
    pub ident: Ident,
    pub select_expr: Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HierarchicalIdent {
    pub root: bool,
    pub ident_selects: Box<IdentSelect>,
}

pub type NodeId = Idx<SyntaxNodePtr>;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct HirFile {
    pub items: SmallVec<[FileItem; 1]>,
    pub data: FileData,
}

// TODO: DataDecl, InterfaceDecl
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum FileItem {
    Module(Idx<ModuleInFile>),
    // DataDecl(Idx<DataDecl>),
    // InterfaceDecl(Idx<InterfaceDecl>),
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct FileData {
    pub modules: Arena<ModuleInFile>,
    // pub data_decls: Arena<DataDecl>,
    // pub interface_decls: Arena<InterfaceDecl>,
}

impl FileData {
    pub fn shrink_to_fit(&mut self) {
        self.modules.shrink_to_fit();
        // self.data_decls.shrink_to_fit();
        // self.interface_decls.shrink_to_fit();
    }
}

impl_index! {FileData for ModuleInFile, modules}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleInFile {
    pub ident: Ident,
}

pub type ModuleSource = InFile<ptr::ModuleDeclarationPtr>;
pub type ModuleId = Idx<ModuleInFile>;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FileSourceMap {
    pub module_map: FxHashMap<ModuleSource, ModuleId>,
    pub module_map_back: ArenaMap<ModuleId, ModuleSource>,
}

pub(crate) fn hir_file_with_source_map_query(
    db: &dyn crate::db::HirDb,
    file_id: crate::HirFileId,
) -> (Arc<HirFile>, Arc<FileSourceMap>) {
    let mut hir_file = HirFile::default();
    let mut source_map = FileSourceMap::default();
    db.syntax_tree(file_id.0).and_then(|tree| {
        let root = ast::SourceFile::cast(tree.root_node())?;
        let file_text = db.file_text(file_id.0);
        let file_text = file_text.as_bytes();
        // FIXME: utf8_text panics if the identifier is not utf8

        for description in root.descriptions() {
            if let Some(module) = description.module_declaration() {
                (|| {
                    let ident = module.identifier()?.syntax().utf8_text(file_text).ok()?;
                    let ident = Ident::new(ident);
                    let ptr = module.to_ptr();
                    let module_id = hir_file.data.modules.alloc(ModuleInFile { ident });
                    hir_file.items.push(FileItem::Module(module_id));

                    let module_source = InFile::new(file_id, ptr);

                    source_map.module_map.insert(module_source.clone(), module_id);
                    source_map.module_map_back.insert(module_id, module_source);

                    Some(())
                })();
            }
        }
        Some(())
    });
    hir_file.data.shrink_to_fit();
    (Arc::new(hir_file), Arc::new(source_map))
}
