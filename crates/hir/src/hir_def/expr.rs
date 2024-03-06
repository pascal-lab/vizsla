use crate::hir_def::data::DataType;
use la_arena::Idx;
use syntax::ast::ptr;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ExprHolder {
    Expr(ptr::ExpressionPtr),
    ConstExpr(ptr::ConstantExpressionPtr),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum MintypmaxExprHolder {
    MintypmaxExpr(ptr::MintypmaxExpressionPtr),
    ConstMintypmaxExpr(ptr::ConstantMintypmaxExpressionPtr),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum SelectHolder {
    Select(ptr::SelectPtr),
    ConstSelect(ptr::ConstantSelectPtr),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    // TODO:
    DataType(DataType),
    This,
    Dollar,
    Null,
    // primary
    NumberLiteral(),
}

pub type ExprId = Idx<Expr>;
