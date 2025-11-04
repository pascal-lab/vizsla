pub mod analysis;
pub mod context;
mod keywords;
mod providers;
mod render;
mod result;

pub use analysis::CompletionAnalysis;
pub use context::{CompletionContext, CompletionContextKind, CompletionMode, CompletionToken};
use hir::completion::CompletionScope;
use ide_db::root_db::RootDb;
use span::FilePosition;
use utils::text_edit::TextEdit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionItemKind {
    Keyword,
    Snippet,
    Module,
    Type,
    Function,
    Variable,
    Field,
    Identifier,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionConfig {
    pub enable_snippets: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionItem {
    pub label: String,
    pub label_detail: Option<String>,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub kind: CompletionItemKind,
    pub score: i32,
    pub primary_edit: Option<TextEdit>,
    pub additional_edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub is_incomplete: bool,
}

impl CompletionResult {
    pub fn empty() -> Self {
        Self { items: Vec::new(), is_incomplete: false }
    }

    pub fn from_items(items: Vec<CompletionItem>) -> Self {
        let mut items: Vec<_> = items.into_iter().filter(|item| item.score > -500).collect();

        items.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.label.cmp(&b.label))
                .then_with(|| a.detail.cmp(&b.detail))
        });

        Self { items, is_incomplete: false }
    }
}

pub(crate) fn completions(
    db: &RootDb,
    position: FilePosition,
    config: CompletionConfig,
    trigger_character: Option<char>,
) -> CompletionResult {
    let ctx_result = CompletionContext::new(db, position, trigger_character);

    let Some(ctx) = ctx_result else {
        return CompletionResult::empty();
    };

    let analysis = CompletionAnalysis::new(db, config);

    analysis.analyze(&ctx)
}

pub(crate) fn compute_score(
    prefix: &str,
    label: &str,
    kind: CompletionItemKind,
    ctx: Option<&CompletionContext>,
    scope: Option<CompletionScope>,
) -> i32 {
    let trimmed_prefix = prefix.trim_start_matches(['.', '#']);
    let prefix_lower = trimmed_prefix.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();

    let mut score = match_quality_score(trimmed_prefix, &prefix_lower, label, &label_lower);
    score += base_kind_bias(kind);
    score += context_bias(kind, ctx);
    score += scope_bias(scope);

    score
}

fn match_quality_score(
    trimmed_prefix: &str,
    prefix_lower: &str,
    label: &str,
    label_lower: &str,
) -> i32 {
    let prefix_len = trimmed_prefix.chars().count() as i32;
    let label_len = label.chars().count() as i32;

    if trimmed_prefix.is_empty() {
        return 300 - label_len * 5;
    }

    let mut score = 200;

    if label == trimmed_prefix {
        score += 5000;
    } else if label_lower == prefix_lower {
        score += 4200;
    }

    if label.starts_with(trimmed_prefix) {
        score += 2000;
    } else if label_lower.starts_with(prefix_lower) {
        score += 1600;
    }

    if camel_or_snake_prefix(prefix_lower, label) {
        score += 1200;
    }

    if label_lower.contains(prefix_lower) && !label_lower.starts_with(prefix_lower) {
        score += 600;
    } else if prefix_lower.len() >= 2 && is_subsequence(prefix_lower, label_lower) {
        score += 400;
    }

    let remainder = (label_len - prefix_len).max(0);
    score -= remainder * 6;
    score -= label_len * 2;

    score
}

fn camel_or_snake_prefix(prefix_lower: &str, label: &str) -> bool {
    if prefix_lower.is_empty() {
        return false;
    }

    let mut acronym = String::new();
    let mut prev_is_separator = true;
    let mut prev_is_lower = false;

    for ch in label.chars() {
        if ch == '_' || ch == '-' {
            prev_is_separator = true;
            prev_is_lower = false;
            continue;
        }

        let is_new_segment = prev_is_separator || (prev_is_lower && ch.is_ascii_uppercase());
        if is_new_segment {
            acronym.push(ch.to_ascii_lowercase());
        }

        prev_is_separator = false;
        prev_is_lower = ch.is_ascii_lowercase();
    }

    if acronym.is_empty() {
        return false;
    }

    acronym.starts_with(prefix_lower)
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for ch in haystack.chars() {
        if let Some(target) = current {
            if ch == target {
                current = needle_chars.next();
                if current.is_none() {
                    return true;
                }
            }
        } else {
            break;
        }
    }

    current.is_none()
}

fn base_kind_bias(kind: CompletionItemKind) -> i32 {
    match kind {
        CompletionItemKind::Variable | CompletionItemKind::Field => 500,
        CompletionItemKind::Function => 350,
        CompletionItemKind::Type => 280,
        CompletionItemKind::Module => 200,
        CompletionItemKind::Keyword => 150,
        CompletionItemKind::Snippet => -250,
        CompletionItemKind::Identifier => 50,
        CompletionItemKind::Unknown => 0,
    }
}

fn context_bias(kind: CompletionItemKind, ctx: Option<&CompletionContext>) -> i32 {
    let Some(ctx) = ctx else { return 0 };
    match ctx.context_kind() {
        CompletionContextKind::Expression => match kind {
            CompletionItemKind::Variable | CompletionItemKind::Field => 700,
            CompletionItemKind::Function => 400,
            CompletionItemKind::Snippet => -200,
            CompletionItemKind::Type | CompletionItemKind::Module => -350,
            CompletionItemKind::Keyword => -150,
            _ => 0,
        },
        CompletionContextKind::TypeReference => match kind {
            CompletionItemKind::Type => 800,
            CompletionItemKind::Keyword => 250,
            CompletionItemKind::Module => -150,
            CompletionItemKind::Variable | CompletionItemKind::Field => -450,
            CompletionItemKind::Function => -400,
            CompletionItemKind::Snippet => -200,
            _ => 0,
        },
        CompletionContextKind::Instantiation => match kind {
            CompletionItemKind::Module => 650,
            CompletionItemKind::Snippet => -250,
            _ => -150,
        },
        CompletionContextKind::PortConnection => match kind {
            CompletionItemKind::Field | CompletionItemKind::Variable => 500,
            CompletionItemKind::Function => -150,
            CompletionItemKind::Module | CompletionItemKind::Keyword => -250,
            CompletionItemKind::Snippet => -150,
            _ => 0,
        },
        CompletionContextKind::ParameterList => match kind {
            CompletionItemKind::Variable | CompletionItemKind::Field => 350,
            CompletionItemKind::Snippet => 150,
            CompletionItemKind::Function => -200,
            CompletionItemKind::Module => -250,
            _ => 0,
        },
        CompletionContextKind::MemberDeclaration => match kind {
            CompletionItemKind::Function
            | CompletionItemKind::Variable
            | CompletionItemKind::Type => 200,
            CompletionItemKind::Snippet => -150,
            _ => 0,
        },
        CompletionContextKind::Unknown => 0,
    }
}

fn scope_bias(scope: Option<CompletionScope>) -> i32 {
    match scope {
        Some(CompletionScope::Local) => 700,
        Some(CompletionScope::Subroutine) => 650,
        Some(CompletionScope::Module) => 500,
        Some(CompletionScope::Class) => 450,
        Some(CompletionScope::File) => 300,
        Some(CompletionScope::Package) => 250,
        Some(CompletionScope::Unit) => 150,
        None => 0,
    }
}
