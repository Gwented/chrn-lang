use crate::config_loader::{ConfigLoader, ConfigLoaderOutput};

use super::helpers::*;
use chrn_utils::source_map::source_diagnostic::annotations::AnnotationKind;

#[test]
fn cfg_at_def_no_separator_before_at_end_test() {
    let res = load_cfg("@def@end").expect_success();
    assert_eq!(res.script_start, 0);
    assert_eq!(res.serial_start, Some(8));

    let res = load_cfg(" @def@end ").expect_success();
    assert_eq!(res.script_start, 1);
    assert_eq!(res.serial_start, Some(9));

    let res = load_cfg("@def \t@end\n\t").expect_success();
    assert_eq!(res.script_start, 0);
    assert_eq!(res.serial_start, Some(10));

    let res = load_cfg(" @def @end").expect_success();
    assert_eq!(res.script_start, 1);
    assert_eq!(res.serial_start, Some(10));

    let res = load_cfg(" @def @end ").expect_success();
    assert_eq!(res.script_start, 1);
    assert_eq!(res.serial_start, Some(10));

    let res = load_cfg("@def\t@\re\rnd");
    match res {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            assert_eq!(region.script_start, 0, "@def at offset 0 in broken case");
            assert!(region.serial_start.is_none(), "Broken => no serial_start");
            assert!(
                !diag.annotations.is_empty(),
                "diagnostic must annotate the @def span"
            );
        }
        other => panic!("Expected Broken with Diagnostic, got {other:?}"),
    }
}

/// A file beginning with `@end` is treated as an empty chrn embedding.
/// The serialized-data offset is immediately after the marker.
#[test]
fn cfg_at_end_without_at_def_marks_empty_embedding_test() {
    let res = load_cfg("@end").expect_success();
    assert_eq!(res.src_bytes, b"@end");
    assert_eq!(res.serial_start, Some(4));
    assert_eq!(res.script_start, 0);
}

/// Marker-like text inside a quoted string is treated as string content rather than as a
/// configuration marker.
#[test]
fn cfg_at_sign_inside_string_is_not_a_marker_test() {
    let input = r#""this has @def inside it" remaining"#;
    let res = load_cfg(input).expect_success();
    assert_eq!(res.script_start, 0, "No @def was ever matched");
    assert!(res.serial_start.is_none(), "No @def was ever matched");
    assert_eq!(
        std::str::from_utf8(&res.src_bytes).unwrap(),
        input,
        "src_bytes must preserve the entire input verbatim"
    );
}

/// Comment delimiters inside a quoted string are treated as string content.
#[test]
fn cfg_multi_comment_syntax_inside_string_is_not_comment_test() {
    let input = r#""/* still just text " trailing"#;
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(
        std::str::from_utf8(&res.src_bytes).unwrap(),
        input,
        "src_bytes must preserve the entire input verbatim"
    );
}

/// Marker-like text inside a line comment is ignored by the loader.
#[test]
fn cfg_at_def_inside_line_comment_is_ignored_test() {
    let input = "// @def @end\nreal code\n";
    let res = load_cfg(input).expect_success();
    assert!(
        res.serial_start.is_none(),
        "@def inside a // comment must not open a block"
    );
    assert_eq!(
        std::str::from_utf8(&res.src_bytes).unwrap(),
        input,
        "src_bytes must preserve the entire input verbatim"
    );
}

/// Marker-like text inside a multi-line comment is ignored, including when comment
/// delimiters are nested.
#[test]
fn cfg_at_def_inside_multi_comment_is_ignored_test() {
    let input = "/* @def @end */\nreal\n";
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(std::str::from_utf8(&res.src_bytes).unwrap(), input);

    let input = "/*@def @end*/\nreal\n";
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(std::str::from_utf8(&res.src_bytes).unwrap(), input);

    let input = "/*@def@end*/\r\nreal\n\x25";
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(std::str::from_utf8(&res.src_bytes).unwrap(), input);
}

/// An escape inside a quoted string consumes the following byte as string content and
/// does not cause an escaped quote to terminate the string.
#[test]
fn cfg_escape_sequence_in_string_test() {
    let input = r#""a\b" after"#;
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(
        std::str::from_utf8(&res.src_bytes).unwrap(),
        input,
        "src_bytes must preserve the entire input verbatim"
    );
}

