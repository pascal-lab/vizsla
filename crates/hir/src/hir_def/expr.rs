use crate::hir_def::{data::DataType, lower::Lower};
use la_arena::{Arena, Idx};
use syntax::ast::{self, ptr};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LocalExprSrc {
    Expr(ptr::ExpressionPtr),
    ConstExpr(ptr::ConstantExpressionPtr),
    ConstantParamExpression(ptr::ConstantParamExpressionPtr),
}

pub type LocalExprSrcId = Idx<LocalExprSrc>;

pub(crate) trait LowerExprSrc: Lower {
    fn arena_expr_srcs(&mut self) -> &mut Arena<LocalExprSrc>;

    fn lower_expr_src(&mut self, expr_node: &ast::Expression) -> LocalExprSrcId {
        self.arena_expr_srcs().alloc(LocalExprSrc::Expr(expr_node.to_ptr()))
    }

    fn lower_const_expr_src(&mut self, expr_node: &ast::ConstantExpression) -> LocalExprSrcId {
        self.arena_expr_srcs().alloc(LocalExprSrc::ConstExpr(expr_node.to_ptr()))
    }

    fn lower_const_param_expr_src(
        &mut self,
        expr_node: &ast::ConstantParamExpression,
    ) -> LocalExprSrcId {
        self.arena_expr_srcs().alloc(LocalExprSrc::ConstantParamExpression(expr_node.to_ptr()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LocalMintypmaxExprSrc {
    MintypmaxExpr(ptr::MintypmaxExpressionPtr),
    ConstMintypmaxExpr(ptr::ConstantMintypmaxExpressionPtr),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LocalSelectSrc {
    Select(ptr::SelectPtr),
    ConstSelect(ptr::ConstantSelectPtr),
}

pub type LocalSelectSrcId = Idx<LocalSelectSrc>;

pub(crate) trait LowerSelectSrc: Lower {
    fn arena_select_srcs(&mut self) -> &mut Arena<LocalSelectSrc>;

    fn lower_select_src(&mut self, select_node: &ast::Select) -> LocalSelectSrcId {
        self.arena_select_srcs().alloc(LocalSelectSrc::Select(select_node.to_ptr()))
    }

    fn lower_const_select_src(&mut self, select_node: &ast::ConstantSelect) -> LocalSelectSrcId {
        self.arena_select_srcs().alloc(LocalSelectSrc::ConstSelect(select_node.to_ptr()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    DataType(DataType),
    This,
    Dollar,
    Null,
    // primary
    NumberLiteral(),
}

pub type ExprId = Idx<Expr>;
