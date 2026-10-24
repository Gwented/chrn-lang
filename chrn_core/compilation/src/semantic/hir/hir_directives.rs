pub mod directive_inline;
mod directive_parameterized;
mod directive_preprocess;
pub mod lexer_processor;

pub use directive_inline::*;
pub use directive_parameterized::*;
pub use directive_preprocess::*;

use chrn_utils::id_types::InternedIdUTF8;
use lang::types::boundaries::TypeBoundaryFlags;
//Might be a little too much here

/// General directives not specific to anything
#[derive(Debug, Clone)]
pub enum Directive {
    /// Directives that need special attention to be processed
    Preprocess(DirectiveParameterizedHir),
    // Better name than comptime
    /// Directives that can be applied in a local context
    Inline(DirectiveUnparameterized),
}

impl Directive {
    /// Returns `true` if any directive has the given `InternedIdUTF8`, false otherwise.
    pub const fn is_directive(interned_id: InternedIdUTF8) -> bool {
        // if let Some(found) = DirectiveUnparameterized::is_directive(interned_id) {
        //     return Some(Directive::Comptime(found));
        // };
        // if let Some(found) = DirectivePreprocess::is_directive(interned_id) {
        //     return Some(Directive::Preprocess(found));
        // };
        todo!()
    }

    pub fn try_from_interned_str(interned_id: InternedIdUTF8) -> Option<Directive> {
        if let Some(found) = DirectiveUnparameterized::try_from_interned_str(interned_id) {
            return Some(Directive::Inline(found));
        };
        // Not final
        // if let Some(found) = DirectivePreprocessHir::try_from_interned_str(interned_id) {
        //     return Some(Directive::Preprocess(found));
        // };
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

// impl ChrnClassifiable for Directive {
//     fn to_classified(&self) -> ChrnClassified {
//         match self {
//             Directive::Preprocess(d) => d.kind.to_classified(),
//             Directive::Inline(d) => d.to_classified(),
//         }
//     }
// }

impl From<DirectiveParameterizedHir> for Directive {
    fn from(v: DirectiveParameterizedHir) -> Self {
        Directive::Preprocess(v)
    }
}

impl From<DirectiveUnparameterized> for Directive {
    fn from(v: DirectiveUnparameterized) -> Self {
        Directive::Inline(v)
    }
}

impl From<TypeDirective> for Directive {
    fn from(val: TypeDirective) -> Self {
        Directive::Inline(DirectiveUnparameterized::Type(val))
    }
}
