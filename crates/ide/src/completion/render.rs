use hir::completion::{CompletionEntry, CompletionEntryKind, CompletionScope};

use super::{CompletionContext, CompletionItem, CompletionItemKind, compute_score};

pub fn render_scope_entry(
    entry: CompletionEntry,
    scope: Option<CompletionScope>,
    prefix: &str,
    ctx: Option<&CompletionContext>,
) -> CompletionItem {
    let label = entry.name.to_string();
    let kind = map_completion_entry_kind(entry.kind);
    CompletionItem {
        score: compute_score(prefix, &label, kind, ctx, scope),
        label,
        label_detail: None,
        detail: entry.detail.clone(),
        insert_text: None,
        filter_text: None,
        kind,
        primary_edit: None,
        additional_edits: Vec::new(),
    }
}

fn map_completion_entry_kind(kind: CompletionEntryKind) -> CompletionItemKind {
    match kind {
        CompletionEntryKind::Module => CompletionItemKind::Module,
        CompletionEntryKind::Port => CompletionItemKind::Field,
        CompletionEntryKind::Parameter => CompletionItemKind::Variable,
        CompletionEntryKind::Variable => CompletionItemKind::Variable,
        CompletionEntryKind::Net => CompletionItemKind::Variable,
        CompletionEntryKind::Instance => CompletionItemKind::Identifier,
        CompletionEntryKind::Block => CompletionItemKind::Identifier,
        CompletionEntryKind::Function => CompletionItemKind::Function,
        CompletionEntryKind::Statement => CompletionItemKind::Identifier,
        CompletionEntryKind::Type => CompletionItemKind::Type,
        CompletionEntryKind::Import => CompletionItemKind::Identifier,
    }
}
