use crate::config_loader::{ConfigLoader, ConfigLoaderOutput};
use crate::lexer::token::TokenKind;
use crate::lexer::trivia::TriviaKind;

use super::helpers::*;

#[test]
fn lex_compound_tokens_with_exact_spans() {
    let src: &[u8] = b":= :: -> => == >= <= != && || ..=";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let toks = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg)
        .tokenize(&mut interner)
        .toks;

    let expected = [
        (TokenKind::Walrus, 0, 2),
        (TokenKind::StaticAccess, 3, 5),
        (TokenKind::SlimArrow, 6, 8),
        (TokenKind::NotSlimArrow, 9, 11),
        (TokenKind::EqualTo, 12, 14),
        (TokenKind::GreaterOrEq, 15, 17),
        (TokenKind::LessOrEq, 18, 20),
        (TokenKind::NotEq, 21, 23),
        (TokenKind::And, 24, 26),
        (TokenKind::Or, 27, 29),
        (TokenKind::DotRangeInclusive, 30, 33),
    ];

    assert_eq!(toks.len(), expected.len() + 1);
    for (tok, (expected_kind, expected_start, expected_end)) in toks.iter().zip(expected) {
        assert_eq!(tok.tok.kind(), expected_kind);
        assert_eq!(tok.span.start, expected_start, "{expected_kind} start");
        assert_eq!(tok.span.end, expected_end, "{expected_kind} end");
    }
    assert_eq!(toks.last().unwrap().tok, Token::EOF);
}

#[test]
fn lex_tok_test() {
    let text = r#"bind "./some/path""#;

    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();

    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(
        None, metadata.serial_start,
        "start_offset without `@def` failed"
    );
    assert_eq!(3, toks.len(), "expected exactly 3 tokens");

    // Keyword("bind") spanning bytes 0..4
    assert!(
        matches!(toks[0].tok, Token::Keyword(Keyword::Bind)),
        "expected Keyword(Bind) token"
    );
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);

    // Str("./some/path") spanning bytes 5..18 (includes both quotes)
    assert!(matches!(toks[1].tok, Token::Str(_)), "expected Str token");
    assert_eq!(toks[1].span.start, 5);
    assert_eq!(toks[1].span.end, 18);

    assert_eq!(toks[2].tok, Token::EOF);
    assert_eq!(toks[2].span.start, 17);
}

