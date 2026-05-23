use nako_core::{
    CanonicalMetadata, LibraryId, LibraryItemState, LocalInferenceEvidence,
    LocalInferenceEvidenceId, LocalInferenceEvidenceSource, MediaItem, MediaItemId, MediaKind,
    MediaSource, MediaSourceId, ScanSnapshotId, SourceState,
};
use nako_naming::{DefaultNameParser, NameEvidenceSource, NameParser, ParsedMediaKind, ParsedName};

use super::scan::DiscoveredMediaSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalInferenceRequest<'a> {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub scan_id: ScanSnapshotId,
    pub discovered: &'a DiscoveredMediaSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalInferencePlan {
    pub media_item: MediaItem,
    pub source_state: SourceState,
    pub media_source: MediaSource,
    pub evidence: LocalInferenceEvidence,
    pub hierarchy: ProvisionalHierarchyPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionalHierarchyPlan {
    pub primary_item_id: MediaItemId,
    pub primary_provisional: bool,
    pub required_ancestors: Vec<ProvisionalAncestorPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionalAncestorPlan {
    pub role: ProvisionalAncestorRole,
    pub kind: MediaKind,
    pub title: String,
    pub release_year: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvisionalAncestorRole {
    Series,
    Season,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MediaItemResolution {
    pub(crate) item: MediaItem,
    pub(crate) provisional: bool,
    pub(crate) supporting_items: Vec<MediaItem>,
    pub(crate) supporting_library_item_states: Vec<LibraryItemState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvisionalItemPlan {
    pub(crate) item: MediaItem,
    pub(crate) created: bool,
}

#[derive(Clone, Debug)]
pub struct LocalInferenceEngine<P = DefaultNameParser> {
    name_parser: P,
}

impl Default for LocalInferenceEngine<DefaultNameParser> {
    fn default() -> Self {
        Self::new(DefaultNameParser)
    }
}

impl LocalInferenceEngine<DefaultNameParser> {
    #[must_use]
    pub fn with_default_parser() -> Self {
        Self::default()
    }
}

impl<P> LocalInferenceEngine<P>
where
    P: NameParser,
{
    pub fn new(name_parser: P) -> Self {
        Self { name_parser }
    }

    pub fn plan_source(&self, request: LocalInferenceRequest<'_>) -> LocalInferencePlan {
        let parsed_name = self
            .name_parser
            .parse_path(request.discovered.uri.path_part());
        let hierarchy = self.plan_hierarchy(request.item_id, &parsed_name);
        let media_item = self.primary_media_item(request.item_id, &parsed_name, &hierarchy);
        let source_state = source_state_from_discovered(
            request.library_id,
            request.source_id,
            request.scan_id,
            request.discovered,
        );
        let media_source = media_source_from_discovered(
            request.source_id,
            request.library_id,
            request.item_id,
            request.discovered,
        );
        let evidence = local_inference_evidence_from_parsed(request.source_id, &parsed_name);

        LocalInferencePlan {
            media_item,
            source_state,
            media_source,
            evidence,
            hierarchy,
        }
    }

    fn plan_hierarchy(
        &self,
        primary_item_id: MediaItemId,
        parsed_name: &ParsedName,
    ) -> ProvisionalHierarchyPlan {
        let required_ancestors = if parsed_name.kind_hint == ParsedMediaKind::Episode
            && parsed_name.season_number.is_some()
        {
            vec![
                ProvisionalAncestorPlan {
                    role: ProvisionalAncestorRole::Series,
                    kind: MediaKind::Series,
                    title: parsed_name.title.clone(),
                    release_year: None,
                },
                ProvisionalAncestorPlan {
                    role: ProvisionalAncestorRole::Season,
                    kind: MediaKind::Season,
                    title: format!("Season {}", parsed_name.season_number.unwrap()),
                    release_year: None,
                },
            ]
        } else {
            Vec::new()
        };

        ProvisionalHierarchyPlan {
            primary_item_id,
            primary_provisional: true,
            required_ancestors,
        }
    }

    fn primary_media_item(
        &self,
        id: MediaItemId,
        parsed_name: &ParsedName,
        _hierarchy: &ProvisionalHierarchyPlan,
    ) -> MediaItem {
        let (kind, title) = if parsed_name.kind_hint == ParsedMediaKind::Episode
            && parsed_name.season_number.is_some()
        {
            (
                MediaKind::Episode,
                parsed_name
                    .episode_number
                    .map(|episode| format!("Episode {episode}"))
                    .unwrap_or_else(|| parsed_name.title.clone()),
            )
        } else {
            (
                media_kind_from_parsed(parsed_name.kind_hint),
                parsed_name.title.clone(),
            )
        };

        MediaItem {
            id,
            kind,
            parent_id: None,
            metadata: CanonicalMetadata {
                title,
                original_title: None,
                sort_title: None,
                overview: None,
                release_date: parsed_name.year.map(|year| year.to_string()),
                external_ids: Vec::new(),
                ..CanonicalMetadata::default()
            },
        }
    }
}

pub(crate) fn resolve_local_inference_plan(
    plan: LocalInferencePlan,
    supporting_items: Vec<ProvisionalItemPlan>,
) -> MediaItemResolution {
    let mut item = plan.media_item;

    if item.kind == MediaKind::Episode {
        item.parent_id = supporting_items
            .iter()
            .rev()
            .find(|candidate| candidate.item.kind == MediaKind::Season)
            .map(|candidate| candidate.item.id);
    }

    let mut resolved_supporting_items = Vec::new();
    let mut supporting_library_item_states = Vec::new();
    for supporting_item in supporting_items {
        if supporting_item.created {
            supporting_library_item_states.push(LibraryItemState {
                library_id: plan.source_state.library_id,
                item_id: supporting_item.item.id,
                provisional: true,
            });
            resolved_supporting_items.push(supporting_item.item);
        }
    }

    MediaItemResolution {
        item,
        provisional: plan.hierarchy.primary_provisional,
        supporting_items: resolved_supporting_items,
        supporting_library_item_states,
    }
}

fn source_state_from_discovered(
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

fn media_source_from_discovered(
    id: MediaSourceId,
    library_id: LibraryId,
    item_id: MediaItemId,
    discovered: &DiscoveredMediaSource,
) -> MediaSource {
    MediaSource {
        id,
        library_id,
        item_id,
        locator: discovered.uri.as_str().to_owned(),
        file_name: discovered.file_name.clone(),
        size_bytes: discovered.size_bytes,
        fingerprint: discovered.fingerprint.clone(),
    }
}

fn local_inference_evidence_from_parsed(
    source_id: MediaSourceId,
    parsed_name: &ParsedName,
) -> LocalInferenceEvidence {
    LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id,
        inferred_kind: media_kind_from_parsed(parsed_name.kind_hint),
        inferred_title: Some(parsed_name.title.clone()),
        inferred_year: parsed_name.year.map(i32::from),
        inferred_season: parsed_name.season_number.map(u32::from),
        inferred_episode: parsed_name.episode_number.map(u32::from),
        confidence_milli: Some(parsed_name.confidence_milli),
        evidence_source: evidence_source_from_name(parsed_name.evidence_source.clone()),
        evidence_value: parsed_name.evidence_value.clone(),
        inference_version: parsed_name.parser_version.clone(),
    }
}

fn media_kind_from_parsed(kind: ParsedMediaKind) -> MediaKind {
    match kind {
        ParsedMediaKind::Movie => MediaKind::Movie,
        ParsedMediaKind::Series => MediaKind::Series,
        ParsedMediaKind::Season => MediaKind::Season,
        ParsedMediaKind::Episode => MediaKind::Episode,
        ParsedMediaKind::Extra => MediaKind::Extra,
        ParsedMediaKind::Collection => MediaKind::Collection,
        ParsedMediaKind::Unknown => MediaKind::Unknown,
    }
}

fn evidence_source_from_name(source: NameEvidenceSource) -> LocalInferenceEvidenceSource {
    match source {
        NameEvidenceSource::Path => LocalInferenceEvidenceSource::Path,
        NameEvidenceSource::FileName => LocalInferenceEvidenceSource::FileName,
        NameEvidenceSource::Directory => LocalInferenceEvidenceSource::Directory,
        NameEvidenceSource::NearbyFile => LocalInferenceEvidenceSource::NearbyFile,
        NameEvidenceSource::MediaProbe => LocalInferenceEvidenceSource::MediaProbe,
        NameEvidenceSource::Other(value) => LocalInferenceEvidenceSource::Other(value),
    }
}

#[cfg(test)]
mod tests {
    use nako_core::{LocalInferenceEvidenceSource, MediaKind};
    use nako_vfs::StorageUri;

    use super::*;

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
}
