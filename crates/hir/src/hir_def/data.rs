use crate::hir_def::{Ident, NodeId};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DataType {
    IntegerType,
    NonIntegerType,
    StructUnion,
    Enum,
    String,
    // TODO: complete all the data types
}

// TODO: associative_dimension | queue_dimension
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Dimension {
    Range(NodeId, NodeId),
    Expr(NodeId),
    Unsized,
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
pub struct DataDeclAssignment {
    pub ident: Ident,
    pub dimensions: Option<Box<[Dimension]>>,
    pub expr: Option<NodeId>,
}

// Todo: [ drive_strength | charge_strength ] [ vectored | scalared ]  [ delay3 ]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NetDecl {
    pub net_type: NetType,
    pub data_type: DataType,
    pub list: Box<[DataDeclAssignment]>,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct VarDecl {
    pub konst: bool,
    pub var: bool,
    pub data_type: DataType,
    pub list: Box<[DataDeclAssignment]>,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamDecl {
    pub local: bool,
    // 6.20.2
    pub data_type: Option<DataType>,
    pub list: Box<[DataDeclAssignment]>,
    pub node_id: NodeId,
}

// TODO: TypeDecl, NetTypeDecl
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DataDecl {
    NetDecl(NetDecl),
    VarDecl(VarDecl),
    ParamDecl(ParamDecl),
}
