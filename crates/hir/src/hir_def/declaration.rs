use la_arena::{Arena, Idx};
use smallvec::SmallVec;
use syntax::{TokenKind, ast};
use utils::define_enum_deriving_from;

use crate::{
    alloc_idx_and_src,
    db::InternDb,
    define_src,
    hir_def::{
        arena_nxt_idx,
        expr::{
            Expr, ExprSrc, LowerExpr, LowerExprCtx,
            data_ty::DataTy,
            declarator::{DeclId, Declarator, DeclaratorSrc, LowerDecl, LowerDeclCtx},
            timing_control::{
                DelayControl, EventExpr, EventExprSrc, LowerEventExpr, LowerEventExprCtx,
            },
        },
        ty::{
            DriveStrength, NetKind, Strength, lower_drive_strength, lower_net_kind, lower_strength,
        },
    },
    source_map::SourceMap,
};

define_enum_deriving_from! {
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Declaration {
        DataDecl,
        NetDecl,
        ParamDecl,
    }
}

pub type DeclarationId = Idx<Declaration>;
define_src!(DeclarationSrc(ast::DataDeclaration, ast::NetDeclaration, ast::ParameterDeclaration));

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataDecl {
    pub ty: DataTy,
    pub const_kw: bool,
    pub var_kw: bool,
    pub decls: SmallVec<[DeclId; 2]>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetDecl {
    pub ty: DataTy,
    pub net_kind: Option<NetKind>,
    pub delay: Option<DelayControl>,
    pub strength: Option<NetStrength>,
    pub decls: SmallVec<[DeclId; 2]>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum NetStrength {
    Pull(Strength),
    Drive(DriveStrength),
    Charge(Strength),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParamDecl {
    pub ty: DataTy,
    pub decls: SmallVec<[DeclId; 2]>,
}

pub(crate) struct LowerDeclarationCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
    pub(crate) declarations: &'a mut Arena<Declaration>,
    pub(crate) declaration_srcs: &'a mut SourceMap<DeclarationSrc, Declaration>,

    pub(crate) decls: &'a mut Arena<Declarator>,
    pub(crate) decl_srcs: &'a mut SourceMap<DeclaratorSrc, Declarator>,

    pub(crate) event_exprs: &'a mut Arena<EventExpr>,
    pub(crate) event_expr_srcs: &'a mut SourceMap<EventExprSrc, EventExpr>,

    pub(crate) exprs: &'a mut Arena<Expr>,
    pub(crate) expr_source_map: &'a mut SourceMap<ExprSrc, Expr>,
}

pub(crate) trait LowerDeclaration: LowerDecl + LowerEventExpr {
    fn declaration_ctx(&mut self) -> LowerDeclarationCtx<'_>;
}

impl LowerDecl for LowerDeclarationCtx<'_> {
    fn decl_ctx(&mut self) -> LowerDeclCtx {
        LowerDeclCtx {
            db: self.db,
            decls: self.decls,
            decl_srcs: self.decl_srcs,
            exprs: self.exprs,
            expr_source_map: self.expr_source_map,
        }
    }
}

impl LowerExpr for LowerDeclarationCtx<'_> {
    fn expr_ctx(&mut self) -> LowerExprCtx {
        LowerExprCtx { db: self.db, exprs: self.exprs, expr_source_map: self.expr_source_map }
    }
}

impl LowerEventExpr for LowerDeclarationCtx<'_> {
    fn event_expr_ctx(&mut self) -> LowerEventExprCtx {
        LowerEventExprCtx {
            db: self.db,
            event_exprs: self.event_exprs,
            event_expr_srcs: self.event_expr_srcs,
            exprs: self.exprs,
            expr_source_map: self.expr_source_map,
        }
    }
}

impl LowerDeclarationCtx<'_> {
    pub(crate) fn lower_data_decl(&mut self, data_decl: ast::DataDeclaration) -> DeclarationId {
        let mut const_kw = false;
        let mut var_kw = false;
        data_decl.modifiers().children().for_each(|tok| match tok.kind() {
            TokenKind::CONST_KEYWORD => const_kw = true,
            TokenKind::VAR_KEYWORD => var_kw = true,
            TokenKind::UNKNOWN => {}
            _ => unreachable!(),
        });

        let ty = self.expr_ctx().lower_data_ty(data_decl.type_());

        let next_declaration_idx = arena_nxt_idx(self.declarations).into();
        let decls = data_decl
            .declarators()
            .children()
            .map(|decl| self.decl_ctx().lower_declarator(decl, next_declaration_idx))
            .collect();
        alloc_idx_and_src! {
            DataDecl { ty, const_kw, var_kw, decls } => self.declarations,
            data_decl => self.declaration_srcs,
        }
    }

    pub(crate) fn lower_net_decl(&mut self, net_decl: ast::NetDeclaration) -> DeclarationId {
        let net_kind = lower_net_kind(net_decl.net_type());
        let ty = self.expr_ctx().lower_data_ty(net_decl.type_());
        let delay = net_decl.delay().map(|delay| {
            use crate::hir_def::expr::timing_control::TimingControl::*;
            match self.event_expr_ctx().lower_timing_control(delay) {
                DelayControl(delay) => delay,
                _ => unreachable!(),
            }
        });
        let next_declaration_idx = arena_nxt_idx(self.declarations).into();
        let decls = net_decl
            .declarators()
            .children()
            .map(|decl| self.decl_ctx().lower_declarator(decl, next_declaration_idx))
            .collect();
        let strength = net_decl.strength().and_then(|strength| {
            use ast::NetStrength::*;
            match strength {
                PullStrength(strength) => {
                    strength.strength().map(lower_strength).map(NetStrength::Pull)
                }
                DriveStrength(strength) => Some(NetStrength::Drive(lower_drive_strength(strength))),
                ChargeStrength(strength) => {
                    strength.strength().map(lower_strength).map(NetStrength::Charge)
                }
            }
        });
        alloc_idx_and_src! {
            NetDecl { ty, net_kind, delay, strength, decls } => self.declarations,
            net_decl => self.declaration_srcs,
        }
    }

    pub(crate) fn lower_param_decl_stmt(
        &mut self,
        param_decl: ast::ParameterDeclarationStatement,
    ) -> DeclarationId {
        use ast::ParameterDeclarationBase::*;
        match param_decl.parameter() {
            ParameterDeclaration(param_decl) => self.lower_param_decl(param_decl),
            TypeParameterDeclaration(param_decl) => unimplemented!(),
        }
    }

    fn lower_param_decl(&mut self, param_decl: ast::ParameterDeclaration) -> DeclarationId {
        let ty = self.expr_ctx().lower_data_ty(param_decl.type_());
        let next_declaration_idx = arena_nxt_idx(self.declarations).into();
        let decls = param_decl
            .declarators()
            .children()
            .map(|decl| self.decl_ctx().lower_declarator(decl, next_declaration_idx))
            .collect();
        alloc_idx_and_src! {
            ParamDecl { ty, decls } => self.declarations,
            param_decl => self.declaration_srcs,
        }
    }
}
