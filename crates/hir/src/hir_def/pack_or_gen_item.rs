use la_arena::Idx;
use syntax::ast;

use crate::hir_def::{
    data::{DataDecl, LowerDataDecl},
    try_match,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PackOrGenItemDecl {
    DataDecl(Idx<DataDecl>),
    // TODO: Add more package or generate item decls
}

pub(crate) trait LowerPackOrGenItemDecl: LowerDataDecl {
    fn lower_pack_or_gen_item_decl(
        &mut self,
        item: &ast::PackageOrGenerateItemDeclaration,
    ) -> Option<PackOrGenItemDecl> {
        let item_decl = try_match! {
            item.data_declaration(), data_decl_node => {
                PackOrGenItemDecl::DataDecl(self.lower_data_decl(&data_decl_node)?)
            },
            item.net_declaration(), net_decl_node => {
                PackOrGenItemDecl::DataDecl(self.lower_net_decl(&net_decl_node)?)
            },
            item.any_parameter_declaration(), any_param_decl_node => {
                PackOrGenItemDecl::DataDecl(self.lower_any_param_decl(&any_param_decl_node)?)
            },
            item.task_declaration(), _task_decl_node => {
                unimplemented!("task_declaration");
            },
            item.function_declaration(), _func_decl_node => {
                unimplemented!("function_declaration");
            },
            item.checker_declaration(), _checker_decl_node => {
                unimplemented!("checker_declaration");
            },
            item.dpi_import_export(), _dpi_import_export_node => {
                unimplemented!("dpi_import_export");
            },
            item.extern_constraint_declaration(), _extern_constraint_decl_node => {
                unimplemented!("extern_constraint_declaration");
            },
            item.class_declaration(), _class_decl_node => {
                unimplemented!("class_declaration");
            },
            item.class_constructor_declaration(), _class_constructor_decl_node => {
                unimplemented!("class_constructor_declaration");
            },
            item.covergroup_declaration(), _covergroup_decl_node => {
                unimplemented!("covergroup_declaration");
            },
            item.assertion_item_declaration(), _assertion_item_decl_node => {
                unimplemented!("assertion_item_declaration");
            },
            _ => { return None; }
        };
        Some(item_decl)
    }
}
