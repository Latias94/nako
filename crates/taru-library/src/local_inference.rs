use taru_core::{
    CanonicalMetadata, LibraryId, LocalInferenceEvidence, LocalInferenceEvidenceId, MediaItem,
    MediaItemId, MediaSource, MediaSourceId, ScanSnapshotId, SourceState,
};

use super::scan::DiscoveredMediaSource;

pub(crate) struct MediaItemResolution {
    pub(crate) item: MediaItem,
    pub(crate) provisional: bool,
}

pub(crate) fn media_item_from_discovered(
    id: MediaItemId,
    discovered: &DiscoveredMediaSource,
) -> MediaItem {
    MediaItem {
        id,
        kind: discovered.parsed_name.kind_hint,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: discovered.parsed_name.title.clone(),
            original_title: None,
            sort_title: None,
            overview: None,
            release_date: discovered.parsed_name.year.map(|year| year.to_string()),
            external_ids: Vec::new(),
            ..CanonicalMetadata::default()
        },
    }
}

pub(crate) fn source_state_from_discovered(
    library_id: LibraryId,
    source_id: MediaSourceId,
    scan_id: ScanSnapshotId,
    discovered: &DiscoveredMediaSource,
) -> SourceState {
    SourceState {
        library_id,
        source_id: Some(source_id),
        uri: discovered.uri.as_str().to_owned(),
        size_bytes: discovered.size_bytes,
        modified_at: discovered.modified_at.clone(),
        etag: discovered.etag.clone(),
        fingerprint: discovered.fingerprint.clone(),
        last_seen_scan_id: scan_id,
        tombstoned: false,
    }
}

pub(crate) fn media_source_from_discovered(
    id: MediaSourceId,
    library_id: LibraryId,
    item_id: MediaItemId,
    discovered: DiscoveredMediaSource,
) -> MediaSource {
    MediaSource {
        id,
        library_id,
        item_id,
        locator: discovered.uri.as_str().to_owned(),
        file_name: discovered.file_name,
        size_bytes: discovered.size_bytes,
        fingerprint: discovered.fingerprint,
    }
}

pub(crate) fn local_inference_evidence_from_discovered(
    source_id: MediaSourceId,
    discovered: &DiscoveredMediaSource,
) -> LocalInferenceEvidence {
    LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id,
        inferred_kind: discovered.parsed_name.kind_hint,
        inferred_title: Some(discovered.parsed_name.title.clone()),
        inferred_year: discovered.parsed_name.year.map(i32::from),
        inferred_season: discovered.parsed_name.season_number.map(u32::from),
        inferred_episode: discovered.parsed_name.episode_number.map(u32::from),
        confidence_milli: Some(discovered.parsed_name.confidence_milli),
        evidence_source: discovered.parsed_name.evidence_source.clone(),
        evidence_value: discovered.parsed_name.evidence_value.clone(),
        inference_version: discovered.parsed_name.parser_version.clone(),
    }
}
