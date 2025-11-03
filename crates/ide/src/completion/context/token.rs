use syntax::{SyntaxNode, SyntaxNodeExt};
use utils::text_edit::TextSize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionToken {
    pub text: String,
    pub is_dot_access: bool,
    pub is_scope_resolution: bool,
    pub prefix_chain: Vec<String>,
}

impl CompletionToken {
    pub fn prefix(&self) -> Option<&str> {
        self.prefix_chain.first().map(|s| s.as_str())
    }
}

impl CompletionToken {
    pub fn new(text: String) -> Self {
        Self { text, is_dot_access: false, is_scope_resolution: false, prefix_chain: Vec::new() }
    }

    pub fn with_dot_access(mut self, prefix_chain: Vec<String>) -> Self {
        self.is_dot_access = true;
        self.prefix_chain = prefix_chain;
        self
    }

    pub fn with_scope_resolution(mut self, prefix_chain: Vec<String>) -> Self {
        self.is_scope_resolution = true;
        self.prefix_chain = prefix_chain;
        self
    }
}

pub fn extract_completion_token(syntax: SyntaxNode, position: TextSize) -> Option<CompletionToken> {
    let token_at_offset = syntax.token_at_offset(position);
    let token = token_at_offset.left_biased();

    if token.is_none() {
        return Some(CompletionToken::new(String::new()));
    }

    let token = token.unwrap();
    let token_text = token.value_text().to_string();
    let token_range = token.tok.range()?;

    let position_usize: usize = position.into();
    if token_range.end() == position_usize && !is_identifier(&token_text) {
        if token_text == "." {
            let chain = extract_chain_before(&syntax, token_range.start()).unwrap_or_default();
            return Some(CompletionToken::new(String::new()).with_dot_access(chain));
        } else if token_text == "::" {
            let chain = extract_chain_before(&syntax, token_range.start()).unwrap_or_default();
            return Some(CompletionToken::new(String::new()).with_scope_resolution(chain));
        }
        return Some(CompletionToken::new(String::new()));
    }

    if token_text == "."
        && let Some(chain) = extract_chain_before(&syntax, token_range.start())
    {
        return Some(CompletionToken::new(String::new()).with_dot_access(chain));
    }

    if token_text == "::"
        && let Some(chain) = extract_chain_before(&syntax, token_range.start())
    {
        return Some(CompletionToken::new(String::new()).with_scope_resolution(chain));
    }

    if token_range.start() > 0 {
        let before_current = token_range.start();
        let tokens_at_pos = syntax.token_at_offset(TextSize::from(before_current as u32));

        if let Some(prev_token) = tokens_at_pos.left_biased() {
            let prev_text = prev_token.value_text().to_string();
            let prev_range = prev_token.tok.range();

            if let Some(pr) = prev_range
                && pr.end() == before_current
            {
                if prev_text == "." {
                    if let Some(chain) = extract_chain_before(&syntax, pr.start()) {
                        return Some(CompletionToken::new(token_text).with_dot_access(chain));
                    }
                } else if prev_text == "::"
                    && let Some(chain) = extract_chain_before(&syntax, pr.start())
                {
                    return Some(CompletionToken::new(token_text).with_scope_resolution(chain));
                }
            }
        }
    }

    Some(CompletionToken::new(token_text))
}

fn extract_chain_before(syntax: &SyntaxNode, position: usize) -> Option<Vec<String>> {
    let mut chain = Vec::new();
    let mut current_pos = position;

    loop {
        if current_pos == 0 {
            break;
        }

        let offset = TextSize::from((current_pos - 1) as u32);
        let token = syntax.token_at_offset(offset).left_biased()?;
        let token_text = token.value_text().to_string();
        let token_range = token.tok.range()?;

        if is_identifier(&token_text) {
            chain.push(token_text);
            current_pos = token_range.start();

            if current_pos > 0 {
                let before_ident = TextSize::from(current_pos as u32);
                if let Some(separator_token) = syntax.token_at_offset(before_ident).left_biased() {
                    let separator_text = separator_token.value_text().to_string();
                    let separator_range = separator_token.tok.range()?;

                    if (separator_text == "." || separator_text == "::")
                        && separator_range.end() == current_pos
                    {
                        current_pos = separator_range.start();
                        continue;
                    }
                }
            }

            break;
        } else if token_text == "." || token_text == "::" {
            current_pos = token_range.start();
            continue;
        } else {
            break;
        }
    }

    if chain.is_empty() {
        return None;
    }

    chain.reverse();
    Some(chain)
}

fn is_identifier(s: &str) -> bool {
    !s.is_empty() && (is_regular_identifier(s) || is_system_identifier(s))
}

fn is_regular_identifier(s: &str) -> bool {
    !s.starts_with(|c: char| c.is_ascii_digit())
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_system_identifier(s: &str) -> bool {
    s.starts_with('$')
        && s.len() > 1
        && !s[1..].starts_with(|c: char| c.is_ascii_digit())
        && s[1..].chars().all(|c| c.is_alphanumeric() || c == '_')
}
