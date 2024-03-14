mod lower;
pub mod module_item;
pub mod port;

use crate::hir_def::{
    data::{
        DataDecl, DataSubDecl, Dimension, LocalDataSubDeclSrc, LocalParamPortDeclSrc, ParamDecl,
        PortAssignmentsList,
    },
    expr::{LocalExprSrc, LocalSelectSrc},
    module::{
        port::{AnsiPortDecl, NonAnsiPort, PortDecl},
        //module_item
    },
    tf::TFDecl,
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ModuleItem {
    PortDecl(Idx<PortDecl>),
    DataDecl(Idx<DataDecl>),
    TFDecl(Idx<TFDecl>),
    // ParamOverride(Idx<ParamOverride>),
    ModuleInstantiation(Idx<ModuleInstantiation>),
    ContinuousAssignment(Idx<ContinuousAssignment>),
    ProcessConstruct(Idx<ProcessConstruct>),
    // TODO: Add more module items
    // InterfaceInstantiation(Idx<InterfaceInstantiation>),
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleData {
    pub port_decls: Arena<PortDecl>,
    pub data_sub_decls: Arena<DataSubDecl>,
    pub data_decls: Arena<DataDecl>,
    pub tf_decls: Arena<TFDecl>,

    // pub stmts: Arena<Stmt>,

    // TODO: ParamOverride
    // pub param_overrides: Arena<ParamOverride>,
    pub module_instantiations: Arena<ModuleInstantiation>,

    pub continuous_assignments: Arena<ContinuousAssignment>,
    pub process_constructs: Arena<ProcessConstruct>,
    // TODO: generate
    // pub genvar_decls: Arena<GenvarDecl>,
    // pub generate_constructs: Arena<GenerateConstruct>,

    // TODO: interface_instantiations
    // pub interface_instantiations: Arena<InterfaceInstantiation>,
}

// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// pub struct ParamOverride {
//     pub hierarchical_ident: HierarchicalIdent,
//     pub expr: NodeId,
//     pub node_id: NodeId,
// }

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HierarchicalInstance {
    pub ident: Ident,
    pub dimensions: Option<Box<[Dimension]>>,
    pub port_list: PortAssignmentsList,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleInstantiation {
    pub module_ident: Ident,
    pub param_list: PortAssignmentsList,
    pub hierarchical_instances: Box<HierarchicalInstance>,
}

// TODO: net: [drive_strength][delay3] variable: [delay_control]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContinuousAssignment {
    // TODO: complete this
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AlwaysType {
    Always,
    AlwaysComb,
    AlwaysLatch,
    AlwaysFf,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ProcessType {
    Initial,
    Always(AlwaysType),
    Final,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ProcessConstruct {
    pub process_type: ProcessType,
    //pub stmt: NodeId,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ModuleSourceMap {
    pub expr_srcs: Arena<LocalExprSrc>,
    pub select_srcs: Arena<LocalSelectSrc>,
    pub data_sub_decls: ArenaMap<Idx<DataSubDecl>, LocalDataSubDeclSrc>,
    pub param_port_decls: ArenaMap<Idx<ParamDecl>, LocalParamPortDeclSrc>,
    pub ports: ArenaMap<Idx<NonAnsiPort>, ptr::PortPtr>,
    pub ansi_port_decls: ArenaMap<Idx<AnsiPortDecl>, ptr::AnsiPortDeclarationPtr>,
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
    let mut module_source_map = ModuleSourceMap::default();

    let module_ptr = &file_source_map.module_map_back[module_id.value];

    try_! {
        let tree = db.hir_syntax_tree(module_id.file_id)?;
        let module_node = module_ptr.value.to_node(tree.tree())?;
        let file_text = db.hir_file_text(module_id.file_id);
        let mut ctx = lower::ModuleLowerCtx {
            module_decl: &mut module_decl,
            module_source_map: &mut module_source_map,
            file_text: file_text.as_ref(),
        };
        ctx.lower_module_decl(&module_node);
    };

    (Arc::new(module_decl), Arc::new(module_source_map))
}
