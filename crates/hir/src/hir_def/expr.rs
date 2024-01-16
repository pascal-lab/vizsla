use crate::hir_def::data::DataType;
use la_arena::Idx;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    Mintypmax(ExprId, ExprId, ExprId),
    DataType(DataType),
    Dollar,
}

pub type ExprId = Idx<Expr>;
