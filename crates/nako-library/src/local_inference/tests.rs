use nako_core::{
    LibraryId, LocalInferenceEvidenceSource, MediaItemId, MediaKind, MediaSourceId, ScanSnapshotId,
};
use nako_naming::{NameEvidenceSource, NameParser, ParsedMediaKind, ParsedName};
use nako_vfs::StorageUri;

use super::{LocalInferenceEngine, LocalInferenceRequest};
use crate::scan::DiscoveredMediaSource;

#[test]
fn local_inference_engine_plans_episode_hierarchy_from_file_name() {
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let item_id = MediaItemId::new();
    let scan_id = ScanSnapshotId::new();
    let discovered = discovered_media_source("local:///TV/Firefly/S01/Firefly.S01E02.mkv");
    let engine = LocalInferenceEngine::with_default_parser();

    let plan = engine.plan_source(LocalInferenceRequest {
        library_id,
        source_id,
        item_id,
        scan_id,
        discovered: &discovered,
    });

    assert_eq!(plan.media_item.kind, MediaKind::Episode);
    assert_eq!(plan.media_item.metadata.title, "Episode 2");
    assert_eq!(plan.hierarchy.primary_item_id, item_id);
    assert_eq!(plan.hierarchy.required_ancestors.len(), 2);
    assert_eq!(plan.hierarchy.required_ancestors[0].kind, MediaKind::Series);
    assert_eq!(plan.hierarchy.required_ancestors[0].title, "Firefly");
    assert_eq!(plan.hierarchy.required_ancestors[1].kind, MediaKind::Season);
    assert_eq!(plan.hierarchy.required_ancestors[1].title, "Season 1");
    assert_eq!(plan.evidence.inferred_kind, MediaKind::Episode);
    assert_eq!(plan.evidence.inferred_title, Some("Firefly".to_owned()));
    assert_eq!(plan.evidence.inferred_season, Some(1));
    assert_eq!(plan.evidence.inferred_episode, Some(2));
    assert_eq!(
        plan.evidence.evidence_source,
        LocalInferenceEvidenceSource::FileName
    );
    assert_eq!(plan.source_state.uri, discovered.uri.as_str());
    assert_eq!(plan.media_source.item_id, item_id);
}

#[test]
fn local_inference_engine_keeps_unknown_source_flat_with_evidence() {
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let item_id = MediaItemId::new();
    let scan_id = ScanSnapshotId::new();
    let discovered = discovered_media_source("local:///Uploads/random.clip.mkv");
    let engine = LocalInferenceEngine::with_default_parser();

    let plan = engine.plan_source(LocalInferenceRequest {
        library_id,
        source_id,
        item_id,
        scan_id,
        discovered: &discovered,
    });

    assert_eq!(plan.media_item.kind, MediaKind::Unknown);
    assert_eq!(plan.media_item.parent_id, None);
    assert_eq!(plan.media_item.metadata.title, "random clip");
    assert!(plan.hierarchy.required_ancestors.is_empty());
    assert_eq!(plan.evidence.inferred_kind, MediaKind::Unknown);
    assert_eq!(plan.evidence.confidence_milli, Some(350));
}

#[test]
fn local_inference_engine_maps_name_parser_output_at_library_boundary() {
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let item_id = MediaItemId::new();
    let scan_id = ScanSnapshotId::new();
    let discovered = discovered_media_source("local:///Hints/custom.sidecar");
    let engine = LocalInferenceEngine::new(FixtureNameParser);

    let plan = engine.plan_source(LocalInferenceRequest {
        library_id,
        source_id,
        item_id,
        scan_id,
        discovered: &discovered,
    });

    assert_eq!(plan.media_item.kind, MediaKind::Movie);
    assert_eq!(plan.media_item.metadata.title, "Mapped Movie");
    assert_eq!(plan.evidence.inferred_kind, MediaKind::Movie);
    assert_eq!(
        plan.evidence.evidence_source,
        LocalInferenceEvidenceSource::Other("fixture-parser".to_owned())
    );
    assert_eq!(plan.evidence.evidence_value, "fixture-value");
    assert_eq!(plan.evidence.inference_version, "fixture:v1");
}

fn discovered_media_source(locator: &str) -> DiscoveredMediaSource {
    let uri = StorageUri::parse(locator).unwrap();
    let file_name = uri
        .path_part()
        .rsplit_once('/')
        .map(|(_parent, file_name)| file_name)
        .unwrap_or_else(|| uri.path_part())
        .to_owned();

    DiscoveredMediaSource {
        uri,
        file_name,
        size_bytes: Some(1),
        modified_at: None,
        etag: None,
        fingerprint: None,
        stale: false,
    }
}

struct FixtureNameParser;

impl NameParser for FixtureNameParser {
    fn parse_path(&self, _path: &str) -> ParsedName {
        ParsedName {
            kind_hint: ParsedMediaKind::Movie,
            title: "Mapped Movie".to_owned(),
            year: Some(2026),
            season_number: None,
            episode_number: None,
            confidence_milli: 640,
            evidence_source: NameEvidenceSource::Other("fixture-parser".to_owned()),
            evidence_value: "fixture-value".to_owned(),
            parser_version: "fixture:v1".to_owned(),
        }
    }
}