/// An unterminated double-quoted string produces a diagnostic whose primary span identifies
/// the opening delimiter.
#[test]
fn cfg_unclosed_double_quote_errors_test() {
    let res = load_cfg("hello \"world");
    match res {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            let primary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Primary)
                .expect("diagnostic should have a primary annotation");
            assert_eq!(
                primary.span.start, 6,
                "span should point at the opening double-quote"
            );
            assert_eq!(primary.span.end, 7, "span should be exactly 1 byte");
        }
        other => panic!("Expected unclosed-quote error with Diagnostic, got {other:?}"),
    }
}

/// An unterminated single-quoted string produces a diagnostic whose primary span identifies
/// the opening delimiter.
#[test]
fn cfg_unclosed_single_quote_errors_test() {
    let res = load_cfg("hello 'world");
    match res {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            let primary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Primary)
                .expect("diagnostic should have a primary annotation");
            assert_eq!(
                primary.span.start, 6,
                "span should point at the opening single-quote"
            );
            assert_eq!(primary.span.end, 7, "span should be exactly 1 byte");
        }
        other => panic!("Expected unclosed-quote error with Diagnostic, got {other:?}"),
    }
}

/// A trailing escape in a quoted string leaves the string unterminated and produces a
/// diagnostic for the opening delimiter.
#[test]
fn cfg_escape_at_eof_in_string_errors_test() {
    let res = load_cfg(r#""abc\"#);
    match res {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            let primary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Primary)
                .expect("diagnostic should have a primary annotation");
            assert_eq!(
                primary.span.start, 0,
                "span should point at the opening double-quote"
            );
            assert_eq!(primary.span.end, 1, "span should be exactly 1 byte");
        }
        other => panic!("Expected unclosed-quote error with Diagnostic, got {other:?}"),
    }
}

/// Empty input yields a valid empty region without embedding offsets.
#[test]
fn cfg_empty_file_test() {
    let res = load_cfg("").expect_success();
    assert_eq!(res.src_bytes, []);
    assert_eq!(res.script_start, 0);
    assert!(res.serial_start.is_none());
}

/// Line comments terminated by CRLF preserve source bytes and do not interfere with
/// subsequent source scanning.
#[test]
fn cfg_crlf_line_endings_test() {
    let input = "\r//\r\r\r header\r\nlet A = 1\r\nlet B = 2\r\n";
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(res.script_start, 0, "no @def seen");
    assert_eq!(
        std::str::from_utf8(&res.src_bytes).unwrap(),
        input,
        "src_bytes must preserve the entire input verbatim"
    );
}

/// An incomplete marker at EOF is treated as ordinary source text and does not produce
/// a loader error.
#[test]
fn cfg_lone_at_sign_at_eof_test() {
    let res = load_cfg("some text @").expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(res.script_start, 0);
    assert_eq!(std::str::from_utf8(&res.src_bytes).unwrap(), "some text @");
}

/// Unmatched `@` characters remain ordinary source text; only complete recognized markers
/// affect embedding boundaries.
#[test]
fn cfg_many_at_signs_in_a_row_test() {
    let input = "@@@@@@@@@@@@ plain@ @ text @@@@@@@@@@@@";
    let res = load_cfg(input).expect_success();
    assert!(res.serial_start.is_none());
    assert_eq!(res.script_start, 0);
    assert_eq!(std::str::from_utf8(&res.src_bytes).unwrap(), input);
}

/// An unterminated multi-line comment returns a broken region with a diagnostic identifying
/// both the opening delimiter and the unexpected EOF.
#[test]
fn cfg_unclosed_multi_line_comment_test() {
    let res = load_cfg("/* this comment never ends");
    match res {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            let secondaries: Vec<_> = diag
                .annotations
                .iter()
                .filter(|a| a.kind == AnnotationKind::Secondary)
                .collect();
            let primaries: Vec<_> = diag
                .annotations
                .iter()
                .filter(|a| a.kind == AnnotationKind::Primary)
                .collect();
            assert_eq!(
                secondaries.len(),
                1,
                "one secondary annotation for comment start"
            );
            assert_eq!(
                secondaries[0].span.start, 0,
                "secondary points at opening `/`"
            );
            assert_eq!(primaries.len(), 1, "one primary annotation for EOF");
        }
        other => panic!("Broken with Diagnostic expected, got {other:?}"),
    }
}

