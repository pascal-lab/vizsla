#![feature(try_blocks)]
#![feature(let_chains)]

pub use base_db::Cancelled;
use syntax::{SyntaxNode, ast, match_ast};
pub type Cancellable<T> = Result<T, Cancelled>;

pub mod analysis;
pub mod analysis_host;
pub mod definitions;
pub mod navigation_target;

pub mod document_symbols;
pub mod goto_definition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolKind {
    Module,
    PortLabel,
    Decl,
    Instance,
    Block,
    Stmt,
}

impl SymbolKind {
    pub fn from_node(node: SyntaxNode) -> SymbolKind {
        match_ast! { node in
            ast::ModuleDeclaration => SymbolKind::Module,
            ast::NonAnsiPort => SymbolKind::PortLabel,
            ast::Declarator => SymbolKind::Decl,
            ast::HierarchicalInstance => SymbolKind::Instance,
            ast::BlockStatement => SymbolKind::Block,
            ast::Statement => SymbolKind::Stmt,
            _ => unreachable!("unexpected node: {:?}", node),
        }
    }
}
