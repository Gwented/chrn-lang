use std::{borrow::Cow, collections::HashSet};

use chrn_utils::{
    err_codes::ErrorCode,
    id_types::{DirectiveId, ModuleId},
    intern,
    intern::Intern,
    source_map::{source_diagnostic::DiagnosticLevel, source_span::SourceSpan},
    utils::containers::SpannedContainer,
};
use lang::directive_consts::BUILTIN_DIRECTIVE_STRS;

use super::helpers::{get_module_region, mock_single_module_compiler};
use crate::{
    script_compiler::compiler_consts::directive_to_id_const,
    semantic::{
        arbitraries::ArbitraryIntKind::{self, I64, U64},
        hir::{
            hir_concepts::IntegerRangeKind,
            hir_directives::{
                Directive, DirectiveUnparameterized, DirectivePreprocessExpectInputKind,
                DirectivePreprocessSchema, DirectivePreprocessSchemaKind, TypeDirective,
                directive_preprocess_args::{
                    DirectiveIntegerArgConstraint, DirectiveIntegerArgConstraintErrorKind,
                    DirectivePreprocessArgConstraint, DirectivePreprocessConstraintErrorKind,
                },
            },
        },
        preset_reporter::{preset_err::DirectiveError, report_preset},
        values::ValueKind,
    },
};

//NOTE: Array positions are the stable pre-registered inline directive IDs. Preprocess
// directives are recognized through schemas and do not occupy these arena slots.
const INLINE_DIRECTIVES: [(&str, DirectiveUnparameterized); 7] = [
    ("warn", DirectiveUnparameterized::Warn),
    ("ignore", DirectiveUnparameterized::Ignore),
    ("scient", DirectiveUnparameterized::Type(TypeDirective::Scient)),
    ("hex", DirectiveUnparameterized::Type(TypeDirective::Hex)),
    ("bin", DirectiveUnparameterized::Type(TypeDirective::Bin)),
    ("octal", DirectiveUnparameterized::Type(TypeDirective::Octal)),
    ("unicode", DirectiveUnparameterized::Type(TypeDirective::Unicode)),
];

#[test]
fn builtin_directives_are_preloaded_and_recognized_by_their_family() {
    let expected_names: HashSet<_> = INLINE_DIRECTIVES
        .iter()
        .map(|(name, _)| *name)
        .chain(["max_numeric_bits"])
        .collect();
    let actual_names: HashSet<_> = BUILTIN_DIRECTIVE_STRS.iter().copied().collect();
    assert_eq!(actual_names, expected_names);
    assert_eq!(BUILTIN_DIRECTIVE_STRS.len(), expected_names.len());

    let interner = Intern::init();
    for &name in &BUILTIN_DIRECTIVE_STRS {
        let id = interner
            .get_interned_id_from_str(name)
            .unwrap_or_else(|| panic!("directive '{name}' is not preloaded"));
        assert!(
            (id.id as usize) < intern::INTERNER_PRELOAD_SIZE,
            "directive '{name}' has interned ID {} outside preload size {}",
            id.id,
            intern::INTERNER_PRELOAD_SIZE,
        );

        let inline = DirectiveUnparameterized::try_from_interned_str(id);
        let preprocess =
            DirectivePreprocessSchema::try_from_interned_id(id).map(|schema| schema.kind);
        if name == "max_numeric_bits" {
            assert_eq!(
                (inline, preprocess),
                (None, Some(DirectivePreprocessSchemaKind::MaxNumericBits)),
                "directive '{name}'",
            );
        } else {
            let expected = INLINE_DIRECTIVES
                .iter()
                .find(|(expected_name, _)| *expected_name == name)
                .unwrap()
                .1;
            assert_eq!(
                (inline, preprocess),
                (Some(expected), None),
                "directive '{name}'"
            );
        }
    }
}

#[test]
fn inline_directive_ids_match_pre_registered_arena_entries() {
    let (_, _, _, compiler) = mock_single_module_compiler("");
    assert_eq!(compiler.directives.len(), INLINE_DIRECTIVES.len());
    for (index, (name, expected)) in INLINE_DIRECTIVES.iter().enumerate() {
        let id = DirectiveId::new(index as u32);
        assert_eq!(directive_to_id_const(expected), id, "directive '{name}'");
        let Directive::Inline(actual) = &compiler.directives[id] else {
            panic!("directive '{name}' has a preprocess entry at inline ID {index}");
        };
        assert_eq!(actual, expected, "directive '{name}' at ID {index}");
    }
}

