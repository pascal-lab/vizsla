use base_db::source_db::SourceDb;
use hir::{hir_def::lower_ident, semantics::Semantics};
use ide_db::root_db::RootDb;
use line_index::{TextRange, TextSize};
use span::FilePosition;
use syntax::{
    SyntaxNode, SyntaxNodeExt, SyntaxTokenWithParent,
    ast::{self, AstNode},
    has_text_range::{HasTextRange, SourceRangeExt},
    match_ast,
    token::TokenKindExt,
};
use thiserror::Error;
use utils::text_edit::TextEdit;

use crate::{
    ScopeVisibility,
    definitions::DefinitionClass,
    navigation_target::ToNav,
    references::{
        ReferencesConfig,
        search::{ReferenceToken, ReferencesCtx},
    },
    source_change::SourceChange,
};

pub type RenameResult<T> = Result<T, RenameError>;

#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub scope_visibility: ScopeVisibility,
}

#[derive(Error, Debug)]
pub enum RenameError {
    #[error("No references found at position")]
    NoRefFound,
    #[error("No definitions found for the token")]
    NoDefFound,
}

pub(crate) fn prepare_rename(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> RenameResult<TextRange> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = pick_token(file.syntax(), offset)?;
    let text_range = token.text_range().ok_or(RenameError::NoRefFound)?;
    DefinitionClass::resolve(&sema, token).ok_or(RenameError::NoDefFound)?;
    Ok(text_range)
}

pub(crate) fn rename(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
    RenameConfig { scope_visibility }: RenameConfig,
    new_name: &str,
) -> RenameResult<SourceChange> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = pick_token(file.syntax(), offset)?;
    let def_class = DefinitionClass::resolve(&sema, token).ok_or(RenameError::NoDefFound)?;
    let def = match &def_class {
        DefinitionClass::Definition(def) => def,
        DefinitionClass::PortConnShorthand { data, .. } => data,
    };

    let old_name = lower_ident(Some(token.tok)).unwrap();
    let new_name = new_name.to_string();

    let mut source_changes = SourceChange::default();
    source_changes.extend(ReferencesCtx::new(&sema, def, ReferencesConfig::new(scope_visibility, None))
        .search()
        .into_iter()
        .map(|(file_id, toks)| {
            let mut text_edit = TextEdit::builder();
            let text = sema.db.file_text(file_id);
            toks.into_iter().for_each(|ReferenceToken { token }| {
                let range = token.range().unwrap().to_text_range();
                let (token, parent) = (token.tok, token.parent);
                // TODO: fixit
                match_ast! { parent,
                    ast::NamedPortConnection[it] if it.name() == Some(token) => {
                        // .port(data), ...
                        //  ^^^^ : rename this
                        match (it.open_paren(), it.close_paren()) {
                            (Some(_), Some(cp)) if it.expr().and_then(|expr| expr.syntax().text_range()).is_some_and(|range| text[range] == new_name) => {
                                // .port(data),  => .data,
                                //  ^^^^
                                let end = cp.text_range().unwrap().end();
                                text_edit.replace(TextRange::new(range.start(), end), new_name.clone());
                            },
                            (None, None) => {
                                // .port,  => .port(data),
                                //  ^^^^
                                text_edit.replace(range, format!("{old_name}({new_name})"));
                            },
                            _ => text_edit.replace(range, new_name.clone()),
                        }
                    },
                    // TODO: .port(data)
                    //             ^^^^
                    _ => text_edit.replace(range, new_name.clone()),
                }
            });
            (file_id, text_edit.finish())
        }));

    let def_edits = def.sources().into_iter().map(|def| {
        let mut text_edit = TextEdit::builder();
        // TODO: optimization??
        let nav = def.to_nav(db);
        text_edit.replace(nav.focus_range.unwrap(), new_name.clone());
        (nav.file_id, text_edit.finish())
    });

    // TODO:
    // source_changes.extend(def_edits);
    Ok(source_changes)
}

fn pick_token(node: SyntaxNode, offset: TextSize) -> RenameResult<SyntaxTokenWithParent> {
    node.token_at_offset(offset)
        .pick_bext_token(|kind| kind.name_like().into())
        .ok_or(RenameError::NoRefFound)
}
