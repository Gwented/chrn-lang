use crate::backend::{ConfigCompletionCandidate, config_completion_items};
use crate::state::{DocumentState, SemanticEntity};
use crate::tests::session::{Session, TempWorkspace, position_of};
use chrn_utils::id_types::{InternedId, SourceRegionId};
use chrn_utils::intern::Intern;
use chrn_utils::source_map::source_span::SourceSpan;
use compilation::script_compiler::ScriptCompiler;
use std::sync::Arc;
use tower_lsp::lsp_types::{
    CompletionItemKind, CompletionResponse, CompletionTextEdit, InsertTextFormat, Range,
};

#[test]
fn nested_config_without_a_member_type_still_offers_member_options() {
    let state = DocumentState::new(
        Arc::new(String::new()),
        Vec::new(),
        Vec::new(),
        Intern::init(),
        0,
        None,
        0,
    );
    let compiler = ScriptCompiler::init(None, chrn_utils::arena::Arena::new());
    let candidate = ConfigCompletionCandidate {
        open: 0,
        close: 1,
        name_start: 0,
        type_id: None,
        scope_id: None,
        is_root: false,
        configured_options: vec![InternedId::new(chrn_utils::intern::INTERNED_IDENTS)],
        configured_members: Vec::new(),
    };

    let labels: Vec<String> = config_completion_items(&state, &compiler, candidate, "")
        .into_iter()
        .map(|item| item.label)
        .collect();

    assert!(labels.iter().any(|label| label == "cases ="));
    assert!(labels.iter().any(|label| label == "default_val ="));
    assert!(!labels.iter().any(|label| label == "idents ="));
}

/// An invoked completion inside the script section offers the language keywords and
/// section markers. Completion is refused in serialized data, so this also confirms the
/// request is being classified as script.
#[tokio::test(start_paused = true)]
async fn completion_in_the_script_section_offers_keywords() {
    let workspace = TempWorkspace::new("script_completion");
    let text = "let flag = 3\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let response = session
        .completion(&uri, position_of(text, "let flag", 0), None)
        .await
        .expect("the script section completes");
    let CompletionResponse::Array(items) = response else {
        panic!("the server answers completion with a plain item array");
    };

    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
        labels.contains(&"let"),
        "keywords are offered, got {labels:?}"
    );
    assert_eq!(
        items
            .iter()
            .find(|item| item.label == "let")
            .unwrap()
            .insert_text,
        None,
        "non-module completions keep their default insertion"
    );
    assert!(
        labels.contains(&"var->"),
        "section markers are offered, got {labels:?}"
    );
}

/// Static access on a built-in type offers its namespace members (`MAX`, `MIN`),
/// which live in builtin-type namespace scopes rather than any module.
#[tokio::test(start_paused = true)]
async fn static_access_on_a_builtin_type_offers_its_namespace_members() {
    let workspace = TempWorkspace::new("builtin_static_completion");
    let text = "let flag = 3\ni32::M\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    // Cursor directly after the typed prefix so the `::` trigger applies.
    let mut pos = position_of(text, "M", 0);
    pos.character += 1;

    let response = session
        .completion(&uri, pos, None)
        .await
        .expect("the script section completes");
    let CompletionResponse::Array(items) = response else {
        panic!("the server answers completion with a plain item array");
    };

    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
        labels.contains(&"MAX"),
        "`i32::` completes MAX, got {labels:?}"
    );
    assert!(
        labels.contains(&"MIN"),
        "`i32::` completes MIN, got {labels:?}"
    );
}

/// Completing through the current module's namespace exposes symbols owned by that
/// module, but not built-in types injected from core. Keeping a same-prefix user type
/// in the fixture proves the prefix itself is not being rejected.
#[tokio::test(start_paused = true)]
async fn current_module_completion_keeps_local_types_and_hides_injected_core_types() {
    let workspace = TempWorkspace::new("module_scope_completion");
    let text = "nest->\nstruct item { value: i32 }\nmain::i\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    // Cursor directly after the typed prefix so the `::` trigger applies.
    let mut pos = position_of(text, "main::i", 0);
    pos.character += "main::i".len() as u32;

    let response = session
        .completion(&uri, pos, None)
        .await
        .expect("the script section completes");
    let CompletionResponse::Array(items) = response else {
        panic!("the server answers completion with a plain item array");
    };

    let item = items
        .iter()
        .find(|completion| completion.label == "item")
        .unwrap_or_else(|| panic!("`main::i` retains the local type, got {items:?}"));
    assert_eq!(
        item.kind,
        Some(tower_lsp::lsp_types::CompletionItemKind::STRUCT),
        "the local declaration remains a struct completion"
    );
    assert!(
        items.iter().all(|completion| completion.label != "i8"),
        "the core-injected `i8` type is not owned by `main`, got {items:?}"
    );
}

