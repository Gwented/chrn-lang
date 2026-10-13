//! Parser intended to process preprocess-directives that MUST be seen before the
//! parsing stage to be meaningful.
use chrn_utils::{id_types::InternedId, intern::Intern};
use toolsc::cursors::BasicCursor;

use crate::{
    lexer::token::{SpannedToken, Token, TokenFloat, TokenInt, TokenKind},
    parser::parser_helpers::DelimiterContext,
    semantic::hir::hir_directives::{
        DirectivePreprocess, DirectivePreprocessExpectInput, DirectivePreprocessSchema,
    },
};

// TEST:
// Also need a higher semantic version to process directives after the typeresolver.
/// Expects to start at `#` in a stream of tokens, then attempts to parse it as a
/// preprocessed directive.
///
/// Syntax: #{directive}({val}, ..)
///
/// Returns `None` for malformed or unsupported directives, including truncated input.
/// NOTE: `hash_idx` must point to a `#` token.
pub fn process_directive(
    toks: &[SpannedToken],
    hash_idx: usize,
    interner: &Intern,
) -> Option<DirectivePreprocess> {
    debug_assert!(hash_idx < toks.len());
    debug_assert_eq!(toks[hash_idx].tok, Token::HashSymbol);

    // At ident
    let mut cursor = BasicCursor::with_pos(toks, hash_idx + 1);

    let id = expect_id(&mut cursor, TokenKind::Id)?;
    let field_schema = DirectivePreprocessSchema::try_from_interned_id(id)?;

    let val_delim_ctx = DelimiterContext::with_comma(TokenKind::OParen, TokenKind::CParen);
    let directive = make_directive(&mut cursor, field_schema, val_delim_ctx, interner)?;

    Some(directive)
}

/// Attempts to parse a directive out of `Token`, expecting the correct syntax
fn make_directive(
    cursor: &mut BasicCursor<SpannedToken>,
    schema: &DirectivePreprocessSchema,
    delim_ctx: DelimiterContext,
    interner: &Intern,
) -> Option<DirectivePreprocess> {
    if !expect_tok(cursor, delim_ctx.opening()) {
        return None;
    };

    //NOTE: Not sure if this can be removed because for an integer token for example,
    //we want the integer value to put inside the actual directive field as well as
    //check against it, which both require the inner info regarding the token, or any input at all.
    let mut inputs: Vec<DirectivePreprocessExpectInput> =
        Vec::with_capacity(schema.arg_layout.arg_len());

    //WARN: The schema bounds the number of arguments consumed.

    //NOTE: If we have an actual parameterized directive field, f(x,y), we probably want to return
    //as a failure rather than keep half of something invalid
    for _ in schema.arg_layout.args() {
        // Leave the closing delimiter for the final check.
        let tok = &cursor.peek_ref_checked()?.tok;
        if tok.kind() == delim_ctx.closing() {
            break;
        }

        // Advancing ref so the lifetime lives long enough for the expect input
        let tok = &cursor.advance_ref_checked()?.tok;
        let input = DirectivePreprocessExpectInput::from_tok(tok, interner)?;

        inputs.push(input);

        if cursor.peek_ref_checked()?.tok.kind() == delim_ctx.closing() {
            break;
        }

        if !expect_tok(cursor, delim_ctx.arg_sep()) {
            return None;
        }
        if cursor.peek_ref_checked()?.tok.kind() == delim_ctx.closing() {
            break;
        }
    }

    if !expect_tok(cursor, delim_ctx.closing()) {
        return None;
    };

    let kind = schema.try_as_directive_preprocess(&inputs)?;
    Some(DirectivePreprocess::new(kind))
}

/// Returns `true` if `expected` == found
fn expect_id(cursor: &mut BasicCursor<SpannedToken>, expected: TokenKind) -> Option<InternedId> {
    let found = cursor.advance_owned_checked()?;
    let tok_opt = match found.tok {
        Token::Id(id)
        | Token::Str(id)
        | Token::Integer(TokenInt {
            interned_id: id, ..
        })
        | Token::Float(TokenFloat {
            interned_id: id, ..
        }) => {
            if found.tok.kind() == expected {
                return Some(id);
            } else {
                None
            }
        }
        _ => None,
    };
    tok_opt
}

/// Returns `true` if `expected` == found
fn expect_tok(cursor: &mut BasicCursor<SpannedToken>, expected: TokenKind) -> bool {
    cursor
        .advance_ref_checked()
        .is_some_and(|found| found.tok.kind() == expected)
}
