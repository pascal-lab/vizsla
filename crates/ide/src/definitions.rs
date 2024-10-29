use hir::{
    container::{InContainer, InModule},
    hir_def::{
        block::BlockId,
        expr::declarator::DeclId,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        stmt::StmtId,
    },
    semantics::{Semantics, pathres::PathResolution},
};
use ide_db::root_db::RootDb;
use smallvec::{SmallVec, smallvec};
use syntax::{SyntaxTokenWithParent, TokenKind, ast, match_ast};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Definition {
    ModuleId(ModuleId),
    BlockId(BlockId),

    NonAnsiPort(InModule<NonAnsiPortId>),
    Decl(InContainer<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
}

impl Definition {
    pub fn resolution(
        sema: &Semantics<'_, RootDb>,
        token_par @ SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
    ) -> Option<SmallVec<[Definition; 3]>> {
        if !matches!(tok.kind(), TokenKind::IDENTIFIER | TokenKind::SYSTEM_IDENTIFIER) {
            return None;
        }

        let res = match_ast! { parent in
            ast::MemberAccessExpression as _ => unimplemented!(),
            ast::ScopedName as _ => unimplemented!(),
            _ => sema.resolve_ident(token_par),
        }?;

        let ans = match res {
            PathResolution::Module(module_id) => smallvec![Self::ModuleId(module_id)],
            PathResolution::Decl(decl_id) => smallvec![Self::Decl(decl_id)],
            PathResolution::Port { label, port_decl, data_decl: decl, module } => {
                let mut defs = SmallVec::new();
                let container = module.into();
                if let Some(label) = label {
                    defs.push(Self::NonAnsiPort(InModule::new(module, label)));
                }
                if let Some(port_decl) = port_decl {
                    defs.push(Self::Decl(InContainer::new(container, port_decl)));
                }
                if let Some(decl) = decl {
                    defs.push(Self::Decl(InContainer::new(container, decl)));
                }
                defs
            }
            PathResolution::Instance(instance_id) => smallvec![Self::Instance(instance_id)],
            PathResolution::Stmt(stmt_id) => smallvec![Self::Stmt(stmt_id)],
            PathResolution::Block(blk_id) => smallvec![Self::BlockId(blk_id)],
        };

        Some(ans)
    }
}
