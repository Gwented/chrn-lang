use chrn_utils::{
    id_types::SourceId,
    source_utils::source_manager::{SourceKind, SourceManager},
};
use std::{collections::VecDeque, io::Cursor};

pub mod module_concepts;
pub mod module_finder;

use crate::{
    chrn_config::{ChrnConfig, chrn_perf::ChrnPerfStage},
    config_loader::{ConfigLoader, ConfigLoaderOutput},
    module::module_concepts::{
        Import, ImportKind, Module, ModuleFlagMetadata, ModuleGraph, ModuleIdent, ModuleKind,
        ModuleState, UserModuleMetadata,
    },
    semantic::checker_helpers::DuplicateTracker,
};
use chrn_utils::{
    arena::Arena,
    core_error::{self, ConfigLoadError, ModuleInitError},
    err_codes::ErrorCode,
    files::file_ops,
    id_types::{InternedIdUTF8, ModuleId, SourceRegionId},
    intern::{self, Intern},
    source_utils::{
        source_diagnostic::{
            DiagnosticLevel, SourceDiagnostic, SourceDiagnosticBuilder, SourceDiagnosticSink,
            SourceDiagnosticSummary, annotations::AnnotationKind,
        },
        source_region::SourceRegion,
    },
};

use crate::{
    module::module_finder::ModuleFinder,
    script_compiler::{
        ScriptCompiler, reporter::Reporter, script_compiler_store::ScriptCompilerStore,
    },
};

// Maybe not
/// Convenience over `extract_main` and `extract_modules`.
pub fn extract_all_modules(
    src_kind: SourceKind,
    interner: Intern,
    src_manager: SourceManager,
    mut cfg: ChrnConfig,
    //TODO: Need to figure out how reporter should be treated
    reporter: &mut Reporter,
) -> Result<(ScriptCompiler, ScriptCompilerStore, SourceDiagnosticSummary), ModuleInitError> {
    let mut summary = SourceDiagnosticSummary::default();

    let (main_mod, graph, interner, new_summary) =
        extract_main(src_kind, interner, src_manager, &mut cfg)?;
    summary.merge(new_summary);

    let (compiler, store, new_summary) = extract_modules(main_mod, graph, reporter, interner, cfg);
    summary.merge(new_summary);

    Ok((compiler, store, summary))
}

