use hir::{
    semantics::Semantics,
    type_infer::{TyMember, members_of_ty, type_of_expr},
};
use ide_db::root_db::RootDb;
use span::FilePosition;
use syntax::{
    SyntaxAncestors, SyntaxNode, SyntaxNodeExt,
    ast::{self, AstNode},
    has_text_range::HasTextRange,
};
use utils::text_edit::TextEditItem;

use super::{CompletionItem, CompletionItemKind};
use crate::completion::context::CompletionContext;

pub(super) fn complete_member_access(
    db: &RootDb,
    position: FilePosition,
    prefix: &str,
    ctx: &CompletionContext,
) -> Vec<CompletionItem> {
    let sema = Semantics::new(db);
    let file = sema.parse(position.file_id);
    let root = file.syntax();

    let members = sema
        .find_node_at_offset::<ast::MemberAccessExpression>(root, position.offset)
        .or_else(|| member_access_at_offset(root, position.offset))
        .and_then(|access| members_for_expr(db, &sema, access.left()))
        .or_else(|| members_for_incomplete_access(db, &sema, root, position.offset));
    let Some(members) = members else {
        return Vec::new();
    };

    members
        .into_iter()
        .map(|member| member.name)
        .filter(|name| name.as_str().starts_with(prefix))
        .map(|name| {
            let label = name.to_string();
            CompletionItem {
                label: label.clone(),
                kind: CompletionItemKind::Text,
                edit: Some(TextEditItem::replace(ctx.replacement, label)),
                snippet_edit: None,
            }
        })
        .collect()
}

fn member_access_at_offset(
    root: SyntaxNode<'_>,
    offset: utils::text_edit::TextSize,
) -> Option<ast::MemberAccessExpression<'_>> {
    let prev = root.token_before_offset(offset)?;
    if prev.kind() != syntax::Token![.] {
        return None;
    }
    SyntaxAncestors::start_from(prev.parent).find_map(ast::MemberAccessExpression::cast)
}

fn members_for_incomplete_access(
    db: &RootDb,
    sema: &Semantics<'_, RootDb>,
    root: SyntaxNode<'_>,
    offset: utils::text_edit::TextSize,
) -> Option<Vec<TyMember>> {
    let dot = root.token_before_offset(offset)?;
    if dot.kind() != syntax::Token![.] {
        return None;
    }

    let dot_start = dot.text_range()?.start();
    let expr = expr_before_dot(dot.parent, dot_start)?;

    members_for_expr(db, sema, expr)
}

fn expr_before_dot(
    parent: SyntaxNode<'_>,
    dot_start: utils::text_edit::TextSize,
) -> Option<ast::Expression<'_>> {
    parent
        .children()
        .filter_map(|elem| elem.as_node())
        .filter_map(ast::Expression::cast)
        .find(|expr| expr.syntax().text_range().is_some_and(|r| r.end() == dot_start))
}

fn members_for_expr(
    db: &RootDb,
    sema: &Semantics<'_, RootDb>,
    expr: ast::Expression<'_>,
) -> Option<Vec<TyMember>> {
    let ty = type_of_expr(db, sema.resolve_expr(expr)).ty;
    let members = members_of_ty(db, &ty);
    (!members.is_empty()).then_some(members)
}
