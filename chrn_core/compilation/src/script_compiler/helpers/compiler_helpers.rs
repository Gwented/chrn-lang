// Not really a deep distinction from core since core is compiler generated, but core is a library
// intrinsically needed by the compiler. Directives are not a part of a library of any kind, but
// what if it was?
// Beep

use chrn_utils::{id_types::InternedId, intern};

use crate::{
    script_compiler::compiler_consts,
    semantic::hir::hir_directives::{DirectiveInline, TypeDirective},
};

pub static DIRECTIVES_DATASET: [(InternedId, DirectiveInline);
    compiler_consts::DIRECTIVE_UNICODE_IDX + 1] = [
    (
        InternedId::new(intern::INTERNED_WARN),
        DirectiveInline::Warn,
    ),
    (
        InternedId::new(intern::INTERNED_IGNORE),
        DirectiveInline::Ignore,
    ),
    (
        InternedId::new(intern::INTERNED_SCIENT),
        DirectiveInline::Type(TypeDirective::Scient),
    ),
    (
        InternedId::new(intern::INTERNED_HEX),
        DirectiveInline::Type(TypeDirective::Hex),
    ),
    (
        InternedId::new(intern::INTERNED_BIN),
        DirectiveInline::Type(TypeDirective::Bin),
    ),
    (
        InternedId::new(intern::INTERNED_OCTAL),
        DirectiveInline::Type(TypeDirective::Octal),
    ),
    (
        InternedId::new(intern::INTERNED_UNICODE),
        DirectiveInline::Type(TypeDirective::Unicode),
    ),
];
