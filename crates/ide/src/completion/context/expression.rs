use either::Either;
use syntax::{SyntaxKind, SyntaxNode, SyntaxNodeExt, has_text_range::HasTextRange};
use utils::text_edit::TextSize;

pub fn is_in_expression_context(root: SyntaxNode, position: TextSize) -> bool {
    let token_or_node = root.token_or_node_at_offset(position);

    let mut node = match token_or_node {
        Either::Left(tok_at_offset) => match tok_at_offset.left_biased() {
            Some(tok) => tok.parent,
            None => return false,
        },
        Either::Right(node) => node,
    };

    loop {
        match node.kind() {
            SyntaxKind::ALWAYS_BLOCK
            | SyntaxKind::ALWAYS_COMB_BLOCK
            | SyntaxKind::ALWAYS_FF_BLOCK
            | SyntaxKind::ALWAYS_LATCH_BLOCK
            | SyntaxKind::INITIAL_BLOCK
            | SyntaxKind::FINAL_BLOCK => return true,

            SyntaxKind::ASSIGNMENT_EXPRESSION | SyntaxKind::NONBLOCKING_ASSIGNMENT_EXPRESSION => {
                return true;
            }

            SyntaxKind::FUNCTION_DECLARATION | SyntaxKind::TASK_DECLARATION => {
                if is_inside_callable_body(node, position) {
                    return true;
                }
            }

            SyntaxKind::BINARY_AND_EXPRESSION
            | SyntaxKind::BINARY_OR_EXPRESSION
            | SyntaxKind::BINARY_XOR_EXPRESSION
            | SyntaxKind::BINARY_XNOR_EXPRESSION => return true,

            SyntaxKind::CONDITIONAL_EXPRESSION => return true,

            SyntaxKind::INVOCATION_EXPRESSION => return true,

            SyntaxKind::PARENTHESIZED_EXPRESSION => return true,

            SyntaxKind::CONDITIONAL_STATEMENT | SyntaxKind::CASE_STATEMENT => return true,

            SyntaxKind::FOR_LOOP_STATEMENT
            | SyntaxKind::FOREACH_LOOP_STATEMENT
            | SyntaxKind::DO_WHILE_STATEMENT
            | SyntaxKind::FOREVER_STATEMENT
            | SyntaxKind::LOOP_STATEMENT => return true,

            SyntaxKind::SEQUENTIAL_BLOCK_STATEMENT | SyntaxKind::PARALLEL_BLOCK_STATEMENT => {
                return true;
            }

            SyntaxKind::EXPRESSION_STATEMENT => return true,

            SyntaxKind::RETURN_STATEMENT => return true,

            _ => {}
        }

        let parent = node.parent();
        if let Some(p) = parent {
            node = p;
        } else {
            break;
        }
    }

    false
}

fn is_inside_callable_body(node: SyntaxNode, position: TextSize) -> bool {
    let mut port_list_end: Option<TextSize> = None;

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i)
            && let Some(child_node) = child.as_node()
        {
            let child_kind = child_node.kind();

            if child_kind == SyntaxKind::FUNCTION_PORT_LIST
                && let Some(range) = child_node.text_range()
            {
                port_list_end = Some(range.end());
            }

            if matches!(
                child_kind,
                SyntaxKind::SEQUENTIAL_BLOCK_STATEMENT
                    | SyntaxKind::PARALLEL_BLOCK_STATEMENT
                    | SyntaxKind::RETURN_STATEMENT
                    | SyntaxKind::EXPRESSION_STATEMENT
                    | SyntaxKind::ASSIGNMENT_EXPRESSION
            ) && let Some(range) = child_node.text_range()
                && range.contains(position)
            {
                return true;
            }
        }
    }

    if let Some(end) = port_list_end
        && position >= end
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn check_expression_context(source: &str, expected: bool) {
        static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

        let (prefix, _) = source.split_once("$0").expect("No $0 marker found");
        let source = source.replace("$0", "");

        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_path = format!("test_expr_{}.sv", counter);
        let binding = syntax::SyntaxTree::from_text(&source, &unique_path, &unique_path);
        let root = binding.root().unwrap();

        let position = TextSize::from(prefix.len() as u32);
        let result = is_in_expression_context(root, position);

        assert_eq!(
            result,
            expected,
            "Expected is_in_expression_context={} at position {}, got {}.\nSource:\n{}",
            expected,
            prefix.len(),
            result,
            source
        );
    }

    #[test]
    fn test_in_assignment_expression() {
        check_expression_context(
            r#"module test;
    logic sig_a, sig_b;
    initial begin
        sig_a = $0
    end
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_in_procedural_block() {
        check_expression_context(
            r#"module test;
    logic sig;
    initial begin
        $0
    end
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_in_always_block() {
        check_expression_context(
            r#"module test;
    logic clk, data;
    always @(posedge clk) begin
        data = $0
    end
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_in_function_body() {
        check_expression_context(
            r#"module test;
    function int my_func(int x);
        int tmp;
        return $0;
    endfunction
endmodule"#,
            true,
        );
    }

    #[test]
    fn test_not_in_function_signature() {
        check_expression_context(
            r#"module test;
    function int my_func(int $0x);
        return x;
    endfunction
endmodule"#,
            false,
        );
    }
}