#[test]
fn lex_tok_test_rev() {
    // Properly closed @def and @end
    let correct = r#"@defbind "./some/path"@end"#;

    let mut interner = mock_interner(1, 1);
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let opt = ConfigLoader::new(
        region_id,
        correct.as_bytes(),
        path_id,
        &ChrnConfig::default(),
    )
    .load_config();

    let region = match opt {
        ConfigLoaderOutput::Success(region, _) => region,
        other => panic!("properly closed @def and @end should succeed, got {other:?}"),
    };

    assert_eq!(region.script_start, 0);
    assert_eq!(region.serial_start, Some(26));

    let toks = Lexer::new(
        region.region_id,
        region.path_id,
        &region.src_bytes,
        region.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    // Expect: Def(0,4), Id("bind")(4,8), Str("./some/path")(9,22), End(22,26)
    assert_eq!(toks.len(), 4);
    assert_eq!(toks[0].tok, Token::Def);
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert!(
        matches!(toks[1].tok, Token::Keyword(Keyword::Bind)),
        "expected Keyword(Bind) after @def"
    );
    assert_eq!(toks[1].span.start, 4);
    assert_eq!(toks[1].span.end, 8);
    assert!(matches!(toks[2].tok, Token::Str(_)));
    assert_eq!(toks[2].span.start, 9);
    assert_eq!(toks[2].span.end, 22);
    assert_eq!(toks[3].tok, Token::End);
    assert_eq!(toks[3].span.start, 22);
    assert_eq!(toks[3].span.end, 26);

    // Improper @def without an @end
    // This malformed region must not destabilize diagnostic reporting.
    let wrong = r#"@defbind "./some/path""#;

    let opt = ConfigLoader::new(region_id, wrong.as_bytes(), path_id, &ChrnConfig::default())
        .load_config();

    match opt {
        ConfigLoaderOutput::Broken(region, _) => {
            assert_eq!(region.script_start, 0, "@def at offset 0");
            assert!(region.serial_start.is_none(), "no @end found");
        }
        other => panic!("improper @def without @end should produce a Broken region, got {other:?}"),
    }
}

#[test]
fn cfg_at_test() {
    // Properly closed @def and @end
    let content = "\n     @e\n";
    let mut interner = mock_interner(1, 1);
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let mut settings = ChrnConfig::default();
    let region = ConfigLoader::new(
        region_id,
        content.as_bytes(),
        path_id,
        &ChrnConfig::default(),
    )
    .load_config()
    .expect_success();
    let region_str = str::from_utf8(&region.src_bytes[..]).unwrap();
    assert_eq!(region_str, content);

    let toks = Lexer::new(
        region_id,
        region.path_id,
        &region.src_bytes,
        region.script_start,
        &mut settings,
    )
    .tokenize(&mut interner)
    .toks;
    assert_eq!(toks.len(), 3);
    assert_eq!(toks[0].tok, Token::At);
    assert_eq!(toks[0].span.start, 6);
    assert_eq!(toks[0].span.end, 7);
    assert!(matches!(toks[1].tok, Token::Id(_)));
    assert_eq!(toks[1].span.start, 7);
    assert_eq!(toks[1].span.end, 8);
    assert_eq!(toks[2].tok, Token::EOF);
    assert_eq!(toks[2].span.start, 8);

    let (_, diags) = parser::parse(&mut settings, &region, &toks, &interner);
    assert!(
        !diags.diags.is_empty(),
        "parser should have picked up at least one error"
    );
}

// -----------------------------------------------------------------------------------------
// Config loader byte-consumption tests
//
// These tests target `ConfigLoader` directly with pathological input. They mirror the
// spirit of `chrn_tests/other.chrn` which uses raw `@` text and odd whitespace, but at the
// byte level. Each test exercises a specific corner of the byte-walking state machine so
// that subtle off-by-one, lookahead, escape, or comment-handling regressions get caught.
// -----------------------------------------------------------------------------------------

/// Helper: runs the config loader on a raw byte slice and returns the resulting region.
#[test]
fn char_literal_test() {
    // Valid single character
    let text = "'a'";
    let mut interner = Intern::init();

    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);

    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Char(_),),
        "Expected char token, got {:?}",
        toks[0].tok
    );
    // `'a'` = 3 bytes: span (0, 3)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 3);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 2);

    // Valid escaped character
    let text = "'\\n'";
    let mut interner = Intern::init();

    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);

    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();

    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Char(_),),
        "Expected char token, got {:?}",
        toks[0].tok
    );
    // `'\n'` = 4 bytes: span (0, 4)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Valid hex escape
    let text = "'\\x2F'";

    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);

    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();

    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Char(_),),
        "Expected char token, got {:?}",
        toks[0].tok
    );
    // `'\x2F'` = 6 bytes: span (0, 6)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 6);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 5);

    // Invalid character
    let text = "'aa'";
    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Invalid(_),),
        "Expected Invalid token, got {:?}",
        toks[0].tok
    );
    // `'aa'` = 4 bytes: span (0, 4)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Invalid hex escape
    let text = "'\\x2'";
    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Invalid(_),),
        "Expected Invalid token, got {:?}",
        toks[0].tok
    );
    // `'\x2'` = 5 bytes: span (0, 5)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 5);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 4);

    // I can't actually read hex
    // Invalid hex digits
    let text = "'\\x255'";
    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Invalid(_),),
        "Expected Invalid token, got {:?}",
        toks[0].tok
    );
    // `'\x255'` = 7 bytes: span (0, 7)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 7);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 6);

    // Unknown escape
    let text = "'\\q'";
    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Invalid(_),),
        "Expected Invalid token, got {:?}",
        toks[0].tok
    );
    // `'\q'` = 4 bytes: span (0, 4)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Out of range escape
    let text = "'\\x1Y'";
    let mut interner = Intern::init();
    let path_id = interner.intern_path(Path::new(""));
    let region_id = SourceRegionId::new(0);
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    assert!(
        matches!(toks[0].tok, Token::Invalid(_),),
        "Expected Invalid token, got {:?}",
        toks[0].tok
    );
    // `'\x1Y'` = 6 bytes: span (0, 6)
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 6);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 5);
}

