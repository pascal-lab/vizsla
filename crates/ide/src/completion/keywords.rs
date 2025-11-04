use hir::completion::CompletionScope;
use utils::text_edit::{TextEdit, TextEditItem, TextRange, TextSize};

use super::{
    CompletionConfig, CompletionContext, CompletionItem, CompletionItemKind, compute_score,
};

pub fn keyword_completions(
    prefix: &str,
    ctx: Option<&CompletionContext>,
    config: &CompletionConfig,
) -> Vec<CompletionItem> {
    KEYWORDS
        .iter()
        .filter(|(keyword, _)| keyword.starts_with(prefix))
        .map(|(keyword, snippet)| {
            let mut item = CompletionItem {
                label: keyword.to_string(),
                label_detail: None,
                detail: Some("keyword".to_string()),
                insert_text: None,
                filter_text: None,
                kind: CompletionItemKind::Keyword,
                score: 0,
                primary_edit: None,
                additional_edits: Vec::new(),
            };

            if config.enable_snippets && !snippet.is_empty() {
                item.insert_text = Some(snippet.to_string());
                item.kind = CompletionItemKind::Snippet;
            }

            item.score = compute_score(prefix, &item.label, item.kind, ctx, None);
            item
        })
        .collect()
}

pub fn system_task_completions(
    prefix: &str,
    ctx: Option<&CompletionContext>,
    _config: &CompletionConfig,
) -> Vec<CompletionItem> {
    if !prefix.starts_with('$') {
        return Vec::new();
    }

    let prefix_without_dollar = &prefix[1..];

    SYS_TASKS
        .iter()
        .filter(|task| task.starts_with(prefix_without_dollar))
        .map(|task| {
            let label = format!("${}", task);

            let primary_edit = ctx.map(|ctx| {
                let token = &ctx.token;
                let token_len = token.text.len();
                let start = ctx
                    .position
                    .offset
                    .checked_sub(TextSize::from(token_len as u32))
                    .unwrap_or(ctx.position.offset);
                let end = ctx.position.offset;
                let range = TextRange::new(start, end);

                TextEdit::from_iter([TextEditItem { del: range, ins: label.clone() }])
            });

            CompletionItem {
                score: compute_score(
                    prefix,
                    &label,
                    CompletionItemKind::Function,
                    ctx,
                    Some(CompletionScope::Unit),
                ),
                label: label.clone(),
                label_detail: None,
                detail: Some("system task".to_string()),
                insert_text: Some(label),
                filter_text: None,
                kind: CompletionItemKind::Function,
                primary_edit,
                additional_edits: Vec::new(),
            }
        })
        .collect()
}

pub fn directive_completions(
    prefix: &str,
    ctx: Option<&CompletionContext>,
    _config: &CompletionConfig,
) -> Vec<CompletionItem> {
    if !prefix.starts_with('`') {
        return Vec::new();
    }

    let prefix_without_tick = &prefix[1..];

    DIRECTIVES
        .iter()
        .filter(|directive| directive.starts_with(prefix_without_tick))
        .map(|directive| {
            let label = format!("`{}", directive);

            let primary_edit = ctx.map(|ctx| {
                let token = &ctx.token;
                let token_len = token.text.len();
                let start = ctx
                    .position
                    .offset
                    .checked_sub(TextSize::from(token_len as u32))
                    .unwrap_or(ctx.position.offset);
                let end = ctx.position.offset;
                let range = TextRange::new(start, end);

                TextEdit::from_iter([TextEditItem { del: range, ins: label.clone() }])
            });

            CompletionItem {
                score: compute_score(
                    prefix,
                    &label,
                    CompletionItemKind::Keyword,
                    ctx,
                    Some(CompletionScope::Unit),
                ),
                label: label.clone(),
                label_detail: None,
                detail: Some("directive".to_string()),
                insert_text: Some(label),
                filter_text: None,
                kind: CompletionItemKind::Keyword,
                primary_edit,
                additional_edits: Vec::new(),
            }
        })
        .collect()
}

