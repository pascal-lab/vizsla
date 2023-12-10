use crate::{
    hir_def::{
        block::Block,
        expr::{Expr, ExprId},
        generate::{GenerateConstruct, GenvarDecl},
        package_or_generate_item::{DataDecl, FunctionDecl, NetDecl, ParamDecl, TaskDecl},
        stmt::Stmt,
        Identifier, ModuleDecl,
    },
    InFile,
};
use la_arena::{Arena, Idx};

pub type ModuleId = InFile<Idx<ModuleDecl>>;

pub struct ModuleData {
    pub ident: Identifier,
    pub port_param_decls: Arena<PortParamDecl>,
    pub port_decls: Arena<PortDecl>,
    pub non_ansi_ports: Arena<NonAnsiPort>,

    // module or generate items
    pub param_overrides: Arena<ParamOverride>,
    pub module_instantiations: Arena<ModuleInstantiation>,

    // generate
    pub genvar_decls: Arena<GenvarDecl>,
    pub generate_constructs: Arena<GenerateConstruct>,

    // module_common_item
    pub interface_instantiations: Arena<InterfaceInstantiation>,
    pub continuous_assignments: Arena<ContinuousAssignment>,
    pub process_constructs: Arena<ProcessConstruct>,

    // package_or_generate_item
    pub net_decls: Arena<NetDecl>,
    pub data_decls: Arena<DataDecl>,
    pub task_decls: Arena<TaskDecl>,
    pub function_decls: Arena<FunctionDecl>,
    pub param_decls: Arena<ParamDecl>,

    // behavioral stmts
    pub stmts: Arena<Stmt>,
    pub blocks: Arena<Block>,
    pub exprs: Arena<Expr>,
    // data types
}

pub struct ModuleSourceMap {}

pub enum ConstantParamExpression {
    ConstExpr(ExprId),
    // DataType(DataTypeRef),
    Dollar,
}

pub struct PortParamDecl {
    pub local: bool,
    // pub data_type: DataTypeRef,
    pub ident: Identifier,
    pub expr: Option<ConstantParamExpression>,
}

pub struct PortDecl {}

pub struct NonAnsiPort {}

pub struct ParamOverride {}

pub struct ModuleInstantiation {}

pub struct InterfaceInstantiation {}

pub struct ContinuousAssignment {}

pub enum AlwaysType {
    Always,
    AlwaysComb,
    AlwaysLatch,
    AlwaysFf,
}

pub enum ProcessType {
    Initial,
    Always(AlwaysType),
    Final,
}

pub struct ProcessConstruct {
    pub process_type: ProcessType,
    pub stmt: Idx<Stmt>,
}
