// Not really a deep distinction from core since core is compiler generated, but core is a library
// intrinsically needed by the compiler. Directives are not a part of a library of any kind, but
// what if it was?
// Beep

use chrn_utils::{id_types::InternedId, intern};

use crate::{
    script_compiler::compiler_consts,
    semantic::hir::directives::{Directive, DirectiveInline, TypeDirective},
};

pub static DIRECTIVES_DATASET: [(InternedId, Directive);
    compiler_consts::DIRECTIVE_UNICODE_IDX + 1] = [
    (
        InternedId::new(intern::INTERNED_WARN),
        Directive::Comptime(DirectiveInline::Warn),
    ),
    (
        InternedId::new(intern::INTERNED_IGNORE),
        Directive::Comptime(DirectiveInline::Ignore),
    ),
    (
        InternedId::new(intern::INTERNED_SCIENT),
        Directive::Comptime(DirectiveInline::Type(TypeDirective::Scient)),
    ),
    (
        InternedId::new(intern::INTERNED_HEX),
        Directive::Comptime(DirectiveInline::Type(TypeDirective::Hex)),
    ),
    (
        InternedId::new(intern::INTERNED_BIN),
        Directive::Comptime(DirectiveInline::Type(TypeDirective::Bin)),
    ),
    (
        InternedId::new(intern::INTERNED_OCTAL),
        Directive::Comptime(DirectiveInline::Type(TypeDirective::Octal)),
    ),
    (
        InternedId::new(intern::INTERNED_UNICODE),
        Directive::Comptime(DirectiveInline::Type(TypeDirective::Unicode)),
    ),
];
