//! Small string-mapping helpers shared by the structured (non-terminal)
//! renderers. Centralizes how diagnostic field values are spelled out as
//! plain strings and how a `SourceRegionId` is resolved to a source label, so a
//! new `DiagnosticLevel` or `AnnotationKind` variant only needs to be wired
//! up in one place.
//!
//! These helpers are format-agnostic on purpose: each structured renderer
//! is free to wrap the resulting strings in JSON, YAML, or anything else
//! using its own escape module.

use std::borrow::Cow;

use chrn_utils::{
    arena::Arena,
    err_codes,
    id_types::{SourceId, SourceRegionId},
    intern::Intern,
    source_utils::source_manager::{SourceKind, SourceManager},
    source_utils::{
        source_diagnostic::{DiagnosticLevel, annotations::AnnotationKind},
        source_region::SourceRegion,
        source_span::SourceSpan,
    },
};

/// Plain string name used for a diagnostic level.
///
/// Lowercase, matches the convention used by every structured renderer
/// (JSON, YAML, and any future ones) so consumers can rely on a single
/// vocabulary for `level`.
pub(super) fn level_str(level: DiagnosticLevel) -> &'static str {
    match level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warn => "warn",
        DiagnosticLevel::Note => "note",
        DiagnosticLevel::Help => "help",
    }
}

/// Plain string name used for an annotation kind.
///
/// Lowercase, same rationale as [`level_str`].
pub(super) fn annotation_kind_str(kind: AnnotationKind) -> &'static str {
    match kind {
        AnnotationKind::Primary => "primary",
        AnnotationKind::Secondary => "secondary",
        AnnotationKind::Note => "note",
        AnnotationKind::Help => "help",
    }
}

/// Stored source metadata. Display fallbacks belong only in terminal output.
#[derive(Default)]
pub(super) struct SourceMetadata<'a> {
    pub name: Option<&'a str>,
    pub path: Option<Cow<'a, str>>,
}

pub(super) fn resolve_source<'a>(
    src_manager: &'a SourceManager,
    interner: &'a Intern,
    src_id: SourceId,
) -> SourceMetadata<'a> {
    match src_manager.extract_source(src_id) {
        SourceKind::Path(source) => SourceMetadata {
            name: None,
            path: Some(source.path.to_string_lossy()),
        },
        SourceKind::InMemory(source) => SourceMetadata {
            name: Some(interner.extract_utf8_str(source.name_id)),
            path: None,
        },
    }
}

/// Resolve metadata through the span's own region; missing regions have no metadata.
pub(super) fn resolve_region_source<'a>(
    region_arena_opt: Option<&Arena<SourceRegion, SourceRegionId>>,
    src_manager: &'a SourceManager,
    interner: &'a Intern,
    region_id: SourceRegionId,
) -> SourceMetadata<'a> {
    region_arena_opt
        .and_then(|arena| arena.get(region_id))
        .map_or_else(SourceMetadata::default, |region| {
            resolve_source(src_manager, interner, region.src_id)
        })
}

/// Projects a span's `start`/`end` from region-relative offsets to
/// absolute file offsets using the owning region's `script_start`.
///
/// The rest of the pipeline (lexing, parsing, semantic analysis) emits
/// `SourceSpan` values whose `start`/`end` are byte offsets into the
/// owning `SourceRegion::src_bytes`, not the underlying file. This is
/// deliberate: the loader only retains the script portion of the file
/// in `src_bytes`, so a relative position is the only thing that is
/// stable across the pipeline. External tooling, however, has no way
/// to recover the `script_start` it needs to translate those relative
/// positions back into the file the user actually edited.
///
/// The structured (JSON, YAML) renderers run at the end of the pipeline
/// where the region arena is available, so they take on that projection
/// themselves: they add the region's `script_start` to the relative
/// `start`/`end` and emit absolute byte offsets that a tool can map
/// directly onto the source file.
///
/// When the region arena is missing or the region id is not present in
/// it, the raw relative values are returned unchanged. The structured
/// renderers still emit a usable, well-formed document in that case —
/// they just cannot guarantee the offsets are absolute.
pub(super) fn project_absolute_span(
    region_arena_opt: Option<&Arena<SourceRegion, SourceRegionId>>,
    span: &SourceSpan,
) -> (u32, u32) {
    let script_start = region_arena_opt
        .and_then(|arena| arena.get(span.region_id))
        .map_or(0, |region| region.script_start as u32);
    (span.start + script_start, span.end + script_start)
}