/// Attempts to extract only the entry point
///
/// Canonicalizes paths for all file sources.
///
/// NOTE: The corresponding `extract_modules` function expects the pure graph output from this
/// function to be given to it, otherwise the behavior is undefined.
pub fn extract_main(
    src_kind: SourceKind,
    mut interner: Intern,
    mut src_manager: SourceManager,
    cfg: &mut ChrnConfig,
) -> Result<(Module, ModuleGraph, Intern, SourceDiagnosticSummary), ModuleInitError> {
    let mut summary = SourceDiagnosticSummary::default();

    //TODO: Maybe warn that "virtual" was intended because there exists a chrn in-memory, or
    //in path
    // let ident = interner.intern_str("CHERNGENTS");
    // src_manager.register_in_memory(ident, b"import \"chrn\"".to_vec());

    let main_src_id = match src_kind {
        SourceKind::Path(src_path) => {
            let path = match src_path.path.canonicalize() {
                Ok(p) => p,
                Err(err) => {
                    return Err(ModuleInitError::new(
                        None,
                        src_manager,
                        interner,
                        ConfigLoadError::IO(err),
                    ));
                }
            };
            src_manager.register_path(&path).inner()
        }
        SourceKind::InMemory(src_in_mem) => src_manager
            .register_in_memory(src_in_mem.name_id, src_in_mem.src)
            .inner(),
    };

    // A little concerning here because you CAN stop after this point, meaning the timer doesn't
    // matter, but that should probably be factored in externally and just disallow perf entirely.
    // Or at least warn it's inaccurate, or just, separate stages.
    cfg.perf_tracker_mut().start(main_src_id);

    //TODO: -- SHARE --
    let mut region_arena: Arena<SourceRegion, SourceRegionId> = Arena::with_capacity(1);
    let main_region_id = SourceRegionId::new(0);

    // Not sure if main should even recover from this beyond having an existent region
    let main_src_kind = src_manager.extract_source(main_src_id);
    let (main_region, main_mod_state) =
        match load_source(main_src_kind, main_src_id, main_region_id, cfg) {
            ConfigLoaderOutput::Success(region, loader_summary) => {
                summary.merge(loader_summary);
                (region, ModuleState::Loaded)
            }
            // This could be pretty bad to leave here because
            ConfigLoaderOutput::Broken(broken_region, cfg_err) => {
                let diag = match cfg_err {
                    ConfigLoadError::Diagnostic(diag) => diag,
                    ConfigLoadError::IO(io_err) => {
                        let src_path = match src_manager.extract_source(main_src_id) {
                            SourceKind::Path(s) => s,
                            SourceKind::InMemory(_) => unreachable!(),
                        };

                        let err_str = core_error::form_string_from_io_err(&io_err, &src_path.path)
                            .unwrap_or(io_err.to_string());
                        SourceDiagnostic::builder(
                            //TODO: Should this have a code?
                            None,
                            DiagnosticLevel::Error,
                            err_str,
                            main_src_id,
                        )
                        .build()
                    }
                };

                summary.push_diag(diag);
                (broken_region, ModuleState::BrokenRegion)
            }
            ConfigLoaderOutput::UnrecoverableErr(cfg_err) => {
                return Err(ModuleInitError::new(None, src_manager, interner, cfg_err));
            }
        };
    //TODO: -- SHARE --

    // NOTE: Aliasing maybe.
    let main_name_id = match src_manager.extract_source(main_src_id) {
        SourceKind::InMemory(src) => src.name_id,
        SourceKind::Path(src) => {
            let Some(file_name) = src.path.file_prefix().and_then(|name| name.to_str()) else {
                let core_msg = format!(
                    "Path \"{}\" does not have a UTF-8 file name usable within the program",
                    src.path.display()
                );
                let diag = SourceDiagnostic::builder(
                    ErrorCode::ImportErr.into(),
                    DiagnosticLevel::Error,
                    core_msg,
                    main_src_id,
                ).add_note("The entry point is the only module that cannot use non-UTF-8 in it's file name")
                .build();
                region_arena.push(main_region);
                return Err(ModuleInitError::new(
                    Some(region_arena),
                    src_manager,
                    interner,
                    ConfigLoadError::Diagnostic(diag),
                ));
            };
            interner.intern_str(file_name)
        }
    };

    //TODO: Aliases for main. Ensure if named core, doesn't matter since alias doesn't align. Beep.
    let main_mod_id = ModuleId::new(0);

    // This is a Vector relationship stored where, the path id of an import is stored along with a
    // module id. So, we store main, go into main's imports then fill in OR create the module id of
    // unknown imports based off of reserved len(). This works during the recursive process because
    // it MUST look at all imports before ever recursing further, and it reserves it's spot as
    // `None`.
    let registered_mod_ids: Vec<(SourceId, ModuleId)> = vec![(main_src_id, main_mod_id)];

    // Maybe don't inherently declare here since it's a little odd to use a returned variable as
    // the main variable to then collect future diagnostics?
    // Maybe not?
    let (main_bind, main_imports, mut finder_summary) =
        ModuleFinder::new(&main_region.src_bytes, cfg, &main_region)
            .collect_imports(&mut interner, &mut src_manager);
    summary.append_summary(&mut finder_summary);

    // Pushing main's region
    region_arena.push(main_region);

    // Purposefully done after data is collected.
    if let Err(builder) = ensure_not_reserved(main_name_id, main_src_id) {
        let cfg_err = ConfigLoadError::Diagnostic(builder.build());
        let init_err = ModuleInitError::new(Some(region_arena), src_manager, interner, cfg_err);
        return Err(init_err);
    }

    let user_meta = UserModuleMetadata::new(main_region_id);

    // No errors are immediately terminal after this point since the main entry point now exists and
    // can be viewed even if it's the only module that was successfully created
    let main_mod = Module::new(
        main_name_id,
        main_mod_state,
        // All modules collected here are only user-defined so no flags can be touched.
        ModuleFlagMetadata::none(),
        main_mod_id,
        main_bind,
        main_imports,
        ModuleKind::User(user_meta),
    );

    let seen: Vec<SourceId> = vec![main_src_id];
    let graph = ModuleGraph::new(region_arena, registered_mod_ids, seen, src_manager);

    Ok((main_mod, graph, interner, summary))
}

