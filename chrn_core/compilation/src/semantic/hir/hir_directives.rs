pub mod directive_inline;
mod directive_preprocess;
pub mod lexer_processor;

pub use directive_inline::*;
pub use directive_preprocess::*;

use chrn_utils::{id_types::InternedId, intern};
use lang::{
    chrn_classifier::{ChrnClassifiable, ChrnClassified},
    types::{boundaries::TypeBoundaryFlags, builtins::BuiltinType},
};
//Might be a little too much here

/// General directives not specific to anything
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// Directives that need special attention to be processed
    Preprocess(DirectivePreprocess),
    // Better name than comptime
    /// Directives that can be applied in a local context
    Inline(DirectiveInline),
}

impl Directive {
    /// Returns `true` if any directive has the given `InternedId`, false otherwise.
    pub const fn is_directive(interned_id: InternedId) -> bool {
        // if let Some(found) = DirectiveInline::is_directive(interned_id) {
        //     return Some(Directive::Comptime(found));
        // };
        // if let Some(found) = DirectivePreprocess::is_directive(interned_id) {
        //     return Some(Directive::Preprocess(found));
        // };
        todo!()
    }

    pub fn try_from_interned_str(interned_id: InternedId) -> Option<Directive> {
        if let Some(found) = DirectiveInline::try_from_interned_str(interned_id) {
            return Some(Directive::Inline(found));
        };
        // Not final
        if let Some(found) = DirectivePreprocess::try_from_interned_str(interned_id) {
            return Some(Directive::Preprocess(found));
        };
        None
    }

    //WARN: Comptime is based off of the input boundary.
    // Inline is the input boundary but from the env
    pub const fn boundaries(&self) -> TypeBoundaryFlags {
        match self {
            Directive::Preprocess(d) => TypeBoundaryFlags::empty(),
            Directive::Inline(d) => d.boundaries(),
        }
    }
}

impl ChrnClassifiable for Directive {
    fn to_classified(&self) -> ChrnClassified {
        match self {
            Directive::Preprocess(d) => d.self_kind.to_classified(),
            Directive::Inline(d) => d.to_classified(),
        }
    }
}

impl From<DirectiveInline> for Directive {
    fn from(v: DirectiveInline) -> Self {
        Directive::Inline(v)
    }
}

impl From<TypeDirective> for Directive {
    fn from(val: TypeDirective) -> Self {
        Directive::Inline(DirectiveInline::Type(val))
    }
}
