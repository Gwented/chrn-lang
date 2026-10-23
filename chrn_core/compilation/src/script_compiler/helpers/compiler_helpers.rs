// Not really a deep distinction from core since core is compiler generated, but core is a library
// intrinsically needed by the compiler. Directives are not a part of a library of any kind, but
// what if it was?
// Beep

use chrn_utils::{id_types::InternedId, intern};

use crate::{
    script_compiler::compiler_consts,
    semantic::hir::hir_directives::{DirectiveUnparameterized, TypeDirective},
};

pub static DIRECTIVES_DATASET: [(InternedId, DirectiveUnparameterized);
    compiler_consts::DIRECTIVE_UNICODE_IDX + 1] = [
    (
        InternedId::new(intern::INTERNED_WARN),
        DirectiveUnparameterized::Warn,
    ),
    (
        InternedId::new(intern::INTERNED_IGNORE),
        DirectiveUnparameterized::Ignore,
    ),
    (
        InternedId::new(intern::INTERNED_SCIENT),
        DirectiveUnparameterized::Type(TypeDirective::Scient),
    ),
    (
        InternedId::new(intern::INTERNED_HEX),
        DirectiveUnparameterized::Type(TypeDirective::Hex),
    ),
    (
        InternedId::new(intern::INTERNED_BIN),
        DirectiveUnparameterized::Type(TypeDirective::Bin),
    ),
    (
        InternedId::new(intern::INTERNED_OCTAL),
        DirectiveUnparameterized::Type(TypeDirective::Octal),
    ),
    (
        InternedId::new(intern::INTERNED_UNICODE),
        DirectiveUnparameterized::Type(TypeDirective::Unicode),
    ),
];
