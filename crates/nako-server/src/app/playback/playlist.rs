use std::path::Path;

use nako_core::{NakoError, PlaybackSessionId, Result, TranscodeSessionId};

pub(super) fn rewrite_hls_playlist(body: &str, session_id: TranscodeSessionId) -> String {
    rewrite_hls_playlist_media_uris(body, &session_id.to_string())
}

pub(super) fn rewrite_hls_playlist_for_playback_session(
    body: &str,
    session_id: PlaybackSessionId,
) -> String {
    rewrite_hls_playlist_media_uris(body, &session_id.to_string())
}

fn rewrite_hls_playlist_media_uris(body: &str, session_id: &str) -> String {
    let mut rewritten = body
        .lines()
        .map(|line| rewrite_hls_playlist_line(line, session_id))
        .collect::<Vec<_>>()
        .join("\n");

    if body.ends_with('\n') {
        rewritten.push('\n');
    }

    rewritten
}

fn rewrite_hls_playlist_line(line: &str, session_id: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_owned();
    }

    if trimmed.starts_with("#EXT-X-MAP:") {
        return rewrite_hls_map_uri(line, session_id).unwrap_or_else(|| line.to_owned());
    }

    if trimmed.starts_with('#') {
        return line.to_owned();
    }

    rewrite_hls_media_uri(trimmed, session_id)
}

fn rewrite_hls_map_uri(line: &str, session_id: &str) -> Option<String> {
    let marker = "URI=\"";
    let start = line.find(marker)? + marker.len();
    let end = line[start..].find('"')? + start;
    let rewritten_uri = rewrite_hls_media_uri(&line[start..end], session_id);

    let mut rewritten = String::with_capacity(line.len() + rewritten_uri.len());
    rewritten.push_str(&line[..start]);
    rewritten.push_str(&rewritten_uri);
    rewritten.push_str(&line[end..]);
    Some(rewritten)
}

fn rewrite_hls_media_uri(uri: &str, session_id: &str) -> String {
    let trimmed = uri.trim();
    if trimmed.contains("://") {
        return trimmed.to_owned();
    }

    if let Some(segment_path) = existing_hls_session_segment_path(trimmed) {
        return hls_segment_route(session_id, segment_path);
    }

    if trimmed.starts_with('/') || trimmed.contains('/') || trimmed.contains('\\') {
        return trimmed.to_owned();
    }

    hls_segment_route(session_id, trimmed)
}

fn existing_hls_session_segment_path(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("/playback/sessions/")?;
    let (_old_session_id, segment_path) = rest.split_once("/hls/segments/")?;
    validate_hls_segment_name(segment_path)
        .is_ok()
        .then_some(segment_path)
}

fn hls_segment_route(session_id: &str, segment_path: &str) -> String {
    format!("/playback/sessions/{session_id}/hls/segments/{segment_path}")
}

pub(super) fn validate_hls_segment_name(segment_name: &str) -> Result<()> {
    if segment_name.is_empty()
        || segment_name.contains('/')
        || segment_name.contains('\\')
        || segment_name.contains("..")
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    let path = Path::new(segment_name);
    if !path
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_hls_playlist_rewrites_fmp4_init_map_and_segments() {
        let session_id = TranscodeSessionId::new();
        let body =
            "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:1,\nsegment_00000.m4s\n#EXT-X-ENDLIST\n";

        let rewritten = rewrite_hls_playlist(body, session_id);

        assert!(rewritten.contains(&format!(
            "#EXT-X-MAP:URI=\"/playback/sessions/{session_id}/hls/segments/init.mp4\""
        )));
        assert!(rewritten.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/segment_00000.m4s"
        )));
        assert!(rewritten.ends_with('\n'));
    }

    #[test]
    fn rewrite_hls_playlist_rewrites_adaptive_variant_playlist_uris() {
        let session_id = TranscodeSessionId::new();
        let body = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=3128000,RESOLUTION=1280x720\nvariant_0.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=1328000,RESOLUTION=854x480\nvariant_1.m3u8\n";

        let rewritten = rewrite_hls_playlist(body, session_id);

        assert!(rewritten.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/variant_0.m3u8"
        )));
        assert!(rewritten.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/variant_1.m3u8"
        )));
    }

    #[test]
    fn rewrite_hls_playlist_for_playback_session_rebinds_existing_segment_routes() {
        let old_session_id = PlaybackSessionId::new();
        let new_session_id = PlaybackSessionId::new();
        let body = format!(
            "#EXTM3U\n#EXT-X-MAP:URI=\"/playback/sessions/{old_session_id}/hls/segments/init.mp4\"\n/playback/sessions/{old_session_id}/hls/segments/segment_00000.ts\n"
        );

        let rewritten = rewrite_hls_playlist_for_playback_session(&body, new_session_id);

        assert!(!rewritten.contains(&old_session_id.to_string()));
        assert!(rewritten.contains(&format!(
            "#EXT-X-MAP:URI=\"/playback/sessions/{new_session_id}/hls/segments/init.mp4\""
        )));
        assert!(rewritten.contains(&format!(
            "/playback/sessions/{new_session_id}/hls/segments/segment_00000.ts"
        )));
    }
}
