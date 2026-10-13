use super::helpers::*;
use crate::lexer::notations::IntegerNotation;
use crate::parser::ast::{
    ast_concepts::{AbstractDecl, AbstractImpl, Item},
    ast_exprs::AstExpr,
};
use crate::semantic::hir::hir_directives::{
    DirectivePreprocess, DirectivePreprocessKind, lexer_processor::process_directive,
};

#[test]
fn preprocess_literal_syntax_agrees_with_parser() {
    for (arguments, spelling, notation, bits) in [
        ("(64)", "64", IntegerNotation::Decimal, 64),
        ("(64,)", "64", IntegerNotation::Decimal, 64),
        ("(0b1000000)", "1000000", IntegerNotation::Bin, 64),
        ("(0o100,)", "100", IntegerNotation::Octal, 64),
        ("(0x40)", "40", IntegerNotation::Hex, 64),
        (
            "( /* before */ 64 /* after */ , )",
            "64",
            IntegerNotation::Decimal,
            64,
        ),
        ("(0)", "0", IntegerNotation::Decimal, 0),
        (
            "(4294967295)",
            "4294967295",
            IntegerNotation::Decimal,
            u32::MAX,
        ),
    ] {
        for with_neighbors in [false, true] {
            // Cover EOF without a final newline and a nonzero hash index with neighbors.
            let text = if with_neighbors {
                format!("let before = 1\n#max_numeric_bits{arguments}\nlet after = 7")
            } else {
                format!("#max_numeric_bits{arguments}")
            };
            let (arena, mut interner, mut settings, compiler) = mock_single_module_compiler(&text);
            let region = get_module_region(&arena, &compiler.mods[ModuleId::new(0)]);
            let output = Lexer::new(
                region.self_id,
                region.path_id,
                &region.src_bytes,
                region.script_start,
                &mut settings,
            )
            .tokenize(&mut interner);
            assert_eq!(output.found_invalid_toks, 0, "{arguments}");
            let hash_idx = output
                .toks
                .iter()
                .position(|t| t.tok == Token::HashSymbol)
                .unwrap();
            assert_eq!(
                process_directive(&output.toks, hash_idx, &interner),
                Some(DirectivePreprocess::new(
                    DirectivePreprocessKind::MaxNumericBits(bits)
                )),
                "{arguments}"
            );
            let (ast, summary) = parser::parse(&mut settings, region, &output.toks, &interner);
            assert!(summary.diags.is_empty(), "{arguments}: {:?}", summary.diags);
            assert_eq!(
                ast.items().len(),
                if with_neighbors { 3 } else { 1 },
                "{arguments}"
            );
            let Item::Impl(AbstractImpl::Directive(directive)) =
                &ast.items()[usize::from(with_neighbors)]
            else {
                panic!("{arguments}: expected preprocessed directive");
            };
            assert_eq!(
                interner.extract_str_from_interned_id(directive.sp_name_id.inner),
                "max_numeric_bits"
            );
            assert_eq!(
                &text[directive.sp_name_id.span.start as usize
                    ..directive.sp_name_id.span.end as usize],
                "max_numeric_bits"
            );
            assert_eq!(
                &text[directive.sp_args.span.start as usize..directive.sp_args.span.end as usize],
                arguments
            );
            assert_eq!(directive.sp_args.inner.len(), 1, "{arguments}");
            let AstExpr::Integer(id, actual_notation) = directive.sp_args.inner[0].inner else {
                panic!("{arguments}: expected integer argument");
            };
            assert_eq!(actual_notation, notation, "{arguments}");
            assert_eq!(
                interner.extract_str_from_interned_id(id),
                spelling,
                "{arguments}"
            );
            if with_neighbors {
                for (index, name, value) in [(0, "before", "1"), (2, "after", "7")] {
                    let Item::Decl(AbstractDecl::Var(var)) = &ast.items()[index] else {
                        panic!("{arguments}: expected preserved {name} declaration");
                    };
                    assert_eq!(interner.extract_str_from_interned_id(var.name_id), name);
                    let AstExpr::Integer(id, IntegerNotation::Decimal) = var.spanned_expr.inner
                    else {
                        panic!("{arguments}: expected preserved {value} value");
                    };
                    assert_eq!(interner.extract_str_from_interned_id(id), value);
                }
            }
        }
    }
}

