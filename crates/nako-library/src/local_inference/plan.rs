use nako_core::{CanonicalMetadata, MediaItem, MediaItemId, MediaKind};
use nako_naming::{DefaultNameParser, NameParser, ParsedMediaKind, ParsedName};

use super::{
    evidence::{local_inference_evidence_from_parsed, media_kind_from_parsed},
    hierarchy::plan_hierarchy,
    source_records::{media_source_from_discovered, source_state_from_discovered},
    types::{LocalInferencePlan, LocalInferenceRequest, ProvisionalHierarchyPlan},
};

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
        let hierarchy = plan_hierarchy(request.item_id, &parsed_name);

        LocalInferencePlan {
            media_item: primary_media_item(request.item_id, &parsed_name, &hierarchy),
            source_state: source_state_from_discovered(&request),
            media_source: media_source_from_discovered(&request),
            evidence: local_inference_evidence_from_parsed(&request, &parsed_name),
            hierarchy,
        }
    }
}

fn primary_media_item(
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
