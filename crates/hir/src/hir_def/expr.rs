use la_arena::Idx;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    // TODO: Add more expressions
    Mintypmax(ExprId, ExprId, ExprId),
}

pub type ExprId = Idx<Expr>;