/// Tab characters are preserved as ordinary source bytes and do not alter marker recognition
/// or offset accounting.
#[test]
fn cfg_tab_characters_around_at_def_test() {
    let res = load_cfg("\t@def\tva\nr->\tx:\ti3\r2\t\r\u{32}@end\t");
    match res {
        ConfigLoaderOutput::Success(region, _) => {
            assert_eq!(region.script_start, 1);
            assert_eq!(region.serial_start, Some(27));
            let s = std::str::from_utf8(&region.src_bytes).unwrap();
            assert!(s.contains("@def"));
            assert!(s.contains("@end"));
        }
        other => panic!("Loader errored on tab-separated @def/@end: {other:?}"),
    }
}

/// Nested multi-line comments are accepted when all delimiters close; an unclosed comment
/// returns a broken region with a diagnostic.
#[test]
fn multi_line_comment_test() {
    // Properly closed nested multi-line comment.
    let correct_input = "
            /* /* */ */
        "
    .as_bytes();

    // Unclosed nested multi-line comment.
    let wrong_input = "
            /* /* */
        "
    .as_bytes();

    let region_id = SourceRegionId::new(0);

    let correct = ConfigLoader::new(
        region_id,
        correct_input,
        PathId::default(),
        &ChrnConfig::default(),
    )
    .load_config();

    let wrong = ConfigLoader::new(
        region_id,
        wrong_input,
        PathId::default(),
        &ChrnConfig::default(),
    )
    .load_config();

    let correct_region = correct.expect_success();
    assert_eq!(
        correct_region.src_bytes, correct_input,
        "correct multi-comment: src_bytes must match input verbatim"
    );

    match wrong {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            let primaries: Vec<_> = diag
                .annotations
                .iter()
                .filter(|a| a.kind == AnnotationKind::Primary)
                .collect();
            assert!(
                !primaries.is_empty(),
                "unclosed multi-comment diagnostic needs a primary annotation"
            );
        }
        other => {
            panic!("unclosed multi-line comment should produce Broken, got {other:?}")
        }
    }
}

#[test]
fn start_and_serial_offset_test() {
    let text = format!("adwh@def var-> int: i32 @endhi");
    let region_id = SourceRegionId::new(0);

    let metadata = ConfigLoader::new(
        region_id,
        text.as_bytes(),
        PathId::default(),
        &ChrnConfig::default(),
    )
    .load_config()
    .expect_success();

    assert_eq!(&text[4..], &text[metadata.script_start..]);
    assert_eq!("hi", &text[metadata.serial_start.unwrap()..]);
    assert_eq!(28, metadata.serial_start.unwrap());
}

/// An escape sequence that reaches the region limit must not let scanning exceed the cap.
/// The loader reports an unclosed quote with an in-bounds opening-quote span.
#[test]
fn cfg_escape_straddling_read_limit_stops_at_cap_test() {
    const LIMIT: usize = chrn_utils::MAX_REGION_SIZE; // 32KB == 32768
    const TOTAL: usize = 64 * 1024;

    // Place the escape at the final byte in the region budget and its target beyond the budget.
    let mut input = String::with_capacity(TOTAL);
    input.push('"');
    input.extend(std::iter::repeat('a').take(LIMIT - 2));
    input.push('\\');
    input.push('x');
    input.push('"');
    assert_eq!(input.len(), LIMIT + 2);
    input.extend(std::iter::repeat('a').take(TOTAL - input.len()));
    assert_eq!(input.len(), TOTAL);
    assert_eq!(
        input.as_bytes()[LIMIT - 1],
        b'\\',
        "escape must sit at the final byte in the region budget"
    );

    match load_cfg(&input) {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            let primary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Primary)
                .expect("unclosed-quote diagnostic should have a primary annotation");
            assert_eq!(
                primary.span.start, 0,
                "span must point at the opening quote"
            );
            assert_eq!(primary.span.end, 1, "span must be exactly 1 byte");
            assert!(
                region.src_bytes.len() <= LIMIT,
                "loader must stop at the 32KB cap, got {} bytes",
                region.src_bytes.len()
            );
            assert!(
                region.src_bytes.len() < input.len(),
                "loader must not consume the entire 64KB buffer, consumed {}",
                region.src_bytes.len()
            );
        }
        other => {
            panic!("escape straddling the read limit must produce Broken (cap held), got {other:?}")
        }
    }
}

