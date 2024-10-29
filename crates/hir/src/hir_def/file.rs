use la_arena::Arena;
use smallvec::SmallVec;
use syntax::ast::{self, AstNode};
use triomphe::Arc;
use utils::define_enum_deriving_from;

use super::{
    block::{BlockInfo, BlockSrc, LocalBlockId},
    declaration::{
        Declaration, DeclarationId, DeclarationSrc, LowerDeclaration, LowerDeclarationCtx,
    },
    expr::{
        Expr, ExprId, ExprSrc, LowerExpr, LowerExprCtx,
        declarator::{DeclId, Declarator, DeclaratorSrc, LowerDecl, LowerDeclCtx},
        timing_control::{EventExpr, EventExprId, EventExprSrc, LowerEventExpr, LowerEventExprCtx},
    },
    lower_ident,
    module::{LocalModuleId, ModuleInfo, ModuleSrc},
    proc::{LowerProc, LowerProcCtx, Proc, ProcId, ProcSrc},
    stmt::{LowerStmt, LowerStmtCtx, Stmt, StmtId, StmtSrc},
};
use crate::{
    alloc_idx_and_src,
    db::{HirDb, InternDb},
    file::HirFileId,
    impl_arena_idx, impl_source_map_idx,
    source_map::SourceMap,
};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct HirFile {
    // Represente the item in order
    pub items: SmallVec<[FileItem; 3]>,

    // TODO: DataDecl, InterfaceDecl
    pub modules: Arena<ModuleInfo>,
    pub procs: Arena<Proc>,
    pub declarations: Arena<Declaration>,

    pub exprs: Arena<Expr>,
    pub event_exprs: Arena<EventExpr>,
    pub decls: Arena<Declarator>,
    pub stmts: Arena<Stmt>,
}

impl_arena_idx! { HirFile =>
    modules[ModuleInfo],
    procs[Proc],
    declarations[Declaration],

    exprs[Expr],
    event_exprs[EventExpr],
    decls[Declarator],
    stmts[Stmt],
    stmts[LocalBlockId => BlockInfo],
}

impl HirFile {
    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
        self.modules.shrink_to_fit();
        self.procs.shrink_to_fit();
        self.declarations.shrink_to_fit();
        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
        self.stmts.shrink_to_fit();
    }
}

define_enum_deriving_from! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub enum FileItem {
        LocalModuleId,
        ProcId,
        DeclarationId,
    }
}

// Definition for HirFileSourceMap
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct FileSourceMap {
    pub modules: SourceMap<ModuleSrc, ModuleInfo>,
    pub procs: SourceMap<ProcSrc, Proc>,
    pub declarations: SourceMap<DeclarationSrc, Declaration>,

    pub exprs: SourceMap<ExprSrc, Expr>,
    pub event_exprs: SourceMap<EventExprSrc, EventExpr>,
    pub decls: SourceMap<DeclaratorSrc, Declarator>,
    pub stmts: SourceMap<StmtSrc, Stmt>,
}

impl_source_map_idx! { FileSourceMap =>
    modules[ModuleSrc, LocalModuleId],
    procs[ProcSrc, ProcId],
    declarations[DeclarationSrc, DeclarationId],
    exprs[ExprSrc, ExprId],
    event_exprs[EventExprSrc, EventExprId],
    decls[DeclaratorSrc, DeclId],
    stmts[StmtSrc, StmtId],
    stmts[BlockSrc, LocalBlockId],
}

impl FileSourceMap {
    pub fn shrink_to_fit(&mut self) {
        self.modules.shrink_to_fit();
        self.procs.shrink_to_fit();
        self.declarations.shrink_to_fit();

        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
        self.stmts.shrink_to_fit();
    }
}

pub(crate) struct LowerFileCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
    pub(crate) file_id: HirFileId,

    pub(crate) file: &'a mut HirFile,
    pub(crate) file_source_map: &'a mut FileSourceMap,
}

impl LowerExpr for LowerFileCtx<'_> {
    fn expr_ctx(&mut self) -> LowerExprCtx {
        LowerExprCtx {
            db: self.db,
            exprs: &mut self.file.exprs,
            expr_source_map: &mut self.file_source_map.exprs,
        }
    }
}

