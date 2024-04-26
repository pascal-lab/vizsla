mod lower;
pub mod module_item;
pub mod port;

use crate::{
    db::HirDb,
    file::InFile,
    hir_def::{
        control::EventExpr,
        data::{DataDecl, DataDeclSrc, DataSubDecl, DataSubDeclSrc},
        expr::{Expr, ExprSrc},
        //tf::TFDecl,
        impl_index,
        module::{
            module_item::{HierarchicalInst, Inst, ModuleItem, ModuleItemSrc},
            port::{AnsiPortDecl, NonAnsiPort, PortDecl},
        },
        stmt::{Stmt, StmtSrc},
        Ident,
        SourceMap,
    },
};
use la_arena::{Arena, Idx, IdxRange};
use std::ops::Index;
use syntax::ast::ptr;
use triomphe::Arc;
use utils::try_;

use super::{
    block::{block_src::BlockSrc, BlockInfo},
    ModuleId,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Module {
    pub ident: Ident,
    pub param_port_list: Option<IdxRange<DataDecl>>,
    pub ansi_port_decls: Arena<AnsiPortDecl>,
    pub non_ansi_ports: Arena<NonAnsiPort>,
    pub module_items: Arena<ModuleItem>,
    pub data: ModuleData,
}

impl_index!(Module for
    AnsiPortDecl, ansi_port_decls,
    NonAnsiPort, non_ansi_ports,
    ModuleItem, module_items,
    Expr, data,
    EventExpr, data,
    DataSubDecl, data,
    DataDecl, data,
    PortDecl, data,
    Stmt, data,
    BlockInfo, data,
    HierarchicalInst, data,
    Inst, data,
);

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleData {
    pub exprs: Arena<Expr>,
    pub event_exprs: Arena<EventExpr>,
    pub data_sub_decls: Arena<DataSubDecl>,
    pub data_decls: Arena<DataDecl>,
    pub port_decls: Arena<PortDecl>,
    pub stmts: Arena<Stmt>,
    pub block_infos: Arena<BlockInfo>,
    pub hierarchical_insts: Arena<HierarchicalInst>,
    pub insts: Arena<Inst>,
}

impl_index!(ModuleData for
    Expr, exprs,
    EventExpr, event_exprs,
    DataSubDecl, data_sub_decls,
    DataDecl, data_decls,
    PortDecl, port_decls,
    Stmt, stmts,
    BlockInfo, block_infos,
    HierarchicalInst, hierarchical_insts,
    Inst, insts,
);

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleSourceMap {
    pub non_ansi_port: SourceMap<InFile<ptr::PortPtr>, NonAnsiPort>,
    pub ansi_port_decl: SourceMap<InFile<ptr::AnsiPortDeclarationPtr>, AnsiPortDecl>,
    pub module_item: SourceMap<ModuleItemSrc, ModuleItem>,
    pub expr: SourceMap<ExprSrc, Expr>,
    pub event_expr: SourceMap<InFile<ptr::EventExpressionPtr>, EventExpr>,
    pub data_sub_decl: SourceMap<DataSubDeclSrc, DataSubDecl>,
    pub data_decl: SourceMap<DataDeclSrc, DataDecl>,
    pub port_decl: SourceMap<InFile<ptr::PortDeclarationPtr>, PortDecl>,
    pub stmt: SourceMap<StmtSrc, Stmt>,
    pub block: SourceMap<BlockSrc, BlockInfo>,
    pub hierarchical_inst: SourceMap<InFile<ptr::HierarchicalInstancePtr>, HierarchicalInst>,
    pub inst: SourceMap<InFile<ptr::InstantiationPtr>, Inst>,
}

pub(crate) fn module_with_source_map_query(
    db: &dyn HirDb,
    module_id: ModuleId,
) -> (Arc<Module>, Arc<ModuleSourceMap>) {
    let (hir_file, file_source_map) = db.hir_file_with_source_map(module_id.file_id);
    let ident = hir_file.data[module_id.value].ident.clone();
    let mut module_decl = Module {
        ident,
        param_port_list: None,
        ansi_port_decls: Arena::default(),
        non_ansi_ports: Arena::default(),
        module_items: Arena::default(),
        data: ModuleData::default(),
    };
    let mut module_src_map = ModuleSourceMap::default();

    try_! {
        let module_ptr = file_source_map.modules.get_src(module_id.value)?;
        let tree = db.hir_syntax_tree(module_id.file_id)?;
        let module_node = module_ptr.value.to_node(tree.tree())?;
        let file_text = db.hir_file_text(module_id.file_id);
        let mut ctx = lower::ModuleLowerCtx {
            db,
            module_id,
            module_decl: &mut module_decl,
            module_src_map: &mut module_src_map,
            file_text: file_text.as_ref(),
        };
        ctx.lower_module_decl(&module_node);
    };

    (Arc::new(module_decl), Arc::new(module_src_map))
}