#[test]
fn lex_notation_test() {
    // Hex Test (Hex Text (Hex Test))
    let text = "0xff";
    let mut interner = mock_interner(1, 1);

    let path_id = PathId::new(0);
    let region_id = SourceRegionId::new(0);

    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();

    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Hex) => {
            assert_eq!("255", interner.search(id));
        }
        _ => panic!("Expected Integer with Hex, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Binary
    let text = "0b1010";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Bin) => {
            assert_eq!("10", interner.search(id));
        }
        _ => panic!("Expected Integer with Binary, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 6);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 5);

    // Octal
    let text = "0o77";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Octal) => {
            assert_eq!("63", interner.search(id));
        }
        _ => panic!("Expected Integer with Octal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Decimal
    let text = "42";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Decimal) => {
            assert_eq!("42", interner.search(id));
        }
        _ => panic!("Expected Integer of Decimal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 2);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 1);

    // Float with decimal
    let text = "3.14";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Float(id, Notation::Decimal) => {
            assert_eq!("3.14", interner.search(id));
        }
        _ => panic!("Expected Float with Decimal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 4);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 3);

    // Positive Scientific Notation
    let text = "1e+23";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Float(id, Notation::Decimal) => {
            assert_eq!("1e+23", interner.search(id));
        }
        _ => panic!("Expected Float with Decimal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 5);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 4);

    // Negative Scientific Notation
    let text = "1e-23";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Float(id, Notation::Decimal) => {
            assert_eq!("1e-23", interner.search(id));
        }
        _ => panic!("Expected Float with Decimal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 5);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 4);

    // Underscored Numbers
    let text = "1_000_000";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Decimal) => {
            assert_eq!("1000000", interner.search(id));
        }
        _ => panic!("Expected Integer with Decimal, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 9);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 8);

    // Underscored Hex
    let text = "0x_ff_ff";
    let metadata = ConfigLoader::new(region_id, text.as_bytes(), path_id, &ChrnConfig::default())
        .load_config()
        .expect_success();
    let toks = Lexer::new(
        metadata.region_id,
        metadata.path_id,
        &metadata.src_bytes,
        metadata.script_start,
        &mut ChrnConfig::default(),
    )
    .tokenize(&mut interner)
    .toks;

    assert_eq!(2, toks.len());
    match toks[0].tok {
        Token::Integer(id, Notation::Hex) => {
            assert_eq!("65535", interner.search(id));
        }
        _ => panic!("Expected Integer with Hex, found {:?}", toks[0].tok),
    }
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 8);
    assert_eq!(toks[1].tok, Token::EOF);
    assert_eq!(toks[1].span.start, 7);
}

#[test]
fn read_ident_includes_trailing_underscore() {
    let src: &[u8] = b"foo_";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let mut lex = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg);
    let toks = lex.tokenize(&mut interner).toks;

    // Expect at least one identifier token: "foo_"
    let id_tok = toks
        .iter()
        .find_map(|st| match st.tok {
            Token::Id(id) => Some((id, st.span)),
            _ => None,
        })
        .expect("expected an Id token for \"foo_\"");

    let (id, span) = id_tok;
    let lexed = interner.search(id);
    assert_eq!(lexed, "foo_", "underscore should be included in ident");

    // Span must cover exactly the four bytes "foo_".
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 4);
}

// You can tell from the very second word the writer of this test.
/// A bare `_` at the end of input must lex without panicking.
#[test]
fn read_ident_handles_bare_underscore() {
    let src: &[u8] = b"_";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let mut lex = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg);
    let toks = lex.tokenize(&mut interner).toks;

    let id = toks
        .iter()
        .find_map(|st| match st.tok {
            Token::Id(id) => Some((id, st.span)),
            _ => None,
        })
        .expect("expected an Id token for \"_\"");

    assert_eq!(interner.search(id.0), "_");
    assert_eq!(id.1.start, 0);
    assert_eq!(id.1.end, 1);
}

/// Identifiers that mix alphanumerics and underscores in various positions
/// must all be lexed correctly.
#[test]
fn read_ident_mixed_alphanumeric_and_underscore() {
    // Separators between identifiers are tokens that don't contain
    // alphanumerics or underscores, so each Id token in the source
    // becomes one Id in the output.
    let src: &[u8] = b"foo_bar+_qux+a_b_c_";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let mut lex = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg);
    let toks = lex.tokenize(&mut interner).toks;

    let names: Vec<(String, u32, u32)> = toks
        .iter()
        .filter_map(|st| match st.tok {
            Token::Id(id) => Some((interner.search(id).to_string(), st.span.start, st.span.end)),
            _ => None,
        })
        .collect();

    assert_eq!(
        names,
        vec![
            ("foo_bar".to_string(), 0, 7),
            ("_qux".to_string(), 8, 12),
            ("a_b_c_".to_string(), 13, 19),
        ]
    );
}
//TODO: Verity
//
// #[test]
// fn embedded_nul_is_invalid_without_truncating_following_tokens() {
//     let src: &[u8] = b"left\0 right";
//     let mut interner = Intern::init();
//     let mut cfg = ChrnConfig::default();
//     let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
//
//     assert_eq!(
//         output
//             .toks
//             .iter()
//             .map(|tok| tok.tok.kind())
//             .collect::<Vec<_>>(),
//         vec![TokenKind::Id, TokenKind::Invalid, TokenKind::Id, TokenKind::EOF],
//     );
//     assert_eq!(output.toks[1].span.start, 4);
//     assert_eq!(output.toks[1].span.end, 5);
//     assert_eq!(output.toks[2].span.start, 6);
//     assert_eq!(output.toks[2].span.end, 11);
//     let Token::Id(id) = output.toks[2].tok else {
//         unreachable!("token kind asserted above")
//     };
//     assert_eq!(interner.search(id), "right");
// }

