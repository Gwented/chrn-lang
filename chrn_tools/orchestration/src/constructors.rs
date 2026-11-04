use chrn_utils::{
    core_error::ModuleInitError,
    intern::Intern,
    source_utils::source_manager::{SourceKind, SourceManager},
};
use compilation::{
    chrn_config::ChrnConfig,
    module,
    script_compiler::{
        ScriptCompiler, reporter::Reporter, script_compiler_store::ScriptCompilerStore,
    },
};

use crate::script_compiler_cache::ScriptCompilerCache;

/// Performs the minimum operates to create a `ScriptCompiler` and `ScriptCompilerStore`.
///
/// Supply an interner initialized with `Intern::init()` that owns all in-memory names
/// in the source manager. Previously registered sources remain available for imports.
/// If the loading stage fails critically, returns `Err(ModuleInitError)`
pub fn create_compiler(
    src_kind: SourceKind,
    interner: Intern,
    src_manager: SourceManager,
    reporter: &mut Reporter,
    cfg: ChrnConfig,
) -> Result<(ScriptCompiler, ScriptCompilerStore), ModuleInitError> {
    let (compiler, store, new_summary) =
        module::extract_all_modules(src_kind, interner, src_manager, cfg, reporter)?;
    reporter.merge_summary_safe(new_summary);
    Ok((compiler, store))
}

// Not sure if this will stay
/// Performs the minimum operates to create a `ScriptCompiler` and `ScriptCompilerStore`, then
/// creates `ScriptCompilerCache` alongside it.
///
/// Supply an interner initialized with `Intern::init()` that owns all in-memory names
/// in the source manager. Previously registered sources remain available for imports.
/// If the loading stage fails critically, returns `Err(ModuleInitError)`
pub fn create_compiler_with_cache(
    src_kind: SourceKind,
    interner: Intern,
    src_manager: SourceManager,
    reporter: &mut Reporter,
    cfg: ChrnConfig,
) -> Result<(ScriptCompiler, ScriptCompilerStore, ScriptCompilerCache), ModuleInitError> {
    let (compiler, compiler_store, new_summary) =
        module::extract_all_modules(src_kind, interner, src_manager, cfg, reporter)?;
    reporter.merge_summary_safe(new_summary);

    let cache = ScriptCompilerCache {
        mod_cache: Default::default(),
    };

    Ok((compiler, compiler_store, cache))
}
