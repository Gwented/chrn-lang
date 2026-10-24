use chrn_utils::id_types::{InternedId, id_tags::UTF8Tag};
// Not really a deep distinction from core since core is compiler generated, but core is a library
// intrinsically needed by the compiler. Directives are not a part of a library of any kind, but
// what if it was?
// Beep

use chrn_utils::{id_types::InternedIdUTF8, intern};

use crate::{
    script_compiler::compiler_consts,
    semantic::hir::hir_directives::{DirectiveUnparameterized, TypeDirective},
};

pub static DIRECTIVES_DATASET: [(InternedIdUTF8, DirectiveUnparameterized);
    compiler_consts::DIRECTIVE_UNICODE_IDX + 1] = [
    (
        InternedId::new(intern::INTERNED_WARN).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Warn,
    ),
    (
        InternedId::new(intern::INTERNED_IGNORE).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Ignore,
    ),
    (
        InternedId::new(intern::INTERNED_SCIENT).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Type(TypeDirective::Scient),
    ),
    (
        InternedId::new(intern::INTERNED_HEX).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Type(TypeDirective::Hex),
    ),
    (
        InternedId::new(intern::INTERNED_BIN).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Type(TypeDirective::Bin),
    ),
    (
        InternedId::new(intern::INTERNED_OCTAL).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Type(TypeDirective::Octal),
    ),
    (
        InternedId::new(intern::INTERNED_UNICODE).into_tagged::<UTF8Tag>(),
        DirectiveUnparameterized::Type(TypeDirective::Unicode),
    ),
];
