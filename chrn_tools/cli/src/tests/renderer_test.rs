use std::path::Path;

use chrn_utils::{
    arena::Arena,
    id_types::SourceRegionId,
    intern::Intern,
    source_utils::{
        source_diagnostic::{DiagnosticLevel, SourceDiagnostic, annotations::AnnotationKind},
        source_manager::SourceManager,
        source_region::SourceRegion,
        source_span::SourceSpan,
    },
};
use colorc::color_type::TerminalColorType;

use crate::renderer::{
    self, json_renderer::json_config::JsonRenderConfig, render_kind::RenderKind,
    terminal_renderer::terminal_config::TerminalRenderConfig,
    yaml_renderer::yaml_config::YamlRenderConfig,
};

#[test]
fn source_names_render_without_regions_and_without_annotations() {
    let mut interner = Intern::init();
    let name = interner.intern_str("編集中: \"source");
    let mut sources = SourceManager::new();
    let path = sources
        .register_path(Path::new("/fixture/main.chrn"))
        .inner();
    let named = sources.register_in_memory(Some(name), Vec::new()).inner();
    let unnamed = sources.register_in_memory(None, Vec::new()).inner();

    for (id, raw_name, escaped_name) in [
        (path, "/fixture/main.chrn", "/fixture/main.chrn"),
        (named, "編集中: \"source", "編集中: \\\"source"),
        (unnamed, "source", "source"),
    ] {
        let diagnostic =
            SourceDiagnostic::builder(None, DiagnosticLevel::Error, "bad type", id).build();
        // Some(empty arena) exercises the terminal renderer's no-annotations path separately
        // from its no-arena path. Neither requires bytes or a filesystem path.
        let arena = Arena::new();
        for regions in [None, Some(&arena)] {
            let terminal = renderer::render(
                std::slice::from_ref(&diagnostic),
                &[],
                regions,
                &sources,
                &interner,
                &RenderKind::Terminal(TerminalRenderConfig::new(false, TerminalColorType::Ansi4)),
            );
            assert_eq!(
                terminal.matches(&format!("PATH => \"{raw_name}\"")).count(),
                1,
                "source {id:?}, arena present {}: {terminal}",
                regions.is_some()
            );
            assert!(terminal.contains("error: bad type"), "{terminal}");

            for (kind, space) in structured_kinds() {
                let rendered = renderer::render(
                    std::slice::from_ref(&diagnostic),
                    &[],
                    regions,
                    &sources,
                    &interner,
                    &kind,
                );
                let name_value = if id != named {
                    "null".to_string()
                } else {
                    format!("\"{escaped_name}\"")
                };
                assert_field_count(&rendered, &kind, space, "source_name", &name_value, 1);
                let path_value = if id == path {
                    if matches!(kind, RenderKind::Json(_)) {
                        "\"/fixture/main.chrn\""
                    } else {
                        "/fixture/main.chrn"
                    }
                } else {
                    "null"
                };
                assert_field_count(&rendered, &kind, space, "path", path_value, 1);
            }
        }
    }
}

#[test]
fn annotation_source_names_follow_their_own_regions() {
    let mut interner = Intern::init();
    let name = interner.intern_str("編集中: \"source");
    let mut sources = SourceManager::new();
    let path = sources
        .register_path(Path::new("/fixture/main.chrn"))
        .inner();
    let named = sources.register_in_memory(Some(name), Vec::new()).inner();
    let unnamed = sources.register_in_memory(None, Vec::new()).inner();
    let mut regions = Arena::new();
    for (i, (id, script_start)) in [(named, 11), (unnamed, 29)].into_iter().enumerate() {
        let region_id = SourceRegionId::new(i as u32);
        assert_eq!(
            regions.push(SourceRegion::new(
                1,
                0,
                if i == 0 {
                    b"one".to_vec()
                } else {
                    b"two".to_vec()
                },
                region_id,
                id,
                script_start,
                None,
            )),
            region_id
        );
    }
    let diagnostic = SourceDiagnostic::builder(None, DiagnosticLevel::Error, "mixed sources", path)
        .add_annotation(
            SourceSpan::new(SourceRegionId::new(0), 0, 3),
            AnnotationKind::Primary,
            Some("named source".to_string()),
        )
        .add_annotation(
            SourceSpan::new(SourceRegionId::new(1), 0, 3),
            AnnotationKind::Secondary,
            Some("unnamed source".to_string()),
        )
        .build();

    let terminal = renderer::render(
        std::slice::from_ref(&diagnostic),
        &[],
        Some(&regions),
        &sources,
        &interner,
        &RenderKind::Terminal(TerminalRenderConfig::new(false, TerminalColorType::Ansi4)),
    );
    assert_eq!(terminal.matches("PATH => ").count(), 2, "{terminal}");
    assert!(
        terminal.contains("PATH => \"編集中: \"source\""),
        "{terminal}"
    );
    assert!(terminal.contains("PATH => \"source\""), "{terminal}");
    assert!(!terminal.contains("/fixture/main.chrn"), "{terminal}");
    let (named_block, unnamed_block) = terminal.split_once("PATH => \"source\"").unwrap();
    assert!(named_block.contains("1 | one"), "{terminal}");
    assert!(named_block.contains("^^^ named source"), "{terminal}");
    assert!(!named_block.contains("unnamed source"), "{terminal}");
    assert!(unnamed_block.contains("1 | two"), "{terminal}");
    assert!(unnamed_block.contains("--- unnamed source"), "{terminal}");

    for (kind, space) in structured_kinds() {
        let rendered = renderer::render(
            std::slice::from_ref(&diagnostic),
            &[],
            Some(&regions),
            &sources,
            &interner,
            &kind,
        );
        let json = matches!(kind, RenderKind::Json(_));
        assert_field_count(&rendered, &kind, space, "region_path", "null", 2);
        for (key, json_value, yaml_value) in [
            ("source_name", "null", "null"),
            ("path", "\"/fixture/main.chrn\"", "/fixture/main.chrn"),
            (
                "region_source_name",
                "\"編集中: \\\"source\"",
                "\"編集中: \\\"source\"",
            ),
            ("region_source_name", "null", "null"),
            ("start", "11", "11"),
            ("end", "14", "14"),
            ("start", "29", "29"),
            ("end", "32", "32"),
        ] {
            let value = if json { json_value } else { yaml_value };
            assert_field_count(&rendered, &kind, space, key, value, 1);
        }
        let label_key = if json { "\"label\":" } else { "label:" };
        let blocks: Vec<_> = rendered.split(label_key).skip(1).collect();
        assert_eq!(blocks.len(), 2, "{kind:?}: {rendered}");
        for (block, name, start, end) in [
            (blocks[0], "\"編集中: \\\"source\"", "11", "14"),
            (blocks[1], "null", "29", "32"),
        ] {
            assert_field_count(block, &kind, space, "region_source_name", name, 1);
            assert_field_count(block, &kind, space, "region_path", "null", 1);
            assert_field_count(block, &kind, space, "start", start, 1);
            assert_field_count(block, &kind, space, "end", end, 1);
        }
    }
}

