use std::path::Path;

use nako_core::{NakoError, PlaybackSessionId, Result, TranscodeSessionId};
use nako_transcode::{HlsArtifactManifest, HlsVariantPolicy};

const HLS_SUBTITLE_GROUP_ID: &str = "nako-subtitles";
const SINGLE_VARIANT_FALLBACK_BANDWIDTH: u64 = 3_128_000;

pub(super) fn author_hls_entry_playlist(
    body: &str,
    manifest: &HlsArtifactManifest,
) -> Result<String> {
    if !manifest.media_renditions().has_subtitles() {
        return Ok(body.to_owned());
    }

    match manifest.output().variant_policy {
        HlsVariantPolicy::SingleVariant => author_single_variant_master_playlist(manifest),
        HlsVariantPolicy::Adaptive => Ok(author_adaptive_master_playlist(body, manifest)),
    }
}

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

    if trimmed.starts_with("#EXT-X-MAP:") || trimmed.starts_with("#EXT-X-MEDIA:") {
        return rewrite_hls_quoted_uri_attribute(line, session_id)
            .unwrap_or_else(|| line.to_owned());
    }

    if trimmed.starts_with('#') {
        return line.to_owned();
    }

    rewrite_hls_media_uri(trimmed, session_id)
}

fn rewrite_hls_quoted_uri_attribute(line: &str, session_id: &str) -> Option<String> {
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

fn author_single_variant_master_playlist(manifest: &HlsArtifactManifest) -> Result<String> {
    let playlist_file_name = hls_playlist_file_name(manifest.primary_playlist_path())?;
    let mut lines = vec!["#EXTM3U".to_owned(), "#EXT-X-VERSION:3".to_owned()];
    lines.extend(subtitle_media_lines(manifest));
    lines.push(format!(
        "#EXT-X-STREAM-INF:BANDWIDTH={SINGLE_VARIANT_FALLBACK_BANDWIDTH},SUBTITLES=\"{HLS_SUBTITLE_GROUP_ID}\""
    ));
    lines.push(playlist_file_name.to_owned());

    Ok(playlist_lines(lines))
}

fn author_adaptive_master_playlist(body: &str, manifest: &HlsArtifactManifest) -> String {
    let media_lines = subtitle_media_lines(manifest);
    let mut authored = Vec::new();
    let mut inserted_media = false;

    for line in body.lines() {
        if !inserted_media && line.trim_start().starts_with("#EXT-X-STREAM-INF:") {
            authored.extend(media_lines.iter().cloned());
            inserted_media = true;
        }
        authored.push(author_master_playlist_line(line));
    }

    if !inserted_media {
        let insert_at = authored
            .iter()
            .position(|line| line.trim() == "#EXTM3U")
            .map(|position| position + 1)
            .unwrap_or_default();
        authored.splice(insert_at..insert_at, media_lines);
    }

    let mut body = authored.join("\n");
    body.push('\n');
    body
}

fn author_master_playlist_line(line: &str) -> String {
    if !line.trim_start().starts_with("#EXT-X-STREAM-INF:") || line.contains("SUBTITLES=") {
        return line.to_owned();
    }

    let trimmed_len = line.trim_end().len();
    let (content, trailing) = line.split_at(trimmed_len);
    format!("{content},SUBTITLES=\"{HLS_SUBTITLE_GROUP_ID}\"{trailing}")
}

fn subtitle_media_lines(manifest: &HlsArtifactManifest) -> Vec<String> {
    manifest
        .media_renditions()
        .subtitles()
        .iter()
        .enumerate()
        .map(|(position, subtitle)| {
            let language = subtitle.language.as_deref().unwrap_or("und");
            let name = if language == "und" {
                format!("Subtitle {}", position + 1)
            } else {
                language.to_owned()
            };
            let default = if position == 0 { "YES" } else { "NO" };
            let mut attributes = vec![
                "TYPE=SUBTITLES".to_owned(),
                format!(
                    "GROUP-ID=\"{}\"",
                    hls_attribute_value(HLS_SUBTITLE_GROUP_ID)
                ),
                format!("NAME=\"{}\"", hls_attribute_value(&name)),
                format!("DEFAULT={default}"),
                "AUTOSELECT=YES".to_owned(),
                "FORCED=NO".to_owned(),
            ];
            if language != "und" {
                attributes.push(format!("LANGUAGE=\"{}\"", hls_attribute_value(language)));
            }
            attributes.push(format!("URI=\"{}\"", subtitle.playlist_file_name()));

            format!("#EXT-X-MEDIA:{}", attributes.join(","))
        })
        .collect()
}

fn hls_attribute_value(value: &str) -> String {
    value
        .chars()
        .filter(|value| !matches!(value, '"' | '\r' | '\n'))
        .collect()
}

fn hls_playlist_file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls playlist path does not have a valid file name".to_owned(),
        })
}

