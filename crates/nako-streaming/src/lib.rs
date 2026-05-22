mod direct;
mod selection;

pub use direct::*;
pub use selection::*;
#[cfg(test)]
mod tests {
    use nako_core::{
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
        assert_eq!(decision.selected_source.source_id, source.id);
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::DirectPlay(_)
        ));
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
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
                output_container: nako_transcode::RemuxContainer::Mp4,
                ..
            })
        ));
    }

    #[test]
    fn selection_request_can_choose_requested_remux_output_container() {
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
        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: Some(&probe),
            client: &ClientPlaybackCapabilities::default(),
            context: PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mkv),
                    ..Default::default()
                },
            },
        });

        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
                output_container: nako_transcode::RemuxContainer::Mkv,
                ..
            })
        ));
    }

    #[test]
    fn selection_request_carries_storage_and_preference_context() {
        let source = media_source("movie.mp4");
        let client = ClientPlaybackCapabilities::default();

        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: None,
            client: &client,
            context: PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(1),
                    requested_subtitle_stream: Some(2),
                    max_video_bitrate: Some(4_000_000),
                    prefer_hdr: Some(false),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mkv),
                    transcode_output_container: None,
                },
            },
        });

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(decision.selected_source.library_id, source.library_id);
        assert_eq!(decision.direct_play.unwrap().supports_range_requests, false);
    }

    #[test]
    fn selection_request_can_require_hls_transcode_output() {
        let source = media_source("movie.mp4");
        let client = ClientPlaybackCapabilities::default();

        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: None,
            client: &client,
            context: PlaybackSelectionContext {
                storage: PlaybackStorageContext::default(),
                preferences: PlaybackPreferenceContext {
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                    ..Default::default()
                },
            },
        });

        assert_eq!(decision.mode, PlaybackMode::Transcode);
        assert!(matches!(
            decision.execution,
            PlaybackExecutionPlan::Transcode(nako_transcode::TranscodePlan {
                output_container: nako_transcode::OutputContainer::Hls,
                ..
            })
        ));
    }

    #[test]
    fn playback_profile_identity_normalizes_capability_order_and_case() {
        let left = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["MP4".to_owned(), "webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["H264".to_owned(), "hevc".to_owned()],
                audio_codecs: vec!["AAC".to_owned(), "opus".to_owned()],
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    requested_subtitle_stream: None,
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mp4),
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                },
            },
        );
        let right = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities {
                direct_play: true,
                containers: vec!["webm".to_owned(), "mp4".to_owned()],
                video_codecs: vec!["hevc".to_owned(), "h264".to_owned()],
                audio_codecs: vec!["opus".to_owned(), "aac".to_owned()],
            },
            PlaybackSelectionContext {
                storage: PlaybackStorageContext {
                    remote: true,
                    range_readable: Some(false),
                },
                preferences: PlaybackPreferenceContext {
                    requested_audio_stream: Some(2),
                    requested_subtitle_stream: None,
                    max_video_bitrate: Some(8_000_000),
                    prefer_hdr: Some(true),
                    remux_output_container: Some(nako_transcode::RemuxContainer::Mp4),
                    transcode_output_container: Some(nako_transcode::OutputContainer::Hls),
                },
            },
        );

        assert_eq!(left.identity_key(), right.identity_key());
        assert!(left.identity_key().contains("containers=mp4|webm"));
        assert!(left.identity_key().contains("audio=2"));
        assert!(left.identity_key().contains("transcode=hls"));
    }

    #[test]
    fn playback_profile_rejects_invalid_runtime_selected_hardware_plan() {
        let profile = PlaybackProfile::from_context(
            &ClientPlaybackCapabilities::default(),
            PlaybackSelectionContext::default(),
        );
        let plan = nako_transcode::TranscodePlan {
            input_locator: "local:///demo.mkv".to_owned(),
            output_container: nako_transcode::OutputContainer::Hls,
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            hardware_acceleration: nako_transcode::HardwareAcceleration::Nvenc,
        };

        let err = profile
            .try_hls_transcode_profile(&plan, nako_transcode::HardwareAcceleration::Nvenc)
            .unwrap_err();

        assert!(err.to_string().contains("hardware acceleration selection"));
        assert!(!err.to_string().contains("local:///"));
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
            library_id: nako_core::LibraryId::new(),
            item_id: nako_core::MediaItemId::new(),
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
