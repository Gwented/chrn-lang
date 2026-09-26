//! Produces workspace edits for renames across analyzed documents.

use crate::references::find_occurrence_ranges;
use crate::state::DocumentCache;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

/// Computes the workspace edits required to rename the symbol at `position`.
/// Module renames are unsupported because they imply a filesystem rename.
pub fn compute_rename(
    uri: &Url,
    position: Position,
    new_name: String,
    doc_cache: &DocumentCache,
) -> Option<WorkspaceEdit> {
    let changes: HashMap<_, _> = find_occurrence_ranges(uri, position, doc_cache)?
        .into_iter()
        .filter_map(|(uri, ranges)| {
            let edits: Vec<_> = ranges
                .into_iter()
                .map(|range| TextEdit::new(range, new_name.clone()))
                .collect();
            (!edits.is_empty()).then_some((uri, edits))
        })
        .collect();

    (!changes.is_empty()).then_some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