#[test]
fn preprocess_schema_rejection_is_distinct_from_parser_syntax() {
    // The parser accepts expression lists; the early processor requires a registered
    // name, exactly one integer token, and the max_numeric_bits schema's range.
    for text in [
        "#unknown(64)",
        "#warn(64)",
        "#max_numeric_bits()",
        "#max_numeric_bits(64,32)",
        "#max_numeric_bits(64,32,)",
        "#max_numeric_bits(\"64\")",
        "#max_numeric_bits(64.0)",
        "#max_numeric_bits(bits)",
        "#max_numeric_bits(32 + 32)",
        "#max_numeric_bits(-1)",
        "#max_numeric_bits(4294967296)",
        "#max_numeric_bits(18446744073709551616)",
    ] {
        let (arena, mut interner, mut settings, compiler) = mock_single_module_compiler(text);
        let region = get_module_region(&arena, &compiler.mods[ModuleId::new(0)]);
        let output = Lexer::new(
            region.self_id,
            region.path_id,
            &region.src_bytes,
            region.script_start,
            &mut settings,
        )
        .tokenize(&mut interner);
        assert_eq!(output.found_invalid_toks, 0, "{text}");
        assert_eq!(
            process_directive(&output.toks, 0, &interner),
            None,
            "{text}"
        );
        let (ast, summary) = parser::parse(&mut settings, region, &output.toks, &interner);
        assert!(summary.diags.is_empty(), "{text}: {:?}", summary.diags);
        assert_eq!(ast.items().len(), 1, "{text}");
        let Item::Impl(AbstractImpl::Directive(directive)) = &ast.items()[0] else {
            panic!("{text}: expected preprocessed directive");
        };
        let name_end = text.find('(').unwrap();
        assert_eq!(
            interner.extract_str_from_interned_id(directive.sp_name_id.inner),
            &text[1..name_end]
        );
        assert_eq!(
            &text[directive.sp_args.span.start as usize..directive.sp_args.span.end as usize],
            &text[name_end..]
        );
        let expected_len = match text {
            "#max_numeric_bits()" => 0,
            "#max_numeric_bits(64,32)" | "#max_numeric_bits(64,32,)" => 2,
            _ => 1,
        };
        assert_eq!(directive.sp_args.inner.len(), expected_len, "{text}");
    }
}

#[test]
fn preprocess_malformed_syntax_and_eof_are_rejected() {
    let mut panics = Vec::new();
    for (text, expected_message) in [
        ("#", "Expected identifier for directive, found `<eof>`"),
        ("#max_numeric_bits", "Expected opening '(', found `<eof>`"),
        ("#max_numeric_bits 64", "Expected opening '(', found \"64\""),
        ("#max_numeric_bits[64]", "Expected opening '(', found `[`"),
        ("#max_numeric_bits(", "Expected closing ')', found `<eof>`"),
        (
            "#max_numeric_bits(64",
            "Expected `,` to separate elements, found `<eof>`",
        ),
        (
            "#max_numeric_bits(64,",
            "Expected closing ')', found `<eof>`",
        ),
        (
            "#max_numeric_bits(,64)",
            "Expected a valid expression, found `,`",
        ),
        (
            "#max_numeric_bits(64,,)",
            "Expected a valid expression, found `,`",
        ),
        (
            "#max_numeric_bits(64 32)",
            "Expected `,` to separate elements, found \"32\"",
        ),
    ] {
        let (arena, mut interner, mut settings, compiler) = mock_single_module_compiler(text);
        let region = get_module_region(&arena, &compiler.mods[ModuleId::new(0)]);
        let output = Lexer::new(
            region.self_id,
            region.path_id,
            &region.src_bytes,
            region.script_start,
            &mut settings,
        )
        .tokenize(&mut interner);
        assert_eq!(output.found_invalid_toks, 0, "{text}");
        let (ast, summary) = parser::parse(&mut settings, region, &output.toks, &interner);
        assert!(
            ast.items().is_empty(),
            "{text}: malformed directive entered AST"
        );
        assert_eq!(summary.diags.len(), 1, "{text}: {:?}", summary.diags);
        assert_eq!(summary.diags[0].core_msg, expected_message, "{text}");
        // Use the lexer's actual EOF-terminated stream, including no final newline.
        match std::panic::catch_unwind(|| process_directive(&output.toks, 0, &interner)) {
            Ok(result) => assert_eq!(result, None, "{text}"),
            Err(_) => panics.push(text),
        }
    }
    assert!(
        panics.is_empty(),
        "malformed preprocessed directives panicked instead of returning None: {panics:?}"
    );
}