#[test]
fn invalid_control_whitespace_recovery_makes_progress() {
    let src: &[u8] = b"\x0bvalid";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(
        output
            .toks
            .iter()
            .map(|tok| tok.tok.kind())
            .collect::<Vec<_>>(),
        vec![TokenKind::Invalid, TokenKind::EOF],
    );
    assert_eq!(output.found_invalid_toks, 1);
    assert_eq!(output.toks[0].span.start, 0);
    assert_eq!(output.toks[0].span.end, 6);
}

// TODO: Not a "failure", but more so a better recovery semantic to hinge off of. Where maybe we
// take in the ctx that we are in a str, which now means recover_invalid looks for the end quote,
// not whitespace. Or maybe just an `Option<char>` which just chooses '\"' and is ' ' by default
// #[test]
// fn invalid_string_escape_recovers_through_closing_quote() {
//     let src: &[u8] = b"\"a\\q rest\" next";
//     let mut interner = Intern::init();
//     let mut cfg = ChrnConfig::default();
//     let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
//
//     assert_eq!(
//         output
//             .toks
//             .iter()
//             .map(|tok| tok.tok.kind())
//             .collect::<Vec<_>>(),
//         vec![TokenKind::Invalid, TokenKind::Id, TokenKind::EOF],
//     );
//     assert_eq!(output.found_invalid_toks, 1);
//     assert_eq!(output.toks[0].span.start, 0);
//     assert_eq!(output.toks[0].span.end, 10);
//     assert_eq!(output.toks[1].span.start, 11);
//     assert_eq!(output.toks[1].span.end, 15);
// }

// Test all invalid tok locations to ensure it doesn't regress.
// Each case targets a distinct `increment_invalid_tok` / `recover_invalid` route in
// `lexer.rs`, first in isolation (exactly one `Invalid` + `EOF`) and then combined
// (emitted `Invalid` count must match `found_invalid_toks`).
#[test]
fn invalid_token_count_matches_emitted_invalid_tokens() {
    // Each entry: (bytes, label identifying the lexer.rs route).
    let cases: Vec<(Vec<u8>, &str)> = vec![
        // tokenize catch-all `_` -> recover_invalid (lexer.rs `match ch { _ => ... }`).
        (b"$".to_vec(), "tokenize catch-all"),
        // read_ident empty escaped `e#` -> recover_invalid.
        (b"e#".to_vec(), "read_ident empty escaped ident"),
        // read_quotes bad escape -> recover_invalid.
        (b"\"a\\q\"".to_vec(), "read_quotes invalid escape"),
        // read_char bad escape -> recover_invalid.
        (b"'\\q'".to_vec(), "read_char invalid escape"),
        // read_char multi-char -> recover_invalid.
        (b"'aa'".to_vec(), "read_char multi-char"),
        // read_num empty prefixed literal -> Invalid (hex / bin / octal prefixes
        // share the `<empty numeric literal>` branch).
        (b"0x".to_vec(), "read_num empty hex"),
        (b"0b".to_vec(), "read_num empty bin"),
        (b"0o".to_vec(), "read_num empty octal"),
        // read_num radix parse failure (i64 overflow) -> Invalid.
        (b"0xFFFFFFFFFFFFFFFFFF".to_vec(), "read_num radix overflow"),
        // read_quotes invalid UTF-8 -> Invalid.
        (vec![b'"', 0xFF, b'"'], "read_quotes invalid utf-8"),
        // read_char empty literal -> Invalid.
        (b"''".to_vec(), "read_char empty literal"),
    ];
    // NOTE: `read_ident` non-UTF8 and `read_num` non-UTF8 slices are defensive-only:
    // both advance on char/ASCII boundaries so the slice stays valid UTF-8 and the
    // branch is unreachable from lexer input; no case can target them.

    for (src, label) in &cases {
        let mut interner = Intern::init();
        let mut cfg = ChrnConfig::default();
        let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

        let invalid_count = output
            .toks
            .iter()
            .filter(|tok| tok.tok.kind() == TokenKind::Invalid)
            .count();
        assert_eq!(
            invalid_count, 1,
            "{label} ({src:?}) must emit exactly one Invalid token"
        );
        assert_eq!(
            output.found_invalid_toks, 1,
            "{label} ({src:?}) counter must match emitted Invalid tokens"
        );
        assert_eq!(
            output.toks.len(),
            2,
            "{label} ({src:?}) must emit Invalid + EOF only"
        );
        assert_eq!(output.toks.last().unwrap().tok, Token::EOF);
    }

    // Combined: every route at once. `recover_invalid` unconditionally advances once
    // before consuming to whitespace, so a single separating space after `e#` / `'aa'`
    // (where recovery starts already on the separator) would skip it and merge with
    // the next case; use a double space so the advance lands on whitespace and each
    // case stays a separate `Invalid`. Total (11) stays below `MAX_INVALID_TOKS` (12)
    // so the cap path does not truncate the count.
    let mut combined: Vec<u8> = Vec::new();
    for (idx, (src, _)) in cases.iter().enumerate() {
        if idx > 0 {
            combined.extend_from_slice(b"  ");
        }
        combined.extend_from_slice(src);
    }

    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), &combined, 0, &mut cfg).tokenize(&mut interner);

    let invalid_count = output
        .toks
        .iter()
        .filter(|tok| tok.tok.kind() == TokenKind::Invalid)
        .count();
    assert_eq!(
        invalid_count,
        cases.len(),
        "combined input must emit one Invalid per route"
    );
    assert_eq!(
        output.found_invalid_toks as usize,
        cases.len(),
        "combined counter must match emitted Invalid tokens"
    );
    assert_eq!(output.toks.len(), cases.len() + 1);
    assert_eq!(output.toks.last().unwrap().tok, Token::EOF);
}

