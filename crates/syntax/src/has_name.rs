use slang::{
    SyntaxToken,
    ast::{
        AstNode, BlockStatement, Declarator, HierarchicalInstance, ModuleDeclaration, NonAnsiPort,
        Statement,
    },
};

pub trait HasName<'a>: AstNode<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>>;
}

impl<'a> HasName<'a> for ModuleDeclaration<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.header().name()
    }
}

impl<'a> HasName<'a> for BlockStatement<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.block_name()?.name()
    }
}

impl<'a> HasName<'a> for NonAnsiPort<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.as_explicit_non_ansi_port()?.name()
    }
}

impl<'a> HasName<'a> for Declarator<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.name()
    }
}

impl<'a> HasName<'a> for HierarchicalInstance<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.decl()?.name()
    }
}

impl<'a> HasName<'a> for Statement<'a> {
    fn name(&self) -> Option<SyntaxToken<'a>> {
        self.label()?.name()
    }
}
