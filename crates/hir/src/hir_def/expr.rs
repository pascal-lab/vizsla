use crate::hir_def::{data::DataType, lower::Lower};
use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use syntax::ast::{
    self,
    ptr::{self, AstNodePtr},
};

use super::literal::Literal;

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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum UnaryOp {
    // `+`
    Pos,
    // `-`
    Neg,
    // `!`
    LogNeg,
    // `~`
    BitNeg,
    // `&`
    ReducAnd,
    // `~&`
    ReducNand,
    // `|`
    ReducOr,
    // `~|`
    ReducNor,
    // `^`
    ReducXor,
    // `~^`, same as `^~`
    ReducXnor,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum BinaryOp {
    // Arithmetic operators
    // `+`
    Add,
    // `-`
    Sub,
    // `*`
    Mul,
    // `/`
    Div,
    // `%`
    Mod,
    // `**`
    Pow,
    // Equality operators
    // `==`
    Eq,
    // `!=`
    Neq,
    // `===`
    CaseEq,
    // `!==`
    CaseNeq,
    // `==?`
    WildEq,
    // `!=?`
    WildNeq,
    // Relational operators
    // `>`
    Gt,
    // `>=`
    Ge,
    // `<`
    Lt,
    // `<=`
    Le,
    // Logical operators
    // `&&`
    LogAnd,
    // `||`
    LogOr,
    // Shift operators
    // `>>`
    ShiftRight,
    // `<<`
    ShiftLeft,
    // `>>>`
    ArithShiftRight,
    // `<<<`
    ArithShiftLeft,
    // Bitwise operators
    // `&`
    BitAnd,
    // `|`
    BitOr,
    // `^`
    BitXor,
    // `~^`, same as `^~`
    BitXnor,
    // TODO: implication and equivalence
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum IncDecOp {
    // `++`
    Inc,
    // `--`
    Dec,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum MinTypMaxExpr {
    MinTypMax { min: ExprId, typ: ExprId, max: ExprId },
    Expr(ExprId),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ConcatExpr {
    pub exprs: Box<[ExprId]>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    // TODO: only used in params
    // DataType(DataType),
    Unary { op: UnaryOp, expr: ExprId },
    Bin { op: BinaryOp, lhs: ExprId, rhs: ExprId },
    Cond { cond: ExprId, true_expr: ExprId, false_expr: ExprId },
    IncDec { op: IncDecOp, expr: ExprId, is_post: bool },

    // Primary
    Literal(Literal),
    TimeLiteral { value: String, unit: String },
    Concat(ConcatExpr),
    MultiConcat { expr: ExprId, count: ConcatExpr },
    Cast { data_type: DataType, expr: ExprId },
    MinTypMax(MinTypMaxExpr),
    FuncCall { callee: Path, args: Box<[ExprId]> },
    // TODO: method call chain?
    TaskCall { callee: Path, args: Box<[ExprId]> },
    // This,
    // Dollar,
    // Null,
    // TODO: add more primary expressions
}

pub enum RangeExpr {
    Indexed { is_addr: bool, index: ExprId, offset: ExprId },
    Range { lsb: ExprId, msb: ExprId },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Box<[SmolStr]>);

pub type ExprId = Idx<Expr>;