/// An import alias is the module binding visible in the importing document.
/// General completion must use that binding, including its module classification,
/// rather than advertising the dependency's file-derived module name.
#[tokio::test(start_paused = true)]
async fn general_completion_exposes_only_the_import_alias_as_a_module() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("aliased_import_general_completion");
    let dependency_uri = workspace.write("dependency.chrn", "export let ITEM = 1\n");
    let dependency_path = dependency_uri
        .to_file_path()
        .expect("the workspace URI is a file path");
    let text = format!(
        "import \"{}\" as public_api\nlet value = \n",
        dependency_path.display()
    );
    let uri = workspace.write("main.chrn", &text);

    let mut session = Session::new().await;
    session.open(&uri, &text).await;

    let mut position = position_of(&text, "let value = ", 0);
    position.character += "let value = ".len() as u32;
    let response = session
        .completion(&uri, position, None)
        .await
        .expect("general completion returns candidates");
    let CompletionResponse::Array(items) = response else {
        panic!("completion must return an item array");
    };

    let alias = items
        .iter()
        .find(|item| item.label == "public_api::")
        .unwrap_or_else(|| panic!("the visible import alias is completed, got {items:?}"));
    assert_eq!(alias.kind, Some(CompletionItemKind::MODULE));
    assert_eq!(alias.insert_text.as_deref(), Some("public_api::"));
    assert!(
        items.iter().all(|item| item.label != "dependency"),
        "an aliased import must not expose its original module name, got {items:?}"
    );
}

/// The editor already has a separator after the cursor; accepting the module
/// must not add a second one.
#[tokio::test(start_paused = true)]
async fn module_completion_before_existing_separator_does_not_duplicate_it() {
    let workspace = TempWorkspace::new("module_existing_separator");
    let dependency_uri = workspace.write("dependency.chrn", "export let ITEM = 1\n");
    let dependency_path = dependency_uri.to_file_path().unwrap();
    let text = format!(
        "import \"{}\" as public_api\npublic_api::ITEM\n",
        dependency_path.display()
    );
    let uri = workspace.write("main.chrn", &text);
    let mut session = Session::new().await;
    session.open(&uri, &text).await;

    let mut position = position_of(&text, "public_api::ITEM", 0);
    position.character += "public_api".len() as u32;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("general completion returns an item array");
    };
    let item = items
        .iter()
        .find(|item| item.label == "public_api::")
        .unwrap();
    assert_eq!(item.kind, Some(CompletionItemKind::MODULE));
    assert_eq!(
        item.insert_text.as_deref(),
        Some("public_api"),
        "the existing `::` supplies the separator"
    );
}

/// A module can contain a visible module binding with the same name. The
/// completion edit must start at the cursor after `::`, not replace the earlier
/// `evil` segment selected by a client's word matching.
#[tokio::test(start_paused = true)]
async fn same_name_module_completion_appends_after_static_access() {
    let workspace = TempWorkspace::new("same_name_module_completion");
    let text = "let value = (evil::)\n";
    let uri = workspace.write("evil.chrn", text);
    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "evil::", 0);
    position.character += "evil::".len() as u32;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("static access completion returns an item array");
    };
    let item = items
        .iter()
        .find(|item| item.label == "evil::")
        .unwrap_or_else(|| panic!("the current module is offered: {items:?}"));
    assert_eq!(item.kind, Some(CompletionItemKind::MODULE));
    assert_eq!(
        item.text_edit,
        Some(CompletionTextEdit::Edit(tower_lsp::lsp_types::TextEdit {
            range: Range::new(position, position),
            new_text: "evil::".into(),
        })),
        "accepting the completion must append `evil::` at the cursor"
    );
    let Some(CompletionTextEdit::Edit(edit)) = &item.text_edit else {
        unreachable!("the completion edit was checked above");
    };
    let offset = crate::text::position_to_offset(text, edit.range.start);
    let mut accepted = text.to_string();
    accepted.insert_str(offset, &edit.new_text);
    assert_eq!(accepted, "let value = (evil::evil::)\n");
}

