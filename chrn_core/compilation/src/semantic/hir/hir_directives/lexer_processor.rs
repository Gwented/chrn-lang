//! Parser intended to process preprocess-directives that MUST be seen before the
//! parsing stage to be meaningful.
use chrn_utils::id_types::InternedId;
use toolsc::cursors::BasicCursor;

use crate::{
    lexer::token::{SpannedToken, Token, TokenFloat, TokenInt, TokenKind},
    parser::parser_helpers::DelimiterContext,
    semantic::hir::hir_directives::{
        DirectivePreprocess, DirectivePreprocessExpectInput, DirectivePreprocessInput,
        DirectivePreprocessSchema,
    },
};

// TEST:
// Also need a higher semantic version to process directives after the typeresolver.
/// Expects to start at `#` in a stream of tokens, then attempts to parse it as a
/// preprocessed directive.
///
/// Syntax: #[{directive}({val}, ..), ..]
///
/// NOTE: Will panic if expected invariants are broken
pub fn process_directive(toks: &[SpannedToken], hash_idx: usize) -> Option<DirectivePreprocess> {
    debug_assert!(hash_idx < toks.len());
    debug_assert_eq!(toks[hash_idx].tok, Token::HashSymbol);

    //TEST: Is this true?
    // At least #{ident}() 3 toks ahead
    _ = toks.get(hash_idx + 3)?;

    // At ident
    let mut cursor = BasicCursor::with_pos(toks, hash_idx + 1);

    let id = expect_id(&mut cursor, TokenKind::Id)?;
    let field_schema = DirectivePreprocessSchema::try_from_interned_id(id)?;

    let val_delim_ctx = DelimiterContext::with_comma(TokenKind::OParen, TokenKind::CParen);
    let directive = make_directive(&mut cursor, field_schema, val_delim_ctx)?;

    Some(directive)
}

fn make_directive(
    cursor: &mut BasicCursor<SpannedToken>,
    schema: &DirectivePreprocessSchema,
    delim_ctx: DelimiterContext,
) -> Option<DirectivePreprocess> {
    if !expect_tok(cursor, delim_ctx.opening()) {
        return None;
    };

    // Simplify pleaseee
    let mut inputs: Vec<(DirectivePreprocessExpectInput, DirectivePreprocessInput)> = Vec::new();

    //WARN: HANDLE SEMANTICS PLEASE
    // So field schemas can enforce their max arg count
    let mut arg_count = 0;
    //TEST: Overly complicated

    //NOTE: If we have an actual parameterized directive field, f(x,y), we probably want to return
    //as a failure rather than keep half of something invalid
    for _ in schema.arg_layout.args() {
        if arg_count + 1 > schema.arg_layout.arg_len() {
            return None;
        }

        // Advancing ref so the lifetime lives long enough for the resolved input
        let tok = &cursor.advance_ref().tok;
        if tok.kind().is_terminator() || tok.kind() == delim_ctx.closing() {
            break;
        }

        let expect = DirectivePreprocessExpectInput::Token(tok.kind());
        let input = DirectivePreprocessInput::Token(tok);

        // if !arg.allows(&input) {
        //     return None;
        // }

        arg_count += 1;
        inputs.push((expect, input));

        // // Tok needs to be put up against constraints
        // let Some(field_kind) = field_schema.try_tok_as_field_kind(&tok, arg) else {
        //     return;
        // };

        //TODO: Fields have arguments therefore we need an aregument abresuaotrction for pre fields

        if cursor.peek_owned().tok.kind() == delim_ctx.arg_sep()
            && cursor.peek_ahead_owned(1).tok.kind() == delim_ctx.closing()
        {
            cursor.skip(1);
            break;
        } else if cursor.peek_owned().tok.kind() == delim_ctx.closing() {
            break;
        }

        if !expect_tok(cursor, delim_ctx.arg_sep()) {
            return None;
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
    let found = cursor.advance_owned();
    let res = match found.tok {
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
    res
}

/// Returns `true` if `expected` == found
fn expect_tok(cursor: &mut BasicCursor<SpannedToken>, expected: TokenKind) -> bool {
    let res = cursor.advance_owned().tok.kind() == expected;
    res
}
