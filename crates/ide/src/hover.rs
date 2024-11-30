use hir::{
    container::InContainer,
    hir_def::{expr::Expr, literal::Literal},
    semantics::Semantics,
};
use ide_db::root_db::RootDb;
use span::{FilePosition, RangeInfo};
use syntax::{
    SVInt, SyntaxNodeExt, SyntaxTokenWithParent, TokenKind,
    ast::{self, AstNode},
    has_text_range::HasTextRange,
    token::TokenKindExt,
};
use utils::get::GetRef;

use crate::markup::Markup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverFormat {
    Markdown,
    PlainText,
}

#[derive(Debug, Clone)]
pub struct HoverConfig {
    pub format: HoverFormat,
}

pub(crate) fn hover(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
    config: HoverConfig,
) -> Option<RangeInfo<Markup>> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = file.syntax().token_at_offset(offset).pick_bext_token(token_precedence)?;

    let res = handle_literal(&sema, token)?;
    Some(RangeInfo::new(token.text_range()?, res))
}

pub(crate) fn token_precedence(kind: TokenKind) -> usize {
    match kind {
        _ if kind.name_like() => 4,
        _ if kind.is_literal() => 3,
        _ => 1,
    }
}

fn handle_literal(
    sema: &Semantics<RootDb>,
    SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
) -> Option<Markup> {
    if !tok.kind().is_literal() {
        return None;
    }

    let db = sema.db;

    let expr = ast::Expression::cast(parent)?;
    let InContainer { value: expr_id, cont_id } = sema.resolve_expr(expr);
    let container = cont_id.to_container(db);
    let Expr::Literal(literal) = container.get(expr_id) else {
        return None;
    };

    let mut res = Markup::new();

    match literal {
        Literal::Int(svint) => {
            let width = svint.get_bit_width();
            let dec = render_svint(svint, 10);
            res.push_with_plain_fence(&format!("width: {width}\ndec: {dec}\n"));
            if let Some(ieee754) = render_svint_as_ieee754(svint) {
                res.push_with_plain_fence(&format!("ieee754: {ieee754}"));
            }

            res.line_break();

            let bin = render_svint(svint, 2);
            let oct = render_svint(svint, 8);
            let hex = render_svint(svint, 16);
            res.push_with_plain_fence(&format!("bin: {bin}\nhex: {hex}\noct: {oct}",));
        }
        Literal::Float(float) => {
            let num = f64::from(*float);
            let bits = float.to_bits();
            res.push_with_plain_fence(&format!("{num}\nbits: {bits:#x}"));
        }
        Literal::Time { val, unit } => {
            let num = f64::from(*val);
            res.push_with_plain_fence(&format!("{num} {unit}"));
        }
        Literal::Str(s) => {
            res.push_with_plain_fence(&format!("{s}"));
        }
        Literal::UnbasedUnsized(bit) => {
            res.push_with_plain_fence(&format!("{bit}"));
        }
    };

    Some(res)
}

fn render_svint(svint: &SVInt, base: usize) -> String {
    let mut s = svint.serialize(base);
    let mut len = s.len();
    let width = svint.get_bit_width();
    if base == 2 || base == 8 || base == 16 {
        let log = match base {
            2 => 1,
            8 => 3,
            16 => 4,
            _ => unreachable!(),
        };
        s.insert_str(0, &"0".repeat(width.div_ceil(log) - len));
        len += width.div_ceil(log) - len;
    }

    let interval = match base {
        2 => 4,
        8 => 3,
        10 => 3,
        16 => 4,
        _ => unreachable!("unexpected base: {base}"),
    };

    let mut result = String::with_capacity(len + len / interval + len / 4);

    for (i, c) in s.chars().enumerate() {
        if i > 0 {
            if base == 2 && (len - i) % 16 == 0 {
                result.push_str(" / ");
            } else if (len - i) % interval == 0 {
                result.push(' ');
            }
        }
        result.push(c);
    }

    result
}

fn render_svint_as_ieee754(svint: &SVInt) -> Option<String> {
    let width = svint.get_bit_width();

    if (width != 32 && width != 64) || svint.has_unknown() {
        return None;
    }

    let word = svint.get_single_word().unwrap();
    if width == 32 {
        let f = f32::from_bits(word as u32);
        Some(format!("{:?}", f))
    } else {
        let f = f64::from_bits(word);
        Some(format!("{:?}", f))
    }
}
