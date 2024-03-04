use crate::hir_def::data::DataType;
use la_arena::Idx;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    // TODO: Mintypmax(ExprId, ExprId, ExprId),
    DataType(DataType),
    This,
    Dollar,
    Null,
    // primary
    NumberLiteral(),
}

pub type ExprId = Idx<Expr>;