#[test]
fn invalid_escape_tokens_obey_invalid_token_cap_without_overflow() {
    let mut src = b"'\\q' ".repeat(256);
    src.extend_from_slice(b"valid");
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), &src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(
        output
            .toks
            .iter()
            .filter(|tok| tok.tok.kind() == TokenKind::Invalid)
            .count(),
        13,
    );
    assert_eq!(output.found_invalid_toks, 13);
    assert_eq!(output.toks.len(), 14);
    assert_eq!(output.toks.last().unwrap().tok, Token::EOF);
}

// -----------------------------------------------------------------------------------------
// Trivia tests
// -----------------------------------------------------------------------------------------

#[test]
fn trivia_kind_predicate_methods() {
    // TriviaKind::Tab
    assert!(TriviaKind::Tab.is_spacing_no_newline());
    assert!(TriviaKind::Tab.is_spacing());
    assert!(!TriviaKind::Tab.is_comment());

    // TriviaKind::Whitespace
    assert!(TriviaKind::Whitespace.is_spacing_no_newline());
    assert!(TriviaKind::Whitespace.is_spacing());
    assert!(!TriviaKind::Whitespace.is_comment());

    // TriviaKind::Newline
    assert!(!TriviaKind::Newline.is_spacing_no_newline());
    assert!(TriviaKind::Newline.is_spacing());
    assert!(!TriviaKind::Newline.is_comment());

    // TriviaKind::SingleComment
    assert!(!TriviaKind::SingleComment.is_spacing_no_newline());
    assert!(!TriviaKind::SingleComment.is_spacing());
    assert!(TriviaKind::SingleComment.is_comment());

    // TriviaKind::MultiComment
    assert!(!TriviaKind::MultiComment.is_spacing_no_newline());
    assert!(!TriviaKind::MultiComment.is_spacing());
    assert!(TriviaKind::MultiComment.is_comment());
}

#[test]
fn trivia_whitespace_coalesces_spaces_and_unicode_whitespace() {
    // Single space
    let src: &[u8] = b" x";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 1);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        b" "
    );
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Consecutive spaces coalesce into a single TriviaKind::Whitespace
    let src: &[u8] = b"    x";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 4);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        b"    "
    );
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Unicode whitespace: NO-BREAK SPACE (\u{00A0}, 2 bytes in UTF-8)
    let src_str = "\u{00A0}x";
    let src = src_str.as_bytes();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 2);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Mixed ASCII spaces and Unicode whitespace coalesce into a single Whitespace trivia
    // ' ' (1 byte) + '\u{00A0}' (2 bytes) + ' ' (1 byte) = 4 bytes
    let src_str = " \u{00A0} x";
    let src = src_str.as_bytes();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 4);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);
}

#[test]
fn trivia_tabs_are_emitted_individually() {
    // Consecutive tabs are NOT coalesced; each tab gets its own TriviaKind::Tab
    let src: &[u8] = b"\t\t\tx";
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 3);
    for (i, trivia) in output.trivia.iter().enumerate() {
        assert_eq!(trivia.kind, TriviaKind::Tab, "trivia[{i}] should be Tab");
        assert_eq!(trivia.span.start, i as u32);
        assert_eq!(trivia.span.end, (i + 1) as u32);
    }
    assert_eq!(output.toks[0].leading_trivia_indices, 0..3);

    // Mixed spaces and tabs alternate
    let src: &[u8] = b" \t  \tx";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 4);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 1);

    assert_eq!(output.trivia[1].kind, TriviaKind::Tab);
    assert_eq!(output.trivia[1].span.start, 1);
    assert_eq!(output.trivia[1].span.end, 2);

    assert_eq!(output.trivia[2].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[2].span.start, 2);
    assert_eq!(output.trivia[2].span.end, 4);

    assert_eq!(output.trivia[3].kind, TriviaKind::Tab);
    assert_eq!(output.trivia[3].span.start, 4);
    assert_eq!(output.trivia[3].span.end, 5);

    assert_eq!(output.toks[0].leading_trivia_indices, 0..4);
}

