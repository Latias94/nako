use super::*;
use nako_core::{
    SourceDuplicateEvidenceKind, SourceDuplicateRelationshipStatus, SourceDuplicateRepository,
};
use nako_vfs::{
    StorageBackend, StorageLinkKind, StorageLinkPlanRequest, StorageLinkPlanStatus, StorageUri,
};

#[tokio::test]
async fn catalog_records_filesystem_link_duplicate_suggestion_without_merging_items() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("movies")).unwrap();
    fs::write(temp.path().join("movies").join("demo.mkv"), b"demo").unwrap();
    fs::write(temp.path().join("movies").join("demo-linked.mkv"), b"demo").unwrap();

    let source_uri = StorageUri::from_parts("local", "movies/demo.mkv").unwrap();
    let target_uri = StorageUri::from_parts("local", "movies/demo-linked.mkv").unwrap();
    let link_plan = LocalFsBackend::new(temp.path())
        .unwrap()
        .plan_link(StorageLinkPlanRequest::new(
            source_uri.clone(),
            target_uri.clone(),
            StorageLinkKind::Hard,
        ))
        .await
        .unwrap();
    assert_eq!(link_plan.status, StorageLinkPlanStatus::TargetExists);

    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let catalog = CatalogAppService::new(store.clone());
    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///movies".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let first_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo Linked".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let first_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: first_item.id,
        locator: source_uri.as_str().to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: second_item.id,
        locator: target_uri.as_str().to_owned(),
        file_name: "demo-linked.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };

    store.upsert_media_item(&first_item).await.unwrap();
    store.upsert_media_item(&second_item).await.unwrap();
    store.upsert_media_source(&first_source).await.unwrap();
    store.upsert_media_source(&second_source).await.unwrap();

    let relationship = catalog
        .record_filesystem_link_duplicate_suggestion(first_source.id, second_source.id, &link_plan)
        .await
        .unwrap();
    let first_source_after = store
        .get_media_source(first_source.id)
        .await
        .unwrap()
        .unwrap();
    let second_source_after = store
        .get_media_source(second_source.id)
        .await
        .unwrap()
        .unwrap();
    let listed = store
        .list_source_duplicate_relationships(first_source.id, PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        relationship.evidence_kind,
        SourceDuplicateEvidenceKind::FilesystemLink
    );
    assert_eq!(
        relationship.status,
        SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(relationship.confidence_milli, Some(700));
    assert_eq!(
        relationship.evidence_value.as_deref(),
        Some("link_plan:scheme=local;kind=hard;status=target_exists")
    );
    assert_eq!(listed, vec![relationship]);
    assert_eq!(first_source_after.item_id, first_item.id);
    assert_eq!(second_source_after.item_id, second_item.id);
    assert_ne!(first_source_after.item_id, second_source_after.item_id);
}

#[tokio::test]
async fn catalog_rejects_non_evidence_link_plan_status_without_relationship() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("movies")).unwrap();
    fs::write(temp.path().join("movies").join("demo.mkv"), b"demo").unwrap();

    let source_uri = StorageUri::from_parts("local", "movies/demo.mkv").unwrap();
    let target_uri = StorageUri::from_parts("local", "missing/demo-linked.mkv").unwrap();
    let link_plan = LocalFsBackend::new(temp.path())
        .unwrap()
        .plan_link(StorageLinkPlanRequest::new(
            source_uri.clone(),
            target_uri.clone(),
            StorageLinkKind::Soft,
        ))
        .await
        .unwrap();
    assert_eq!(link_plan.status, StorageLinkPlanStatus::TargetParentMissing);

    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let catalog = CatalogAppService::new(store.clone());
    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///movies".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let first_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Missing Target".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let first_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: first_item.id,
        locator: source_uri.as_str().to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: second_item.id,
        locator: target_uri.as_str().to_owned(),
        file_name: "demo-linked.mkv".to_owned(),
        size_bytes: None,
        fingerprint: None,
    };

    store.upsert_media_item(&first_item).await.unwrap();
    store.upsert_media_item(&second_item).await.unwrap();
    store.upsert_media_source(&first_source).await.unwrap();
    store.upsert_media_source(&second_source).await.unwrap();

    let error = catalog
        .record_filesystem_link_duplicate_suggestion(first_source.id, second_source.id, &link_plan)
        .await
        .unwrap_err();
    let listed = store
        .list_source_duplicate_relationships(first_source.id, PageRequest::first_page())
        .await
        .unwrap();

    assert!(
        error
            .to_string()
            .contains("target_parent_missing cannot create source duplicate evidence")
    );
    assert!(listed.is_empty());
}
