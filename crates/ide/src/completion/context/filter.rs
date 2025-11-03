use syntax::{SyntaxNode, SyntaxNodeExt, TokenKind, TriviaKind};
use utils::text_edit::TextSize;

pub fn should_complete(syntax: SyntaxNode, position: TextSize) -> bool {
    let pos: usize = position.into();

    if is_inside_comment(syntax, pos) {
        return false;
    }

    let token = syntax.token_at_offset(position).left_biased();

    if token.is_none() {
        return true;
    }

    let token = token.unwrap();
    let kind = token.tok.kind();

    if is_string_token(kind) || is_numeric_literal(kind) {
        return false;
    }

    true
}

fn is_inside_comment(syntax: SyntaxNode, pos: usize) -> bool {
    fn check_node(node: SyntaxNode, pos: usize) -> bool {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if let Some(token) = child.as_token() {
                    for ((start, end), trivia) in token.trivias_with_loc() {
                        if matches!(
                            trivia.kind(),
                            TriviaKind::LINE_COMMENT | TriviaKind::BLOCK_COMMENT
                        ) && start <= pos
                            && pos <= end
                        {
                            return true;
                        }
                    }
                } else if let Some(child_node) = child.as_node()
                    && check_node(child_node, pos)
                {
                    return true;
                }
            }
        }
        false
    }

    check_node(syntax, pos)
}

fn is_string_token(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::STRING_LITERAL)
}

fn is_numeric_literal(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::INTEGER_LITERAL
            | TokenKind::INTEGER_BASE
            | TokenKind::REAL_LITERAL
            | TokenKind::UNBASED_UNSIZED_LITERAL
            | TokenKind::TIME_LITERAL
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn check_should_complete(source: &str, expected: bool) {
        static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

        let (prefix, _) = source.split_once("$0").expect("No $0 marker found");
        let source = source.replace("$0", "");

        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_path = format!("test_filter_{}.sv", counter);
        let binding = syntax::SyntaxTree::from_text(&source, &unique_path, &unique_path);
        let root = binding.root().unwrap();

        let position = TextSize::from(prefix.len() as u32);
        let result = should_complete(root, position);

        assert_eq!(
            result,
            expected,
            "Expected should_complete={} at position {}, got {}",
            expected,
            prefix.len(),
            result
        );
    }

    #[test]
    fn test_allow_empty_file() {
        check_should_complete("$0", true);
    }

    #[test]
    fn test_allow_normal_identifier() {
        check_should_complete(
            r#"module test;
    logic sig$0nal;
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_filter_string_literal() {
        check_should_complete(
            r#"module test;
    initial $display("hello wor$0ld");
endmodule"#,
            false,
        );
    }

    #[test]
    fn test_filter_line_comment() {
        check_should_complete(
            r#"module test;
    // This is a comm$0ent
    logic sig;
endmodule"#,
            false,
        );
    }

    #[test]
    fn test_filter_block_comment() {
        check_should_complete(
            r#"module test;
    /* comment $0 text */
    logic sig;
endmodule"#,
            false,
        );
    }

    #[test]
    fn test_allow_after_dot() {
        check_should_complete(
            r#"module test;
    typedef struct { int field; } my_struct_t;
    my_struct_t s;
    initial s.$0
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_allow_in_expression() {
        check_should_complete(
            r#"module test;
    logic signal;
    initial sig$0
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_filter_numeric_literal() {
        check_should_complete(
            r#"module test;
    logic [7:0] data = 8'hF$0F;
endmodule"#,
            false,
        );
    }
}
