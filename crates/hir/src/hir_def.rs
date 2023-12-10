mod block;
mod expr;
mod generate;
mod module;
mod package_or_generate_item;
mod stmt;

use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use std::ops::Index;
use std::sync::Arc;
use syntax::ast::ptr;

use crate::hir_def::package_or_generate_item::{
    DataDecl, FunctionDecl, NetDecl, ParamDecl, TaskDecl,
};

pub type Identifier = SmolStr;

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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FileItems {
    module_decls: Arena<ModuleDecl>,
    interface_decls: Arena<InterfaceDecl>,

    // package_or_generate_items
    net_decls: Arena<NetDecl>,
    data_decls: Arena<DataDecl>,
    task_decls: Arena<TaskDecl>,
    function_decls: Arena<FunctionDecl>,
    param_decls: Arena<ParamDecl>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FileSourceMap {
    module_decls: Arena<ptr::ModuleDeclarationPtr>,
    interface_decls: Arena<ptr::InterfaceDeclarationPtr>,

    // package_or_generate_item
    net_decls: Arena<ptr::NetDeclarationPtr>,
    data_decls: Arena<ptr::DataDeclarationPtr>,
    task_decls: Arena<ptr::TaskDeclarationPtr>,
    function_decls: Arena<ptr::FunctionDeclarationPtr>,
    param_decls: Arena<ptr::ParameterDeclarationPtr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleDecl {
    pub ident: Identifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterfaceDecl {
    pub ident: Identifier,
}

pub(crate) fn file_items_with_source_map_query(
    db: &dyn crate::db::HirDb,
    file_id: crate::HirFileId,
) -> (Arc<FileItems>, Arc<FileSourceMap>) {
    unimplemented!()
}