#[test]
fn config_option_completion_inserts_assignment_syntax() {
    let state = DocumentState::new(
        Arc::new(String::new()),
        Vec::new(),
        Vec::new(),
        Intern::init(),
        0,
        None,
        0,
    );
    let compiler = ScriptCompiler::init(None, chrn_utils::arena::Arena::new());
    let candidate = ConfigCompletionCandidate {
        open: 0,
        close: 1,
        name_start: 0,
        type_id: None,
        scope_id: None,
        is_root: false,
        configured_options: Vec::new(),
        configured_members: Vec::new(),
    };
    let items = config_completion_items(&state, &compiler, candidate, "iden");
    let [item] = items.as_slice() else {
        panic!("the option prefix resolves to one item: {items:?}");
    };
    assert_eq!(item.label, "idents =");
    assert_eq!(item.kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(item.insert_text.as_deref(), Some("idents = "));
}

/// A call-capable predicate is completed as a call only in a condition. Snippet
/// syntax must never be sent to clients that do not advertise snippet support.
#[tokio::test(start_paused = true)]
async fn condition_function_completion_respects_client_snippet_support() {
    let workspace = TempWorkspace::new("condition_call_completion");
    let text = "nest->\nstruct Record { value: i32 [Ran] }\n";
    let uri = workspace.write("main.chrn", text);
    let mut position = position_of(text, "Ran", 0);
    position.character += 3;

    for (supports_snippets, expected_text, expected_format) in [
        (false, "Range()", None),
        (true, "Range($0)", Some(InsertTextFormat::SNIPPET)),
    ] {
        let mut session = Session::new_with_snippet_support(supports_snippets).await;
        session.open(&uri, text).await;
        let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
        else {
            panic!("condition completion returns an item array");
        };
        let item = items
            .iter()
            .find(|item| item.label == "Range(..)")
            .unwrap_or_else(|| panic!("Range is offered in a condition: {items:?}"));
        assert_eq!(item.kind, Some(CompletionItemKind::FUNCTION));
        assert_eq!(item.insert_text.as_deref(), Some(expected_text));
        assert_eq!(item.insert_text_format, expected_format);
    }
}

#[tokio::test(start_paused = true)]
async fn callable_completion_outside_condition_shows_call_shape_without_inserting_it() {
    let workspace = TempWorkspace::new("plain_callable_completion");
    let text = "let chosen = Ran\n";
    let uri = workspace.write("main.chrn", text);
    let mut session = Session::new_with_snippet_support(true).await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "Ran", 0);
    position.character += 3;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("general completion returns an item array");
    };
    let item = items
        .iter()
        .find(|item| item.label == "Range(..)")
        .unwrap_or_else(|| panic!("the callable symbol remains available: {items:?}"));
    assert_eq!(item.kind, Some(CompletionItemKind::FUNCTION));
    assert_eq!(item.insert_text.as_deref(), Some("Range"));
    assert_eq!(item.insert_text_format, None);
}

#[tokio::test(start_paused = true)]
async fn general_completion_uses_visible_symbols_without_raw_token_duplicates() {
    let workspace = TempWorkspace::new("general_visible_symbols");
    let text = "let visible = missing_name\nlet answer = vis\n";
    let uri = workspace.write("main.chrn", text);
    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "vis", 1);
    position.character += 3;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("general completion returns an item array");
    };
    let visible: Vec<_> = items
        .iter()
        .filter(|item| item.label == "visible")
        .collect();
    assert_eq!(
        visible.len(),
        1,
        "the in-scope declaration appears once: {items:?}"
    );
    assert_eq!(visible[0].kind, Some(CompletionItemKind::VARIABLE));

    let mut position = position_of(text, "missing_name", 0);
    position.line += 1;
    position.character = "let answer = ".len() as u32;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("general completion returns an item array");
    };
    assert!(
        items.iter().all(|item| item.label != "missing_name"),
        "an unresolved source token is not a visible symbol: {items:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn neutral_completion_excludes_declared_var_section_symbols() {
    let workspace = TempWorkspace::new("general_scope_filter");
    let text = "let visible = 1\nlet answer = vis\nvar->\nvisible_var: i32\nnest->\nvis\n";
    let uri = workspace.write("main.chrn", text);
    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "vis", 1);
    position.character += 3;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("neutral completion returns an item array");
    };
    let visible: Vec<_> = items
        .iter()
        .filter(|item| item.label == "visible")
        .collect();
    assert_eq!(
        visible.len(),
        1,
        "visible neutral symbol is offered once: {items:?}"
    );
    assert_eq!(visible[0].kind, Some(CompletionItemKind::VARIABLE));
    assert!(
        items.iter().all(|item| item.label != "visible_var"),
        "a var-section declaration is not visible in the neutral section: {items:?}"
    );

    let mut position = position_of(text, "vis", 3);
    position.character += 3;
    let Some(CompletionResponse::Array(items)) = session.completion(&uri, position, None).await
    else {
        panic!("nest completion returns an item array");
    };
    assert!(
        items.iter().any(|item| item.label == "visible_var"),
        "the var-section declaration is registered and visible from nest: {items:?}"
    );
}

