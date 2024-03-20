mod lower;
pub mod module_item;
pub mod port;

use crate::hir_def::{
    data::{DataSubDecl, DataSubDeclSrc, ParamDecl, ParamPortDeclSrc},
    expr::{Expr, ExprSrc},
    impl_index,
    module::{
        module_item::{HierarchicalInstance, ModuleItem, ModuleItemSrc},
        port::{AnsiPortDecl, NonAnsiPort},
    },
    //tf::TFDecl,
    Ident,
    InFile,
    SourceMap,
};
use la_arena::{Arena, Idx};
use std::ops::Index;
use syntax::ast::ptr;
use triomphe::Arc;
use utils::try_;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleDecl {
    pub ident: Ident,
    pub param_port_list: Arena<ParamDecl>,
    pub ansi_port_decls: Arena<AnsiPortDecl>,
    pub non_ansi_ports: Arena<NonAnsiPort>,
    pub module_items: Arena<ModuleItem>,
    pub data: ModuleData,
}

impl_index!(ModuleDecl for
    ParamDecl, param_port_list,
    AnsiPortDecl, ansi_port_decls,
    NonAnsiPort, non_ansi_ports,
    ModuleItem, module_items,
    Expr, data,
    DataSubDecl, data,
    HierarchicalInstance, data,
);

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleData {
    pub exprs: Arena<Expr>,
    pub data_sub_decls: Arena<DataSubDecl>,
    // TODO: pub stmts: Arena<Stmt>,
    pub hierarchical_instances: Arena<HierarchicalInstance>,
}

impl_index!(ModuleData for
    Expr, exprs,
    DataSubDecl, data_sub_decls,
    HierarchicalInstance, hierarchical_instances,
);

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleSourceMap {
    pub expr: SourceMap<ExprSrc, Expr>,
    pub data_sub_decl: SourceMap<DataSubDeclSrc, DataSubDecl>,
    pub param_port_decl: SourceMap<ParamPortDeclSrc, ParamDecl>,
    pub port: SourceMap<InFile<ptr::PortPtr>, NonAnsiPort>,
    pub ansi_port_decl: SourceMap<InFile<ptr::AnsiPortDeclarationPtr>, AnsiPortDecl>,
    pub hierarchical_instance:
        SourceMap<InFile<ptr::HierarchicalInstancePtr>, HierarchicalInstance>,
    pub module_item: SourceMap<ModuleItemSrc, ModuleItem>,
}

pub(crate) fn module_with_source_map_query(
    db: &dyn crate::db::HirDb,
    module_id: crate::hir_def::ModuleId,
) -> (Arc<ModuleDecl>, Arc<ModuleSourceMap>) {
    let (hir_file, file_source_map) = db.hir_file_with_source_map(module_id.file_id);
    let ident = hir_file.data[module_id.value].ident.clone();
    let mut module_decl = ModuleDecl {
        ident,
        param_port_list: Arena::default(),
        ansi_port_decls: Arena::default(),
        non_ansi_ports: Arena::default(),
        module_items: Arena::default(),
        data: ModuleData::default(),
    };
    let mut module_src_map = ModuleSourceMap::default();

    let module_ptr = &file_source_map.module.idx2src[module_id.value];

    try_! {
        let tree = db.hir_syntax_tree(module_id.file_id)?;
        let module_node = module_ptr.value.to_node(tree.tree())?;
        let file_text = db.hir_file_text(module_id.file_id);
        let mut ctx = lower::ModuleLowerCtx {
            hir_file_id: module_id.file_id,
            module_decl: &mut module_decl,
            module_src_map: &mut module_src_map,
            file_text: file_text.as_ref(),
        };
        ctx.lower_module_decl(&module_node);
    };

    (Arc::new(module_decl), Arc::new(module_src_map))
}