#[test]
fn integer_constraint_preserves_named_and_explicit_range_boundaries() {
    for (range, accepted, rejected) in [
        (
            IntegerRangeKind::U8,
            vec![I64(0), I64(i64::from(u8::MAX)), U64(u64::from(u8::MAX))],
            vec![I64(-1), U64(u64::from(u8::MAX) + 1)],
        ),
        (
            IntegerRangeKind::U16,
            vec![I64(0), I64(i64::from(u16::MAX)), U64(u64::from(u16::MAX))],
            vec![I64(-1), U64(u64::from(u16::MAX) + 1)],
        ),
        (
            IntegerRangeKind::U64,
            vec![I64(0), I64(i64::MAX), U64(u64::MAX)],
            vec![I64(-1)],
        ),
        (
            IntegerRangeKind::I8,
            vec![I64(i64::from(i8::MIN)), I64(-1), U64(i8::MAX as u64)],
            vec![I64(i64::from(i8::MIN) - 1), I64(i64::from(i8::MAX) + 1)],
        ),
        (
            IntegerRangeKind::I16,
            vec![I64(i64::from(i16::MIN)), I64(-1), U64(i16::MAX as u64)],
            vec![I64(i64::from(i16::MIN) - 1), I64(i64::from(i16::MAX) + 1)],
        ),
        (
            IntegerRangeKind::I64,
            vec![I64(i64::MIN), I64(-1), I64(i64::MAX), U64(i64::MAX as u64)],
            vec![
                ArbitraryIntKind::BigInt(dashu_int::IBig::from(i64::MIN) - 1),
                U64(i64::MAX as u64 + 1),
                U64(u64::MAX),
            ],
        ),
        (
            IntegerRangeKind::U32,
            vec![I64(0), I64(i64::from(u32::MAX)), U64(u64::from(u32::MAX))],
            vec![I64(-1), U64(u64::from(u32::MAX) + 1), U64(u64::MAX)],
        ),
        (
            IntegerRangeKind::I32,
            vec![I64(i64::from(i32::MIN)), I64(-1), U64(i32::MAX as u64)],
            vec![
                I64(i64::from(i32::MIN) - 1),
                I64(i64::from(i32::MAX) + 1),
                U64(u64::MAX),
            ],
        ),
        (
            IntegerRangeKind::RangeI64(-2..2),
            vec![I64(-2), I64(-1), U64(1)],
            vec![I64(-3), I64(2), U64(u64::MAX)],
        ),
        (
            IntegerRangeKind::RangeU64((u64::MAX - 2)..u64::MAX),
            vec![U64(u64::MAX - 2), U64(u64::MAX - 1)],
            vec![I64(-2), U64(u64::MAX - 3), U64(u64::MAX)],
        ),
    ] {
        let constraint = DirectivePreprocessArgConstraint::Integer(
            DirectiveIntegerArgConstraint::new(Some(range.clone())),
        );
        for value in accepted {
            let input = DirectivePreprocessExpectInputKind::Integer(Cow::Borrowed(&value));
            assert!(constraint.allows(&input).is_ok(), "{range}: {value}");
        }
        for value in rejected.into_iter().chain([ArbitraryIntKind::BigInt(
            dashu_int::IBig::from(u64::MAX) + 1,
        )]) {
            let input = DirectivePreprocessExpectInputKind::Integer(Cow::Borrowed(&value));
            let error = constraint
                .allows(&input)
                .expect_err(&format!("{range}: {value}"));
            let DirectivePreprocessConstraintErrorKind::Integer(
                DirectiveIntegerArgConstraintErrorKind::RangeMismatch { expected },
            ) = error
            else {
                panic!("{range}: {value}: wrong constraint error: {error:?}");
            };
            assert_eq!(expected, range, "{value}: rejected range identity was lost");
        }
    }
}

#[test]
fn integer_constraint_diagnostics_preserve_type_names_and_explicit_ranges() {
    let (arena, interner, settings, compiler) = mock_single_module_compiler("18446744073709551616");
    let region = get_module_region(&arena, &compiler.mods[ModuleId::new(0)]);
    let value = ArbitraryIntKind::BigInt(dashu_int::IBig::from(u64::MAX) + 1);
    let input = DirectivePreprocessExpectInputKind::Integer(Cow::Borrowed(&value));
    let span = SourceSpan::new(region.self_id, 0, 20);
    for (range, name) in [
        (IntegerRangeKind::U8, "u8"),
        (IntegerRangeKind::U16, "u16"),
        (IntegerRangeKind::U64, "u64"),
        (IntegerRangeKind::I8, "i8"),
        (IntegerRangeKind::I16, "i16"),
        (IntegerRangeKind::I64, "i64"),
        (IntegerRangeKind::U32, "u32"),
        (IntegerRangeKind::I32, "i32"),
        (IntegerRangeKind::RangeI64(-2..2), "-2..2"),
        (IntegerRangeKind::RangeU64(0..2), "0..2"),
    ] {
        let constraint = DirectivePreprocessArgConstraint::Integer(
            DirectiveIntegerArgConstraint::new(Some(range)),
        );
        let mut diagnostics = Vec::new();
        report_preset(
            &compiler,
            &mut diagnostics,
            DirectiveError::ParameterMismatch {
                direct_schema_kind: DirectivePreprocessSchemaKind::MaxNumericBits,
                arg_idx: 0,
                constraint_err: constraint.allows(&input).unwrap_err(),
                sp_found_val_kind: SpannedContainer::new(ValueKind::BigInt, span),
            }
            .into(),
            region,
            &settings,
            &interner,
        );
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(
            diagnostic.core_msg,
            format!("expected value in range `{name}`, found `BigInt`")
        );
        assert_eq!(diagnostic.err_code, Some(ErrorCode::DirectiveErr));
        assert_eq!(diagnostic.level, DiagnosticLevel::Error);
        assert_eq!(diagnostic.path_id, region.path_id);
        assert_eq!(diagnostic.annotations.len(), 1);
        assert_eq!(diagnostic.annotations[0].span, span);
    }
}
