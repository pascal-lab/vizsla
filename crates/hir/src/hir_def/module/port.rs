use crate::hir_def::{
    data::{NetDecl, VarDecl},
    expr::SelectHolder,
    Ident,
};
use la_arena::{Arena, Idx};

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
pub struct PortReference {
    pub ident: Ident,
    pub select: Idx<SelectHolder>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct NonAnsiPort {
    pub ident: Option<Ident>,
    pub port_expr: Arena<PortReference>,
}
