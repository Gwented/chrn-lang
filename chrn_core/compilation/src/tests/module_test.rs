use chrn_utils::id_types::{InternedId, id_tags::UTF8Tag};
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use super::helpers::*;
use crate::config_loader::ConfigLoader;
use crate::module::extract_all_modules;
use crate::module::extract_main;
use crate::module::module_concepts::{ImportKind, Module, ModuleState};
use crate::module::module_finder::ModuleFinder;
use crate::script_compiler::reporter::Reporter;
use chrn_utils::{
    core_error::ConfigLoadError,
    id_types::{ModuleId, PathId, SourceRegionId},
    intern::Intern,
    source_map::{
        source_diagnostic::{DiagnosticLevel, annotations::AnnotationKind},
        source_span::SourceSpan,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a unique temporary directory for a test. The caller should clean up
/// with `remove_dir_all` when done.
fn create_temp_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("chrn_modules_test_{}", label));
    _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Writes `content` to `path`, creating parent directories as needed.
fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(path, content).expect("failed to write file");
}

/// Creates a `.chrn` file and returns its canonical path.
fn create_chrn_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    write_file(&path, content);
    fs::canonicalize(&path).expect("failed to canonicalize path")
}

/// Returns the module at the given index from a compiler, or panics if out of
/// range.  Index 0 is the first user module; the core module is always last.
fn get_user_mod_count(compiler: &crate::script_compiler::ScriptCompiler) -> usize {
    // All modules except the implicit core module at the end
    compiler.mods.len().saturating_sub(1)
}

/// Finds a module in the compiler by name.  Returns `None` if not found.
fn find_module_by_name<'a>(
    compiler: &'a crate::script_compiler::ScriptCompiler,
    interner: &Intern,
    name: &str,
) -> Option<&'a Module> {
    let name_id = interner.get_interned_id_from_str(name)?;
    for i in 0..compiler.mods.len() {
        let m = &compiler.mods[ModuleId::new(i as u32)];
        if m.name_id == name_id {
            return Some(m);
        }
    }
    None
}

/// Creates a simple script file with no imports.  Returns its canonical path.
fn make_simple_script(dir: &Path) -> PathBuf {
    create_chrn_file(dir, "main.chrn", "let x = 5\nlet y = 10\n")
}

/// Creates a script file that imports a sub-module. `sub_path` must be the
/// *canonical* absolute path to the sub file (import statements use absolute
/// paths so the working directory is irrelevant).
fn make_importing_script(dir: &Path, sub_canonical: &Path) -> PathBuf {
    let import_stmt = format!("import \"{}\"\n", sub_canonical.display());
    create_chrn_file(dir, "main.chrn", &format!("{}let x = 5\n", import_stmt))
}

/// Creates a sub-module script with simple content.
fn make_sub_script(dir: &Path) -> PathBuf {
    create_chrn_file(dir, "sub.chrn", "let y = 42\n")
}

/// Creates a file with `@def` / `@end` boundaries.
fn make_def_script(dir: &Path) -> PathBuf {
    create_chrn_file(dir, "def_test.chrn", "serial_data\n@def\nlet x = 1\n@end\n")
}

/// Creates a file with a broken `@def` (no matching `@end`).
fn make_broken_script(dir: &Path) -> PathBuf {
    create_chrn_file(dir, "broken.chrn", "@def\nlet x = 1\n")
}

/// Creates a file with a `bind` statement pointing to a real file.
fn make_bind_script(dir: &Path, bind_target: &Path) -> PathBuf {
    let bind_stmt = format!("bind \"{}\"\n", bind_target.display());
    create_chrn_file(dir, "with_bind.chrn", &format!("{}let x = 5\n", bind_stmt))
}

fn mod_id_for_path(reserved: &[(PathId, ModuleId)], path_id: PathId) -> Option<ModuleId> {
    reserved
        .iter()
        .find(|(p, _)| *p == path_id)
        .map(|(_, m)| *m)
}

/// Extracts the `ModuleId` from an `Import` if its kind carries one.
/// Returns `None` for unresolved or error-source imports.
fn import_mod_id(import: &Import) -> Option<ModuleId> {
    match &import.kind {
        ImportKind::Source(_, mod_id) => Some(*mod_id),
        ImportKind::Core(mod_id) => Some(*mod_id),
        _ => None,
    }
}

// ===========================================================================
// ModuleFinder unit tests
// ===========================================================================

/// `ModuleFinder` should return zero imports and no bind when the src text
/// contains neither.
#[test]
fn modfinder_no_imports_or_bind() {
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("dummy.chrn"));
    let region_id = SourceRegionId::new(0);

    let region = ConfigLoader::new(
        region_id,
        Cursor::new("let x = 5\nlet y = 10\n"),
        path_id,
        &cfg,
    )
    .load_config()
    .expect_success();

    let (bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(bind.is_none(), "no bind expected, got {:?}", bind);
    assert!(
        imports.is_empty(),
        "no imports expected, got {}",
        imports.len()
    );
    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
}