/// An unclosed multi-line comment at EOF must produce diagnostic spans within the recovered
/// region, including when the opening delimiter reaches EOF.
#[test]
fn cfg_unclosed_multi_comment_eof_span_bounds_test() {
    for (input, expected_span) in [("/*", 0..2), ("/* this comment never ends", 25..26)] {
        let len = input.len() as u32;
        let res = load_cfg(input);
        match res {
            ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
                let secondaries: Vec<_> = diag
                    .annotations
                    .iter()
                    .filter(|a| a.kind == AnnotationKind::Secondary)
                    .collect();
                let primaries: Vec<_> = diag
                    .annotations
                    .iter()
                    .filter(|a| a.kind == AnnotationKind::Primary)
                    .collect();
                assert_eq!(
                    secondaries.len(),
                    1,
                    "one secondary for comment start in {input:?}"
                );
                assert_eq!(
                    primaries.len(),
                    1,
                    "one primary for comment start in {input:?}"
                );

                let secondary = secondaries[0];
                assert_eq!(
                    secondary.span.start, 0,
                    "secondary must start at `/` in {input:?}"
                );
                assert_eq!(
                    secondary.span.end - secondary.span.start,
                    2,
                    "secondary must cover both bytes of `/*` in {input:?}"
                );

                assert_eq!(primaries.len(), 1, "one primary for EOF in {input:?}");
                let primary = primaries[0];
                assert_eq!(
                    primary.span.start..primary.span.end,
                    expected_span,
                    "primary must cover the final byte in {input:?}"
                );
                assert_eq!(
                    primary.span.end, len,
                    "primary EOF span must remain in bounds in {input:?}"
                );
            }
            other => {
                panic!("unclosed multi-line comment must produce Broken, got {other:?}")
            }
        }
    }
}

/// A marker-less input may exceed the region limit without being malformed; the loader returns
/// a successful region containing only the bytes within the limit.
#[test]
fn cfg_oversized_markerless_file_is_capped_test() {
    const LIMIT: usize = chrn_utils::MAX_REGION_SIZE;
    let input = vec![b'a'; LIMIT + 1024];
    assert_eq!(input.len(), 33 * 1024);

    match load_cfg_bytes(&input) {
        ConfigLoaderOutput::Success(region, _) => {
            assert_eq!(
                region.src_bytes.len(),
                LIMIT,
                "marker-less region must be capped at exactly 32KiB"
            );
            assert_eq!(region.src_bytes, input[..LIMIT]);
            assert_eq!(region.script_start, 0);
            assert!(region.serial_start.is_none());
        }
        other => panic!("oversized marker-less input must remain loadable, got {other:?}"),
    }
}

/// An escaped newline in quoted text updates line and column metadata as a newline before
/// the loader records a following embedding's starting position.
#[test]
fn cfg_backslash_newline_in_string_bumps_line_test() {
    let input = "\"\\\n\" @def@end";
    assert_eq!(
        &input.as_bytes()[1..3],
        b"\\\n",
        "test input must contain backslash-newline"
    );

    let region = load_cfg(input).expect_success();
    assert_eq!(region.script_start, 5, "script must start at `@def`");
    assert_eq!(
        region.serial_start,
        Some(13),
        "serial must start one past `@end`"
    );
    assert_eq!(
        region.abs_ln_num_start, 2,
        "escaped newline must increment the region start line (got line {})",
        region.abs_ln_num_start
    );
    assert_eq!(
        region.abs_col_start, 3,
        "column must reset after the escaped newline (got col {})",
        region.abs_col_start
    );
    assert_eq!(region.src_bytes, b"@def@end");
}

/// A complete embedding preserves its source boundaries, serialized-data offset, and starting
/// line and column metadata.
#[test]
fn cfg_at_def_end_single_region_offsets_test() {
    let input = "pre @def body @endpost";

    let region = load_cfg(input).expect_success();
    assert_eq!(region.script_start, 4, "script must start at `@def`");
    assert_eq!(
        region.serial_start,
        Some(18),
        "serial must start one past `@end`"
    );
    assert_eq!(
        region.src_bytes, b"@def body @end",
        "region bytes must be exact"
    );
    assert_eq!(
        &input[region.serial_start.unwrap()..],
        "post",
        "trailing serial slice must be exact"
    );
    assert_eq!(region.abs_ln_num_start, 1, "no newlines precede `@def`");
    assert_eq!(
        region.abs_col_start, 5,
        "four `pre ` advances precede `@def`"
    );
}