#[test]
fn trivia_newlines_lf_and_crlf() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Single LF (\n)
    let src: &[u8] = b"\nx";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 1);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Single CRLF (\r\n) -> 2-byte span
    let src: &[u8] = b"\r\nx";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 2);
    assert_eq!(&src[0..2], b"\r\n");
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Mixed consecutive newlines
    let src: &[u8] = b"\r\n\n\r\nx";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 3);

    assert_eq!(output.trivia[0].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 2);

    assert_eq!(output.trivia[1].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[1].span.start, 2);
    assert_eq!(output.trivia[1].span.end, 3);

    assert_eq!(output.trivia[2].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[2].span.start, 3);
    assert_eq!(output.trivia[2].span.end, 5);

    assert_eq!(output.toks[0].leading_trivia_indices, 0..3);
}

#[test]
fn trivia_single_line_comments_basic() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Standard comment followed by newline
    let src: &[u8] = b"// a single line comment\nfoo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 2);
    assert_eq!(output.trivia[0].kind, TriviaKind::SingleComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 24);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        b"// a single line comment"
    );

    assert_eq!(output.trivia[1].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[1].span.start, 24);
    assert_eq!(output.trivia[1].span.end, 25);

    assert_eq!(output.toks[0].leading_trivia_indices, 0..2);

    // Empty single comment: `//\n`
    let src: &[u8] = b"//\nfoo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 2);
    assert_eq!(output.trivia[0].kind, TriviaKind::SingleComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 2);
    assert_eq!(output.trivia[1].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[1].span.start, 2);
    assert_eq!(output.trivia[1].span.end, 3);

    // Single comment at EOF without trailing newline
    let src: &[u8] = b"foo // trailing at eof";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.toks.len(), 2); // Id("foo"), EOF
    assert_eq!(output.toks[0].leading_trivia_indices, 0..0);
    assert_eq!(output.toks[1].leading_trivia_indices, 0..2); // attached to EOF

    assert_eq!(output.trivia.len(), 2);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 3);
    assert_eq!(output.trivia[0].span.end, 4);

    assert_eq!(output.trivia[1].kind, TriviaKind::SingleComment);
    assert_eq!(output.trivia[1].span.start, 4);
    assert_eq!(output.trivia[1].span.end, src.len() as u32);
    assert_eq!(
        &src[output.trivia[1].span.start as usize..output.trivia[1].span.end as usize],
        b"// trailing at eof"
    );

    // Single comment with Unicode text
    let src_str = "// 🦀 Ferris\nfoo";
    let src = src_str.as_bytes();
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia[0].kind, TriviaKind::SingleComment);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        "// 🦀 Ferris".as_bytes()
    );
}

#[test]
fn trivia_multi_line_comments_basic() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Standard block comment
    let src: &[u8] = b"/* hello block */ foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 2);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 17);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        b"/* hello block */"
    );

    assert_eq!(output.trivia[1].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[1].span.start, 17);
    assert_eq!(output.trivia[1].span.end, 18);

    assert_eq!(output.toks[0].leading_trivia_indices, 0..2);

    // Empty block comment: `/**/`
    let src: &[u8] = b"/**/foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 4);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Block comment spanning multiple lines (newlines inside are not separate trivia)
    let src: &[u8] = b"/* line 1\nline 2\r\nline 3 */foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 27);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);

    // Block comment containing extra asterisks and slashes
    let src: &[u8] = b"/*** not close / nor * ***/foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 27);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);
}

#[test]
fn trivia_nested_multi_line_comments() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Nested multi-line comment: `/* outer /* inner */ outer */`
    let src: &[u8] = b"/* outer /* inner */ outer */foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 29);
    assert_eq!(
        &src[output.trivia[0].span.start as usize..output.trivia[0].span.end as usize],
        b"/* outer /* inner */ outer */"
    );
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);
}

#[test]
fn trivia_empty_source_and_trivia_only_source() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Empty source
    let src: &[u8] = b"";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);
    assert_eq!(output.toks.len(), 1);
    assert_eq!(output.toks[0].tok, Token::EOF);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..0);
    assert!(output.trivia.is_empty());

    // Source with only trivia (no semantic tokens)
    let src: &[u8] = b"  // comment\n/* block */\n\t ";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.toks.len(), 1);
    assert_eq!(output.toks[0].tok, Token::EOF);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..7);

    assert_eq!(output.trivia.len(), 7);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[1].kind, TriviaKind::SingleComment);
    assert_eq!(output.trivia[2].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[3].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[4].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[5].kind, TriviaKind::Tab);
    assert_eq!(output.trivia[6].kind, TriviaKind::Whitespace);
}

