use hir::semantics::Semantics;
use ide_db::root_db::RootDb;

use crate::completion::{
    CompletionConfig, CompletionContext, CompletionItem,
    keywords::{directive_completions, keyword_completions, system_task_completions},
    render::render_scope_entry,
    result::dedup_by_label,
};

pub(crate) fn complete_identifier(
    db: &RootDb,
    ctx: &CompletionContext,
    config: &CompletionConfig,
) -> Vec<CompletionItem> {
    let prefix = ctx.prefix();
    let mut items = Vec::new();

    items.extend(keyword_completions(prefix, Some(ctx), config));

    if prefix.starts_with('$') {
        items.extend(system_task_completions(prefix, Some(ctx), config));
    }

    if prefix.starts_with('`') {
        items.extend(directive_completions(prefix, Some(ctx), config));
    }

    let sema = Semantics::new(db);
    let scope_completions = sema.scope_completions(ctx.position.file_id, ctx.position.offset);

    let scope_items = scope_completions
        .into_iter()
        .map(|scoped| render_scope_entry(scoped.entry, Some(scoped.scope), prefix, Some(ctx)))
        .collect::<Vec<_>>();

    items.extend(scope_items);

    dedup_by_label(items)
}