/// `ModuleFinder` should find a single import with a valid path.
#[test]
fn modfinder_single_import() {
    let dir = create_temp_dir("modfinder_single_import");
    let sub_path = make_sub_script(&dir);
    let sub_canonical = sub_path.to_string_lossy();

    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = format!("import \"{}\"\nlet x = 5\n", sub_canonical);
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let (bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(bind.is_none(), "no bind expected");
    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    assert_eq!(imports.len(), 1, "exactly one import expected");

    let import = &imports[0];
    let sub_path_id = interner.intern_path(&sub_path);
    // ModuleFinder returns UnresolvedSource imports (module id is assigned
    // during module resolution). Verify the import kind carries the correct path
    // id and span.
    // Source is `import "PATH"\nlet x = 5\n`; module finder's `parse_import` starts
    // at the opening `"` (byte 7), advances past it, so the span covers the path
    // without quotes:  start = 8,  end = 8 + path.len().
    let import_span = match &import.kind {
        ImportKind::UnresolvedSource(sp_path_id) => {
            assert_eq!(
                sp_path_id.inner, sub_path_id,
                "import must carry the correct sub path id"
            );
            let expected_start = 8;
            let expected_end = expected_start + sub_canonical.len() as u32;
            assert_eq!(
                sp_path_id.span.start, expected_start,
                "import span must start at byte 8 (right after opening \"), got {}",
                sp_path_id.span.start
            );
            assert_eq!(
                sp_path_id.span.end,
                expected_end,
                "import span must end at byte {} (right after path of length {}), got {}",
                expected_end,
                sub_canonical.len(),
                sp_path_id.span.end
            );
            sp_path_id.span
        }
        _ => panic!(
            "expected UnresolvedSource import kind, got {:?}",
            import.kind
        ),
    };
    // Verify the span slices out the exact path from src bytes
    let span_bytes = &region.src_bytes[import_span.start as usize..import_span.end as usize];
    let span_str = String::from_utf8_lossy(span_bytes);
    assert_eq!(
        span_str, sub_canonical,
        "import span must extract the exact path string from src bytes"
    );

    // ModuleFinder records unresolved import paths; module ids and graph state belong to
    // module resolution.

    _ = fs::remove_dir_all(&dir);
}

/// `ModuleFinder` should find two distinct imports.
#[test]
fn modfinder_multiple_imports() {
    let dir = create_temp_dir("modfinder_multi_import");
    let sub1_path = make_sub_script(&dir);
    let sub2_path = create_chrn_file(&dir, "another.chrn", "let z = 99\n");

    let sub1_str = sub1_path.to_string_lossy();
    let sub2_str = sub2_path.to_string_lossy();

    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = format!(
        "import \"{}\"\nimport \"{}\"\nlet x = 5\n",
        sub1_str, sub2_str
    );
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let (bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(bind.is_none());
    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    assert_eq!(imports.len(), 2, "exactly two imports expected");

    // ModuleFinder returns UnresolvedSource imports; module ids are assigned
    // later by resolve_module.  Both imports should be unresolved.
    for (idx, imp) in imports.iter().enumerate() {
        assert!(
            matches!(imp.kind, ImportKind::UnresolvedSource(_)),
            "import {} must be UnresolvedSource, got {:?}",
            idx,
            imp.kind
        );
    }

    _ = fs::remove_dir_all(&dir);
}

/// `ModuleFinder` should capture the `as` alias when present.
#[test]
fn modfinder_import_with_alias() {
    let dir = create_temp_dir("modfinder_alias");
    let sub_path = make_sub_script(&dir);
    let sub_canonical = sub_path.to_string_lossy();

    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = format!("import \"{}\" as my_mod\nlet x = 5\n", sub_canonical);
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let (_, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert!(
        import.sp_alias_id.is_some(),
        "import should have an alias_id"
    );
    let alias_name = interner.extract_utf8_str(import.sp_alias_id.as_ref().unwrap().inner);
    assert_eq!(alias_name, "my_mod", "alias should be 'my_mod'");

    _ = fs::remove_dir_all(&dir);
}

/// `ModuleFinder` should find a `bind` statement.
#[test]
fn modfinder_bind() {
    let dir = create_temp_dir("modfinder_bind");
    let bind_target = create_chrn_file(&dir, "target.bin", "not actually a binary\n");
    let bind_canonical = bind_target.to_string_lossy();

    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = format!("bind \"{}\"\nlet x = 5\n", bind_canonical);
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let (bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(diags.diags.is_empty(), "no diagnostics expected");
    assert!(imports.is_empty(), "no imports expected");
    assert!(bind.is_some(), "bind should be present");
    assert_eq!(
        bind.as_ref().unwrap().path_id,
        interner.intern_path(&bind_target),
        "bind path must match target"
    );

    _ = fs::remove_dir_all(&dir);
}

/// A backslash in an import path must produce an error diagnostic (only '/' is
/// allowed as a path separator).
#[test]
fn modfinder_backslash_error() {
    let dir = create_temp_dir("modfinder_backslash");
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    // Use a backslash in the import path — even though the real file exists,
    // the backslash triggers the error *before* canonicalization is attempted.
    let src = r#"import "bad\path" "#;
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        imports.is_empty(),
        "no valid import must be produced for a backslash path"
    );
    assert!(
        !diags.diags.is_empty(),
        "backslash should produce a diagnostic"
    );
    let has_backslash_msg = diags.diags.iter().any(|d| d.core_msg.contains("'/'"));
    assert!(
        has_backslash_msg,
        "diagnostic should mention that only '/' is allowed"
    );

    _ = fs::remove_dir_all(&dir);
}

/// An import written inside a line comment must be ignored.
#[test]
fn modfinder_import_inside_line_comment() {
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("dummy.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = Cursor::new(b"// import \"sub.chrn\"\nlet x = 5\n");
    let region = ConfigLoader::new(region_id, src, path_id, &cfg)
        .load_config()
        .expect_success();

    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        imports.is_empty(),
        "import inside line comment must be ignored"
    );
    assert!(diags.diags.is_empty(), "no diagnostics expected");
}

/// An import written inside a multi-line comment must be ignored.
#[test]
fn modfinder_import_inside_block_comment() {
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("dummy.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = Cursor::new(b"/* import \"sub.chrn\" */\nlet x = 5\n");
    let region = ConfigLoader::new(region_id, src, path_id, &cfg)
        .load_config()
        .expect_success();

    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        imports.is_empty(),
        "import inside block comment must be ignored"
    );
    assert!(diags.diags.is_empty(), "no diagnostics expected");
}

/// Duplicate imports to the same path should reuse the same module id via the
/// `seen` vector.
#[test]
fn modfinder_duplicate_import_path_reuses_mod_id() {
    let dir = create_temp_dir("modfinder_dup");
    let sub_path = make_sub_script(&dir);
    let sub_canonical = sub_path.to_string_lossy();

    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("main.chrn"));
    let region_id = SourceRegionId::new(0);

    // Two imports to the same path
    let src = format!(
        "import \"{}\"\nimport \"{}\"\nlet x = 5\n",
        sub_canonical, sub_canonical
    );
    let region = ConfigLoader::new(region_id, src.as_bytes(), path_id, &cfg)
        .load_config()
        .expect_success();

    let sub_path_id = interner.intern_path(&sub_path);
    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    assert_eq!(imports.len(), 2, "both imports should be recorded");

    // ModuleFinder returns UnresolvedSource imports; both should carry
    // the same path_id since they reference the same path.
    for imp in &imports {
        assert!(
            matches!(imp.kind, ImportKind::UnresolvedSource(ref sp) if sp.inner == sub_path_id),
            "import must reference the sub path"
        );
    }

    _ = fs::remove_dir_all(&dir);
}

/// Non-existent paths in an import statement must produce a diagnostic and the
/// import must be dropped.
#[test]
fn modfinder_nonexistent_path() {
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("dummy.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = Cursor::new(b"import \"/does/not/exist.chrn\"\nlet x = 5\n");
    let region = ConfigLoader::new(region_id, src, path_id, &cfg)
        .load_config()
        .expect_success();

    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        imports.is_empty(),
        "import for non-existent path must be dropped"
    );
    assert!(
        !diags.diags.is_empty(),
        "non-existent path must produce a diagnostic"
    );
}

/// `ModuleFinder` should handle a string inside the src that contains
/// something that looks like `import` but is inside quotes.
#[test]
fn modfinder_import_inside_string_is_not_parsed() {
    let mut interner = Intern::init();
    let cfg = ChrnConfig::default();
    let path_id = interner.intern_path(Path::new("dummy.chrn"));
    let region_id = SourceRegionId::new(0);

    let src = Cursor::new(b"\"this has import inside a string\" let x = 5\n");
    let region = ConfigLoader::new(region_id, src, path_id, &cfg)
        .load_config()
        .expect_success();

    let (_bind, imports, diags) =
        ModuleFinder::new(&region.src_bytes, &cfg, &region).collect_imports(&mut interner);

    assert!(
        imports.is_empty(),
        "'import' inside a string must not be parsed as an import"
    );
    assert!(diags.diags.is_empty(), "no diagnostics expected");
}

// ===========================================================================
// extract_main tests
// ===========================================================================

/// `extract_main` on a simple script file with no imports must return a
/// `Module` with `Loaded` state, no imports, no bind, and a correctly
/// initialised `ModuleGraph`.
#[test]
fn extract_main_simple_script() {
    let dir = create_temp_dir("extract_main_simple");
    let main_path = make_simple_script(&dir);
    let mut cfg = ChrnConfig::default();

    let (main_mod, graph, mut interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    // ---- Module assertions ----
    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "main module should be Loaded"
    );
    assert_eq!(main_mod.self_id, ModuleId::new(0), "main gets mod_id 0");
    let main_name = interner.extract_utf8_str(main_mod.name_id);
    assert_eq!(
        main_name, "main",
        "module name should be 'main' (file stem)"
    );
    assert!(
        main_mod.imports.is_empty(),
        "no imports expected in simple script"
    );
    assert!(main_mod.bind.is_none(), "no bind expected in simple script");
    assert!(
        main_mod.region_id.is_some(),
        "main module should have a region_id"
    );

    // ---- Graph assertions ----
    // reserved_mod_ids must contain main's path -> mod_id 0
    let main_path_id = interner.intern_path(&main_path);
    let main_entry = mod_id_for_path(graph.registered_mod_ids(), main_path_id);
    assert_eq!(main_entry, Some(ModuleId::new(0)));
    assert_eq!(graph.registered_mod_ids().len(), 1, "only main in reserved");
    // `seen` must contain main's path id
    assert_eq!(graph.seen().len(), 1, "only main in seen");
    assert!(graph.seen().contains(&main_path_id));
    // `other_mods` starts empty
    // assert!(graph.other_mods().is_empty(), "other_mods should be empty");
    // region_arena must have one entry
    assert_eq!(graph.region_arena().len(), 1, "one region pushed");
    assert_eq!(graph.region_arena().capacity(), 1);

    assert!(diags.diags.is_empty(), "no diagnostics expected");

    _ = fs::remove_dir_all(&dir);
}

/// `extract_main` on a file with a valid import should attach the import to
/// the main module.  Module ids are not yet assigned (that happens later in
/// `extract_modules`), so imports remain `UnresolvedSource`.
#[test]
fn extract_main_with_import() {
    let dir = create_temp_dir("extract_main_import");
    let sub_path = make_sub_script(&dir);
    let main_path = make_importing_script(&dir, &sub_path);
    let mut cfg = ChrnConfig::default();

    let (main_mod, graph, mut interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    assert_eq!(main_mod.state, ModuleState::Loaded);
    // Wait for import - there should be one import found
    assert_eq!(
        main_mod.imports.len(),
        1,
        "main should have 1 import from the sub module"
    );

    // Imports returned by extract_main are UnresolvedSource; no module id
    // is assigned yet.
    let import = &main_mod.imports[0];
    let sub_path_id = interner.intern_path(&sub_path);
    let sub_path_str = sub_path.to_string_lossy();
    let import_span = match &import.kind {
        ImportKind::UnresolvedSource(sp_path_id) => {
            assert_eq!(
                sp_path_id.inner, sub_path_id,
                "import must carry the correct sub path id"
            );
            let expected_start = 8;
            let expected_end = expected_start + sub_path_str.len() as u32;
            assert_eq!(
                sp_path_id.span.start, expected_start,
                "import span must start at byte 8 (right after opening \"), got {}",
                sp_path_id.span.start
            );
            assert_eq!(
                sp_path_id.span.end,
                expected_end,
                "import span must end at byte {} (after path of length {}), got {}",
                expected_end,
                sub_path_str.len(),
                sp_path_id.span.end
            );
            sp_path_id.span
        }
        ImportKind::Core(_) => panic!("expected UnresolvedSource, got Core"),
        _ => panic!("expected UnresolvedSource, got {:?}", import.kind),
    };
    // Verify the span slices out the exact path from src bytes
    let region = &graph.region_arena()[main_mod.region_id.expect("main mod has region_id")];
    let span_bytes = &region.src_bytes[import_span.start as usize..import_span.end as usize];
    let span_str = String::from_utf8_lossy(span_bytes);
    assert_eq!(
        span_str, sub_path_str,
        "import span must extract the exact path string from src bytes"
    );

    // reserved_mod_ids only contains main (sub is registered later in
    // extract_modules by resolve_module).
    let main_path_id = interner.intern_path(&main_path);
    assert_eq!(graph.registered_mod_ids().len(), 1, "only main in reserved");
    assert!(
        graph
            .registered_mod_ids()
            .iter()
            .any(|(p, _)| *p == main_path_id),
        "main path must be in reserved"
    );

    // `seen` must contain main
    assert!(graph.seen().contains(&main_path_id));
    assert_eq!(
        graph.seen().len(),
        1,
        "only main in seen (sub not visited yet)"
    );

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );

    _ = fs::remove_dir_all(&dir);
}

/// `extract_main` on a file with a broken `@def` (no `@end`) must produce a
/// module with `ModuleState::BrokenRegion` and a diagnostic.
#[test]
fn extract_main_broken_config() {
    let dir = create_temp_dir("extract_main_broken");
    let main_path = make_broken_script(&dir);
    let mut cfg = ChrnConfig::default();

    let (main_mod, graph, _interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main should still return Ok with BrokenRegion");

    assert_eq!(
        main_mod.state,
        ModuleState::BrokenRegion,
        "broken config should result in BrokenRegion state"
    );
    assert_eq!(main_mod.self_id, ModuleId::new(0));
    assert!(
        main_mod.region_id.is_some(),
        "even a broken region gets a region_id"
    );
    // The region was pushed even for broken config
    assert_eq!(
        graph.region_arena().len(),
        1,
        "broken region still pushed into arena"
    );

    // There should be at least one diagnostic about the missing @end
    assert!(
        !diags.diags.is_empty(),
        "broken config must produce diagnostics"
    );
    let has_missing_end = diags.diags.iter().any(|d| d.core_msg.contains("@end"));
    assert!(has_missing_end, "diagnostic should mention missing @end");

    _ = fs::remove_dir_all(&dir);
}

/// `extract_main` on a file with a `bind` statement must capture the bind in
/// the returned module.
#[test]
fn extract_main_with_bind() {
    let dir = create_temp_dir("extract_main_bind");
    let bind_target = create_chrn_file(&dir, "bind_target.bin", "content\n");
    let main_path = make_bind_script(&dir, &bind_target);
    let mut cfg = ChrnConfig::default();

    let (main_mod, _graph, mut interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    assert_eq!(main_mod.state, ModuleState::Loaded);
    assert!(
        main_mod.bind.is_some(),
        "bind should be present in the module"
    );
    let bind = main_mod.bind.as_ref().unwrap();
    let expected_path_id = interner.intern_path(&bind_target);
    assert_eq!(
        bind.path_id, expected_path_id,
        "bind path must match the target file"
    );

    assert!(diags.diags.is_empty(), "no diagnostics expected");

    // Also verify that bind is the only thing found (no imports)
    assert!(
        main_mod.imports.is_empty(),
        "no imports expected in bind-only script"
    );

    _ = fs::remove_dir_all(&dir);
}

/// `extract_main` on a file with an `@def`/`@end` block must set the region's
/// script_start and serial_start correctly.
#[test]
fn extract_main_with_at_def_block() {
    let dir = create_temp_dir("extract_main_def");
    let main_path = make_def_script(&dir);
    let mut cfg = ChrnConfig::default();

    let (main_mod, graph, _interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    assert_eq!(main_mod.state, ModuleState::Loaded);

    // The region for main should have the correct script and serial offsets.
    // File content from `make_def_script`:  "serial_data\n@def\nlet x = 1\n@end\n"
    //   bytes 0-11  = "serial_data\n"  (12 bytes of serial data)
    //   bytes 12-16 = "@def\n"         (5 bytes)
    //   bytes 17-26 = "let x = 1\n"    (10 bytes of script)
    //   bytes 27-30 = "@end"           (4 bytes, the \n at byte 31 is trailing)
    // So: script_start = 12 (offset of @def), serial_start = 31 (offset after @end).
    let region = &graph.region_arena()[main_mod.region_id.unwrap()];
    assert_eq!(
        region.script_start, 12,
        "script_start should be at the @def directive (byte 12), got {}",
        region.script_start
    );
    assert_eq!(
        region.serial_start,
        Some(31),
        "serial_start should be after @end (byte 31), got {:?}",
        region.serial_start
    );

    assert!(diags.diags.is_empty(), "no diagnostics expected");

    // Verify the region bytes contain @def and @end
    let src_str = String::from_utf8_lossy(&region.src_bytes);
    assert!(src_str.contains("@def"), "region bytes must contain @def");
    assert!(src_str.contains("@end"), "region bytes must contain @end");

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// extract_all_modules tests  (full pipeline)
// ===========================================================================

/// `extract_all_modules` with a single module (no imports) should produce a
/// `ScriptCompiler` containing exactly one user module plus the core module.
#[test]
fn extract_all_modules_single() {
    let dir = create_temp_dir("extract_all_single");
    let main_path = make_simple_script(&dir);
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    // Number of user modules = total mods - 1 (core module is injected last)
    let user_count = get_user_mod_count(&compiler);
    assert_eq!(user_count, 1, "expected 1 user module");

    // The single user module must be loaded
    let main_mod = &compiler.mods[ModuleId::new(0)];
    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "single module should be Loaded"
    );

    // Core module must exist
    let core_mod_id = ModuleId::new(compiler.mods.len() as u32 - 1);
    let core_mod = &compiler.mods[core_mod_id];
    assert_eq!(
        core_mod.name_id,
        InternedId::new(chrn_utils::intern::INTERNED_CORE).into_tagged::<UTF8Tag>(),
        "last module must be core"
    );

    // The store is filled once per compiler module by the orchestration stage.
    assert_eq!(store.toks.capacity(), compiler.mods.len());
    assert_eq!(store.trivias.capacity(), compiler.mods.len());
    assert_eq!(store.asts.capacity(), compiler.mods.len());
    assert_eq!(store.compilation_syms.capacity(), compiler.mods.len());

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );

    _ = fs::remove_dir_all(&dir);
}

/// `extract_all_modules` with an importing main module must resolve the sub
/// module and include it in the compiler.
#[test]
fn extract_all_modules_with_sub_module() {
    let dir = create_temp_dir("extract_all_sub");
    let sub_path = make_sub_script(&dir);
    let main_path = make_importing_script(&dir, &sub_path);
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    let user_count = get_user_mod_count(&compiler);
    assert_eq!(user_count, 2, "expected 2 user modules (main + sub)");

    // Find the sub module by name
    let interner = &store.interner;
    let sub_mod = find_module_by_name(&compiler, interner, "sub")
        .expect("sub module must be present in compiler");
    assert_eq!(
        sub_mod.state,
        ModuleState::Loaded,
        "sub module must be Loaded"
    );
    assert!(sub_mod.region_id.is_some(), "sub module must have a region");

    // Main module must still be Loaded
    let main_mod =
        find_module_by_name(&compiler, interner, "main").expect("main module must be present");
    assert_eq!(main_mod.state, ModuleState::Loaded);

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );

    _ = fs::remove_dir_all(&dir);
}

/// When a sub-module has a broken config, the pipeline must still complete but
/// the sub-module should have `BrokenRegion` state and diagnostics should be
/// emitted.
#[test]
fn extract_all_modules_submodule_broken() {
    let dir = create_temp_dir("extract_all_broken_sub");
    let sub_path = make_broken_script(&dir);
    let main_path = make_importing_script(&dir, &sub_path);
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    let interner = &store.interner;

    // Main module stays Loaded
    let main_mod =
        find_module_by_name(&compiler, interner, "main").expect("main module must be present");
    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "main must remain Loaded when sub is broken"
    );

    // Sub module must be BrokenRegion
    let sub_mod = find_module_by_name(&compiler, interner, "broken")
        .expect("sub (broken) module must be present");
    assert_eq!(
        sub_mod.state,
        ModuleState::BrokenRegion,
        "sub module with broken config must be BrokenRegion"
    );

    // Diagnostics must be emitted (at least the missing @end diagnostic)
    assert!(
        !diags.diags.is_empty(),
        "broken sub-module must produce diagnostics"
    );
    let has_end_diag = diags
        .diags
        .iter()
        .any(|d| d.core_msg.contains("@end") || d.core_msg.contains("broken"));
    assert!(
        has_end_diag,
        "at least one diagnostic should mention @end or 'broken'"
    );

    _ = fs::remove_dir_all(&dir);
}

/// When the main module imports a file that does not exist, the import should
/// be dropped and a diagnostic emitted, but `extract_all_modules` should still
/// return Ok with the main module.
#[test]
fn extract_all_modules_import_to_nonexistent() {
    let dir = create_temp_dir("extract_all_nonexistent_import");
    let main_path = create_chrn_file(
        &dir,
        "main.chrn",
        "import \"/definitely/does/not/exist.chrn\"\nlet x = 5\n",
    );
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed even with bad import");

    // Main module should still be present
    let interner = &store.interner;
    let main_mod =
        find_module_by_name(&compiler, interner, "main").expect("main module must be present");
    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "main module should be Loaded despite bad import"
    );

    // The main module may or may not have an import entry (ModuleFinder
    // drops imports with non-existent paths, so the import is absent).
    // Either way, only 1 user module should exist.
    assert_eq!(
        get_user_mod_count(&compiler),
        1,
        "only main module should exist"
    );

    // Diagnostics must be emitted about the missing import
    assert!(
        !diags.diags.is_empty(),
        "non-existent import must produce diagnostics"
    );

    _ = fs::remove_dir_all(&dir);
}

