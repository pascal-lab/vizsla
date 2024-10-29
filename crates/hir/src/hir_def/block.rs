use base_db::intern::Lookup;
use la_arena::Arena;
use smallvec::SmallVec;
use syntax::{
    TokenKind,
    ast::{self, AstNode},
    match_ast,
};
use triomphe::Arc;
use utils::{
    define_enum_deriving_from,
    get::{Get, GetRef},
};

use super::{
    Ident,
    declaration::{
        Declaration, DeclarationId, DeclarationSrc, LowerDeclaration, LowerDeclarationCtx,
    },
    expr::{
        Expr, ExprId, ExprSrc, LowerExpr, LowerExprCtx,
        declarator::{DeclId, Declarator, DeclaratorSrc, LowerDecl, LowerDeclCtx},
        timing_control::{EventExpr, EventExprId, EventExprSrc, LowerEventExpr, LowerEventExprCtx},
    },
    stmt::{LowerStmt, LowerStmtCtx, Stmt, StmtId, StmtKind, StmtSrc},
};
use crate::{
    container::{ContainerId, InFile},
    db::{HirDb, InternDb},
    define_src,
    file::HirFileId,
    hir_def::lower_ident_opt,
    impl_arena_idx, impl_source_map_idx,
    source_map::{SourceMap, ToAstNode},
};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub name: Option<Ident>,
    pub kind: BlockKind,
    pub items: SmallVec<[BlockItem; 2]>,

    pub declarations: Arena<Declaration>,
    pub stmts: Arena<Stmt>,
    pub exprs: Arena<Expr>,
    pub event_exprs: Arena<EventExpr>,
    pub decls: Arena<Declarator>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub enum BlockKind {
    #[default]
    Sequential,
    Parallel(ParBlockKind),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ParBlockKind {
    Join,
    JoinAny,
    JoinNone,
}

define_src!(BlockSrc(ast::BlockStatement));

impl From<BlockSrc> for StmtSrc {
    fn from(src: BlockSrc) -> Self {
        StmtSrc::new(src.0)
    }
}

impl Get<LocalBlockId> for SourceMap<StmtSrc, Stmt> {
    type Output = BlockSrc;

    fn get_opt(&self, block_id: LocalBlockId) -> Option<Self::Output> {
        let stmt_id = block_id.0;
        Some(BlockSrc(self.get(stmt_id).ptr()))
    }
}

impl Get<BlockSrc> for SourceMap<StmtSrc, Stmt> {
    type Output = LocalBlockId;

    fn get_opt(&self, block_src: BlockSrc) -> Option<Self::Output> {
        let src: StmtSrc = block_src.into();
        Some(LocalBlockId(self.get(src)))
    }
}

impl GetRef<LocalBlockId> for Arena<Stmt> {
    type Output = BlockInfo;

    fn get_opt(&self, block_id: LocalBlockId) -> Option<&Self::Output> {
        let stmt_id = block_id.0;
        let Stmt { kind: StmtKind::Block(block_info), .. } = &self[stmt_id] else {
            unreachable!();
        };
        Some(block_info)
    }
}

impl_arena_idx! { Block =>
    declarations[Declaration],
    stmts[Stmt],
    stmts[LocalBlockId => BlockInfo],
    exprs[Expr],
    event_exprs[EventExpr],
    decls[Declarator],
}

define_enum_deriving_from! {
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum BlockItem {
        DeclarationId,
        StmtId,
    }
}

impl Block {
    pub fn shrink_to_fit(&mut self) {
        self.declarations.shrink_to_fit();
        self.stmts.shrink_to_fit();
        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockInfo {
    pub name: Option<Ident>,
    pub block_id: BlockId,
}

pub struct LocalBlockId(StmtId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BlockId(pub salsa::InternId);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BlockLoc {
    pub cont_id: ContainerId,
    pub src: InFile<BlockSrc>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct BlockSourceMap {
    pub declarations: SourceMap<DeclarationSrc, Declaration>,
    pub stmts: SourceMap<StmtSrc, Stmt>,
    pub exprs: SourceMap<ExprSrc, Expr>,
    pub event_exprs: SourceMap<EventExprSrc, EventExpr>,
    pub decls: SourceMap<DeclaratorSrc, Declarator>,
}

impl_source_map_idx! { BlockSourceMap =>
    declarations[DeclarationSrc, DeclarationId],
    stmts[StmtSrc, StmtId],
    stmts[BlockSrc, LocalBlockId],
    exprs[ExprSrc, ExprId],
    event_exprs[EventExprSrc, EventExprId],
    decls[DeclaratorSrc, DeclId],
}

impl BlockSourceMap {
    pub fn shrink_to_fit(&mut self) {
        self.declarations.shrink_to_fit();
        self.stmts.shrink_to_fit();
        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
    }
}

pub(crate) struct LowerBlockCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
    pub(crate) file_id: HirFileId,
    pub(crate) block_id: BlockId,

    pub(crate) block: &'a mut Block,
    pub(crate) block_source_map: &'a mut BlockSourceMap,
}

impl LowerExpr for LowerBlockCtx<'_> {
    fn expr_ctx(&mut self) -> LowerExprCtx {
        LowerExprCtx {
            db: self.db,
            exprs: &mut self.block.exprs,
            expr_source_map: &mut self.block_source_map.exprs,
        }
    }
}

impl LowerDecl for LowerBlockCtx<'_> {
    fn decl_ctx(&mut self) -> LowerDeclCtx {
        LowerDeclCtx {
            db: self.db,
            decls: &mut self.block.decls,
            decl_srcs: &mut self.block_source_map.decls,

            exprs: &mut self.block.exprs,
            expr_source_map: &mut self.block_source_map.exprs,
        }
    }
}

impl LowerEventExpr for LowerBlockCtx<'_> {
    fn event_expr_ctx(&mut self) -> LowerEventExprCtx {
        LowerEventExprCtx {
            db: self.db,
            event_exprs: &mut self.block.event_exprs,
            event_expr_srcs: &mut self.block_source_map.event_exprs,

            exprs: &mut self.block.exprs,
            expr_source_map: &mut self.block_source_map.exprs,
        }
    }
}

impl LowerStmt for LowerBlockCtx<'_> {
    fn stmt_ctx(&mut self) -> LowerStmtCtx<'_> {
        LowerStmtCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.block_id.into(),
            stmts: &mut self.block.stmts,
            stmt_srcs: &mut self.block_source_map.stmts,

            exprs: &mut self.block.exprs,
            expr_source_map: &mut self.block_source_map.exprs,

            event_exprs: &mut self.block.event_exprs,
            event_expr_srcs: &mut self.block_source_map.event_exprs,

            decls: &mut self.block.decls,
            decl_srcs: &mut self.block_source_map.decls,
        }
    }
}