#[test]
fn trivia_single_line_comment_crlf_handling() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // In a CRLF-terminated file, `// comment\r\n` ends with a 2-byte CRLF newline.
    // The comment content is `// comment` (bytes 0..10), and the newline is `\r\n` (bytes 10..12).
    let src: &[u8] = b"// comment\r\nfoo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.trivia.len(), 2);
    assert_eq!(output.trivia[0].kind, TriviaKind::SingleComment);
    assert_eq!(
        output.trivia[0].span.end, 10,
        "SingleComment span should end before '\\r', but got end={}",
        output.trivia[0].span.end
    );
    assert_eq!(output.trivia[1].kind, TriviaKind::Newline);
    assert_eq!(
        output.trivia[1].span.start, 10,
        "Newline span should start at '\\r', but got start={}",
        output.trivia[1].span.start
    );
    assert_eq!(
        output.trivia[1].span.end, 12,
        "Newline span should cover both '\\r' and '\\n' (span 10..12), but got end={}",
        output.trivia[1].span.end
    );
}

#[test]
fn trivia_nested_multi_line_comment_delimiter_tracking() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Source contains: outer `/*`, inner `/*/` (starts nested comment), and only ONE closing `*/`.
    // Since there are two `/*` openings and only one `*/` closing, depth should be 1 at `*/` and
    // remain unclosed through `foo` up to EOF.
    let src: &[u8] = b"/* a /*/ b */ foo";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    // If depth tracking skipped only 1 byte for `/*`, the `*` and `/` in `/*/` are matched as
    // both opening and immediately closing the nested comment, causing the single `*/` to close
    // the outer comment early and emitting `foo` as a token instead of treating it as part of
    // the unclosed comment.
    assert_eq!(
        output.toks.len(),
        1,
        "Expected unclosed multi-comment to swallow 'foo' until EOF, but emitted tokens: {:?}",
        output.toks
    );
    assert_eq!(output.toks[0].tok, Token::EOF);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.end as usize, src.len());
}

#[test]
fn trivia_interleaved_mixed_sequence() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    let src: &[u8] = b"  /* block */ \t // line\n\r\n\tbar";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    // Expected sequence:
    // 0: Whitespace "  " (0..2)
    // 1: MultiComment "/* block */" (2..13)
    // 2: Whitespace " " (13..14)
    // 3: Tab "\t" (14..15)
    // 4: Whitespace " " (15..16)
    // 5: SingleComment "// line" (16..23)
    // 6: Newline "\n" (23..24)
    // 7: Newline "\r\n" (24..26)
    // 8: Tab "\t" (26..27)
    // Followed by Id("bar") at 27..30.
    assert_eq!(output.trivia.len(), 9);

    let expected_kinds = [
        TriviaKind::Whitespace,
        TriviaKind::MultiComment,
        TriviaKind::Whitespace,
        TriviaKind::Tab,
        TriviaKind::Whitespace,
        TriviaKind::SingleComment,
        TriviaKind::Newline,
        TriviaKind::Newline,
        TriviaKind::Tab,
    ];

    for (i, (&expected_kind, trivia)) in expected_kinds.iter().zip(&output.trivia).enumerate() {
        assert_eq!(trivia.kind, expected_kind, "mismatch at trivia[{i}]");
    }

    // Verify byte coverage continuity
    let mut prev_end = 0u32;
    for (i, trivia) in output.trivia.iter().enumerate() {
        assert_eq!(trivia.span.start, prev_end, "trivia[{i}] start != prev_end");
        prev_end = trivia.span.end;
    }
    assert_eq!(prev_end, 27);

    assert_eq!(output.toks[0].leading_trivia_indices, 0..9);
    assert_eq!(output.toks[0].span.start, 27);
    assert_eq!(output.toks[0].span.end, 30);
}

