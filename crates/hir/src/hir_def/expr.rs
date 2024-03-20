use crate::hir_def::{data::DataType, lower::Lower, Ident, InFile, SourceMap};
use la_arena::{Arena, Idx};
use smallvec::SmallVec;
use smol_str::SmolStr;
use syntax::ast::{self, ptr};

use super::literal::Literal;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LocalExprSrc {
    Expr(ptr::ExpressionPtr),
    ConstExpr(ptr::ConstantExpressionPtr),
    ParamExpr(ptr::ParamExpressionPtr),
    ConstantParamExpression(ptr::ConstantParamExpressionPtr),
}

pub type ExprSrc = InFile<LocalExprSrc>;

pub(crate) trait LowerExpr: Lower {
    fn arena_expr(&mut self) -> &mut Arena<Expr>;

    fn src_map_expr(&mut self) -> &mut SourceMap<ExprSrc, Expr>;

    fn lower_expr(&mut self, expr_node: &ast::Expression) -> Option<ExprId> {
        unimplemented!("lower_expr")
    }

    fn lower_const_expr(&mut self, expr_node: &ast::ConstantExpression) -> Option<ExprId> {
        unimplemented!("lower_const_expr")
    }

    fn lower_param_expr(&mut self, expr_node: &ast::ParamExpression) -> Option<ExprId> {
        unimplemented!("lower_param_expr")
    }

    fn lower_const_param_expr(
        &mut self,
        expr_node: &ast::ConstantParamExpression,
    ) -> Option<ExprId> {
        unimplemented!("lower_const_param_expr_src")
    }

    fn lower_const_select(
        &mut self,
        select_node: &ast::ConstantSelect,
    ) -> Option<SmallVec<[Select; 1]>> {
        unimplemented!("lower_const_select")
    }
}

// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// pub enum LocalMintypmaxExprSrc {
//     MintypmaxExpr(ptr::MintypmaxExpressionPtr),
//     ConstMintypmaxExpr(ptr::ConstantMintypmaxExpressionPtr),
// }

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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum RangeExpr {
    Indexed { is_addr: bool, index: ExprId, offset: ExprId },
    Range { lsb: ExprId, msb: ExprId },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(Box<[SmolStr]>);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Select {
    Member(Ident),
    BitSelect(ExprId),
    Range(RangeExpr),
}

pub type ExprId = Idx<Expr>;
