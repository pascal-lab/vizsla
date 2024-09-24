use la_arena::Arena;
use smallvec::SmallVec;

use super::{
    Ident,
    expr::{
        Expr,
        data_ty::DataTy,
        declarator::{DeclId, Declarator},
        timing_control::EventExpr,
    },
    module::port::PortDirection,
};
use crate::db::InternDb;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Subroutine {
    pub name: Ident,
    pub sig: SubroutineSig,

    pub exprs: Arena<Expr>,
    pub event_exprs: Arena<EventExpr>,
    pub decls: Arena<Declarator>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SubroutineSig {
    Task { ports: SmallVec<[SubroutinePort; 3]> },
    Fn { ports: SmallVec<[SubroutinePort; 3]>, ret_ty: DataTy },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubroutinePort {
    dir: PortDirection,
    var_kw: bool,
    ty: DataTy,
    decl: Option<DeclId>,
}

pub struct LowerSubroutineCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
}
