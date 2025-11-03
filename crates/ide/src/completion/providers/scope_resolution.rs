use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use smol_str::SmolStr;

use crate::completion::{
    CompletionConfig, CompletionContext, CompletionItem, render::render_scope_entry,
};

/// Resolves the package/module and provides completions for its members.
pub(crate) fn complete_scope_resolution(
    db: &RootDb,
    ctx: &CompletionContext,
    _config: &CompletionConfig,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    if ctx.token.prefix_chain.is_empty() {
        return items;
    }

    let sema = Semantics::new(db);
    let chain: Vec<SmolStr> =
        ctx.token.prefix_chain.iter().map(|segment| SmolStr::new(segment.clone())).collect();

    let scope_entries =
        sema.scope_resolution_completions(ctx.position.file_id, &chain, ctx.prefix());
    let prefix = ctx.prefix();
    items.extend(
        scope_entries
            .into_iter()
            .map(|scoped| render_scope_entry(scoped.entry, Some(scoped.scope), prefix, Some(ctx))),
    );

    items
}
