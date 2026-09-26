//! # references
//!
//! Computes "find all references" results for a symbol under the cursor.
//!
//! The single public entry point is [`compute_references`], called from
//! [`crate::backend::Backend::references`].
//!
//! ## Search strategy
//!
//! * **Local bindings** (alias type parameters etc.) are searched only within the
//!   current file, keying on the declaration span and owning symbol ID.
//! * **All other symbols** (types, variables, module members) are searched across
//!   every document currently held in the [`DocumentCache`](crate::state::DocumentCache),
//!   matching by `(definition_path, definition_span, owning_symbol_id)`.
//!
//! After collecting all candidate [`Location`] values, overlapping/redundant ranges
//! within each file are removed with [`crate::text::deduplicate_range_indices`].

use crate::state::STATE_LOCK_TIMEOUT;
use crate::state::{DocumentCache, DocumentState, SemanticEntity};
use crate::text::{LineIndex, position_to_offset};
use chrn_utils::id_types::SymbolId;
use chrn_utils::source_map::source_span::SourceSpan;
use tower_lsp::lsp_types::{Location, Position, Range, Url};

/// Finds all symbol-map entries in the current file that share the same
/// `(decl_span, owner_sym_id)` key — used for local bindings.
fn collect_local_occurrences(
    state: &DocumentState,
    def_span: SourceSpan,
    def_owner_sym_id: Option<SymbolId>,
) -> Vec<Range> {
    let mut results = Vec::new();
    let lines = LineIndex::new(&state.text);
    for (span, ent) in &state.symbol_map {
        if let SemanticEntity::Local {
            decl_span,
            owner_sym_id,
            ..
        } = ent
            && *decl_span == def_span
            && *owner_sym_id == def_owner_sym_id
        {
            // `span` is relative to the region's `src_bytes`; shift to absolute
            // coordinates by adding `script_start` before converting to an LSP
            // `Position`.
            let abs_start = crate::text::rel_to_abs_offset(span.start, state.script_start) as usize;
            let abs_end = crate::text::rel_to_abs_offset(span.end, state.script_start) as usize;
            results.push(Range {
                start: lines.position(abs_start),
                end: lines.position(abs_end),
            });
        }
    }
    results
}

/// Resolve local or cross-document occurrences once for references and rename.
/// Release the current state guard before the cross-document search, which
/// acquires it again.
pub(crate) fn find_occurrence_ranges(
    uri: &Url,
    position: Position,
    doc_cache: &DocumentCache,
) -> Option<Vec<(Url, Vec<Range>)>> {
    let uri_str = uri.to_string();
    let state_arc = doc_cache.get(&uri_str)?;

    // The local search reads `state`; the cross-module search re-reads every
    // cached document, *including this one*.  `parking_lot`'s `RwLock` is not
    // reentrant, so holding this guard across `find_matching_entities` deadlocks
    // as soon as a writer (an analysis task) is queued between the two reads.
    // Resolve the definition key under the guard, then drop it before searching.
    let (def_path, def_span, def_owner_sym_id, is_local, local_ranges) = {
        let state = state_arc.try_read_for(STATE_LOCK_TIMEOUT)?;

        let byte_offset = position_to_offset(&state.text, position);
        if state.offset_in_comment(byte_offset) {
            return None;
        }

        let entity = state.get_entity_at_offset(byte_offset)?;

        // Module entities are not handled by these searches.
        if matches!(entity, SemanticEntity::Module(_)) {
            return None;
        }

        let (def_path, def_span, def_owner_sym_id) = state.definition_site(entity)?;
        let def_path = def_path.to_path_buf();
        let is_local = matches!(entity, SemanticEntity::Local { .. });
        let local_ranges = if is_local {
            collect_local_occurrences(&state, def_span, def_owner_sym_id)
        } else {
            Vec::new()
        };
        (def_path, def_span, def_owner_sym_id, is_local, local_ranges)
    };

    let ranges = if is_local {
        vec![(uri.clone(), local_ranges)]
    } else {
        let entities =
            DocumentState::find_matching_entities(doc_cache, &def_path, def_span, def_owner_sym_id);
        crate::text::occurrences_to_ranges(entities)
            .into_iter()
            .filter_map(|(state_uri, ranges)| Url::parse(&state_uri).ok().map(|uri| (uri, ranges)))
            .collect()
    };

    Some(ranges)
}

/// Computes the list of locations where the symbol at `position` is referenced.
pub fn compute_references(
    uri: &Url,
    position: Position,
    doc_cache: &DocumentCache,
) -> Option<Vec<Location>> {
    let locations: Vec<_> = find_occurrence_ranges(uri, position, doc_cache)?
        .into_iter()
        .flat_map(|(uri, ranges)| {
            ranges.into_iter().map(move |range| Location {
                uri: uri.clone(),
                range,
            })
        })
        .collect();
    (!locations.is_empty()).then_some(locations)
}