/// Override paths use compiler-provided namespace scopes rather than module
/// exports. Completing a partially typed `java::int` must still expose the
/// terminal extern type.
#[tokio::test(start_paused = true)]
async fn static_access_on_an_intrinsic_namespace_offers_extern_types() {
    let workspace = TempWorkspace::new("intrinsic_static_completion");
    let text = "complex->\n    override JAVA {\n        types {\n            change i8 = java::i\n        }\n    }\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut pos = position_of(text, "java::i", 0);
    pos.character += "java::i".len() as u32;

    let response = session
        .completion(&uri, pos, None)
        .await
        .expect("the override path completes");
    let CompletionResponse::Array(items) = response else {
        panic!("the server answers completion with a plain item array");
    };

    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    let int = items
        .iter()
        .find(|item| item.label == "int")
        .unwrap_or_else(|| panic!("`java::i` completes the extern type `int`, got {labels:?}"));
    assert_eq!(
        int.kind,
        Some(tower_lsp::lsp_types::CompletionItemKind::CLASS)
    );
}

/// A repeated intrinsic segment name such as `types` must be resolved through
/// the preceding root namespace. Looking it up globally can select the sibling
/// platform's `types` scope and return the wrong language namespace.
#[tokio::test(start_paused = true)]
async fn multi_segment_intrinsic_completion_stays_under_its_root_namespace() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("qualified_intrinsic_static_completion");
    let mut session = Session::new().await;

    for (root, expected) in [("JAVA", "java"), ("RUST", "rust")] {
        let qualified = format!("{root}::types::");
        let text = format!(
            "complex->\n    override {root} {{\n        types {{\n            change i8 = {qualified}\n        }}\n    }}\n"
        );
        let uri = workspace.write(&format!("{}.chrn", root.to_lowercase()), &text);
        session.open(&uri, &text).await;

        let mut pos = position_of(&text, &qualified, 0);
        pos.character += qualified.len() as u32;
        let response = session
            .completion(&uri, pos, None)
            .await
            .unwrap_or_else(|| panic!("`{qualified}` completes"));
        let CompletionResponse::Array(items) = response else {
            panic!("completion must return an item array");
        };

        let mut actual: Vec<_> = items
            .into_iter()
            .map(|item| (item.label, item.kind))
            .collect();
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            actual,
            vec![(expected.to_string(), Some(CompletionItemKind::VARIABLE))],
            "`{qualified}` completes only its own language namespace"
        );
    }
}

