use nako_core::{
    LibraryId, LibraryItemState, LocalInferenceEvidence, MediaItem, MediaItemId, MediaKind,
    MediaSource, MediaSourceId, ScanSnapshotId, SourceState,
};

use crate::scan::DiscoveredMediaSource;

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
