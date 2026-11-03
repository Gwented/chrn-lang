use std::{collections::HashSet, panic::catch_unwind, path::Path};

use crate::{
    id_types::SourceId,
    intern::Intern,
    source_utils::source_manager::{InMemoryTag, PathTag, SourceKind, SourceManager},
};

#[test]
fn source_registration_preserves_identity_and_kind_through_growth() {
    let mut interner = Intern::init();
    let named = interner.intern_str("編集中のソース");
    let empty = interner.intern_str("");
    let stock = interner.intern_str("source");
    let names = [
        (None, "source"),
        (Some(named), "編集中のソース"),
        (Some(named), "編集中のソース"),
        (Some(empty), ""),
        (Some(stock), "source"),
    ];
    for mut manager in [
        SourceManager::new(),
        SourceManager::default(),
        SourceManager::with_capacity(0),
        SourceManager::with_capacity(512),
    ] {
        assert!(manager.get_source(SourceId::new(0)).is_none());
        let missing_path = Path::new("unregistered/source.chrn");
        assert_eq!(manager.get_source_id_from_path(missing_path), None);

        let mut seen_ids = HashSet::new();
        let mut registered = Vec::new();
        for i in 0..256 {
            let path = format!("unregistered/水/module_{i}.chrn");
            let path = Path::new(&path);
            assert_eq!(manager.get_source_id_from_path(path), None);
            let path_id = manager.register_path(path);
            assert!(seen_ids.insert(path_id.inner()), "duplicate path ID at {i}");
            assert_eq!(manager.register_path(path), path_id);
            assert_eq!(manager.get_source_id_from_path(path), Some(path_id));
            let (name, display_name) = names[i % names.len()];
            let memory_id = manager.register_in_memory(name, Vec::new());
            assert!(
                seen_ids.insert(memory_id.inner()),
                "duplicate memory ID at {i}"
            );
            registered.push((path.to_path_buf(), path_id, memory_id, name, display_name));
        }

        for (path, path_id, memory_id, name, display_name) in registered {
            assert_eq!(
                manager
                    .get_path(path_id)
                    .map(|source| source.path.as_path()),
                Some(path.as_path())
            );
            assert_eq!(manager.extract_path(path_id).path, path);
            assert_eq!(
                manager.display_source(path_id.inner(), &interner),
                path.to_str().unwrap()
            );
            assert_eq!(manager.register_path(&path), path_id);
            assert_eq!(manager.get_source_id_from_path(&path), Some(path_id));
            match manager.get_source(path_id.inner()) {
                Some(SourceKind::Path(stored)) => assert_eq!(&stored.path, &path),
                other => panic!("path ID must retain its kind and path: {other:?}"),
            }
            match manager.extract_source(path_id.inner()) {
                SourceKind::Path(stored) => assert_eq!(&stored.path, &path),
                other => panic!("path extraction returned the wrong kind: {other:?}"),
            }
            match manager.get_source(memory_id.inner()) {
                Some(SourceKind::InMemory(stored)) => assert_eq!(stored.name, name),
                other => panic!("memory ID must retain its kind and name: {other:?}"),
            }
            match manager.extract_source(memory_id.inner()) {
                SourceKind::InMemory(stored) => assert_eq!(stored.name, name),
                other => panic!("memory extraction returned the wrong kind: {other:?}"),
            }
            assert_eq!(
                manager.get_in_memory(memory_id).map(|source| source.name),
                Some(name)
            );
            assert_eq!(
                manager.display_source(memory_id.inner(), &interner),
                display_name
            );
        }

        let missing = SourceId::new(seen_ids.len() as u32);
        assert!(manager.get_source(missing).is_none());
        assert!(manager.get_path(missing.into_tagged::<PathTag>()).is_none());
        assert!(
            manager
                .get_in_memory(missing.into_tagged::<InMemoryTag>())
                .is_none()
        );
        assert_eq!(manager.get_source_id_from_path(missing_path), None);
        assert_eq!(
            manager.register_path(missing_path).inner(),
            missing,
            "lookups and duplicate registrations must not allocate source IDs"
        );
    }
}

#[test]
fn tagged_access_rejects_valid_ids_with_incorrect_kind_proof() {
    let mut manager = SourceManager::new();
    let path_id = manager.register_path(Path::new("source.chrn"));
    let memory_id = manager.register_in_memory(None, Vec::new());
    let false_path = memory_id.inner().into_tagged::<PathTag>();
    let false_memory = path_id.inner().into_tagged::<InMemoryTag>();

    assert!(
        catch_unwind(|| manager.get_path(false_path)).is_err(),
        "checked path access must reject an in-memory source"
    );
    assert!(
        catch_unwind(|| manager.extract_path(false_path)).is_err(),
        "path extraction must reject an in-memory source"
    );
    assert!(
        catch_unwind(|| manager.get_in_memory(false_memory)).is_err(),
        "checked in-memory access must reject a path source"
    );
}