/// An embedded override introduces its own intrinsic namespace. Completion
/// immediately after its shorthand arrow must offer that namespace's children
/// even when the incomplete document contains an invalid shorthand child after
/// the cursor. Losing the override HIR must not fall back to the enclosing config.
#[tokio::test(start_paused = true)]
async fn embedded_override_arrow_completes_the_intrinsic_namespace() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("embedded_override_arrow_completion");
    let text = "nest->\nstruct Structure { field1: i32, field2: u32 }\ncomplex->\nfor Structure {\n    field1 {\n        override JAVA=>idents { change i32 = rust::char }\n    }\n}\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "override JAVA=>", 0);
    position.character += "override JAVA=>".len() as u32;
    let response = session
        .completion(&uri, position, Some(">"))
        .await
        .expect("the embedded override completes");
    let CompletionResponse::Array(items) = response else {
        panic!("completion must return an item array");
    };
    let mut actual: Vec<_> = items
        .iter()
        .map(|item| (item.label.as_str(), item.kind))
        .collect();
    actual.sort_by_key(|(label, _)| *label);
    assert_eq!(
        actual,
        vec![("types", Some(CompletionItemKind::VARIABLE))],
        "`JAVA=>` exposes only its namespace child despite invalid shorthand content"
    );
    {
        let state = session.backend().doc_cache.get(uri.as_str()).unwrap();
        let state = state.read();
        let java_start = text.find("JAVA").unwrap() - state.script_start;
        let java_span = SourceSpan::new(
            SourceRegionId::new(0),
            java_start as u32,
            (java_start + "JAVA".len()) as u32,
        );
        let exact_entities: Vec<_> = state
            .symbol_map
            .iter()
            .filter_map(|(span, entity)| (span == &java_span).then_some(entity))
            .collect();
        assert_eq!(
            exact_entities.len(),
            1,
            "the shorthand override name has one semantic identity, got {exact_entities:?}"
        );
        let entity = exact_entities[0];
        let SemanticEntity::Symbol(sym_id) = entity else {
            panic!("the shorthand override namespace is a symbol, not a config member: {entity:?}");
        };
        assert!(
            matches!(
                state.compiler.as_ref().unwrap().syms[*sym_id].kind,
                compilation::semantic::hir::hir_symbols::SymbolKind::Namespace
            ),
            "the duplicate-span lookup selects the intrinsic namespace symbol"
        );
    }
}

/// Every shorthand arrow follows the namespace selected by the full preceding
/// override path. The second transition must resolve the repeated `types` name
/// under its platform root, not globally or through ordinary config completion.
#[tokio::test(start_paused = true)]
async fn chained_override_arrow_completes_the_next_namespace_segment() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("chained_override_arrow_completion");
    let mut session = Session::new().await;

    for (root, prefix, expected) in [("JAVA", "j", "java"), ("RUST", "r", "rust")] {
        let target = format!("{root}=>types=>{prefix}");
        let text = format!(
            "nest->\nstruct Structure {{ field1: i32 }}\ncomplex->\nfor Structure {{\n    field1 {{\n        override {target}idents {{ change i32 = rust::char }}\n    }}\n}}\n"
        );
        let uri = workspace.write(&format!("{}.chrn", root.to_lowercase()), &text);
        session.open(&uri, &text).await;

        let mut position = position_of(&text, &target, 0);
        position.character += target.len() as u32;
        let response = session
            .completion(&uri, position, None)
            .await
            .unwrap_or_else(|| panic!("the `{target}` override path completes"));
        let CompletionResponse::Array(items) = response else {
            panic!("completion must return an item array");
        };
        let actual: Vec<_> = items
            .into_iter()
            .map(|item| (item.label, item.kind))
            .collect();
        assert_eq!(
            actual,
            vec![(expected.into(), Some(CompletionItemKind::VARIABLE))],
            "`{target}` completes only its platform's child namespace despite invalid remaining shorthand content"
        );
    }
}

/// A braced override is a namespace-backed config root. Prefix completion in
/// its body must expose the same intrinsic child as `override JAVA=>`.
#[tokio::test(start_paused = true)]
async fn braced_override_body_completes_namespace_children() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("braced_override_namespace_completion");
    let text = "nest->\nstruct Structure { field1: i32 }\ncomplex->\nfor Structure {\n    field1 {\n        override JAVA {\n            t\n        }\n    }\n}\n";
    let uri = workspace.write("main.chrn", text);

    let mut session = Session::new().await;
    session.open(&uri, text).await;

    let mut position = position_of(text, "            t", 0);
    position.character += "            t".len() as u32;
    let response = session
        .completion(&uri, position, None)
        .await
        .expect("the braced override body completes");
    let CompletionResponse::Array(items) = response else {
        panic!("completion must return an item array");
    };
    let actual: Vec<_> = items
        .into_iter()
        .map(|item| (item.label, item.kind))
        .collect();
    assert_eq!(
        actual,
        vec![("types".into(), Some(CompletionItemKind::VARIABLE))],
        "the `t` prefix selects only JAVA's `types` namespace child"
    );
}