#[test]
fn annotation_file_paths_and_unavailable_regions_keep_distinct_metadata() {
    let interner = Intern::init();
    let mut sources = SourceManager::new();
    let diagnostic_source = sources.register_in_memory(None, Vec::new()).inner();
    let file_source = sources
        .register_path(Path::new("/fixture/import.chrn"))
        .inner();
    let region_id = SourceRegionId::new(0);
    let regions = Arena::from(vec![SourceRegion::new(
        1,
        0,
        b"one".to_vec(),
        region_id,
        file_source,
        17,
        None,
    )]);
    let diagnostic = SourceDiagnostic::builder(
        None,
        DiagnosticLevel::Error,
        "imported source",
        diagnostic_source,
    )
    .add_annotation(
        SourceSpan::new(region_id, 0, 3),
        AnnotationKind::Primary,
        None,
    )
    .add_annotation(
        SourceSpan::new(SourceRegionId::new(99), 4, 9),
        AnnotationKind::Secondary,
        None,
    )
    .build();

    for arena in [Some(&regions), None] {
        for (kind, space) in structured_kinds() {
            let rendered = renderer::render(
                std::slice::from_ref(&diagnostic),
                &[],
                arena,
                &sources,
                &interner,
                &kind,
            );
            let json = matches!(kind, RenderKind::Json(_));
            assert_field_count(&rendered, &kind, space, "path", "null", 1);
            assert_field_count(&rendered, &kind, space, "source_name", "null", 1);
            for key in ["region_path", "region_source_name"] {
                assert_field_count(
                    &rendered,
                    &kind,
                    space,
                    key,
                    "null",
                    if key == "region_path" && arena.is_some() {
                        1
                    } else {
                        2
                    },
                );
                if key == "region_path" && arena.is_some() {
                    let value = if json {
                        "\"/fixture/import.chrn\""
                    } else {
                        "/fixture/import.chrn"
                    };
                    assert_field_count(&rendered, &kind, space, key, value, 1);
                }
            }
            for (key, value) in [
                ("start", if arena.is_some() { "17" } else { "0" }),
                ("end", if arena.is_some() { "20" } else { "3" }),
                ("start", "4"),
                ("end", "9"),
            ] {
                assert_field_count(&rendered, &kind, space, key, value, 1);
            }
        }
    }
}

fn assert_field_count(
    rendered: &str,
    kind: &RenderKind,
    space: &str,
    key: &str,
    value: &str,
    expected: usize,
) {
    let field = if matches!(kind, RenderKind::Json(_)) {
        format!("\"{key}\":{space}{value}")
    } else {
        format!("{key}: {value}")
    };
    let count = rendered
        .match_indices(&field)
        .filter(|(start, _)| {
            let before = rendered[..*start].chars().next_back();
            let after = rendered.as_bytes().get(start + field.len());
            // Require complete keys and values, so `path` cannot match `region_path`
            // and offset 1 cannot match 17.
            !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                && matches!(after, Some(b',' | b'\n' | b'}'))
        })
        .count();
    assert_eq!(count, expected, "{kind:?}, {key}={value}: {rendered}");
}

fn structured_kinds() -> [(RenderKind, &'static str); 4] {
    [
        (RenderKind::Json(JsonRenderConfig::new(false)), " "),
        (RenderKind::Json(JsonRenderConfig::new(true)), ""),
        (RenderKind::Yaml(YamlRenderConfig::new(false)), " "),
        (RenderKind::Yaml(YamlRenderConfig::new(true)), " "),
    ]
}
