use chrn_utils::{id_types::InternedId, intern::Intern, utils::containers::SpannedContainer};

use crate::{
    lexer::token::{SpannedToken, Token, TokenKind},
    parser::{ast::ast_stmts::AbstractOptionAssignment, parser_helpers::DelimiterContext},
    semantic::hir::hir_directives::{
        DirectivePreprocess, DirectivePreprocessKind, DirectivePreprocessValue,
    },
};

// TEST:
pub fn produce_directives(
    toks: &[SpannedToken],
    hash_idx: usize,
    interner: &Intern,
) -> Option<DirectivePreprocess> {
    debug_assert!(hash_idx < toks.len());
    debug_assert_eq!(toks[hash_idx].tok, Token::HashSymbol);

    // At least #{ident}[] 3 toks ahead
    _ = toks.get(hash_idx + 3)?;

    // At ident
    let mut pos = hash_idx + 1;

    // let ident_span = toks[pos].span;
    let id = expect_id(toks, &mut pos, TokenKind::Id)?;
    let kind = DirectivePreprocessKind::try_from_interned_str(id)?;

    let delim_ctx = DelimiterContext::with_comma(TokenKind::OBracket, TokenKind::CBracket);

    let vals = collect_directive_val_idents(toks, &mut pos, kind, delim_ctx);
    dbg!(&vals);
    panic!();
    let mut directive = DirectivePreprocess::new(kind, vals);

    for sp_tok in &toks[hash_idx..] {
        dbg!(sp_tok);
    }
    panic!("Hi");
    Some(directive)
}

fn collect_directive_val_idents(
    toks: &[SpannedToken],
    pos: &mut usize,
    kind: DirectivePreprocessKind,
    delim_ctx: DelimiterContext,
) -> Vec<DirectivePreprocessValue> {
    let vals: Vec<DirectivePreprocessValue> = Vec::new();
    if !expect_tok(toks, pos, delim_ctx.opening()) {
        return vals;
    };
    let val_delim_ctx = DelimiterContext::with_comma(TokenKind::OParen, TokenKind::CParen);

    while toks[*pos].tok.kind() != delim_ctx.closing() {
        // Not sure what to call this
        let Some(field_ident) = expect_id(toks, pos, TokenKind::Id) else {
            return vals;
        };
        let vals = collect_directive_vals(toks, pos, val_delim_ctx);
    }

    // TEST: It doesn't actually depend on this working so maybe it's fine? The parser would still
    // report.
    // if !expect_tok(toks, pos, delim_ctx.closing()) {
    //     return vals;
    // };
    vals
}

fn collect_directive_vals(
    toks: &[SpannedToken],
    pos: &mut usize,
    delim_ctx: DelimiterContext,
) -> Vec<DirectivePreprocessValue> {
    let vals = Vec::new();
    if !expect_tok(toks, pos, delim_ctx.opening()) {
        return vals;
    };
    todo!()
}

/// Returns `true` if `expected` == found
fn expect_id(toks: &[SpannedToken], pos: &mut usize, expected: TokenKind) -> Option<InternedId> {
    let found = &toks[*pos];
    advance(pos);
    let res = match found.tok {
        Token::Id(id) | Token::Str(id) | Token::Integer(id, _) | Token::Float(id, _) => {
            if found.tok.kind() == expected {
                return Some(id);
            } else {
                None
            }
        }
        _ => None,
    };
    res
}

/// Returns `true` if `expected` == found
fn expect_tok(toks: &[SpannedToken], pos: &mut usize, expected: TokenKind) -> bool {
    let res = toks[*pos].tok.kind() == expected;
    advance(pos);
    res
}

fn advance(pos: &mut usize) {
    *pos += 1;
}