/// `override` selects an intrinsic configuration root rather than an ordinary
/// type-based config target. Completion at that grammar position must expose
/// exactly the compiler's available platform namespaces.
#[tokio::test(start_paused = true)]
async fn override_root_completion_offers_intrinsic_namespaces() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("override_root_completion");
    let mut session = Session::new().await;

    for (name, prefix, expected) in [
        ("empty", "", &["JAVA", "RUST"][..]),
        ("java_prefix", "J", &["JAVA"][..]),
        ("rust_prefix", "RU", &["RUST"][..]),
    ] {
        let target = format!("override {prefix}");
        let text = format!("complex->\n{target}\n");
        let uri = workspace.write(&format!("{name}.chrn"), &text);
        session.open(&uri, &text).await;

        let mut position = position_of(&text, &target, 0);
        position.character += target.len() as u32;
        let response = session
            .completion(&uri, position, None)
            .await
            .unwrap_or_else(|| panic!("the `{target}` target completes"));
        let CompletionResponse::Array(items) = response else {
            panic!("completion must return an item array");
        };

        let mut actual: Vec<_> = items
            .into_iter()
            .map(|item| (item.label, item.kind))
            .collect();
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        let expected: Vec<_> = expected
            .iter()
            .map(|label| (label.to_string(), Some(CompletionItemKind::VARIABLE)))
            .collect();
        assert_eq!(
            actual, expected,
            "`{target}` offers only matching intrinsic platform namespaces"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn arrow_config_completion_matches_braces_for_struct_members() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("arrow_config_completion");
    let mut session = Session::new().await;
    let declarations = "nest->\nstruct Inner { available: i32 }\nstruct Outer { inner: Inner, unrelated: i32 }\ncomplex->\n";

    for (name, config, trigger) in [
        ("braces", "for Outer { inner {~\n} }\n", None),
        ("arrow", "for Outer { inner =>~\n} \n", Some(">")),
        (
            "arrow_whitespace",
            "for Outer { inner =>\n    ~\n} \n",
            None,
        ),
    ] {
        let marked = format!("{declarations}{config}");
        let position = position_of(&marked, "~", 0);
        let text = marked.replace('~', "");
        let uri = workspace.write(&format!("{name}.chrn"), &text);
        let diagnostics = session.open(&uri, &text).await;
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
        assert_eq!(
            session
                .backend()
                .docs
                .read()
                .get(uri.as_str())
                .unwrap()
                .as_str(),
            text
        );

        let response = session.completion(&uri, position, trigger).await.unwrap();
        let CompletionResponse::Array(items) = response else {
            panic!("{name}: completion must return an item array");
        };
        let mut actual: Vec<_> = items
            .into_iter()
            .map(|item| (item.label, item.kind))
            .collect();
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            actual,
            vec![
                ("available".into(), Some(CompletionItemKind::FIELD)),
                ("cases =".into(), Some(CompletionItemKind::PROPERTY)),
                ("default_val =".into(), Some(CompletionItemKind::PROPERTY)),
                ("idents =".into(), Some(CompletionItemKind::PROPERTY)),
            ],
            "{name}: complete the inner member using its type and the member option schema"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn scalar_arrow_completion_uses_member_options_and_stops_at_parent_close() {
    use tower_lsp::lsp_types::CompletionItemKind;

    let workspace = TempWorkspace::new("scalar_arrow_completion");
    let text =
        "nest->\nstruct Outer { value: i32, sibling: i32 }\ncomplex->\nfor Outer { value =>\n}\n\n";
    let uri = workspace.write("main.chrn", text);
    let mut session = Session::new().await;
    let diagnostics = session.open(&uri, text).await;
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(
        session
            .backend()
            .docs
            .read()
            .get(uri.as_str())
            .unwrap()
            .as_str(),
        text
    );

    let mut position = position_of(text, "value =>", 0);
    position.character += "value =>".len() as u32;
    let response = session.completion(&uri, position, Some(">")).await.unwrap();
    let CompletionResponse::Array(items) = response else {
        panic!("completion must return an item array");
    };
    let mut actual: Vec<_> = items
        .into_iter()
        .map(|item| (item.label, item.kind))
        .collect();
    actual.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        actual,
        vec![
            ("cases =".into(), Some(CompletionItemKind::PROPERTY)),
            ("default_val =".into(), Some(CompletionItemKind::PROPERTY)),
            ("idents =".into(), Some(CompletionItemKind::PROPERTY)),
        ]
    );

    let mut after_close = position_of(text, "}\n\n", 0);
    after_close.line += 1;
    after_close.character = 0;
    let response = session.completion(&uri, after_close, None).await.unwrap();
    let CompletionResponse::Array(items) = response else {
        panic!("completion must return an item array");
    };
    assert!(items.iter().any(|item| item.label == "let"));
}
