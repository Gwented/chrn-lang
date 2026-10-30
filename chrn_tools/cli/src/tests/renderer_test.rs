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
    let named = sources.register_in_memory(Some(name)).inner();
    let unnamed = sources.register_in_memory(None).inner();

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
                let field = if matches!(kind, RenderKind::Json(_)) {
                    format!("\"path\":{space}\"{escaped_name}\"")
                } else if id == named {
                    format!("path: \"{escaped_name}\"")
                } else {
                    format!("path: {raw_name}")
                };
                assert_eq!(
                    rendered.matches(&field).count(),
                    1,
                    "{kind:?}, source {id:?}: {rendered}"
                );
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
    let named = sources.register_in_memory(Some(name)).inner();
    let unnamed = sources.register_in_memory(None).inner();
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
        for (key, json_value, yaml_value) in [
            ("path", "\"/fixture/main.chrn\"", "/fixture/main.chrn"),
            (
                "region_path",
                "\"編集中: \\\"source\"",
                "\"編集中: \\\"source\"",
            ),
            ("region_path", "\"source\"", "source"),
            ("start", "11", "11"),
            ("end", "14", "14"),
            ("start", "29", "29"),
            ("end", "32", "32"),
        ] {
            let value = if json { json_value } else { yaml_value };
            let field = if json {
                format!("\"{key}\":{space}{value}")
            } else {
                format!("{key}: {value}")
            };
            assert_eq!(
                rendered.matches(&field).count(),
                1,
                "{kind:?}, {key}={value}: {rendered}"
            );
            let end = rendered.find(&field).unwrap() + field.len();
            assert!(
                matches!(rendered.as_bytes().get(end), Some(b',' | b'\n' | b'}')),
                "{kind:?}, {key} must equal {value}, not a longer value: {rendered}"
            );
        }
    }
}

fn structured_kinds() -> [(RenderKind, &'static str); 4] {
    [
        (RenderKind::Json(JsonRenderConfig::new(false)), " "),
        (RenderKind::Json(JsonRenderConfig::new(true)), ""),
        (RenderKind::Yaml(YamlRenderConfig::new(false)), " "),
        (RenderKind::Yaml(YamlRenderConfig::new(true)), " "),
    ]
}
