use la_arena::Arena;

use crate::hir_def::{expr::ExprHolder, Ident};
use la_arena::Idx;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DataType {
    IntegerType,
    NonIntegerType,
    StructUnion,
    Enum,
    String,
    // TODO: complete all the data types
}

// TODO: associative_dimension | queue_dimension | Unsized
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Dimension {
    Range(Idx<ExprHolder>, Idx<ExprHolder>),
    Expr(Idx<ExprHolder>),
    // Unsized,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NetType {
    Supply0,
    Supply1,
    Tri,
    Triand,
    Trior,
    Tri0,
    Tri1,
    Wire,
    Wand,
    Wor,
    Uwire,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DataSubDecl {
    pub ident: Ident,
    pub dimensions: Option<Box<[Dimension]>>,
    pub expr: Option<Idx<ExprHolder>>,
}

// Todo: [ drive_strength | charge_strength ] [ vectored | scalared ]  [ delay3 ]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NetDecl {
    pub net_type: NetType,
    pub data_type: DataType,
    pub sub_decls: Arena<DataSubDecl>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct VarDecl {
    pub konst: bool,
    pub data_type: DataType,
    pub sub_decls: Arena<DataSubDecl>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamDecl {
    pub local: bool,
    // 6.20.2
    pub data_type: Option<DataType>,
    pub sub_decls: Arena<DataSubDecl>,
}

// 23.3.2 Module instantiation syntax
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct OrderedPortAssignment {
    expr: Idx<ExprHolder>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NamedPortAssignment {
    ident: Ident,
    expr: Option<Idx<ExprHolder>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PortAssignmentsList {
    Ordered(Box<[OrderedPortAssignment]>),
    Named(Box<[NamedPortAssignment]>),
}

// TODO: TypeDecl, NetTypeDecl
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DataDecl {
    NetDecl(NetDecl),
    VarDecl(VarDecl),
    ParamDecl(ParamDecl),
}