impl LowerDecl for LowerFileCtx<'_> {
    fn decl_ctx(&mut self) -> LowerDeclCtx {
        LowerDeclCtx {
            db: self.db,
            decls: &mut self.file.decls,
            decl_srcs: &mut self.file_source_map.decls,

            exprs: &mut self.file.exprs,
            expr_source_map: &mut self.file_source_map.exprs,
        }
    }
}

impl LowerEventExpr for LowerFileCtx<'_> {
    fn event_expr_ctx(&mut self) -> LowerEventExprCtx {
        LowerEventExprCtx {
            db: self.db,
            event_exprs: &mut self.file.event_exprs,
            event_expr_srcs: &mut self.file_source_map.event_exprs,

            exprs: &mut self.file.exprs,
            expr_source_map: &mut self.file_source_map.exprs,
        }
    }
}

impl LowerDeclaration for LowerFileCtx<'_> {
    fn declaration_ctx(&mut self) -> LowerDeclarationCtx<'_> {
        LowerDeclarationCtx {
            db: self.db,
            declarations: &mut self.file.declarations,
            declaration_srcs: &mut self.file_source_map.declarations,

            decls: &mut self.file.decls,
            decl_srcs: &mut self.file_source_map.decls,

            event_exprs: &mut self.file.event_exprs,
            event_expr_srcs: &mut self.file_source_map.event_exprs,

            exprs: &mut self.file.exprs,
            expr_source_map: &mut self.file_source_map.exprs,
        }
    }
}

impl LowerStmt for LowerFileCtx<'_> {
    fn stmt_ctx(&mut self) -> LowerStmtCtx<'_> {
        LowerStmtCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.file_id.into(),
            stmts: &mut self.file.stmts,
            stmt_srcs: &mut self.file_source_map.stmts,

            exprs: &mut self.file.exprs,
            expr_source_map: &mut self.file_source_map.exprs,

            event_exprs: &mut self.file.event_exprs,
            event_expr_srcs: &mut self.file_source_map.event_exprs,

            decls: &mut self.file.decls,
            decl_srcs: &mut self.file_source_map.decls,
        }
    }
}

impl LowerProc for LowerFileCtx<'_> {
    fn proc_ctx(&mut self) -> LowerProcCtx<'_> {
        LowerProcCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.file_id.into(),
            procs: &mut self.file.procs,
            proc_srcs: &mut self.file_source_map.procs,

            stmts: &mut self.file.stmts,
            stmt_srcs: &mut self.file_source_map.stmts,

            exprs: &mut self.file.exprs,
            expr_srcs: &mut self.file_source_map.exprs,

            event_exprs: &mut self.file.event_exprs,
            event_expr_srcs: &mut self.file_source_map.event_exprs,

            decls: &mut self.file.decls,
            decl_srcs: &mut self.file_source_map.decls,
        }
    }
}

impl LowerFileCtx<'_> {
    pub(crate) fn lower_file(&mut self, root: ast::CompilationUnit) {
        for member in root.members().children() {
            use ast::Member::*;
            let idx = match member {
                ModuleDeclaration(decl) => {
                    let name = lower_ident(decl.header().name());

                    alloc_idx_and_src! {
                        ModuleInfo { name } => self.file.modules,
                        decl => self.file_source_map.modules,
                    }
                    .into()
                }
                ProceduralBlock(proc) => self.proc_ctx().lower_proc(proc).into(),
                DataDeclaration(data_decl) => {
                    self.declaration_ctx().lower_data_decl(data_decl).into()
                }
                NetDeclaration(net_decl) => self.declaration_ctx().lower_net_decl(net_decl).into(),
                _ => unimplemented!(),
            };
            self.file.items.push(idx);
        }
    }
}

pub(crate) fn hir_file_with_source_map_query(
    db: &dyn HirDb,
    file_id: HirFileId,
) -> (Arc<HirFile>, Arc<FileSourceMap>) {
    let mut hir_file = HirFile::default();
    let mut source_map = FileSourceMap::default();

    let tree = db.parse(file_id);
    let Some(root) = tree.root().and_then(ast::CompilationUnit::cast) else {
        return (Arc::new(hir_file), Arc::new(source_map));
    };

    let mut lower_ctx =
        LowerFileCtx { db, file_id, file: &mut hir_file, file_source_map: &mut source_map };
    lower_ctx.lower_file(root);

    hir_file.shrink_to_fit();
    source_map.shrink_to_fit();

    (Arc::new(hir_file), Arc::new(source_map))
}
