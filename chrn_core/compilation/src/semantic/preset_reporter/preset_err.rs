use chrn_utils::{
    id_types::{InternedId, ModuleId, SymbolId, TypeId},
    source_map::{source_diagnostic::SourceDiagnosticBuilder, source_span::SourceSpan},
    utils::containers::SpannedContainer,
};
use lang::{chrn_classifier::ChrnClassified, types::boundaries::TypeBoundaryFlags};

use crate::{
    constraints::FuncArgConstraint,
    lookup::scopes::scopes_concepts::AssociatedScopeKind,
    parser::ast::ast_concepts::{BinaryOp, UnaryOp},
    resolvers::typechecker::typechecker_concepts::{ExpectedKind, ExpectedKindType},
    semantic::{
        hir::{
            hir_directives::{
                Directive, DirectiveInline, DirectivePreprocessSchema,
                DirectivePreprocessSchemaKind, DirectivePreprocessValueSource,
                directive_preprocess_args::DirectivePreprocessConstraintErrorKind,
            },
            hir_symbols::{BuiltinFuncKind, SymbolKindFlat},
        },
        values::{Value, ValueKind},
    },
};

#[derive(Debug)]
pub enum PresetErr {
    /// Intended so that diagnostics can be made inline and still align with the same type
    General(SourceDiagnosticBuilder),
    /// Constraint, found type(builtin or user), spans
    Lookup(LookupError),
    Directive(DirectiveError),
    SymbolMismatch {
        expected_kind: SymbolKindFlat,
        sp_found_sym_id: SpannedContainer<SymbolId>,
    },
    TypeMismatch {
        expected_kind: ExpectedKindType,
        sp_found_type_id: SpannedContainer<TypeId>,
    },
    FuncConstraintMismatch {
        constraint: FuncArgConstraint,
        fmtted_ty: ChrnClassified,
        spans: Vec<SourceSpan>,
    },
    // maybe have this be general at some point
    ArgCountMismatch {
        expected: u16,
        found: u16,
        span: SourceSpan,
    },
    /// Spanned unknown identifier that was expeceted to be a directive
    UnknownDirective(SpannedContainer<InternedId>),
    // TypecheckFailed {
    //     invalid_ty: SpannedContainer<TypeId>,
    // },
    /// Constraint, amount of incorrect params found, spans
    DirectiveCountMismatch {
        constraint: FuncArgConstraint,
        count: u32,
        spans: Vec<SourceSpan>,
    },
    /// Constraint, Incorrect type found, spans
    TypeBoundaryMismatch {
        given_constraints: TypeBoundaryFlags,
        found_ty: ChrnClassified,
        spans: Vec<SourceSpan>,
    },
    /// Duplicate identiiers were found
    DuplicateIdents {
        sp_original: SpannedContainer<InternedId>,
        sp_dup: SpannedContainer<InternedId>,
        /// What the duplicate actually was.
        /// Like if it should output "duplicate field/variant/parameter" etsy
        classifier: ChrnClassified,
    },
    /// Currently inferred constraints, Conflicting other constraints, spans
    TypeBoundaryBoundConflict {
        inferred: TypeBoundaryFlags,
        conflicting: TypeBoundaryFlags,
        spans: Vec<SourceSpan>,
    },
    /// Spanned Directive failed at, Error Symbol span
    UnsupportedDirective {
        sp_directive: SpannedContainer<Directive>,
        classified_direct: ChrnClassified,
        sym_span: SourceSpan,
    },
    /// SpannedArg,
    // Interesting name
    VagueDirective(SpannedContainer<DirectiveInline>),
    // CircularRef
    // Change this
    /// Occurs when an argument is applied to a type, that recursively holds itself inside of
    /// itself
    ///
    /// (Parent declaration span, Spanned directive failed at, Type span failed at)
    //TODO: Combine
    CircularDirective {
        sp_fmtted_parent: SpannedContainer<ChrnClassified>,
        // Actual parent name
        // SpannedContainer<InternedId>,
        sp_directive: SpannedContainer<DirectiveInline>,
        err_ty_span: SourceSpan,
    },
    /// SpannedInterned number, type overflown
    //WARN: Should rename this to something else.
    NumericParseError {
        sp_num: SpannedContainer<InternedId>,
        fmtted_ty: ChrnClassified,
    },
    /// A literal or constant computation exceeded the configured numeric limit.
    NumericLimitExceeded {
        spans: Vec<SourceSpan>,
        max_bits: u32,
    },
    //TODO: Maybe option name id?
    UndefinedMember(SourceSpan),
    Math(MathError),
    // Schema(*const u8),
}

