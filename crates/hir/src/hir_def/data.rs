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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnpackedDimension {
    Range(NodeId, NodeId),
    Expr(NodeId),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PackedDimension {
    Range(NodeId, NodeId),
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
pub struct NetDeclAssignment {
    pub ident: Ident,
    pub dimensions: Option<Box<[UnpackedDimension]>>,
    pub expr: Option<NodeId>,
    pub node_id: NodeId,
}

// Todo: [ drive_strength | charge_strength ] [ vectored | scalared ]  [ delay3 ]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NetDecl {
    pub net_type: NetType,
    pub data_type: DataType,
    pub list: Box<[NetDeclAssignment]>,
    pub node_id: NodeId,
}

// TODO: associative_dimension | queue_dimension
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum VarDimension {
    Unpacked(UnpackedDimension),
    Packed(PackedDimension),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct VarDeclAssignment {
    pub ident: Ident,
    pub dimensions: Option<Box<[VarDimension]>>,
    pub expr: Option<NodeId>,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct VarDecl {
    pub konst: bool,
    pub var: bool,
    pub data_type: DataType,
    pub list: Box<[VarDeclAssignment]>,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ParamExpression {
    Expr(NodeId),
    DataType(DataType),
    Dollar,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamAssignment {
    pub ident: Ident,
    pub dimensions: Box<[UnpackedDimension]>,
    pub param_expr: Option<ParamExpression>,
    pub node_id: NodeId,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParamDecl {
    pub local: bool,
    // 6.20.2
    pub data_type: Option<DataType>,
    pub param_assignments: Box<[ParamAssignment]>,
    pub node_id: NodeId,
}

// TODO: TypeDecl, NetTypeDecl
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DataDecl {
    NetDecl(NetDecl),
    VarDecl(VarDecl),
    ParamDecl(ParamDecl),
}
