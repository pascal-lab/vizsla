use crate::{
    hir_def::{data::DataType, lower::Lower, Ident, InFile, SourceMap},
    try_match,
};
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
    NetLValue(ptr::NetLvaluePtr),
    VarLValue(ptr::VariableLvaluePtr),
    // TODO: NoneRangeVarLValue(ptr::NonerangeVariableLvaluePtr),
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

    fn lower_net_lvalue(&mut self, lvalue_node: &ast::NetLvalue) -> Option<ExprId> {
        unimplemented!("lower_net_lvalue")
    }

    fn lower_var_lvalue(&mut self, lvalue_node: &ast::VariableLvalue) -> Option<ExprId> {
        unimplemented!("lower_var_lvalue")
    }

    fn lower_delay_value(&mut self, delay_value_node: &ast::DelayValue) -> Option<MinTypMaxExpr> {
        unimplemented!("lower_delay_value")
    }

    fn lower_min_typ_max_expr(
        &mut self,
        min_typ_max_expr_node: &ast::MintypmaxExpression,
    ) -> Option<MinTypMaxExpr> {
        unimplemented!("lower_min_typ_max_expr")
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum AssignOp {
    // `=`
    Assign,
    // `+=`
    BinaryOpAssign(BinaryOp),
}

pub(crate) fn lower_assign_op(op: &ast::AssignmentOperator) -> Option<AssignOp> {
    try_match! {
        op.token_eq(), _ => Some(AssignOp::Assign),
        op.token_plus_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::Add)),
        op.token_minus_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::Sub)),
        op.token_star_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::Mul)),
        op.token_slash_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::Div)),
        op.token_percent_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::Mod)),
        op.token_and_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::BitAnd)),
        op.token_or_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::BitOr)),
        op.token_xor_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::BitXor)),
        op.token_lshift_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::ShiftLeft)),
        op.token_rshift_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::ShiftRight)),
        op.token_arith_lshift_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::ArithShiftLeft)),
        op.token_arith_rshift_eq(), _ => Some(AssignOp::BinaryOpAssign(BinaryOp::ArithShiftRight)),
        _ => None,
    }
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
