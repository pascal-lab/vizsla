use std::borrow::Cow;

use ide_db::root_db::RootDb;

use super::{
    CompletionConfig, CompletionContext, CompletionContextKind, CompletionMode, CompletionResult,
    keywords::{directive_completions, system_task_completions},
    providers::{
        complete_dot_access, complete_identifier, complete_parameter_list,
        complete_port_connection, complete_scope_resolution, complete_type_reference,
    },
};

pub struct CompletionAnalysis<'a> {
    db: &'a RootDb,
    config: CompletionConfig,
}

impl<'a> CompletionAnalysis<'a> {
    pub(crate) fn new(db: &'a RootDb, config: CompletionConfig) -> Self {
        CompletionAnalysis { db, config }
    }

    pub(crate) fn analyze(&self, ctx: &CompletionContext) -> CompletionResult {
        match ctx.mode() {
            CompletionMode::Dot => {
                match ctx.context_kind() {
                    CompletionContextKind::PortConnection => {
                        return CompletionResult::from_items(complete_port_connection(
                            self.db,
                            ctx,
                            &self.config,
                        ));
                    }
                    CompletionContextKind::ParameterList => {
                        return CompletionResult::from_items(complete_parameter_list(
                            self.db,
                            ctx,
                            &self.config,
                        ));
                    }
                    _ => {}
                }
                CompletionResult::from_items(complete_dot_access(self.db, ctx, &self.config))
            }
            CompletionMode::ScopeResolution => {
                CompletionResult::from_items(complete_scope_resolution(self.db, ctx, &self.config))
            }
            CompletionMode::SystemTask => {
                CompletionResult::from_items(self.system_task_completions(ctx))
            }
            CompletionMode::Directive => {
                CompletionResult::from_items(self.directive_completions(ctx))
            }
            CompletionMode::Parameter => {
                CompletionResult::from_items(complete_parameter_list(self.db, ctx, &self.config))
            }
            CompletionMode::Identifier => self.analyze_identifier_completion(ctx),
        }
    }

    fn analyze_identifier_completion(&self, ctx: &CompletionContext) -> CompletionResult {
        if ctx.context_kind() == CompletionContextKind::PortConnection {
            return CompletionResult::from_items(complete_port_connection(
                self.db,
                ctx,
                &self.config,
            ));
        }

        if ctx.context_kind() == CompletionContextKind::ParameterList {
            return CompletionResult::from_items(complete_parameter_list(
                self.db,
                ctx,
                &self.config,
            ));
        }

        if ctx.context_kind() == CompletionContextKind::TypeReference {
            return CompletionResult::from_items(complete_type_reference(
                self.db,
                ctx,
                &self.config,
            ));
        }

        CompletionResult::from_items(complete_identifier(self.db, ctx, &self.config))
    }

    fn system_task_completions(&self, ctx: &CompletionContext) -> Vec<super::CompletionItem> {
        let prefix = ctx.prefix();

        let search_prefix: Cow<'_, str> = if prefix.is_empty() {
            Cow::Borrowed("$")
        } else if prefix.starts_with('$') {
            Cow::Borrowed(prefix)
        } else {
            Cow::Owned(format!("${}", prefix))
        };

        system_task_completions(search_prefix.as_ref(), Some(ctx), &self.config)
    }

    fn directive_completions(&self, ctx: &CompletionContext) -> Vec<super::CompletionItem> {
        let prefix = ctx.prefix();

        let search_prefix: Cow<'_, str> = if prefix.is_empty() {
            Cow::Borrowed("`")
        } else if prefix.starts_with('`') {
            Cow::Borrowed(prefix)
        } else {
            Cow::Owned(format!("`{}", prefix))
        };

        directive_completions(search_prefix.as_ref(), Some(ctx), &self.config)
    }
}