/// Is intended to pick up **EXACTLY** where `extract_main` left off.
///
/// Returns compiler, it's data storing counter-part, and possible diagnostics
/// No `Result` is returned, but internally there may be many states, such as module states, which
/// might be incomplete, as well as diagnostics.
pub fn extract_modules(
    main_mod: Module,
    mut graph: ModuleGraph,
    reporter: &mut Reporter,
    mut interner: Intern,
    mut cfg: ChrnConfig,
) -> (ScriptCompiler, ScriptCompilerStore, SourceDiagnosticSummary) {
    debug_assert_eq!(main_mod.self_id.id, 0);
    let mut summary = SourceDiagnosticSummary::default();

    // This ONLY contains verified modules.
    // Modules are only pushed when their all their imports are processed.
    let mut valid_mods: Arena<Module, ModuleId> = Arena::with_capacity(main_mod.imports.len());

    // If this is true then the outer loop must stop (HELLO I AM A LOOP LABEL)
    // Only used when max modules have been exceeded
    let mut should_break_outer = false;

    const MAX_MODS_USIZE: usize = chrn_utils::MAX_MODULES as usize;

    // Duplicate tracker that will be re-used across different module contexts.
    let mut mod_ident_tracker: DuplicateTracker<ModuleIdent> =
        DuplicateTracker::with_capacities(main_mod.imports.len() + 1, 0);

    // Starting at [root]
    let mut pending_mods: VecDeque<Module> = vec![main_mod].into();

    // Work-list processing of each module
    //
    // If the import from the popped module has not been seen before, mark it as seen so that it is
    // not processed again, create it's module, then push it to the end of the queue so that it's
    // imports can also be processed as modules.
    while let Some(mut importer_mod) = pending_mods.pop_front() {
        let importer_region_id = importer_mod.kind.expect_user().region_id;
        let importer_src_id = graph.region_arena[importer_region_id].src_id;

        // A module binds its own name, and only its imports can carry an alias, so this is never
        // an alias
        let root_ident = ModuleIdent::new(importer_mod.name_id, false, None);

        mod_ident_tracker.insert_or_store(root_ident);

        // This diner still makes Coke the old-fashioned way
        for imp_idx in 0..importer_mod.imports.len() {
            let imp = importer_mod.imports[imp_idx].clone();

            // We haven't made the compiler yet so no other types of imports should exist
            let ImportKind::UnresolvedSource(ref sp_src_id) = imp.kind else {
                unreachable!();
            };

            //NOTE: If this were allowed, some odd behavior would exist, which should likely just
            //be omitted entirely. This is the only domain where anything "reserved" in identifier
            //is stopped.
            if let Err(mut builder) = ensure_not_reserved(imp.name_id, importer_src_id) {
                builder = builder.add_annotation(sp_src_id.span, AnnotationKind::Primary, None);
                summary.push_diag(builder.build());
                importer_mod.imports[imp_idx].kind = ImportKind::ErrorSource(sp_src_id.clone());
                continue;
            }

            if let Some(sp_alias_id) = imp.sp_alias_id.clone() {
                if let Err(mut builder) = ensure_not_reserved(sp_alias_id.inner, importer_src_id) {
                    builder =
                        builder.add_annotation(sp_alias_id.span, AnnotationKind::Primary, None);
                    summary.push_diag(builder.build());
                    importer_mod.imports[imp_idx].kind = ImportKind::ErrorSource(sp_src_id.clone());
                    continue;
                }
            }

            // If is_importer, that means it shouldn't count as a duplicate identifier. So, if `main`
            // imports `main` and both mains are the same, no error is emitted because that would be
            // the wrong behavior. This is solely aa self import guard.
            let is_importer = sp_src_id.inner == importer_src_id && imp.sp_alias_id.is_none();

            // The identifier of an import is mutually exclusive. If it has an alias, it can't use
            // the generated identifier, if it only has an identifier it only uses the identifier.
            let (official_ident, official_span) = if let Some(sp_alias_id) = imp.sp_alias_id.clone()
            {
                (sp_alias_id.inner, sp_alias_id.span)
            } else {
                (imp.name_id, sp_src_id.span)
            };

            let mod_ident = ModuleIdent::new(
                official_ident,
                imp.sp_alias_id.is_some(),
                official_span.into(),
            );

            if !is_importer {
                mod_ident_tracker.insert_or_store(mod_ident);
            } else {
                // We just warn on self import because it's not useful but it's not an error
                let core_msg = "Self imports have no effect";
                let builder = SourceDiagnostic::builder(
                    None,
                    DiagnosticLevel::Warn,
                    core_msg,
                    importer_src_id,
                )
                .add_annotation(
                    official_span,
                    AnnotationKind::Secondary,
                    "Does nothing".to_string().into(),
                );
                summary.push_diag(builder.build());
            }

            // If the path from a given import was seen already then it skips
            if graph.seen.contains(&sp_src_id.inner) {
                // Checking if there is a module id associated with it's path before skipping so
                // that the import can be updated if possible.
                let mod_id_opt = graph
                    .registered_mod_ids
                    .iter()
                    .find(|(s_id, _)| *s_id == sp_src_id.inner)
                    .map(|(_, m_id)| *m_id);

                // Valid import
                if let Some(mod_id) = mod_id_opt {
                    importer_mod.imports[imp_idx].kind =
                        ImportKind::Source(sp_src_id.clone(), mod_id);
                } else {
                    // Meaning the import was an error source because resolve_module did register a
                    // module id for this import, and it was already seen
                    importer_mod.imports[imp_idx].kind = ImportKind::ErrorSource(sp_src_id.clone());
                }
                continue;
            }

            // It will now continue on the next iteration this import that has already been
            // registered is seen.
            graph.seen.push(sp_src_id.inner);

            // The module produced by this import (Happy birthday)
            let new_mod = match resolve_module(
                imp.clone(),
                // official_ident,
                // &importer_mod,
                &mut graph,
                &mut summary,
                &cfg,
                &mut interner,
            ) {
                Ok(m) => {
                    // Need to set the current import to a resolved source or it stays unresolved
                    importer_mod.imports[imp_idx].kind =
                        ImportKind::Source(sp_src_id.clone(), m.self_id);
                    m
                }
                // Need to transition state from unresolved source to error so future users of this
                // state machine know
                Err(_) => {
                    importer_mod.imports[imp_idx].kind = ImportKind::ErrorSource(sp_src_id.clone());
                    continue;
                }
            };

            // The amount of reserved module ids grows O(modules registered), which is
            // all_mods_len + pending_mods_len, which ensures modules not yet registered are also
            // checked for over-allocation
            let expected_len = graph.registered_mod_ids.len();

            //NOTE: Check if externals (like lsp) can share this if not already
            //SAFETY
            // Ensuring before any resizing that the total modules never exceed MAX_MODULES
            if expected_len > MAX_MODS_USIZE {
                // Checking if the last processed is in the queue
                let last_processed_name = if let Some(module) = pending_mods.iter().last() {
                    interner.extract_utf8_str(module.name_id)
                // Checking if the last processed has been pushed into the final module list
                } else if let Some(module) = valid_mods.iter().last() {
                    interner.extract_utf8_str(module.name_id)
                } else {
                    // All checks were exhausted so this was a failure at the entry point
                    interner.extract_utf8_str(importer_mod.name_id)
                };

                let core_msg = format!("Exceeded max module count of {}", chrn_utils::MAX_MODULES);

                let src_diag = SourceDiagnostic::builder(
                    ErrorCode::CompilerInternals.into(),
                    DiagnosticLevel::Error,
                    core_msg,
                    sp_src_id.inner,
                )
                .add_note(format!("Last processed module was `{last_processed_name}`"));

                summary.push_diag(src_diag.build());
                reporter.summary.exceeded_max_mods = Some(chrn_utils::MAX_MODULES);
                should_break_outer = true;
                break;
            }

            // Pushing back so that it's imports can be viewed BEFORE putting it into valid_mods
            pending_mods.push_back(new_mod);
        }

        // This is only met if max modules have been exceeded.
        if should_break_outer {
            let valid_mods_len = valid_mods.len();
            //SAFETY:
            // Goes through valid module and checks if any module id from one of their imports
            // correspond to an invalid module. All modules from this iteration, including the
            // importer, are dropped.
            //
            // This loop is required because if say module0 imported module1, module1 was a
            // valid module, but then module2 reaches the capacity, that would mean module0
            // already set it's imported associated with module1 as a valid source module. This
            // corrects that by setting it to an error source, which DOESN'T really matter since
            // the caller should be terminating after this anyways but if it ever weren't done
            // this would prevent said bug.
            for valid_mod in valid_mods.iter_mut() {
                for imp in valid_mod.imports.iter_mut() {
                    if let ImportKind::Source(sp_path_id, m_id) = &imp.kind {
                        if m_id.id as usize >= valid_mods_len {
                            imp.kind = ImportKind::ErrorSource(sp_path_id.clone());
                        }
                    }
                }
            }
            // It could guarantee the current module, which would mean main is ALWAYS processed, but
            // not sure if that really matters.
            // valid_mods.push(importer_mod);
            break;
        }

        for found in mod_ident_tracker.found_dups.drain(..) {
            let dup_ident = interner.extract_utf8_str(found.dup.ident_id);

            // An alias is already present so changes msg
            let add_help = !found.dup.is_alias;

            let core_msg = if found.dup.is_alias {
                format!("Duplicate import alias `{dup_ident}`")
            } else {
                format!("Duplicate identifier `{dup_ident}` generated by this import")
            };

            let imp_span = found
                .dup
                .span
                .expect("Previous loop should guarantee import span");

            let mut builder = SourceDiagnostic::builder(
                ErrorCode::ImportErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                importer_src_id,
            )
            .add_annotation(imp_span, AnnotationKind::Primary, None);

            // The original is the root module when it has no span
            if let Some(original_span) = found.original.span {
                builder = builder.add_annotation(
                    original_span,
                    AnnotationKind::Secondary,
                    "Identifier first generated here".to_string().into(),
                );
            } else {
                builder = builder.add_note("Original generated identifier is from the root module");
            }

            if add_help {
                builder = builder.add_help("Consider giving the import an alias");
            }

            summary.push_diag(builder.build());
        }

        // Clearing current context of what should be considered a duplicate
        mod_ident_tracker.clear();

        // The pending module's imports have all been seen making this the final assignment.
        // Module ids are sequential so this is fine.
        valid_mods.push(importer_mod);
    }

    // let x = valid_mods.iter().for_each(|e| {
    //     dbg!(interner.extract_utf8_str(e.name_id));
    // });

    // Still concerning
    cfg.perf_tracker_mut().stop(ChrnPerfStage::ModuleGraph);

    let compiler = ScriptCompiler::init(valid_mods);

    // The store's arrays are dense and indexed by `ModuleId`, so they must cover every module the
    // compiler holds -- the implicit `core` module injected by `init` included, not just the user
    // modules in `valid_mods`.
    let store_capacity = compiler.mods.len();
    let comp_store = ScriptCompilerStore::new(
        cfg,
        graph.region_arena,
        interner,
        graph.src_manager,
        Vec::with_capacity(store_capacity),
        Vec::with_capacity(store_capacity),
        Vec::with_capacity(store_capacity),
        Vec::with_capacity(store_capacity),
    );

    (compiler, comp_store, summary)
}

