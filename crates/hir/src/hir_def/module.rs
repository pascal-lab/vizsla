mod lower;
pub mod module_item;
pub mod port;

use crate::hir_def::{
    data::{DataSubDecl, LocalDataSubDeclSrc, LocalParamPortDeclSrc, ParamDecl},
    expr::{LocalExprSrc, LocalSelectSrc},
    module::{
        module_item::{HierarchicalInstance, LocalModuleItemSrc, ModuleItem},
        port::{AnsiPortDecl, NonAnsiPort},
    },
    //tf::TFDecl,
    Ident,
};
use la_arena::{Arena, ArenaMap, Idx};
use smallvec::SmallVec;
use syntax::ast::ptr;
use triomphe::Arc;
use utils::try_;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleDecl {
    pub ident: Ident,
    pub param_port_list: Arena<ParamDecl>,
    pub ansi_port_decls: Arena<AnsiPortDecl>,
    pub non_ansi_ports: Arena<NonAnsiPort>,
    pub module_items: SmallVec<[ModuleItem; 1]>,
    pub data: ModuleData,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleData {
    pub data_sub_decls: Arena<DataSubDecl>,
    // TODO: pub stmts: Arena<Stmt>,
    pub hierarchical_instances: Arena<HierarchicalInstance>,
    pub module_items: Arena<ModuleItem>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleSourceMap {
    pub expr_srcs: Arena<LocalExprSrc>,
    pub select_srcs: Arena<LocalSelectSrc>,
    pub data_sub_decls: ArenaMap<Idx<DataSubDecl>, LocalDataSubDeclSrc>,
    pub param_port_decls: ArenaMap<Idx<ParamDecl>, LocalParamPortDeclSrc>,
    pub ports: ArenaMap<Idx<NonAnsiPort>, ptr::PortPtr>,
    pub ansi_port_decls: ArenaMap<Idx<AnsiPortDecl>, ptr::AnsiPortDeclarationPtr>,
    pub hierarchical_instances: ArenaMap<Idx<HierarchicalInstance>, ptr::HierarchicalInstancePtr>,
    pub module_items: ArenaMap<Idx<ModuleItem>, LocalModuleItemSrc>,
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
        module_items: SmallVec::new(),
        data: ModuleData::default(),
    };
    let mut module_src_map = ModuleSourceMap::default();

    let module_ptr = &file_source_map.module_map_back[module_id.value];

    try_! {
        let tree = db.hir_syntax_tree(module_id.file_id)?;
        let module_node = module_ptr.value.to_node(tree.tree())?;
        let file_text = db.hir_file_text(module_id.file_id);
        let mut ctx = lower::ModuleLowerCtx {
            module_decl: &mut module_decl,
            module_src_map: &mut module_src_map,
            file_text: file_text.as_ref(),
        };
        ctx.lower_module_decl(&module_node);
    };

    (Arc::new(module_decl), Arc::new(module_src_map))
}