/// The 32KiB limit applies to the chrn region beginning at `@def`, not to its
/// absolute position in a file that contains a serialized-data prefix.
#[test]
fn cfg_at_end_budget_is_relative_to_nonzero_script_start_test() {
    const LIMIT: usize = chrn_utils::MAX_REGION_SIZE;
    const PREFIX_LEN: usize = 100;
    const END_START: usize = LIMIT - 3;

    let mut input = vec![b'p'; PREFIX_LEN];
    input.extend_from_slice(b"@def");
    input.extend(std::iter::repeat_n(b'a', END_START - input.len()));
    assert_eq!(input.len(), END_START, "`@end` boundary setup drifted");
    input.extend_from_slice(b"@end");

    let region = match load_cfg_bytes(&input) {
        ConfigLoaderOutput::Success(region, _) => region,
        ConfigLoaderOutput::Broken(_, err) => {
            panic!("in-budget `@end` must close the region, got Broken: {err:?}")
        }
        ConfigLoaderOutput::UnrecoverableErr(err) => {
            panic!("in-budget `@end` must close the region, got UnrecoverableErr: {err:?}")
        }
    };
    assert_eq!(region.script_start, PREFIX_LEN);
    assert_eq!(region.serial_start, Some(END_START + 4));
    assert_eq!(
        region.src_bytes.len(),
        END_START + 4 - PREFIX_LEN,
        "the complete in-budget region must be retained"
    );
    assert_eq!(&region.src_bytes[..4], b"@def");
    assert_eq!(&region.src_bytes[region.src_bytes.len() - 4..], b"@end");
}

/// Diagnostics attached to a recovered region use offsets relative to that region.
/// A nonzero `script_start` must not be included in the missing-`@end` EOF span.
#[test]
fn cfg_missing_at_end_eof_span_is_region_relative_test() {
    let input = "prefix@def xyz";

    match load_cfg(input) {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            assert_eq!(region.script_start, 6);
            assert_eq!(region.src_bytes, b"@def xyz");

            let primary = diag
                .annotations
                .iter()
                .find(|annotation| annotation.kind == AnnotationKind::Primary)
                .expect("missing-`@end` diagnostic must identify the unexpected EOF");
            let region_len = region.src_bytes.len() as u32;
            assert_eq!(
                primary.span.start..primary.span.end,
                region_len - 1..region_len,
                "EOF span must cover the final byte relative to the recovered region"
            );
            assert_eq!(
                primary.span.end as usize + region.script_start,
                input.len(),
                "converting the relative EOF span back to file coordinates must reach true EOF"
            );
        }
        other => panic!("missing `@end` must return a broken region and diagnostic, got {other:?}"),
    }
}

/// A multi-line comment delimiter that crosses the region limit is not recognized as complete.
/// Out-of-budget lookahead must not close an in-budget comment.
#[test]
fn cfg_multi_comment_close_straddling_read_limit_is_unclosed_test() {
    const LIMIT: usize = chrn_utils::MAX_REGION_SIZE;

    let mut input = Vec::with_capacity(LIMIT + 1);
    input.extend_from_slice(b"/*");
    input.extend(std::iter::repeat_n(b'a', LIMIT - 3));
    input.extend_from_slice(b"*/");
    assert_eq!(input.len(), LIMIT + 1);
    assert_eq!(input[LIMIT - 1], b'*');
    assert_eq!(input[LIMIT], b'/');

    match load_cfg_bytes(&input) {
        ConfigLoaderOutput::Broken(_, ConfigLoadError::Diagnostic(diag)) => {
            assert_eq!(diag.core_msg, "Unclosed multi-line comment");
            let primary = diag
                .annotations
                .iter()
                .find(|annotation| annotation.kind == AnnotationKind::Primary)
                .expect("straddling comment diagnostic must identify the in-budget EOF");
            assert_eq!(
                primary.span.start..primary.span.end,
                (LIMIT - 1) as u32..LIMIT as u32,
                "the diagnostic must stop at the last byte inside the region budget"
            );
        }
        ConfigLoaderOutput::Success(_, _) => {
            panic!("a comment closed only beyond the region limit must not produce loader success")
        }
        ConfigLoaderOutput::UnrecoverableErr(err) => {
            panic!(
                "an unclosed multi-line comment must produce Broken, got UnrecoverableErr: {err:?}"
            )
        }
        ConfigLoaderOutput::Broken(_, err) => {
            panic!("expected diagnostic for unclosed multi-line comment, got: {err:?}")
        }
    }
}

