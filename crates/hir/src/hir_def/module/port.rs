use crate::hir_def::{
    data::{NetDecl, VarDecl},
    expr::LocalSelectSrcId,
    Ident,
};
use la_arena::Arena;

// TODO: ref and interface port
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PortDecl {
    IODecl(IODecl),
    // RefDecl,
    // InterfacePortDecl,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IODecl {
    pub direction: PortDirection,
    pub data_decl: PortDataDecl,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PortDataDecl {
    NetDecl(NetDecl),
    VarDecl(VarDecl),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PortReference {
    pub ident: Ident,
    pub select: LocalSelectSrcId,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct NonAnsiPort {
    pub ident: Option<Ident>,
    pub port_expr: Arena<PortReference>,
}