/// Takes an import and attempts to turn it into a module using the current graph progress
fn resolve_module(
    // The import to turn into a module
    imp: Import,
    // official_ident: InternedIdUTF8,
    // Module that imported this module, which triggered this module's processing,
    // importer_mod: &Module,
    graph: &mut ModuleGraph,
    summary: &mut SourceDiagnosticSummary,
    cfg: &ChrnConfig,
    interner: &mut Intern,
) -> Result<Module, ()> {
    //WARN: If there is ever an instance where reserved_mod_ids reserves a module id that is valid,
    //but fails with "Err(())" below, it has broken module id alignment

    let ImportKind::UnresolvedSource(sp_src_id) = imp.kind.clone() else {
        unreachable!();
    };

    let importer_src_id = graph.region_arena[sp_src_id.span.region_id].src_id;

    // Tracks the id of the current module by tracking however many imports were seen, which
    // all represent one module
    //
    // This uses expect() because module_finder is the only collector imports.
    // We are iterating through said imports. Meaning for this iteration to happen module_finder
    // would have to register the path first.

    //TEST: Maybe supply src first

    // Using region id before pushing
    let sub_region_id = graph.region_arena.make_id();

    let src_kind = graph.src_manager.extract_source(sp_src_id.inner);
    let loader_out = load_source(src_kind, sp_src_id.inner, sub_region_id, cfg);

    let (sub_region, sub_state) = match loader_out {
        ConfigLoaderOutput::Success(region, new_summary) => {
            summary.merge(new_summary);
            (region, ModuleState::Loaded)
        }
        //FIX: Works but code liability
        ConfigLoaderOutput::Broken(broken_region, cfg_err) => {
            match cfg_err {
                ConfigLoadError::Diagnostic(diag) => {
                    summary.push_diag(diag);
                }
                ConfigLoadError::IO(e) => {
                    // This is matched even though file IO is the only current means in the case of
                    // a different source which is also IO, which would make this a comptime error
                    // if not updated.
                    let core_msg = match &graph.src_manager.extract_source(sp_src_id.inner) {
                        SourceKind::Path(src_path) => {
                            core_error::form_string_from_io_err(&e, &src_path.path)
                                .unwrap_or(e.to_string())
                        }
                        SourceKind::InMemory(_) => unreachable!(),
                    };

                    let src_diag = SourceDiagnostic::builder(
                        None,
                        DiagnosticLevel::Error,
                        core_msg,
                        importer_src_id,
                    )
                    .add_annotation(sp_src_id.span, AnnotationKind::Primary, None)
                    .build();

                    summary.push_diag(src_diag);
                }
            }
            (broken_region, ModuleState::BrokenRegion)
        }
        ConfigLoaderOutput::UnrecoverableErr(cfg_err) => {
            match cfg_err {
                ConfigLoadError::Diagnostic(diag) => {
                    summary.push_diag(diag);
                }
                ConfigLoadError::IO(e) => {
                    let core_msg = match &graph.src_manager.extract_source(sp_src_id.inner) {
                        SourceKind::Path(src_path) => {
                            core_error::form_string_from_io_err(&e, &src_path.path)
                                .unwrap_or(e.to_string())
                        }
                        SourceKind::InMemory(_) => unreachable!(),
                    };

                    let src_diag = SourceDiagnostic::builder(
                        None,
                        DiagnosticLevel::Error,
                        core_msg,
                        importer_src_id,
                    )
                    .add_annotation(sp_src_id.span, AnnotationKind::Primary, None)
                    .build();

                    summary.push_diag(src_diag);
                }
            }
            return Err(());
        }
    };

    // The check in the main loop MUST have ensured the module id created here is going to be new
    let current_mod_id = ModuleId::new(graph.registered_mod_ids.len() as u32);
    debug_assert!(
        graph
            .registered_mod_ids
            .iter()
            .all(|(_, m_id)| *m_id != current_mod_id)
    );

    graph
        .registered_mod_ids
        .push((sp_src_id.inner, current_mod_id));

    let (bind, sub_imports, finder_summary) =
        ModuleFinder::new(&sub_region.src_bytes, cfg, &sub_region)
            .collect_imports(interner, &mut graph.src_manager);
    summary.merge(finder_summary);

    // As opposed to how modules are pushed, regions are pushed before recursively descending
    // so no special cases needed for indexing it
    graph.region_arena.push(sub_region);

    let user_meta = UserModuleMetadata::new(sub_region_id);
    let sub_mod = Module::new(
        imp.name_id,
        sub_state,
        // All modules collected here are only user-defined so no flags can be touched.
        ModuleFlagMetadata::none(),
        current_mod_id,
        bind,
        sub_imports,
        ModuleKind::User(user_meta),
    );

    Ok(sub_mod)
}

