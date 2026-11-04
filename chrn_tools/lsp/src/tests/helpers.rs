use std::{collections::HashMap, sync::Arc};

use chrn_utils::{
    id_types::SourceRegionId, intern::Intern, source_utils::source_manager::SourceManager,
};
use compilation::{
    chrn_config::ChrnConfig,
    config_loader::ConfigLoaderOutput,
    module::{load_source, module_concepts::ModuleState},
};
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;

use crate::{
    analyser::{PreparedDocument, resolve_document_modules},
    state::DocumentCache,
};

/// Resolves registered virtual sources through the production LSP module loader.
/// Registration order deliberately remains independent of module discovery order.
pub(super) fn resolve_memory_sources(
    main_name: &str,
    sources: &[(&str, &str)],
) -> PreparedDocument {
    let mut interner = Intern::init();
    let mut manager = SourceManager::new();
    for &(name, text) in sources {
        let name_id = interner.intern_str(name);
        assert!(manager.get_source_id_from_interned_id(name_id).is_none());
        manager.register_in_memory(name_id, text.as_bytes().to_vec());
    }
    let name_id = interner.get_interned_id_from_str(main_name).unwrap();
    let main_id = manager.get_source_id_from_interned_id(name_id).unwrap();
    let text = Arc::new(
        String::from_utf8(
            manager
                .get_in_memory_from_interned_id(name_id)
                .unwrap()
                .src
                .clone(),
        )
        .unwrap(),
    );
    let mut cfg = ChrnConfig::default();
    let ConfigLoaderOutput::Success(region, summary) = load_source(
        manager.extract_source(main_id),
        main_id,
        SourceRegionId::new(0),
        &cfg,
    ) else {
        panic!("the test's entry source must load successfully");
    };
    assert!(summary.diags.is_empty(), "{summary:?}");
    // The URI is intentionally unrelated: the registered source owns its identity.
    resolve_document_modules(
        &Url::parse("file:///unrelated-to-virtual-entry.chrn").unwrap(),
        text,
        region,
        ModuleState::Loaded,
        &mut cfg,
        &DocumentCache::new(10),
        &RwLock::new(HashMap::new()),
        1,
        interner,
        manager,
    )
}