/// When two imports point to the same sub-module path, `extract_all_modules`
/// must produce only one sub-module in the compiler (deduplicated by the
/// `reserved_mod_ids` registration mechanism).
#[test]
fn extract_all_modules_duplicate_import() {
    let dir = create_temp_dir("extract_all_dup");
    let sub_path = make_sub_script(&dir);
    let sub_canonical = sub_path.to_string_lossy();

    let main_content = format!(
        "import \"{}\"\nimport \"{}\"\nlet x = 5\n",
        sub_canonical, sub_canonical
    );
    let main_path = create_chrn_file(&dir, "main.chrn", &main_content);
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    let interner = &store.interner;

    // Only 2 user modules (main + sub), not 3
    assert_eq!(
        get_user_mod_count(&compiler),
        2,
        "deduplication: expected 2 user modules, not 3"
    );

    // The sub module exists
    let sub_mod =
        find_module_by_name(&compiler, interner, "sub").expect("sub module must be present");
    assert_eq!(sub_mod.state, ModuleState::Loaded);

    // Importing the same path twice still registers `sub` twice
    assert_eq!(
        diags.diags.len(),
        1,
        "expected one duplicate identifier error"
    );
    assert!(
        diags
            .diags
            .iter()
            .any(|diag| diag.core_msg.contains("generated")),
        "expected a generated-identifier diagnostic"
    );

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// ImportKind span() accessor tests
// ===========================================================================

/// Tests that ImportKind::Source carries a SourceSpan that is preserved
/// through the pipeline.
#[test]
fn import_kind_span_preserved() {
    let dir = create_temp_dir("import_span");
    let sub_path = make_sub_script(&dir);
    let main_path = make_importing_script(&dir, &sub_path);
    let mut cfg = ChrnConfig::default();

    let sub_path_str = sub_path.to_string_lossy();

    let (main_mod, graph, _interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    assert!(!main_mod.imports.is_empty(), "at least one import expected");
    let import = &main_mod.imports[0];
    // Extract the span from ImportKind via pattern matching.
    // After extract_main, imports are still UnresolvedSource.
    let span = match &import.kind {
        ImportKind::UnresolvedSource(sp_path_id) => sp_path_id.span,
        ImportKind::Core(_) => panic!("expected UnresolvedSource import kind, got Core"),
        _ => panic!("expected UnresolvedSource, got {:?}", import.kind),
    };
    // The span should cover the import path without quotes.
    // Source is `import "PATH"\nlet x = 5\n`; span starts at byte 8, ends at 8 + path.len().
    let expected_start = 8;
    let expected_end = expected_start + sub_path_str.len() as u32;
    assert_eq!(
        span.start, expected_start,
        "import span must start at byte 8 (right after opening \"), got {}",
        span.start
    );
    assert_eq!(
        span.end,
        expected_end,
        "import span must end at byte {} (after path of length {}), got {}",
        expected_end,
        sub_path_str.len(),
        span.end
    );
    // Verify the span slices out the exact path from src bytes
    let region = &graph.region_arena()[main_mod.region_id.expect("main mod has region_id")];
    let span_bytes = &region.src_bytes[span.start as usize..span.end as usize];
    let span_str = String::from_utf8_lossy(span_bytes);
    assert_eq!(
        span_str, sub_path_str,
        "import span must extract the exact path string from src bytes, got '{}'",
        span_str
    );
    assert!(diags.diags.is_empty(), "no diagnostics expected");

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// ModuleGraph state assertions
// ===========================================================================

/// After `extract_main`, the `ModuleGraph` should have:
/// - `region_arena` of length 1
/// - `reserved_mod_ids` containing only the main module
/// - `other_mods` empty
/// - `seen` containing only the main path id
#[test]
fn module_graph_initial_state() {
    let dir = create_temp_dir("mod_graph_init");
    let main_path = make_simple_script(&dir);
    let mut cfg = ChrnConfig::default();

    let (_main_mod, graph, mut interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed");

    let main_path_id = interner.intern_path(&main_path);

    assert_eq!(
        graph.region_arena().len(),
        1,
        "region_arena must have 1 region"
    );
    assert_eq!(
        graph.registered_mod_ids().len(),
        1,
        "reserved_mod_ids must have 1 entry"
    );
    assert_eq!(
        graph.registered_mod_ids()[0],
        (main_path_id, ModuleId::new(0)),
        "reserved_mod_ids[0] must be (main_path, mod_id 0)"
    );
    // assert!(
    //     graph.other_mods().is_empty(),
    //     "other_mods must be empty initially"
    // );
    assert_eq!(graph.seen().len(), 1, "seen must have 1 entry");
    assert!(
        graph.seen().contains(&main_path_id),
        "seen must contain main"
    );
    assert!(diags.diags.is_empty());

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// Edge cases
// ===========================================================================

/// An empty file must produce a Loaded module with no imports and no bind.
#[test]
fn extract_main_empty_file() {
    let dir = create_temp_dir("extract_main_empty");
    let main_path = create_chrn_file(&dir, "empty.chrn", "");
    let mut cfg = ChrnConfig::default();

    let (main_mod, _graph, _interner, diags) = extract_main(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        &mut cfg,
    )
    .expect("extract_main must succeed on empty file");

    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "empty file should be Loaded"
    );
    assert!(main_mod.imports.is_empty(), "empty file has no imports");
    assert!(main_mod.bind.is_none(), "empty file has no bind");
    assert!(diags.diags.is_empty(), "empty file produces no diagnostics");

    _ = fs::remove_dir_all(&dir);
}

/// A file whose name is not valid UTF-8 should produce a terminal error. This
/// is tested by using a path that can't produce a file_prefix().
#[test]
fn extract_main_invalid_utf8_filename() {
    let dir = create_temp_dir("extract_main_invalid_utf8");
    // Create a file with a name that is not valid UTF-8 on unix
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let invalid_name = OsStr::from_bytes(&[0xFF, 0xFE]);
        let file_path = dir.join(invalid_name);
        fs::write(&file_path, "let x = 5\n").expect("failed to write invalid UTF-8 file");
        let mut cfg = ChrnConfig::default();

        let result = extract_main(
            &file_path,
            std::fs::File::open(&file_path).unwrap(),
            &mut cfg,
        );
        assert!(
            result.is_err(),
            "file with invalid UTF-8 name should produce Err"
        );
    }

    // On other platforms we skip this test as the file system may not
    // allow non-UTF-8 file names.

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// Module id assignment tests
// ===========================================================================

/// Verifies that module ids are assigned sequentially for a simple chain:
///   main (0) → a (1) → b (2)
///
/// Import layout (core is injected post-resolution with its own mod_id):
///   main.imports = [Source(a, 1), Core(3)]
///   a.imports    = [Source(b, 2), Core(3)]
///   b.imports    = [Core(3)]
#[test]
fn mod_ids_sequential_chain() {
    let dir = create_temp_dir("mod_ids_chain");

    // b.chrn (leaf)
    let b_path = create_chrn_file(&dir, "b.chrn", "let z = 1\n");
    // a.chrn imports b
    let b_canonical = b_path.to_string_lossy();
    let a_content = format!("import \"{}\"\nlet y = b::z\n", b_canonical);
    let a_path = create_chrn_file(&dir, "a.chrn", &a_content);
    // main.chrn imports a
    let a_canonical = a_path.to_string_lossy();
    let main_content = format!("import \"{}\"\nlet x = a::y\n", a_canonical);
    let main_path = create_chrn_file(&dir, "main.chrn", &main_content);

    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    let user_count = 3;
    assert_eq!(
        get_user_mod_count(&compiler),
        user_count,
        "expected 3 user modules (main, a, b)"
    );

    let interner = &store.interner;

    // -- Exact mod_id checks --
    let main_mod = find_module_by_name(&compiler, interner, "main").expect("main must be present");
    assert_eq!(main_mod.self_id, ModuleId::new(0), "main gets mod_id 0");

    let a_mod = find_module_by_name(&compiler, interner, "a").expect("a must be present");
    assert_eq!(
        a_mod.self_id,
        ModuleId::new(1),
        "a (imported first) gets mod_id 1"
    );

    let b_mod = find_module_by_name(&compiler, interner, "b").expect("b must be present");
    assert_eq!(b_mod.self_id, ModuleId::new(2), "b (leaf) gets mod_id 2");

    // -- Import numbering --
    let core_mod_id = ModuleId::new(user_count as u32); // 3 = last index

    // main imports a then core
    assert_eq!(main_mod.imports.len(), 2, "main: 1 user import + 1 core");
    assert_eq!(
        import_mod_id(&main_mod.imports[0]),
        Some(ModuleId::new(1)),
        "main.imports[0] must be Source(a, 1)"
    );
    assert!(
        matches!(main_mod.imports[1].kind, ImportKind::Core(id) if id == core_mod_id),
        "main.imports[1] must be Core(3), got {:?}",
        main_mod.imports[1].kind
    );

    // a imports b then core
    assert_eq!(a_mod.imports.len(), 2, "a: 1 user import + 1 core");
    assert_eq!(
        import_mod_id(&a_mod.imports[0]),
        Some(ModuleId::new(2)),
        "a.imports[0] must be Source(b, 2)"
    );
    assert!(
        matches!(a_mod.imports[1].kind, ImportKind::Core(id) if id == core_mod_id),
        "a.imports[1] must be Core(3), got {:?}",
        a_mod.imports[1].kind
    );

    // b only has core
    assert_eq!(b_mod.imports.len(), 1, "b: only core import");
    assert!(
        matches!(b_mod.imports[0].kind, ImportKind::Core(id) if id == core_mod_id),
        "b.imports[0] must be Core(3), got {:?}",
        b_mod.imports[0].kind
    );

    _ = fs::remove_dir_all(&dir);
}

/// Diamond dependency: main imports a and b, both a and b import shared.
/// shared is resolved only once; deduplication is verified through exact
/// module id checks.
///
/// Resolution order (deterministic):
///   main(0, pre-registered), a(1), b(2), shared(3, resolved as a's import)
///
/// Import layout (core is injected post-resolution):
///   main.imports   = [Source(a, 1), Source(b, 2), Core(4)]
///   a.imports      = [Source(shared, 3), Core(4)]
///   b.imports      = [Source(shared, 3), Core(4)]
///   shared.imports = [Core(4)]
#[test]
fn mod_ids_diamond_dedup() {
    let dir = create_temp_dir("mod_ids_diamond");

    // shared.chrn (leaf)
    let shared_path = create_chrn_file(&dir, "shared.chrn", "let val = 42\n");
    let shared_canonical = shared_path.to_string_lossy();

    // a.chrn imports shared
    let a_content = format!("import \"{}\"\nlet a_val = shared::val\n", shared_canonical);
    let a_path = create_chrn_file(&dir, "a.chrn", &a_content);

    // b.chrn imports shared
    let b_content = format!("import \"{}\"\nlet b_val = shared::val\n", shared_canonical);
    let b_path = create_chrn_file(&dir, "b.chrn", &b_content);

    // main.chrn imports a and b
    let a_canonical = a_path.to_string_lossy();
    let b_canonical = b_path.to_string_lossy();
    let main_content = format!(
        "import \"{}\"\nimport \"{}\"\nlet x = a::a_val + b::b_val\n",
        a_canonical, b_canonical
    );
    let main_path = create_chrn_file(&dir, "main.chrn", &main_content);

    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    assert!(
        diags.diags.is_empty(),
        "no diagnostics expected, got {:?}",
        diags
    );
    let user_count = 4;
    assert_eq!(
        get_user_mod_count(&compiler),
        user_count,
        "expected 4 user modules (main, a, b, shared)"
    );

    let interner = &store.interner;

    // — Exact mod_id checks (deterministic registration order) —
    let main_mod = find_module_by_name(&compiler, interner, "main").expect("main must be present");
    assert_eq!(main_mod.self_id, ModuleId::new(0), "main gets mod_id 0");

    let a_mod = find_module_by_name(&compiler, interner, "a").expect("a must be present");
    assert_eq!(
        a_mod.self_id,
        ModuleId::new(1),
        "a (first import of main) gets mod_id 1"
    );

    let b_mod = find_module_by_name(&compiler, interner, "b").expect("b must be present");
    assert_eq!(
        b_mod.self_id,
        ModuleId::new(2),
        "b (second import of main) gets mod_id 2"
    );

    let shared_mod =
        find_module_by_name(&compiler, interner, "shared").expect("shared must be present");
    assert_eq!(
        shared_mod.self_id,
        ModuleId::new(3),
        "shared (resolved as a's import) gets mod_id 3"
    );

    // — Import numbering —
    let core_mod_id = ModuleId::new(user_count as u32); // 4

    // main: a, then b, then core
    assert_eq!(main_mod.imports.len(), 3, "main: 2 user imports + 1 core");
    assert_eq!(
        import_mod_id(&main_mod.imports[0]),
        Some(ModuleId::new(1)),
        "main.imports[0] must be Source(a, 1)"
    );
    assert_eq!(
        import_mod_id(&main_mod.imports[1]),
        Some(ModuleId::new(2)),
        "main.imports[1] must be Source(b, 2)"
    );
    assert!(
        matches!(main_mod.imports[2].kind, ImportKind::Core(id) if id == core_mod_id),
        "main.imports[2] must be Core(4), got {:?}",
        main_mod.imports[2].kind
    );

    // a: shared, then core
    assert_eq!(a_mod.imports.len(), 2, "a: 1 user import + 1 core");
    assert_eq!(
        import_mod_id(&a_mod.imports[0]),
        Some(ModuleId::new(3)),
        "a.imports[0] must be Source(shared, 3)"
    );
    assert!(
        matches!(a_mod.imports[1].kind, ImportKind::Core(id) if id == core_mod_id),
        "a.imports[1] must be Core(4), got {:?}",
        a_mod.imports[1].kind
    );

    // b: shared (dedup) at imports[0], then core
    assert_eq!(
        b_mod.imports.len(),
        2,
        "b: 1 user import + 1 core (shared deduped)"
    );
    assert_eq!(
        import_mod_id(&b_mod.imports[0]),
        Some(ModuleId::new(3)),
        "b.imports[0] must be Source(shared, 3) (same mod_id as a.imports[0])"
    );
    assert!(
        matches!(b_mod.imports[1].kind, ImportKind::Core(id) if id == core_mod_id),
        "b.imports[1] must be Core(4), got {:?}",
        b_mod.imports[1].kind
    );

    // shared: only core
    assert_eq!(shared_mod.imports.len(), 1, "shared: only core import");
    assert!(
        matches!(shared_mod.imports[0].kind, ImportKind::Core(id) if id == core_mod_id),
        "shared.imports[0] must be Core(4), got {:?}",
        shared_mod.imports[0].kind
    );

    // — Deduplication cross-check —
    assert_eq!(
        import_mod_id(&a_mod.imports[0]),
        import_mod_id(&b_mod.imports[0]),
        "a and b carry the same shared mod_id"
    );

    _ = fs::remove_dir_all(&dir);
}

/// When an import points to a non-existent file, the import is dropped
/// at ModuleFinder level (no `Import` record is created) and no module id
/// is consumed.  The next valid import gets the next sequential id.
///
/// Import layout:
///   main.imports = [Source(sub, 1), Core(2)]
///   sub.imports  = [Core(2)]
#[test]
fn mod_ids_nonexistent_import_does_not_consume_id() {
    let dir = create_temp_dir("mod_ids_missing");

    // sub.chrn exists
    let sub_path = make_sub_script(&dir);
    let sub_canonical = sub_path.to_string_lossy();

    // main.chrn imports a missing file then the valid sub
    let main_content = format!(
        "import \"/definitely/does/not/exist.chrn\"\nimport \"{}\"\nlet x = 5\n",
        sub_canonical
    );
    let main_path = create_chrn_file(&dir, "main.chrn", &main_content);

    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    let interner = &store.interner;

    // -- Main assertions --
    let user_count = 2;
    assert_eq!(
        get_user_mod_count(&compiler),
        user_count,
        "expected 2 user modules (main + sub)"
    );
    let core_mod_id = ModuleId::new(user_count as u32); // 2

    let main_mod = find_module_by_name(&compiler, interner, "main").expect("main must be present");
    assert_eq!(main_mod.self_id, ModuleId::new(0), "main gets mod_id 0");
    assert_eq!(
        main_mod.state,
        ModuleState::Loaded,
        "main should be Loaded despite bad import"
    );

    // The missing-file import is dropped entirely at ModuleFinder level:
    // parse_import returns Err before any import record is created.
    assert_eq!(main_mod.imports.len(), 2, "main: 1 Source (sub) + 1 Core");

    // imports[0] = Source(sub, 1)
    let sub_import_mod_id = match &main_mod.imports[0].kind {
        ImportKind::Source(_, m_id) => Some(*m_id),
        _ => None,
    }
    .expect("main.imports[0] should be Source(resolved sub)");
    assert_eq!(
        sub_import_mod_id,
        ModuleId::new(1),
        "main.imports[0] must be Source(sub, 1)"
    );

    // imports[1] = Core(2)
    assert!(
        matches!(main_mod.imports[1].kind, ImportKind::Core(id) if id == core_mod_id),
        "main.imports[1] must be Core(2), got {:?}",
        main_mod.imports[1].kind
    );

    // -- Sub assertions --
    let sub_mod = find_module_by_name(&compiler, interner, "sub").expect("sub must be present");
    assert_eq!(
        sub_mod.self_id,
        ModuleId::new(1),
        "sub gets mod_id 1 (sequential, no gap)"
    );
    assert_eq!(
        sub_import_mod_id, sub_mod.self_id,
        "main's import to sub must carry sub's mod_id"
    );

    // sub has only the core import
    assert_eq!(sub_mod.imports.len(), 1, "sub: only core import");
    assert!(
        matches!(sub_mod.imports[0].kind, ImportKind::Core(id) if id == core_mod_id),
        "sub.imports[0] must be Core(2), got {:?}",
        sub_mod.imports[0].kind
    );

    // Diagnostics must be emitted for the missing file
    assert!(
        !diags.diags.is_empty(),
        "non-existent import must produce diagnostics"
    );

    _ = fs::remove_dir_all(&dir);
}

/// Verifies that the main module always gets mod_id 0 in the final compiler
/// and its only import is the core module.
///
/// Import layout:
///   main.imports = [Core(1)]
#[test]
fn mod_ids_main_module_id_is_zero() {
    let dir = create_temp_dir("mod_ids_main_zero");
    let main_path = make_simple_script(&dir);
    let cfg = ChrnConfig::default();
    let mut reporter = Reporter::new(100);

    let (compiler, _store, _diags) = extract_all_modules(
        &main_path,
        std::fs::File::open(&main_path).unwrap(),
        cfg,
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    let user_count = 1;
    assert_eq!(get_user_mod_count(&compiler), user_count);
    let core_mod_id = ModuleId::new(user_count as u32); // 1

    assert_eq!(
        compiler.mods[ModuleId::new(0)].self_id,
        ModuleId::new(0),
        "main module at index 0 must have mod_id 0"
    );

    let main_mod = &compiler.mods[ModuleId::new(0)];
    assert_eq!(main_mod.imports.len(), 1, "main: only core import");
    assert!(
        matches!(main_mod.imports[0].kind, ImportKind::Core(id) if id == core_mod_id),
        "main.imports[0] must be Core(1), got {:?}",
        main_mod.imports[0].kind
    );

    _ = fs::remove_dir_all(&dir);
}

// ===========================================================================
// Duplicate module identifier tests
// ===========================================================================

/// Runs the full extraction over `main_path` and returns every diagnostic
/// header message in emission order.
fn extract_msgs(main_path: &Path) -> Vec<String> {
    let mut reporter = Reporter::new(100);
    let (_, _, diags) = extract_all_modules(
        main_path,
        fs::File::open(main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    diags.diags.iter().map(|d| d.core_msg.clone()).collect()
}

fn extract_has_errors(main_path: &Path) -> bool {
    let mut reporter = Reporter::new(100);
    let (_, _, diags) = extract_all_modules(
        main_path,
        fs::File::open(main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("extract_all_modules must succeed");

    diags.has_err()
}

/// Writes `main.chrn` in `dir` whose body is `imports` followed by one `let`.
fn make_main_with(dir: &Path, imports: &[String]) -> PathBuf {
    let mut content = String::new();
    for imp in imports {
        content.push_str(imp);
        content.push('\n');
    }
    content.push_str("let x = 5\n");
    create_chrn_file(dir, "main.chrn", &content)
}

fn import_stmt(path: &Path) -> String {
    format!("import \"{}\"", path.display())
}

fn import_stmt_as(path: &Path, alias: &str) -> String {
    format!("import \"{}\" as {alias}", path.display())
}

fn assert_has_diagnostic_containing(main_path: &Path, expected_count: usize, text: &str) {
    let messages = extract_msgs(main_path);
    assert_eq!(messages.len(), expected_count);
    assert!(
        messages.iter().any(|message| message.contains(text)),
        "expected a diagnostic containing `{text}`, got {messages:?}"
    );
}

/// Two distinct files sharing a file name collide on that name.
#[test]
fn dup_import_same_name_different_dirs() {
    let dir = create_temp_dir("dup_same_name_dirs");
    let a = create_chrn_file(&dir, "a/sub.chrn", "let y = 1\n");
    let b = create_chrn_file(&dir, "b/sub.chrn", "let z = 2\n");

    let main_path = make_main_with(&dir, &[import_stmt(&a), import_stmt(&b)]);

    assert_has_diagnostic_containing(&main_path, 1, "generated");

    _ = fs::remove_dir_all(&dir);
}

/// An import whose file name matches the importing module's own name collides
/// with that module.
#[test]
fn dup_import_matches_importer_name() {
    let dir = create_temp_dir("dup_matches_importer");
    let other_main = create_chrn_file(&dir, "nested/main.chrn", "let y = 1\n");

    let main_path = make_main_with(&dir, &[import_stmt(&other_main)]);

    assert_has_diagnostic_containing(&main_path, 1, "generated");

    _ = fs::remove_dir_all(&dir);
}

/// A module importing itself is not a duplicate identifier.
#[test]
fn self_import_is_not_duplicate() {
    let dir = create_temp_dir("self_import");
    let main_path = dir.join("main.chrn");
    // Written twice: the import needs the canonical path of the file it lives in.
    write_file(&main_path, "let x = 5\n");
    let canonical = fs::canonicalize(&main_path).expect("canonicalize");
    write_file(
        &main_path,
        &format!("{}\nlet x = 5\n", import_stmt(&canonical)),
    );

    assert!(
        !extract_has_errors(&canonical),
        "self import must not be reported as a duplicate identifier",
    );

    _ = fs::remove_dir_all(&dir);
}

/// Distinct aliases keep two same-named imports apart.
#[test]
fn import_same_name_distinct_aliases_is_not_duplicate() {
    let dir = create_temp_dir("distinct_aliases");
    let a = create_chrn_file(&dir, "a/sub.chrn", "let y = 1\n");
    let b = create_chrn_file(&dir, "b/sub.chrn", "let z = 2\n");

    let main_path = make_main_with(&dir, &[import_stmt_as(&a, "s1"), import_stmt_as(&b, "s2")]);

    assert!(
        extract_msgs(&main_path).is_empty(),
        "distinct aliases must not collide",
    );

    _ = fs::remove_dir_all(&dir);
}

/// An alias that matches another import's file name collides with it.
#[test]
fn dup_import_alias_matches_other_import_name() {
    let dir = create_temp_dir("alias_vs_name");
    let a = create_chrn_file(&dir, "a.chrn", "let y = 1\n");
    let sub = make_sub_script(&dir);

    let main_path = make_main_with(&dir, &[import_stmt_as(&a, "sub"), import_stmt(&sub)]);

    assert_has_diagnostic_containing(&main_path, 1, "generated");

    _ = fs::remove_dir_all(&dir);
}

/// Two differently named imports sharing one alias collide on that alias.
#[test]
fn dup_import_aliases_collide_across_names() {
    let dir = create_temp_dir("alias_vs_alias");
    let a = create_chrn_file(&dir, "a.chrn", "let y = 1\n");
    let b = create_chrn_file(&dir, "b.chrn", "let z = 2\n");

    let main_path = make_main_with(&dir, &[import_stmt_as(&a, "x"), import_stmt_as(&b, "x")]);

    assert_has_diagnostic_containing(&main_path, 1, "alias");

    _ = fs::remove_dir_all(&dir);
}

/// An alias that matches the importing module's own name collides with it.
#[test]
fn dup_import_alias_matches_importer_name() {
    let dir = create_temp_dir("alias_vs_importer");
    let sub = make_sub_script(&dir);

    let main_path = make_main_with(&dir, &[import_stmt_as(&sub, "main")]);

    assert_has_diagnostic_containing(&main_path, 1, "alias");

    _ = fs::remove_dir_all(&dir);
}

/// Duplicate detection is scoped per module: two modules may each import a
/// file named `sub` without colliding with one another.
#[test]
fn duplicate_tracking_is_per_module() {
    let dir = create_temp_dir("dup_per_module");
    let leaf = create_chrn_file(&dir, "leaf/sub.chrn", "let w = 3\n");
    let mid = create_chrn_file(&dir, "mid/mid.chrn", &format!("{}\n", import_stmt(&leaf)));
    let other = create_chrn_file(&dir, "other/sub.chrn", "let y = 1\n");

    let main_path = make_main_with(&dir, &[import_stmt(&mid), import_stmt(&other)]);

    assert!(
        extract_msgs(&main_path).is_empty(),
        "identifiers in different modules must not collide",
    );

    _ = fs::remove_dir_all(&dir);
}

/// A bind can intern a module's path before its first import. Repeated imports
/// must still share one module even when path IDs differ from discovery order.
#[test]
fn bind_path_interning_preserves_module_deduplication() {
    let dir = create_temp_dir("bind_path_dedup");
    let shared = create_chrn_file(&dir, "shared.chrn", "@end\n{}\n");
    let a = create_chrn_file(&dir, "a.chrn", &format!("{}\n", import_stmt(&shared)));
    let main_path = create_chrn_file(
        &dir,
        "main.chrn",
        &format!(
            "bind \"{}\"\n{}\n{}\n",
            shared.display(),
            import_stmt(&a),
            import_stmt(&shared),
        ),
    );
    let mut reporter = Reporter::new(100);
    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("module extraction must succeed");
    _ = fs::remove_dir_all(&dir);

    assert!(diags.diags.is_empty(), "unexpected diagnostics: {diags:?}");
    assert_eq!(get_user_mod_count(&compiler), 3, "main, a, and shared only");
    let main_mod = &compiler.mods[ModuleId::new(0)];
    let a_mod = find_module_by_name(&compiler, &store.interner, "a").unwrap();
    let shared_mod = find_module_by_name(&compiler, &store.interner, "shared").unwrap();
    assert_eq!(
        import_mod_id(&main_mod.imports[1]),
        Some(shared_mod.self_id)
    );
    assert_eq!(import_mod_id(&a_mod.imports[0]), Some(shared_mod.self_id));
}

/// An alias belongs to its importing module and must not collide with an
/// otherwise distinct import inside the aliased module.
#[test]
fn parent_import_alias_does_not_rename_child_module() {
    let dir = create_temp_dir("parent_alias_scope");
    let x = create_chrn_file(&dir, "x.chrn", "let value = 1\n");
    let a = create_chrn_file(&dir, "a.chrn", &format!("{}\n", import_stmt(&x)));
    let main_path = make_main_with(&dir, &[import_stmt_as(&a, "x")]);
    let mut reporter = Reporter::new(100);
    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("module extraction must succeed");
    _ = fs::remove_dir_all(&dir);

    assert!(
        diags.diags.is_empty(),
        "parent aliases are local to the parent: {diags:?}"
    );
    assert_eq!(get_user_mod_count(&compiler), 3);
    assert_eq!(
        store
            .interner
            .extract_utf8_str(compiler.mods[ModuleId::new(1)].name_id),
        "a"
    );
    assert_eq!(
        import_mod_id(&compiler.mods[ModuleId::new(1)].imports[0]),
        Some(ModuleId::new(2))
    );
}

/// Loading a path through another name must not let a later import claim the
/// compiler-owned `core` identifier.
#[test]
fn reserved_core_alias_is_rejected_for_already_loaded_path() {
    let dir = create_temp_dir("core_alias_seen_path");
    let sub = make_sub_script(&dir);
    let main_path = make_main_with(&dir, &[import_stmt(&sub), import_stmt_as(&sub, "core")]);
    let mut reporter = Reporter::new(100);
    let (compiler, _, diags) = extract_all_modules(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("module extraction must recover from the reserved alias");
    _ = fs::remove_dir_all(&dir);

    let messages: Vec<_> = diags
        .diags
        .iter()
        .map(|diag| diag.core_msg.as_str())
        .collect();
    assert_eq!(messages, ["`core` is a reserved module identifier"]);
    assert!(diags.has_err());
    assert_eq!(get_user_mod_count(&compiler), 2);
    let main_mod = &compiler.mods[ModuleId::new(0)];
    assert_eq!(import_mod_id(&main_mod.imports[0]), Some(ModuleId::new(1)));
    assert!(matches!(
        main_mod.imports[1].kind,
        ImportKind::ErrorSource(_)
    ));
    assert!(matches!(main_mod.imports[2].kind, ImportKind::Core(_)));
}

/// A reserved alias is an error in the importing source, regardless of whether
/// its target has already been loaded through another import.
#[test]
fn reserved_core_alias_diagnostic_identifies_importer() {
    let dir = fs::canonicalize(create_temp_dir("core_alias_diag_importer")).unwrap();
    let sub = make_sub_script(&dir);
    let mut diagnostic_paths = Vec::new();
    for already_loaded in [false, true] {
        let mut imports = Vec::new();
        if already_loaded {
            imports.push(import_stmt(&sub));
        }
        imports.push(import_stmt_as(&sub, "core"));
        let main_path = make_main_with(&dir, &imports);
        let mut reporter = Reporter::new(100);
        let (compiler, store, diags) = extract_all_modules(
            &main_path,
            fs::File::open(&main_path).unwrap(),
            ChrnConfig::default(),
            &mut reporter,
        )
        .expect("reserved aliases must allow extraction to recover");
        assert_eq!(diags.diags.len(), 1);
        let diag = &diags.diags[0];
        assert_eq!(diag.core_msg, "`core` is a reserved module identifier");
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert!(matches!(
            compiler.mods[ModuleId::new(0)].imports[usize::from(already_loaded)].kind,
            ImportKind::ErrorSource(_)
        ));
        diagnostic_paths.push(
            store
                .interner
                .extract_path_from_path_id(diag.path_id)
                .to_path_buf(),
        );
    }
    _ = fs::remove_dir_all(&dir);
    assert_eq!(
        diagnostic_paths,
        [dir.join("main.chrn"), dir.join("main.chrn")]
    );
}

/// An import-open error must locate the import statement in its source file,
/// rather than pairing that source's annotation with the target directory.
#[cfg(unix)]
#[test]
fn directory_import_diagnostic_identifies_importer() {
    let dir = create_temp_dir("directory_import_diag");
    let target = dir.join("target");
    fs::create_dir_all(&target).unwrap();
    let target = fs::canonicalize(target).unwrap();
    let main_path = make_main_with(&dir, &[import_stmt(&target)]);
    let mut reporter = Reporter::new(100);
    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("an import-open failure must allow extraction to recover");
    _ = fs::remove_dir_all(&dir);

    assert_eq!(get_user_mod_count(&compiler), 1);
    assert_eq!(diags.diags.len(), 1);
    let diag = &diags.diags[0];
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(
        diag.core_msg,
        format!("The path \"{}\" is a directory", target.display())
    );
    assert_eq!(diag.annotations.len(), 1);
    assert_eq!(diag.annotations[0].kind, AnnotationKind::Primary);
    assert_eq!(
        diag.annotations[0].span,
        SourceSpan::new(
            SourceRegionId::new(0),
            8,
            8 + target.as_os_str().len() as u32
        )
    );
    assert!(matches!(
        compiler.mods[ModuleId::new(0)].imports[0].kind,
        ImportKind::ErrorSource(_)
    ));
    assert_eq!(
        store.interner.extract_path_from_path_id(diag.path_id),
        main_path
    );
}

/// A read error after opening an import must attribute its annotation to the
/// importer, just like an import-open error. Linux procfs exposes a file that
/// opens successfully but reliably returns EIO when read at offset zero.
#[cfg(target_os = "linux")]
#[test]
fn import_read_error_diagnostic_identifies_importer() {
    let dir = create_temp_dir("import_read_error_diag");
    let target = fs::canonicalize("/proc/self/mem").unwrap();
    let main_path = make_main_with(&dir, &[import_stmt(&target)]);
    let mut reporter = Reporter::new(100);
    let (compiler, store, diags) = extract_all_modules(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("an import-read failure must allow extraction to recover");
    _ = fs::remove_dir_all(&dir);

    assert_eq!(get_user_mod_count(&compiler), 1);
    assert_eq!(store.region_arena.len(), 1);
    assert!(matches!(
        compiler.mods[ModuleId::new(0)].imports[0].kind,
        ImportKind::ErrorSource(_)
    ));
    assert_eq!(diags.diags.len(), 1);
    let diag = &diags.diags[0];
    assert_eq!(diag.level, DiagnosticLevel::Error);
    // EIO proves the file opened and reached ConfigLoader's read-error path.
    assert_eq!(
        diag.core_msg,
        std::io::Error::from_raw_os_error(5).to_string()
    );
    assert_eq!(diag.annotations.len(), 1);
    assert_eq!(diag.annotations[0].kind, AnnotationKind::Primary);
    assert_eq!(
        diag.annotations[0].span,
        SourceSpan::new(
            SourceRegionId::new(0),
            8,
            8 + target.as_os_str().len() as u32
        )
    );
    assert_eq!(
        store.interner.extract_path_from_path_id(diag.path_id),
        main_path,
        "the read-error diagnostic must identify the file containing its annotation"
    );
}

/// Different spellings of the entry file must still resolve a self import to
/// module zero, without loading the same declarations in a second module.
#[test]
fn noncanonical_entry_path_preserves_self_import_identity() {
    let dir = create_temp_dir("noncanonical_entry_identity");
    fs::create_dir_all(dir.join("nested")).unwrap();
    let main_path = create_chrn_file(&dir, "main.chrn", "let x = 5\n");
    write_file(
        &main_path,
        &format!("{}\nlet x = 5\n", import_stmt(&main_path)),
    );
    let entry_path = dir.join("nested/../main.chrn");
    let mut reporter = Reporter::new(100);
    let (compiler, _, diags) = extract_all_modules(
        &entry_path,
        fs::File::open(&entry_path).unwrap(),
        ChrnConfig::default(),
        &mut reporter,
    )
    .expect("a valid entry path containing a parent component must load");
    _ = fs::remove_dir_all(&dir);

    assert_eq!(
        get_user_mod_count(&compiler),
        1,
        "a self import must not load the entry file twice"
    );
    assert_eq!(
        import_mod_id(&compiler.mods[ModuleId::new(0)].imports[0]),
        Some(ModuleId::new(0))
    );
    assert_eq!(diags.diags.len(), 1);
    assert_eq!(diags.diags[0].level, DiagnosticLevel::Warn);
    assert_eq!(diags.diags[0].core_msg, "Self imports have no effect");
}

/// A source named `core` cannot be used as the entry module.
#[test]
fn reserved_core_entry_is_rejected() {
    let dir = create_temp_dir("reserved_core_entry");
    let main_path = create_chrn_file(&dir, "core.chrn", "let value = 1\n");
    let result = extract_main(
        &main_path,
        fs::File::open(&main_path).unwrap(),
        &mut ChrnConfig::default(),
    );
    _ = fs::remove_dir_all(&dir);

    let init_err = match result {
        Err(init_err) => init_err,
        Ok(_) => panic!("a source named core must terminate entry-module extraction"),
    };
    let ConfigLoadError::Diagnostic(diag) = init_err.cfg_err else {
        panic!("a reserved entry-module name must produce a language diagnostic");
    };
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.core_msg, "`core` is a reserved module identifier");
    assert_eq!(
        init_err.interner.extract_path_from_path_id(diag.path_id),
        main_path
    );
}

/// An import alias cannot bypass the prohibition on a source named `core`.
/// Rejecting that source must preserve unrelated imports and dense module IDs.
#[test]
fn reserved_core_source_is_rejected_even_with_alias() {
    let dir = create_temp_dir("reserved_core_source");
    let core_source = create_chrn_file(&dir, "core.chrn", "let value = 1\n");
    let valid_source = make_sub_script(&dir);
    for (aliased, core_import) in [
        (false, import_stmt(&core_source)),
        (true, import_stmt_as(&core_source, "defs")),
    ] {
        let main_path = make_main_with(&dir, &[core_import, import_stmt(&valid_source)]);
        let mut reporter = Reporter::new(100);
        let (compiler, store, diags) = extract_all_modules(
            &main_path,
            fs::File::open(&main_path).unwrap(),
            ChrnConfig::default(),
            &mut reporter,
        )
        .expect("a prohibited import must allow extraction of unrelated modules");
        let messages: Vec<_> = diags
            .diags
            .iter()
            .map(|diag| (diag.level, diag.core_msg.as_str()))
            .collect();
        assert_eq!(
            messages,
            [(DiagnosticLevel::Error, "`core` is a reserved module identifier")],
            "reserved source rejection must hold when aliased={aliased}"
        );
        assert_eq!(
            store.interner.extract_path_from_path_id(diags.diags[0].path_id),
            main_path
        );
        assert_eq!(get_user_mod_count(&compiler), 2);
        let main_mod = &compiler.mods[ModuleId::new(0)];
        assert!(matches!(main_mod.imports[0].kind, ImportKind::ErrorSource(_)));
        assert_eq!(import_mod_id(&main_mod.imports[1]), Some(ModuleId::new(1)));
        assert_eq!(
            compiler.mods[ModuleId::new(1)].name_id,
            store.interner.get_interned_id_from_str("sub").unwrap()
        );
    }
    _ = fs::remove_dir_all(&dir);
}
