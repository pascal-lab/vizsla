mod block;
mod data;
mod expr;
mod generate;
mod module;
mod stmt;
mod tf;

use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use std::sync::Arc;
use syntax::{ast::ptr, SyntaxNodePtr};

use crate::hir_def::{
    data::DataDecl,
    module::{InterfaceDecl, ModuleDecl},
};

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

pub type NodeId = Idx<SyntaxNodePtr>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct File {
    pub items: FileItems,
    pub data: FileData,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum FileItems {
    DataDecl(Idx<DataDecl>),
    ModuleDecl(Idx<ModuleDecl>),
    InterfaceDecl(Idx<InterfaceDecl>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FileData {
    idents: Arena<Ident>,
    data_decls: Arena<DataDecl>,
    module_decls: Arena<ModuleDecl>,
    interface_decls: Arena<InterfaceDecl>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NodeIdMap {}

pub(crate) fn file_items_with_source_map_query(
    db: &dyn crate::db::HirDb,
    file_id: crate::HirFileId,
) -> (Arc<FileItems>, Arc<NodeIdMap>) {
    unimplemented!()
}
