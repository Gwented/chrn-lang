//! Parser intended to process preprocess-directives that MUST be seen before the
//! parsing stage to be meaningful.
use chrn_utils::id_types::InternedId;
use toolsc::cursors::BasicCursor;

use crate::{
    lexer::token::{SpannedToken, Token, TokenFloat, TokenInt, TokenKind},
    parser::parser_helpers::DelimiterContext,
    semantic::hir::hir_directives::{
        DirectivePreprocessExpectInput, DirectivePreprocessField, DirectivePreprocessFieldSchema,
        DirectivePreprocessHir, DirectivePreprocessInput, DirectivePreprocessKind,
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
pub fn process_directive(toks: &[SpannedToken], hash_idx: usize) -> Option<DirectivePreprocessHir> {
    debug_assert!(hash_idx < toks.len());
    debug_assert_eq!(toks[hash_idx].tok, Token::HashSymbol);

    // At least #{ident}[] 3 toks ahead
    _ = toks.get(hash_idx + 3)?;

    // At directive ident
    let mut cursor = BasicCursor::with_pos(toks, hash_idx + 1);

    let name_span = cursor.peek_ref().span;
    // Ident of directive
    let id = expect_id(&mut cursor, TokenKind::Id)?;
    let kind = DirectivePreprocessKind::try_from_interned_str(id)?;

    let delim_ctx = DelimiterContext::with_comma(TokenKind::OBracket, TokenKind::CBracket);

    let fields = collect_directive_fields(&mut cursor, kind, delim_ctx);
    let directive = DirectivePreprocessHir::new(name_span, kind, fields);

    Some(directive)
}

// #{ident}[{ident}({val}), {ident}({val}), ..]
//TODO: Should maybe have basic duplicate field prevention
fn collect_directive_fields(
    cursor: &mut BasicCursor<SpannedToken>,
    kind: DirectivePreprocessKind,
    delim_ctx: DelimiterContext,
) -> Vec<DirectivePreprocessField> {
    let mut fields: Vec<DirectivePreprocessField> = Vec::new();
    // OBracket
    if !expect_tok(cursor, delim_ctx.opening()) {
        return fields;
    };

    // Syntax for the array of values inside directives
    let val_delim_ctx = DelimiterContext::with_comma(TokenKind::OParen, TokenKind::CParen);

    while !cursor.peek_owned().tok.kind().is_terminator()
        && cursor.peek_owned().tok.kind() != delim_ctx.closing()
    {
        // Not sure what to call this
        let Some(field_ident) = expect_id(cursor, TokenKind::Id) else {
            break;
        };

        if cursor.peek_owned().tok.kind() == delim_ctx.arg_sep()
            && cursor.peek_ahead_owned(1).tok.kind() == delim_ctx.closing()
        {
            cursor.skip(1);
            break;
        } else if cursor.peek_owned().tok.kind() == delim_ctx.closing() {
            break;
        }

        let Some(schema) = kind.get_field(field_ident) else {
            return fields;
        };

        // #d[f(x,y)]
        let Some(field) = collect_directive_field_vals(cursor, schema, val_delim_ctx) else {
            return fields;
        };
        fields.push(field);

        // comma
        if !expect_tok(cursor, delim_ctx.arg_sep()) {
            break;
        }
    }
    // Doesn't check CBracket because it makes no meaningful difference

    fields
}

//TODO: Idea here is we do [(arg1, input1), (arg2, input2), (argn, inputn)] and for each
//we store the input if the arg passes the constraint. When done, we should have all valid inputs
//and feed that to the schema, which produces whatever it should produce for it's schema given
//the items.
fn collect_directive_field_vals(
    cursor: &mut BasicCursor<SpannedToken>,
    field_schema: &DirectivePreprocessFieldSchema,
    delim_ctx: DelimiterContext,
) -> Option<DirectivePreprocessField> {
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
    for _ in field_schema.arg_layout.args() {
        if arg_count + 1 > field_schema.arg_layout.arg_len() {
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

    // Terminates here because assuming everything later will be fine by letting this through as
    // though it were a success is not valid.
    if !expect_tok(cursor, delim_ctx.closing()) {
        return None;
    };

    let kind = field_schema.try_as_field_kind(&inputs)?;
    Some(DirectivePreprocessField::new(kind))
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
