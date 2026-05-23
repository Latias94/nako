use nako_core::{MediaSource, SourceState};

use super::types::LocalInferenceRequest;

pub(super) fn source_state_from_discovered(request: &LocalInferenceRequest<'_>) -> SourceState {
    SourceState {
        library_id: request.library_id,
        source_id: Some(request.source_id),
        uri: request.discovered.uri.as_str().to_owned(),
        size_bytes: request.discovered.size_bytes,
        modified_at: request.discovered.modified_at.clone(),
        etag: request.discovered.etag.clone(),
        fingerprint: request.discovered.fingerprint.clone(),
        last_seen_scan_id: request.scan_id,
        tombstoned: false,
    }
}

pub(super) fn media_source_from_discovered(request: &LocalInferenceRequest<'_>) -> MediaSource {
    MediaSource {
        id: request.source_id,
        library_id: request.library_id,
        item_id: request.item_id,
        locator: request.discovered.uri.as_str().to_owned(),
        file_name: request.discovered.file_name.clone(),
        size_bytes: request.discovered.size_bytes,
        fingerprint: request.discovered.fingerprint.clone(),
    }
}