fn playlist_lines(lines: Vec<String>) -> String {
    let mut body = lines.join("\n");
    body.push('\n');
    body
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
    use nako_transcode::{
        HlsMediaRenditionPlan, HlsOutputRequirement, HlsRendition, HlsSegmentContainer,
        HlsSubtitleRendition,
    };

    #[test]
    fn author_hls_entry_playlist_generates_single_variant_master_for_subtitles() {
        let manifest = single_variant_manifest_with_subtitles();
        let body = "#EXTM3U\n#EXTINF:1,\nsegment_00000.ts\n#EXT-X-ENDLIST\n";

        let authored = author_hls_entry_playlist(body, &manifest).unwrap();

        assert!(authored.starts_with("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-MEDIA:"));
        assert!(authored.contains("TYPE=SUBTITLES"));
        assert!(authored.contains("GROUP-ID=\"nako-subtitles\""));
        assert!(authored.contains("NAME=\"jpn\""));
        assert!(authored.contains("DEFAULT=YES"));
        assert!(authored.contains("LANGUAGE=\"jpn\""));
        assert!(authored.contains("URI=\"subtitle_0.m3u8\""));
        assert!(authored.contains("SUBTITLES=\"nako-subtitles\"\nplaylist.m3u8"));
        assert!(!authored.contains("segment_00000.ts"));
        assert!(authored.ends_with('\n'));
    }

    #[test]
    fn author_hls_entry_playlist_enriches_adaptive_master_for_subtitles() {
        let manifest = adaptive_manifest_with_subtitles();
        let body = "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-STREAM-INF:BANDWIDTH=3128000,RESOLUTION=1280x720\nvariant_0.m3u8\n";

        let authored = author_hls_entry_playlist(body, &manifest).unwrap();

        assert!(authored.starts_with("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-MEDIA:"));
        assert!(authored.contains("#EXT-X-MEDIA:TYPE=SUBTITLES"));
        assert!(authored.contains("URI=\"subtitle_0.m3u8\""));
        assert!(authored.contains(
            "#EXT-X-STREAM-INF:BANDWIDTH=3128000,RESOLUTION=1280x720,SUBTITLES=\"nako-subtitles\""
        ));
        assert!(authored.contains("\nvariant_0.m3u8\n"));
    }

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
    fn rewrite_hls_playlist_rewrites_webvtt_subtitle_segments() {
        let session_id = TranscodeSessionId::new();
        let body = "#EXTM3U\n#EXTINF:1,\nsubtitle_0_00000.vtt\n#EXT-X-ENDLIST\n";

        let rewritten = rewrite_hls_playlist(body, session_id);

        assert!(rewritten.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/subtitle_0_00000.vtt"
        )));
    }

    #[test]
    fn rewrite_hls_playlist_rewrites_subtitle_media_playlist_uri() {
        let session_id = TranscodeSessionId::new();
        let body = "#EXTM3U\n#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"nako-subtitles\",NAME=\"jpn\",URI=\"subtitle_0.m3u8\"\n#EXT-X-STREAM-INF:BANDWIDTH=3128000,SUBTITLES=\"nako-subtitles\"\nplaylist.m3u8\n";

        let rewritten = rewrite_hls_playlist(body, session_id);

        assert!(rewritten.contains(&format!(
            "URI=\"/playback/sessions/{session_id}/hls/segments/subtitle_0.m3u8\""
        )));
        assert!(rewritten.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/playlist.m3u8"
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

    fn single_variant_manifest_with_subtitles() -> HlsArtifactManifest {
        HlsArtifactManifest::single_variant(
            "hls",
            "hls/playlist.m3u8",
            "hls/segment_%05d.ts",
            HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::SingleVariant,
                segment_container: HlsSegmentContainer::MpegTs,
            },
        )
        .unwrap()
        .with_media_renditions(subtitle_renditions())
        .unwrap()
    }

    fn adaptive_manifest_with_subtitles() -> HlsArtifactManifest {
        HlsArtifactManifest::adaptive_fmp4_with_audio(
            "hls",
            "hls/master.m3u8",
            vec![HlsRendition::new(0, 1280, 720, 3_000_000, 128_000)],
            true,
        )
        .unwrap()
        .with_media_renditions(subtitle_renditions())
        .unwrap()
    }

    fn subtitle_renditions() -> HlsMediaRenditionPlan {
        HlsMediaRenditionPlan::from_subtitles(vec![HlsSubtitleRendition::new(
            0,
            2,
            Some("jpn".to_owned()),
        )])
        .unwrap()
    }
}
