#![feature(try_blocks)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(trait_upcasting)]

pub use base_db::Cancelled;
use syntax::{SyntaxNode, ast, match_ast};
pub type Cancellable<T> = Result<T, Cancelled>;

pub mod analysis;
pub mod analysis_host;
pub mod definitions;
pub mod navigation_target;
pub mod source_change;

pub mod document_highlight;
pub mod document_symbols;
pub mod goto_definition;
pub mod references;
pub mod rename;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolKind {
    Module,
    PortLabel,
    Decl,
    Instance,
    Block,
    Stmt,
    Fn,
    Generate,
    Interface,
}

impl SymbolKind {
    pub fn from_node(node: SyntaxNode) -> SymbolKind {
        match_ast! { node,
            ast::ModuleHeader[it] => {
                use ast::ModuleHeader::*;
                match it {
                    ModuleHeader(_) => SymbolKind::Module,
                    InterfaceHeader(_) => SymbolKind::Interface,
                    _ => unimplemented!(),
                }
            },
            ast::ModuleDeclaration[it] => {
                use ast::ModuleDeclaration::*;
                match it {
                    ModuleDeclaration(_) => SymbolKind::Module,
                    InterfaceDeclaration(_) => SymbolKind::Interface,
                    _ => unimplemented!(),
                }
            },
            ast::NonAnsiPort => SymbolKind::PortLabel,
            ast::Declarator => SymbolKind::Decl,
            ast::HierarchicalInstance => SymbolKind::Instance,
            ast::BlockStatement => SymbolKind::Block,
            ast::Statement => SymbolKind::Stmt,
            ast::FunctionDeclaration => SymbolKind::Fn,
            ast::GenerateBlock => SymbolKind::Generate,
            _ => unreachable!("unexpected node: {:?}", node),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeVisibility {
    Public,
    Private,
}