impl LowerDeclaration for LowerBlockCtx<'_> {
    fn declaration_ctx(&mut self) -> LowerDeclarationCtx<'_> {
        LowerDeclarationCtx {
            db: self.db,
            declarations: &mut self.block.declarations,
            declaration_srcs: &mut self.block_source_map.declarations,

            decls: &mut self.block.decls,
            decl_srcs: &mut self.block_source_map.decls,

            event_exprs: &mut self.block.event_exprs,
            event_expr_srcs: &mut self.block_source_map.event_exprs,

            exprs: &mut self.block.exprs,
            expr_source_map: &mut self.block_source_map.exprs,
        }
    }
}

impl LowerBlockCtx<'_> {
    pub(crate) fn lower_block(&mut self, block: ast::BlockStatement) {
        // TODO: label? end_block_name?
        self.block.name = block.block_name().and_then(|name| lower_ident_opt(name.name()));
        self.block.kind = match block.end().map(|end| end.kind()) {
            Some(TokenKind::JOIN_KEYWORD) => BlockKind::Parallel(ParBlockKind::Join),
            Some(TokenKind::JOIN_ANY_KEYWORD) => BlockKind::Parallel(ParBlockKind::JoinAny),
            Some(TokenKind::JOIN_NONE_KEYWORD) => BlockKind::Parallel(ParBlockKind::JoinNone),
            _ => BlockKind::Sequential, // Some(TokenKind::END_KEYWORD) | None | Others
        };
        self.block.items = block
            .items()
            .children()
            .map(|node| {
                let node = node.syntax();
                match_ast! { node in
                    ast::Statement as it => self.stmt_ctx().lower_stmt(it).into(),
                    ast::DataDeclaration as it => self.declaration_ctx().lower_data_decl(it).into(),
                    _ => unimplemented!("{:?}", node.kind()),
                }
            })
            .collect();
    }
}

pub(crate) fn block_with_source_map_query(
    db: &dyn HirDb,
    block_id: BlockId,
) -> (Arc<Block>, Arc<BlockSourceMap>) {
    let InFile { cont_id: file_id, value: block_src } = block_id.lookup(db).src;
    let tree = db.parse(file_id);

    let mut block = Block::default();
    let mut block_source_map = BlockSourceMap::default();
    let Some(ast_block) = block_src.to_node(&tree) else {
        return (Arc::new(block), Arc::new(block_source_map));
    };

    let mut lower_ctx = LowerBlockCtx {
        db,
        file_id,
        block_id,
        block: &mut block,
        block_source_map: &mut block_source_map,
    };
    lower_ctx.lower_block(ast_block);

    block.shrink_to_fit();
    block_source_map.shrink_to_fit();
    (Arc::new(block), Arc::new(block_source_map))
}
