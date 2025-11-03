use rustc_hash::FxHashMap;

use super::{CompletionItem, CompletionItemKind};

pub(crate) fn dedup_by_label(items: Vec<CompletionItem>) -> Vec<CompletionItem> {
    let mut seen = FxHashMap::default();
    let mut deduped = Vec::with_capacity(items.len());

    for item in items {
        if let Some(&idx) = seen.get(&item.label) {
            if should_replace(&deduped[idx], &item) {
                deduped[idx] = item;
            }
            continue;
        }

        seen.insert(item.label.clone(), deduped.len());
        deduped.push(item);
    }

    deduped
}

pub(crate) fn should_replace(existing: &CompletionItem, candidate: &CompletionItem) -> bool {
    if candidate.score != existing.score {
        return candidate.score > existing.score;
    }

    match (&existing.detail, &candidate.detail) {
        (None, Some(_)) => return true,
        (Some(_), None) => return false,
        _ => {}
    }

    matches!(existing.kind, CompletionItemKind::Identifier | CompletionItemKind::Unknown)
        && !matches!(candidate.kind, CompletionItemKind::Identifier | CompletionItemKind::Unknown)
}