/// Attempts to loads a registered source.
///
/// `region_id` should be a reserved ID that will be used for the `ConfigLoaderOutput` region,
/// if successful.
pub fn load_source(
    src_kind: &SourceKind,
    src_id: SourceId,
    region_id: SourceRegionId,
    cfg: &ChrnConfig,
) -> ConfigLoaderOutput {
    match src_kind {
        SourceKind::Path(src_path) => {
            //WARN: A canonical path can still name a directory or become unreadable before opening.
            let file = match file_ops::fopen(&src_path.path) {
                Ok(f) => f,
                Err((e, _)) => {
                    return ConfigLoaderOutput::UnrecoverableErr(ConfigLoadError::IO(e));
                }
            };
            ConfigLoader::new(region_id, file, src_id, cfg).load_config()
        }
        SourceKind::InMemory(s) => {
            ConfigLoader::new(region_id, Cursor::new(&s.src), src_id, cfg).load_config()
        }
    }
}

/// Rejects reserved module identifiers, attributing the diagnostic to `src_id`.
/// Callers can annotate the returned builder with the declaration or import span.
pub fn ensure_not_reserved(
    name_id: InternedIdUTF8,
    src_id: SourceId,
) -> Result<(), SourceDiagnosticBuilder> {
    if name_id.inner().id == intern::INTERNED_CORE {
        let core_msg = "`core` is a reserved module identifier";
        return Err(SourceDiagnostic::builder(
            ErrorCode::CompilerInternals.into(),
            DiagnosticLevel::Error,
            core_msg,
            src_id,
        ));
    }
    Ok(())
}
