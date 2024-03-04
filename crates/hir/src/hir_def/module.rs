use crate::hir_def::{
    data::{DataDecl, Dimension, NetDecl, ParamDecl, PortAssignmentsList, VarDecl},
    generate::{GenerateConstruct, GenvarDecl},
    tf::TFDecl,
    HierarchicalIdent, Ident, NodeId,
};
use la_arena::{Arena, Idx};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleDecl {
    pub ident: Ident,
    pub param_port_list: Arena<ParamDecl>,
    pub port_decls: Arena<PortDecl>,
    pub module_items: Box<ModuleItem>,
    pub data: ModuleData,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ModuleItem {
    NonAnsiPort(Idx<NonAnsiPort>),
    DataDecl(Idx<DataDecl>),
    TFDecl(Idx<TFDecl>),
    ParamOverride(Idx<ParamOverride>),
    ModuleInstantiation(Idx<ModuleInstantiation>),
    InterfaceInstantiation(Idx<InterfaceInstantiation>),
    ContinuousAssignment(Idx<ContinuousAssignment>),
    ProcessConstruct(Idx<ProcessConstruct>),
    // TODO: Add more module items
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleData {
    pub non_ansi_ports: Arena<NonAnsiPort>,

    pub data_decls: Arena<DataDecl>,
    pub tf_decls: Arena<TFDecl>,

    pub param_overrides: Arena<ParamOverride>,
    pub module_instantiations: Arena<ModuleInstantiation>,
    pub interface_instantiations: Arena<InterfaceInstantiation>,
    pub continuous_assignments: Arena<ContinuousAssignment>,
    pub process_constructs: Arena<ProcessConstruct>,
    // TODO: generate
    // pub genvar_decls: Arena<GenvarDecl>,
    // pub generate_constructs: Arena<GenerateConstruct>,
}

// TODO: ref and interface port
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PortDecl {
    IODecl(IODecl),
    // RefDecl,
    // InterfacePortDecl,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IODecl {
    pub port_type: IOType,
    pub data_decl: PortDataDecl,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum IOType {
    Input,
    Output,
    Inout,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PortDataDecl {
    NetDecl(NetDecl),
    VarDecl(VarDecl),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NonAnsiPort {
    ident: Option<Ident>,
    port_expr: Option<NodeId>,
    node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamOverride {
    pub hierarchical_ident: HierarchicalIdent,
    pub expr: NodeId,
    pub node_id: NodeId,
}

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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterfaceInstantiation {
    // TODO: complete this
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
    pub stmt: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterfaceDecl {
    // TODO: complete this
}