// (keyword, snippet)
const KEYWORDS: &[(&str, &str)] = &[
    ("always", "always @($1) begin\n\t$2\nend"),
    ("always_comb", "always_comb begin\n\t$1\nend"),
    ("always_ff", "always_ff @($1) begin\n\t$2\nend"),
    ("always_latch", "always_latch begin\n\t$1\nend"),
    ("and", ""),
    ("assert", ""),
    ("assign", ""),
    ("assume", ""),
    ("automatic", ""),
    ("begin", "begin\n\t$1\nend"),
    ("bit", ""),
    ("break", ""),
    ("buf", ""),
    ("byte", ""),
    ("case", "case ($1)\n\t$2\nendcase"),
    ("casex", "casex ($1)\n\t$2\nendcase"),
    ("casez", "casez ($1)\n\t$2\nendcase"),
    ("class", "class $1;\n\t$2\nendclass"),
    ("clocking", "clocking $1;\n\t$2\nendclocking"),
    ("const", ""),
    ("constraint", ""),
    ("continue", ""),
    ("cover", ""),
    ("covergroup", ""),
    ("default", ""),
    ("disable", ""),
    ("do", ""),
    ("edge", ""),
    ("else", ""),
    ("end", ""),
    ("endcase", ""),
    ("endclass", ""),
    ("endfunction", ""),
    ("endgenerate", ""),
    ("endmodule", ""),
    ("endpackage", ""),
    ("endtask", ""),
    ("enum", ""),
    ("event", ""),
    ("expect", ""),
    ("export", ""),
    ("extends", ""),
    ("extern", ""),
    ("final", ""),
    ("for", "for ($1; $2; $3) begin\n\t$4\nend"),
    ("forever", "forever begin\n\t$1\nend"),
    ("fork", "fork\n\t$1\njoin"),
    ("function", "function $1;\n\t$2\nendfunction"),
    ("generate", "generate\n\t$1\nendgenerate"),
    ("genvar", ""),
    ("if", "if ($1) begin\n\t$2\nend"),
    ("iff", ""),
    ("import", ""),
    ("initial", "initial begin\n\t$1\nend"),
    ("inout", ""),
    ("input", ""),
    ("inside", ""),
    ("int", ""),
    ("integer", ""),
    ("interface", "interface $1;\n\t$2\nendinterface"),
    ("join", ""),
    ("join_any", ""),
    ("join_none", ""),
    ("let", ""),
    ("local", ""),
    ("localparam", ""),
    ("logic", ""),
    ("longint", ""),
    ("module", "module $1($2);\n\t$3\nendmodule"),
    ("modport", ""),
    ("negedge", ""),
    ("new", ""),
    ("null", ""),
    ("or", ""),
    ("output", ""),
    ("package", "package $1;\n\t$2\nendpackage"),
    ("packed", ""),
    ("parameter", ""),
    ("posedge", ""),
    ("primitive", ""),
    ("priority", ""),
    ("program", "program $1;\n\t$2\nendprogram"),
    ("property", "property $1;\n\t$2\nendproperty"),
    ("protected", ""),
    ("pure", ""),
    ("rand", ""),
    ("randc", ""),
    ("randcase", ""),
    ("ref", ""),
    ("reg", ""),
    ("repeat", ""),
    ("return", ""),
    ("sequence", "sequence $1;\n\t$2\nendsequence"),
    ("shortint", ""),
    ("shortreal", ""),
    ("signed", ""),
    ("static", ""),
    ("string", ""),
    ("struct", ""),
    ("super", ""),
    ("supply0", ""),
    ("supply1", ""),
    ("task", "task $1;\n\t$2\nendtask"),
    ("this", ""),
    ("time", ""),
    ("typedef", ""),
    ("union", ""),
    ("unique", ""),
    ("unsigned", ""),
    ("var", ""),
    ("virtual", ""),
    ("void", ""),
    ("wait", ""),
    ("while", "while ($1) begin\n\t$2\nend"),
    ("wire", ""),
    ("with", ""),
];

const SYS_TASKS: &[&str] = &[
    "display",
    "write",
    "monitor",
    "strobe",
    "finish",
    "stop",
    "exit",
    "fatal",
    "error",
    "warning",
    "info",
    "time",
    "realtime",
    "stime",
    "random",
    "urandom",
    "urandom_range",
    "fopen",
    "fclose",
    "fgetc",
    "fgets",
    "fscanf",
    "sscanf",
    "sformat",
    "sformatf",
    "readmemb",
    "readmemh",
    "writememb",
    "writememh",
    "dumpfile",
    "dumpvars",
    "dumpoff",
    "dumpon",
    "dumpall",
    "clog2",
    "bits",
    "cast",
];

const DIRECTIVES: &[&str] = &[
    "define",
    "undef",
    "ifdef",
    "ifndef",
    "elsif",
    "else",
    "endif",
    "include",
    "timescale",
    "default_nettype",
    "resetall",
    "celldefine",
    "endcelldefine",
    "pragma",
    "line",
];
