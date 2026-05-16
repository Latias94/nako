mod direct;
mod selection;

pub use direct::*;
pub use selection::*;
#[cfg(test)]
mod tests {
    use taru_core::{
        MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind,
    };

    use super::*;

    #[test]
    fn direct_play_is_allowed_for_compatible_mp4() {
        let source = media_source("movie.mp4");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = decide_playback(
            &source,
            Some(&probe),
            &ClientPlaybackCapabilities::default(),
        );

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(
            decision.direct_play.unwrap().content_type,
            "video/mp4".to_owned()
        );
    }

    #[test]
    fn unsupported_container_with_supported_codecs_requests_remux() {
        let source = media_source("movie.mkv");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = decide_playback(
            &source,
            Some(&probe),
            &ClientPlaybackCapabilities::default(),
        );

        assert_eq!(decision.mode, PlaybackMode::Remux);
    }

    #[test]
    fn direct_play_response_plan_handles_full_empty_and_partial_ranges() {
        let empty = plan_direct_play_response(0, "video/mp4", DirectPlayRangeRequest::None);
        assert_eq!(empty.status, DirectPlayResponseStatus::Ok);
        assert_eq!(empty.body_len, 0);
        assert_eq!(empty.seek_offset, 0);
        assert_eq!(empty.content_range, None);

        let requested = parse_http_range_header("bytes=2-5").unwrap();
        let partial =
            plan_direct_play_response(10, "video/mp4", DirectPlayRangeRequest::Range(requested));
        assert_eq!(partial.status, DirectPlayResponseStatus::PartialContent);
        assert_eq!(partial.body_len, 4);
        assert_eq!(partial.seek_offset, 2);
        assert_eq!(partial.content_range.as_deref(), Some("bytes 2-5/10"));
    }

    #[test]
    fn direct_play_response_plan_maps_invalid_ranges_to_416() {
        let out_of_bounds = RequestedByteRange {
            start: Some(20),
            end: Some(30),
        };
        let invalid = plan_direct_play_response(
            10,
            "video/mp4",
            DirectPlayRangeRequest::Range(out_of_bounds),
        );
        assert_eq!(
            invalid.status,
            DirectPlayResponseStatus::RangeNotSatisfiable
        );
        assert_eq!(invalid.body_len, 0);
        assert_eq!(invalid.content_range.as_deref(), Some("bytes */10"));

        let malformed = plan_direct_play_response(10, "video/mp4", DirectPlayRangeRequest::Invalid);
        assert_eq!(
            malformed.status,
            DirectPlayResponseStatus::RangeNotSatisfiable
        );
        assert_eq!(malformed.content_range.as_deref(), Some("bytes */10"));
    }

    #[test]
    fn range_parser_resolves_open_and_suffix_ranges() {
        let open = parse_http_range_header("bytes=2-").unwrap();
        let suffix = parse_http_range_header("bytes=-4").unwrap();

        assert_eq!(
            resolve_byte_range(Some(open), 10).unwrap(),
            Some(ResolvedByteRange { start: 2, end: 9 })
        );
        assert_eq!(
            resolve_byte_range(Some(suffix), 10).unwrap(),
            Some(ResolvedByteRange { start: 6, end: 9 })
        );
    }

    fn media_source(file_name: &str) -> MediaSource {
        MediaSource {
            id: MediaSourceId::new(),
            library_id: taru_core::LibraryId::new(),
            item_id: taru_core::MediaItemId::new(),
            locator: format!("local:///{file_name}"),
            file_name: file_name.to_owned(),
            size_bytes: Some(1_000),
            fingerprint: None,
        }
    }

    fn stream(kind: MediaStreamKind, codec: Option<&str>) -> MediaStreamInfo {
        MediaStreamInfo {
            index: 0,
            kind,
            codec: codec.map(ToOwned::to_owned),
            language: None,
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
        }
    }
}
