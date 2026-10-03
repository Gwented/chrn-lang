//! Parser intended to process preprocess-directives that MUST be seen before the
//! parsing stage to be meaningful.
use chrn_utils::{id_types::InternedId, utils::cursors::CursorBasic};

use crate::{
    lexer::token::{SpannedToken, Token, TokenKind},
    parser::parser_helpers::DelimiterContext,
    semantic::hir::hir_directives::{
        DirectivePreprocess, DirectivePreprocessField, DirectivePreprocessKind,
        DirectivePreprocessSchema,
    },
};

// TEST:
// Also need a higher semantic version to process directives after the typeresolver.
/// Expects to start at `#` in a stream of tokens, then attempts to parse it as a
/// preprocessed directive.
///
/// Syntax: {directive_ident}[{directive_field}({val}, ..), ..]
///
/// NOTE: Will panic if expected invariants are broken
pub fn process_directive(toks: &[SpannedToken], hash_idx: usize) -> Option<DirectivePreprocess> {
    debug_assert!(hash_idx < toks.len());
    debug_assert_eq!(toks[hash_idx].tok, Token::HashSymbol);

    // At least #{ident}[] 3 toks ahead
    _ = toks.get(hash_idx + 3)?;

    // At directive ident
    let mut cursor = CursorBasic::with_pos(toks, hash_idx + 1);

    // Ident of directive
    let id = expect_id(&mut cursor, TokenKind::Id)?;
    let kind = DirectivePreprocessKind::try_from_interned_str(id)?;

    let delim_ctx = DelimiterContext::with_comma(TokenKind::OBracket, TokenKind::CBracket);

    let vals = collect_directive_fields(&mut cursor, kind, delim_ctx);
    // dbg!(&vals);
    // panic!();
    let mut directive = DirectivePreprocess::new(kind, vals);

    // for sp_tok in &toks[hash_idx..] {
    //     dbg!(sp_tok);
    // }
    Some(directive)
}

// #{ident}[{ident}({val}), {ident}({val}), ..]
fn collect_directive_fields(
    cursor: &mut CursorBasic<SpannedToken>,
    kind: DirectivePreprocessKind,
    delim_ctx: DelimiterContext,
) -> Vec<DirectivePreprocessField> {
    let mut vals: Vec<DirectivePreprocessField> = Vec::new();
    if !expect_tok(cursor, TokenKind::OBracket) {
        return vals;
    };

    let val_delim_ctx = DelimiterContext::with_comma(TokenKind::OParen, TokenKind::CParen);

    while !cursor.peek_owned().tok.kind().is_terminator()
        && cursor.peek_owned().tok != Token::CBracket
    {
        // Not sure what to call this
        let Some(field_ident) = expect_id(cursor, TokenKind::Id) else {
            return vals;
        };

        if cursor.peek_owned().tok.kind() == delim_ctx.arg_sep()
            && cursor.peek_ahead_owned(1).tok.kind() == delim_ctx.closing()
        {
            cursor.advance_owned();
            break;
        } else if cursor.peek_owned().tok.kind() == delim_ctx.closing() {
            break;
        }

        let Some(schema) = kind.get_field(field_ident) else {
            return vals;
        };

        collect_directive_field_vals(cursor, schema, &mut vals, val_delim_ctx);

        //break?
        if !expect_tok(cursor, TokenKind::Comma) {
            return vals;
        }
    }

    // TEST: It doesn't actually depend on this working so maybe it's fine? The parser would still
    // report.
    // if !expect_tok(toks, pos, delim_ctx.closing()) {
    //     return vals;
    // };
    vals
}

fn collect_directive_field_vals(
    cursor: &mut CursorBasic<SpannedToken>,
    schema: &DirectivePreprocessSchema,
    fields: &mut Vec<DirectivePreprocessField>,
    delim_ctx: DelimiterContext,
) {
    if !expect_tok(cursor, TokenKind::OParen) {
        return;
    };

    while !cursor.peek_owned().tok.kind().is_terminator()
        && cursor.peek_owned().tok != Token::CParen
    {
        let tok = cursor.peek_owned().tok;
        // May be permissive
        let Some(val) = schema.try_tok_as_val(&tok) else {
            return;
        };

        if cursor.peek_owned().tok.kind() == delim_ctx.arg_sep()
            && cursor.peek_ahead_owned(1).tok.kind() == delim_ctx.closing()
        {
            cursor.advance_owned();
            break;
        } else if cursor.peek_owned().tok.kind() == delim_ctx.closing() {
            break;
        }

        fields.push(val);

        if !expect_tok(cursor, TokenKind::Comma) {
            return;
        }
    }

    if !expect_tok(cursor, TokenKind::OParen) {
        return;
    };
}

/// Returns `true` if `expected` == found
fn expect_id(cursor: &mut CursorBasic<SpannedToken>, expected: TokenKind) -> Option<InternedId> {
    let found = cursor.advance_owned();
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
fn expect_tok(cursor: &mut CursorBasic<SpannedToken>, expected: TokenKind) -> bool {
    let res = cursor.advance_owned().tok.kind() == expected;
    res
}
