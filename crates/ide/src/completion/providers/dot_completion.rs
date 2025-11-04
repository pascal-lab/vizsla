use hir::{
    completion::{CompletionScope, DotFieldKind},
    semantics::Semantics,
};
use ide_db::root_db::RootDb;
use smol_str::SmolStr;

use crate::completion::{
    CompletionConfig, CompletionContext, CompletionItem, CompletionItemKind, compute_score,
};

/// Provides completions for struct/class' members (fields, methods, etc.)
pub(crate) fn complete_dot_access(
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
        ctx.token.prefix_chain.iter().map(|s| SmolStr::new(s.clone())).collect();

    let fields =
        sema.dot_completions(ctx.position.file_id, ctx.position.offset, &chain, ctx.prefix());

    for field in fields {
        let kind = match field.kind {
            DotFieldKind::Field => CompletionItemKind::Field,
            DotFieldKind::Method => CompletionItemKind::Function,
        };

        let label = field.name.to_string();
        items.push(CompletionItem {
            score: compute_score(
                ctx.prefix(),
                &label,
                kind,
                Some(ctx),
                Some(CompletionScope::Class),
            ),
            label,
            label_detail: None,
            detail: field.detail.clone(),
            insert_text: None,
            filter_text: None,
            kind,
            primary_edit: None,
            additional_edits: Vec::new(),
        });
    }

    items
}