/// Region-relative lookahead remains valid after a serialized-data prefix, so a valid
/// multi-line comment in the embedding is recognized and the embedding closes successfully.
#[test]
fn cfg_multi_comment_past_search_limit_with_prefix_succeeds_test() {
    const PREFIX_LEN: usize = 32_000;
    let mut input = vec![b'p'; PREFIX_LEN];
    input.extend_from_slice(b"@def ");
    input.extend(std::iter::repeat_n(b' ', 1_000));
    input.extend_from_slice(b"/* valid closed comment */ @end");

    let region = load_cfg_bytes(&input).expect_success();
    assert_eq!(region.script_start, PREFIX_LEN);
    assert!(region.src_bytes.ends_with(b"@end"));
}

/// Region-relative lookahead remains valid after a serialized-data prefix, so escaped quotes
/// remain inside quoted strings and a valid embedding closes successfully.
#[test]
fn cfg_string_escape_past_search_limit_with_prefix_succeeds_test() {
    const PREFIX_LEN: usize = 32_000;
    let mut input = vec![b'p'; PREFIX_LEN];
    input.extend_from_slice(b"@def ");
    input.extend(std::iter::repeat_n(b' ', 1_000));
    input.extend_from_slice(b"\"escaped \\\" quote\" @end");

    let region = load_cfg_bytes(&input).expect_success();
    assert_eq!(region.script_start, PREFIX_LEN);
    assert!(region.src_bytes.ends_with(b"@end"));
}

/// A marker whose bytes cross the region limit is not recognized as a complete `@end` marker.
/// The loader returns a bounded broken region and diagnostic instead of exceeding the limit.
#[test]
fn cfg_at_end_straddling_read_limit_is_rejected_test() {
    const LIMIT: usize = chrn_utils::MAX_REGION_SIZE;
    let mut input = Vec::with_capacity(LIMIT + 16);
    input.extend_from_slice(b"@def");
    // Place the marker's first byte at the final byte in the region budget.
    input.extend(std::iter::repeat_n(b'a', (LIMIT - 1) - 4));
    assert_eq!(input.len(), LIMIT - 1);
    input.extend_from_slice(b"@end\nserial");

    match load_cfg_bytes(&input) {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            assert!(
                region.src_bytes.len() <= LIMIT,
                "region must not exceed 32KiB cap, got {} bytes",
                region.src_bytes.len()
            );
            assert_eq!(diag.core_msg, "Could not find `@end` after `@def`");
        }
        other => panic!("expected Broken with Diagnostic, got: {other:?}"),
    }
}

/// An unterminated comment retains all consumed bytes and keeps its EOF diagnostic span within
/// the recovered region.
#[test]
fn cfg_unclosed_multi_comment_retains_final_byte_and_in_bounds_span_test() {
    let input = "/* hello";
    match load_cfg(input) {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            assert_eq!(
                region.src_bytes,
                input.as_bytes(),
                "all input bytes must be retained in region.src_bytes"
            );
            let primary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Primary)
                .expect("must have primary annotation");
            assert_eq!(
                primary.span.end as usize,
                region.src_bytes.len(),
                "primary EOF span end must equal region length"
            );
            let sliced = &region.src_bytes[primary.span.start as usize..primary.span.end as usize];
            assert_eq!(
                sliced, b"o",
                "EOF span must point to the final byte of the region"
            );
        }
        other => panic!("expected Broken with Diagnostic, got: {other:?}"),
    }
}

/// Diagnostic spans for an unclosed comment are relative to the recovered region, even when
/// the embedding follows a serialized-data prefix.
#[test]
fn cfg_unclosed_multi_comment_inside_at_def_spans_are_region_relative_test() {
    let input = "prefix serialized data @def /* unclosed comment";
    match load_cfg(input) {
        ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(diag)) => {
            assert_eq!(region.script_start, 23);
            let secondary = diag
                .annotations
                .iter()
                .find(|a| a.kind == AnnotationKind::Secondary)
                .expect("must have comment start annotation");
            assert_eq!(
                secondary.span.start..secondary.span.end,
                5..7,
                "comment start span must be relative to recovered region"
            );
            let sliced =
                &region.src_bytes[secondary.span.start as usize..secondary.span.end as usize];
            assert_eq!(
                sliced, b"/*",
                "secondary span must point to /* in region bytes"
            );
        }
        other => panic!("expected Broken with Diagnostic, got: {other:?}"),
    }
}
