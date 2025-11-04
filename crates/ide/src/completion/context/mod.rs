mod analyzer;
mod expression;
mod filter;
mod token;

pub use analyzer::CompletionContextKind;
use ide_db::root_db::RootDb;
use span::FilePosition;
pub use token::CompletionToken;

use self::{
    analyzer::analyze_context, expression::is_in_expression_context, filter::should_complete,
    token::extract_completion_token,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionMode {
    Dot,
    ScopeResolution,
    SystemTask,
    Directive,
    Parameter,
    Identifier,
}

/// Context information for code completion
pub struct CompletionContext {
    pub position: FilePosition,
    pub token: CompletionToken,
    pub trigger_character: Option<char>,
    pub in_expression_context: bool,
    pub context_kind: CompletionContextKind,
}

impl CompletionContext {
    pub fn new(
        db: &RootDb,
        position: FilePosition,
        trigger_character: Option<char>,
    ) -> Option<Self> {
        use hir::semantics::Semantics;
        use syntax::ast::AstNode;

        let sema = Semantics::new(db);
        let parse = sema.parse(position.file_id);
        let root = parse.syntax();

        if !should_complete(root, position.offset) {
            return None;
        }

        let token = extract_completion_token(root, position.offset)?;
        if trigger_character == Some(':') && !token.is_scope_resolution {
            return None;
        }
        let in_expression_context = is_in_expression_context(root, position.offset);

        let context_kind = analyze_context(root, position.offset);

        Some(CompletionContext {
            position,
            token,
            trigger_character,
            in_expression_context,
            context_kind,
        })
    }

    pub fn mode(&self) -> CompletionMode {
        if self.is_dot_completion() {
            CompletionMode::Dot
        } else if self.is_scope_resolution() {
            CompletionMode::ScopeResolution
        } else if self.is_system_task_trigger() {
            CompletionMode::SystemTask
        } else if self.is_directive_trigger() {
            CompletionMode::Directive
        } else if self.is_parameter_trigger() {
            CompletionMode::Parameter
        } else {
            CompletionMode::Identifier
        }
    }

    pub fn is_dot_completion(&self) -> bool {
        self.token.is_dot_access
    }

    pub fn is_scope_resolution(&self) -> bool {
        self.token.is_scope_resolution
    }

    pub fn is_system_task_trigger(&self) -> bool {
        self.trigger_character == Some('$')
    }

    pub fn is_directive_trigger(&self) -> bool {
        self.trigger_character == Some('`')
    }

    pub fn is_parameter_trigger(&self) -> bool {
        self.trigger_character == Some('#')
    }

    pub fn prefix(&self) -> &str {
        &self.token.text
    }

    pub fn is_in_expression_context(&self) -> bool {
        self.in_expression_context
    }

    pub fn context_kind(&self) -> CompletionContextKind {
        self.context_kind
    }
}
