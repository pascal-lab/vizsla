use la_arena::{Arena, Idx};
use syntax::ast;

use crate::{
    alloc_idx_and_src,
    container::ContainerId,
    db::InternDb,
    define_src,
    file::HirFileId,
    hir_def::{
        expr::{
            Expr, ExprSrc, LowerExpr, LowerExprCtx,
            declarator::{Declarator, DeclaratorSrc, LowerDecl, LowerDeclCtx},
            timing_control::{EventExpr, EventExprSrc, LowerEventExpr, LowerEventExprCtx},
        },
        stmt::{LowerStmt, LowerStmtCtx, Stmt, StmtId, StmtSrc},
    },
    source_map::SourceMap,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AlwaysKeyword {
    Always,
    AlwaysComb,
    AlwaysLatch,
    AlwaysFf,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ProcType {
    #[default]
    Initial,

    Always(AlwaysKeyword),
    Final,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Proc {
    pub proc_ty: ProcType,
    pub stmt: StmtId,
}

pub type ProcId = Idx<Proc>;

define_src!(ProcSrc(ast::ProceduralBlock));

pub(crate) trait LowerProc: LowerStmt {
    fn proc_ctx(&mut self) -> LowerProcCtx;
}

pub(crate) struct LowerProcCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
    pub(crate) file_id: HirFileId,
    pub(crate) cont_id: ContainerId,
    pub(crate) procs: &'a mut Arena<Proc>,
    pub(crate) proc_srcs: &'a mut SourceMap<ProcSrc, Proc>,

    pub(crate) stmts: &'a mut Arena<Stmt>,
    pub(crate) stmt_srcs: &'a mut SourceMap<StmtSrc, Stmt>,

    pub(crate) event_exprs: &'a mut Arena<EventExpr>,
    pub(crate) event_expr_srcs: &'a mut SourceMap<EventExprSrc, EventExpr>,

    pub(crate) exprs: &'a mut Arena<Expr>,
    pub(crate) expr_srcs: &'a mut SourceMap<ExprSrc, Expr>,

    pub(crate) decls: &'a mut Arena<Declarator>,
    pub(crate) decl_srcs: &'a mut SourceMap<DeclaratorSrc, Declarator>,
}

impl LowerDecl for LowerProcCtx<'_> {
    fn decl_ctx(&mut self) -> LowerDeclCtx {
        LowerDeclCtx {
            db: self.db,
            decls: self.decls,
            decl_srcs: self.decl_srcs,
            exprs: self.exprs,
            expr_source_map: self.expr_srcs,
        }
    }
}

impl LowerStmt for LowerProcCtx<'_> {
    fn stmt_ctx(&mut self) -> LowerStmtCtx {
        LowerStmtCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.cont_id,
            stmts: self.stmts,
            stmt_srcs: self.stmt_srcs,
            event_exprs: self.event_exprs,
            event_expr_srcs: self.event_expr_srcs,
            exprs: self.exprs,
            expr_source_map: self.expr_srcs,
            decls: self.decls,
            decl_srcs: self.decl_srcs,
        }
    }
}

impl LowerEventExpr for LowerProcCtx<'_> {
    fn event_expr_ctx(&mut self) -> LowerEventExprCtx {
        LowerEventExprCtx {
            db: self.db,
            event_exprs: self.event_exprs,
            event_expr_srcs: self.event_expr_srcs,
            exprs: self.exprs,
            expr_source_map: self.expr_srcs,
        }
    }
}

impl LowerExpr for LowerProcCtx<'_> {
    fn expr_ctx(&mut self) -> LowerExprCtx {
        LowerExprCtx { db: self.db, exprs: self.exprs, expr_source_map: self.expr_srcs }
    }
}

impl LowerProcCtx<'_> {
    pub(crate) fn lower_proc(&mut self, proc: ast::ProceduralBlock) -> ProcId {
        use ast::ProceduralBlock::*;
        let proc_ty = match proc {
            AlwaysFFBlock(_) => ProcType::Always(AlwaysKeyword::AlwaysFf),
            AlwaysBlock(_) => ProcType::Always(AlwaysKeyword::Always),
            AlwaysCombBlock(_) => ProcType::Always(AlwaysKeyword::AlwaysComb),
            AlwaysLatchBlock(_) => ProcType::Always(AlwaysKeyword::AlwaysLatch),
            InitialBlock(_) => ProcType::Initial,
            FinalBlock(_) => ProcType::Final,
        };

        let stmt = self.stmt_ctx().lower_stmt(proc.statement());

        alloc_idx_and_src! {
            Proc { proc_ty, stmt } => self.procs,
            proc => self.proc_srcs,
        }
    }
}
