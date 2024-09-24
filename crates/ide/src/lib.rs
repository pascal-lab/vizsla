#![feature(try_blocks)]

pub use base_db::Cancelled;
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
