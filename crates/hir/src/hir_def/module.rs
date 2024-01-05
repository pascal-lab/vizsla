use crate::hir_def::{
    block::Block,
    data::{DataDecl, ParamDecl},
    generate::{GenerateConstruct, GenvarDecl},
    stmt::Stmt,
    tf::TFDecl,
    Ident, NodeId,
};
use la_arena::{Arena, Idx};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleDecl {
    pub ident: Ident,
    pub port_param_decls: Arena<ParamDecl>,
    pub non_ansi_ports: Arena<NonAnsiPort>,
    pub port_decls: Arena<PortDecl>,
    pub module_items: Box<ModuleItems>,
    pub data: ModuleData,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ModuleItems {
    DataDecl(Idx<DataDecl>),
    TFDecl(Idx<TFDecl>),
    ParamOverride(Idx<ParamOverride>),
    // TODO: Add more module items
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterfaceDecl {
    pub ident: Ident,
    // TODO: complete this
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleData {
    pub data_decls: Arena<DataDecl>,
    pub blocks: Arena<Block>,
    pub tf_decls: Arena<TFDecl>,

    // module or generate items
    pub param_overrides: Arena<ParamOverride>,
    pub module_instantiations: Arena<ModuleInstantiation>,

    // module_common_item
    pub interface_instantiations: Arena<InterfaceInstantiation>,
    pub continuous_assignments: Arena<ContinuousAssignment>,
    pub process_constructs: Arena<ProcessConstruct>,

    // data types

    // generate
    pub genvar_decls: Arena<GenvarDecl>,
    pub generate_constructs: Arena<GenerateConstruct>,
}

// TODO: complete the following content
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PortParamDecl {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PortDecl {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NonAnsiPort {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamOverride {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleInstantiation {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterfaceInstantiation {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContinuousAssignment {}

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
    pub stmt: Idx<Stmt>,
}