#[test]
fn trivia_token_stream_contiguity_and_association() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    let src: &[u8] = b"let x: int = 42 // assign\n/* next */ struct Point { x: int }";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    // Verify contiguity across all tokens:
    // Every token's leading_trivia_indices.start must equal the previous token's leading_trivia_indices.end.
    assert_eq!(output.toks[0].leading_trivia_indices.start, 0);
    for i in 1..output.toks.len() {
        assert_eq!(
            output.toks[i].leading_trivia_indices.start,
            output.toks[i - 1].leading_trivia_indices.end,
            "gap or overlap in trivia indices between token {} and {}",
            i - 1,
            i
        );
    }

    // Total trivia coverage must equal output.trivia.len()
    assert_eq!(
        output.toks.last().unwrap().leading_trivia_indices.end as usize,
        output.trivia.len(),
        "last token end must equal total trivia count"
    );

    // Verify tokens with NO preceding trivia have empty leading_trivia_indices
    // ":" has no space before it (after "x")
    let colon_toks: Vec<_> = output
        .toks
        .iter()
        .filter(|t| t.tok == Token::Colon)
        .collect();
    assert_eq!(colon_toks.len(), 2);
    assert!(
        colon_toks[0].leading_trivia_indices.is_empty(),
        "first colon preceded directly by 'x' should have empty trivia range"
    );
    assert!(
        colon_toks[1].leading_trivia_indices.is_empty(),
        "second colon preceded directly by 'x' should have empty trivia range"
    );

    // "struct" is preceded by " // assign\n/* next */ "
    let struct_tok = output
        .toks
        .iter()
        .find(|t| matches!(t.tok, Token::Keyword(Keyword::Struct)))
        .unwrap();
    let struct_trivia: Vec<TriviaKind> = output.trivia[struct_tok.leading_trivia_indices.start
        as usize
        ..struct_tok.leading_trivia_indices.end as usize]
        .iter()
        .map(|t| t.kind)
        .collect();

    assert_eq!(
        struct_trivia,
        vec![
            TriviaKind::Whitespace,
            TriviaKind::SingleComment,
            TriviaKind::Newline,
            TriviaKind::MultiComment,
            TriviaKind::Whitespace,
        ]
    );
}

#[test]
fn trivia_with_embedding_def_and_end() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    let src: &[u8] = b"@def\n  var x = 1\n@end";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    // Tokens: Def, var, x, =, 1, End
    assert_eq!(output.toks[0].tok, Token::Def);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..0);

    assert!(matches!(output.toks[1].tok, Token::Keyword(Keyword::Var)));
    assert_eq!(output.toks[1].leading_trivia_indices, 0..2); // \n, "  "
    assert_eq!(output.trivia[0].kind, TriviaKind::Newline);
    assert_eq!(output.trivia[1].kind, TriviaKind::Whitespace);

    assert_eq!(output.toks.last().unwrap().tok, Token::End);
    let end_tok = output.toks.last().unwrap();
    assert_eq!(end_tok.leading_trivia_indices, 5..6); // \n
    assert_eq!(output.trivia[5].kind, TriviaKind::Newline);
}

#[test]
fn trivia_associated_with_invalid_tokens() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    let src: &[u8] = b"  $  valid";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.toks.len(), 3); // Invalid, Id("valid"), EOF
    assert!(matches!(output.toks[0].tok, Token::Invalid(_)));
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);
    assert_eq!(output.trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, 2);

    assert!(matches!(output.toks[1].tok, Token::Id(_)));
    assert_eq!(output.toks[1].leading_trivia_indices, 1..2);
    assert_eq!(output.trivia[1].kind, TriviaKind::Whitespace);
    assert_eq!(output.trivia[1].span.start, 3);
    assert_eq!(output.trivia[1].span.end, 5);

    assert_eq!(output.toks[2].tok, Token::EOF);
    assert_eq!(output.toks[2].leading_trivia_indices, 2..2);
}

#[test]
fn trivia_unclosed_multi_line_comment_at_eof() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    let src: &[u8] = b"/* unclosed comment at eof";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    assert_eq!(output.toks.len(), 1);
    assert_eq!(output.toks[0].tok, Token::EOF);
    assert_eq!(output.toks[0].leading_trivia_indices, 0..1);
    assert_eq!(output.trivia.len(), 1);
    assert_eq!(output.trivia[0].kind, TriviaKind::MultiComment);
    assert_eq!(output.trivia[0].span.start, 0);
    assert_eq!(output.trivia[0].span.end, src.len() as u32);
}

#[test]
fn trivia_token_variety_leading_trivia_association() {
    let mut interner = Intern::init();
    let mut cfg = ChrnConfig::default();

    // Exercise diverse token types each preceded by a unique whitespace span:
    // Id, Integer, Float, Str, Char, Bool, and compound symbols
    let src: &[u8] = b"  ident  100  3.14  \"hello\"  'c'  true  ::  :=  ->  =>  ..=";
    let output = Lexer::new(SourceRegionId::new(0), PathId::new(0), src, 0, &mut cfg).tokenize(&mut interner);

    // 11 semantic tokens + 1 EOF = 12 tokens
    // Each of the 11 semantic tokens has exactly 1 leading whitespace trivia
    assert_eq!(output.toks.len(), 12);
    assert_eq!(output.trivia.len(), 11);

    for (i, tok) in output.toks[..11].iter().enumerate() {
        assert_eq!(
            tok.leading_trivia_indices,
            (i as u32)..(i as u32 + 1),
            "token {i} ({:?}) should have leading_trivia_indices {}..{}",
            tok.tok,
            i,
            i + 1
        );
        assert_eq!(
            output.trivia[i].kind,
            TriviaKind::Whitespace,
            "trivia {i} should be Whitespace"
        );
    }

    // EOF has no leading trivia (it immediately follows the last token)
    assert_eq!(output.toks[11].leading_trivia_indices, 11..11);
}