// #[derive(Debug)]
// pub enum SchemaError {
//     BoundaryMismatch {
//         sp_err_boundaries: SpannedContainer<TypeBoundaryFlags>,
//         required_boundaries: TypeBoundaryFlags,
//     },
// }

#[derive(Debug)]
pub enum MathError {
    /// Spanned lhs, Spanned rhs, Op
    BinaryOpMismatch {
        sp_lhs: SpannedContainer<ValueKind>,
        sp_rhs: SpannedContainer<ValueKind>,
        op: BinaryOp,
    },
    /// Spanned Operand, operator
    UnaryOpMismatch {
        sp_operand: SpannedContainer<ValueKind>,
        op: UnaryOp,
    },
    /// lhs span, rhs span
    DivideByZero {
        lhs_span: SourceSpan,
        rhs_span: SourceSpan,
    },
    /// Shift amount is negative, exceeds `u32`, or exceeds the compiler shift limit
    InvalidShift {
        lhs_span: SourceSpan,
        rhs_span: SourceSpan,
    },
}

// MemberLookupError, SymbolLookupError?
#[derive(Debug)]
pub enum LookupError {
    /// Search context was given an identifier and scope, but the identifier does not exist in the
    /// given scope.
    SymbolNotFound {
        sp_invalid_name_id: SpannedContainer<InternedId>,
        scope_searched: AssociatedScopeKind,
    },
    /// Search context was expecting a type but found a non-type
    NotAType {
        invalid_sym_id: SymbolId,
        sp_invalid_name_id: SpannedContainer<InternedId>,
        scope_found_in: AssociatedScopeKind,
    },
    /// Search context found a type, but it isn't accessible from `current_mod_id`
    PrivateTypeAccess {
        sp_found_type_id: SpannedContainer<TypeId>,
        found_sym_id: SymbolId,
        current_mod_id: ModuleId,
    },
    /// Spanned Type that is impossible to member access
    ImpossibleTypeMemberAccess(SpannedContainer<ChrnClassified>),
    /// Spanned type's identifier which has no members, Identifier of member looked up
    MemberNotFound {
        searched_type_id: TypeId,
        sp_searched_type_name_id: SpannedContainer<InternedId>,
        /// Member looked up but not found
        not_found_name_id: InternedId,
    },
    /// Spanned Formatted Symbol
    /// (Symbol with no members is `Formatted` because it's a language level symbol construct, not a
    /// possibly user defined structure)
    InvalidSymbolMemberAccess(SpannedContainer<ChrnClassified>),
}

#[derive(Debug)]
pub enum DirectiveError {
    ParameterMismatch {
        /// Directive schema
        direct_schema_kind: DirectivePreprocessSchemaKind,
        arg_idx: usize,
        constraint_err: DirectivePreprocessConstraintErrorKind,
        // expected_arg_idx: usize,
        // expected_constraint_idx: usize,
        /// Rejected value
        sp_found_val_kind: SpannedContainer<ValueKind>,
    },
}

#[derive(Debug)]
pub enum FuncConstraints {
    /// Constraint, found type, function kind, spans
    FuncConstraintMismatch {
        constraint: FuncArgConstraint,
        fmtted_ty: ChrnClassified,
        func_kind: BuiltinFuncKind,
        spans: Vec<SourceSpan>,
    },
    /// Constraint, function type, amount of incorrect params found, spans
    ArgCountMismatch {
        constraint: FuncArgConstraint,
        func_kind: BuiltinFuncKind,
        count: u32,
        spans: Vec<SourceSpan>,
    },
}

impl From<DirectiveError> for PresetErr {
    fn from(direct_err: DirectiveError) -> Self {
        PresetErr::Directive(direct_err)
    }
}

impl From<MathError> for PresetErr {
    fn from(math_err: MathError) -> Self {
        PresetErr::Math(math_err)
    }
}

impl From<LookupError> for PresetErr {
    fn from(lookup_err: LookupError) -> Self {
        PresetErr::Lookup(lookup_err)
    }
}
