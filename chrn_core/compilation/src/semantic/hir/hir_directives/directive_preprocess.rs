use chrn_utils::{id_types::InternedId, intern};
use lang::{
    chrn_classifier::{ChrnClassifiable, ChrnClassified},
    types::boundaries::TypeBoundaryFlags,
};

use crate::resolvers::resolver_state::ResolverState;
/// General directives not specific to anything
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectivePreprocess {
    pub self_kind: DirectivePreprocessKind,
    pub values: Vec<DirectivePreprocessValue>,
}

impl DirectivePreprocess {
    pub fn new(self_kind: DirectivePreprocessKind, values: Vec<DirectivePreprocessValue>) -> Self {
        Self { self_kind, values }
    }

    pub const fn try_from_interned_str(interned_id: InternedId) -> Option<Self> {
        // None right now
        match interned_id {
            _ => None,
        }
    }
}

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessKind {
    Chrn,
}

// Wow
pub struct DirectivePreprocessField {
    /// Identifier of `self`
    pub ident: InternedId,
    // this
    /// This
    pub boundaries: TypeBoundaryFlags,
    /// Stage when the compiler has enough information for this option to be processed
    pub ready_stage: ResolverState,
    // pub constraints: &'static [DirectivePreprocessConstraintKind],
}

impl DirectivePreprocessField {
    pub const fn new(
        ident: InternedId,
        boundaries: TypeBoundaryFlags,
        ready_stage: ResolverState,
    ) -> Self {
        Self {
            ident,
            boundaries,
            ready_stage,
        }
    }
}

static DIRECTIVE_CHRN_FIELDS: [InternedId; 1] =
    [InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS)];

impl DirectivePreprocessKind {
    pub const fn get_field_constraints(
        &self,
        ident: InternedId,
    ) -> Option<DirectivePreprocessField> {
        match self {
            DirectivePreprocessKind::Chrn => {
                todo!()
            }
        }
    }

    // Calling them directive fields for now
    pub const fn contains_field(&self, ident: InternedId) -> bool {
        match self {
            DirectivePreprocessKind::Chrn => match ident.id {
                // maybe as static array
                intern::INTERNED_MAX_NUMERIC_BITS => true,
                _ => false,
            },
        }
    }

    pub fn try_from_interned_str(interned_id: InternedId) -> Option<Self> {
        let kind = match interned_id.id {
            intern::INTERNED_CHRN => DirectivePreprocessKind::Chrn,
            _ => return None,
        };
        Some(kind)
    }
}

impl ChrnClassifiable for DirectivePreprocessKind {
    fn to_classified(&self) -> ChrnClassified {
        match self {
            DirectivePreprocessKind::Chrn => ChrnClassified::Chrn,
        }
    }
}

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessValue {
    MaxNumericBits(InternedId),
}
