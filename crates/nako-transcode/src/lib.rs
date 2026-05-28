mod artifact;
mod engine;
mod ffmpeg;
mod hardware;
mod hls;
mod pipeline;
mod plan;
mod policy;
mod probe;
mod profile;
mod progress;
mod remux;
mod runner_util;
mod runtime;
mod session;

pub use artifact::*;
pub use engine::*;
pub use ffmpeg::*;
pub use hardware::*;
pub use hls::*;
pub use pipeline::*;
pub use plan::*;
pub use policy::*;
pub use probe::*;
pub use profile::*;
pub use progress::*;
pub use remux::*;
pub use runtime::*;
pub use session::*;

#[cfg(test)]
use runner_util::command_with_output_path;
#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::Duration,
    };

    use nako_core::{MediaSourceId, MediaStreamInfo, MediaStreamKind, MediaStreamTechnicalFacts};
    use tokio::time;

    use super::*;

    fn hls_artifacts(
        output_dir: impl Into<PathBuf>,
        playlist_path: impl Into<PathBuf>,
        segment_pattern: impl Into<PathBuf>,
        output: HlsOutputRequirement,
    ) -> HlsArtifactManifest {
        HlsArtifactManifest::single_variant(output_dir, playlist_path, segment_pattern, output)
            .unwrap()
    }

    #[test]
    fn ffmpeg_builder_plans_remux_without_transcoding_streams() {
        let source_id = MediaSourceId::new();
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = RemuxRequest {
            source_id,
            input_path: PathBuf::from("input.mkv"),
            output_path: PathBuf::from("output.mp4"),
            output_container: RemuxContainer::Mp4,
            overwrite: FfmpegOverwritePolicy::Never,
        };

        let command = builder.remux(&request).unwrap();

        assert_eq!(command.program, PathBuf::from("ffmpeg"));
        assert_eq!(
            command.argv_lossy(),
            vec![
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "warning",
                "-n",
                "-i",
                "input.mkv",
                "-map",
                "0",
                "-c",
                "copy",
                "-f",
                "mp4",
                "output.mp4",
            ]
        );
        assert!(
            command
                .args
                .iter()
                .any(|arg| arg == &FfmpegArg::path("input.mkv"))
        );
    }

    #[test]
    fn ffmpeg_builder_rejects_in_place_remux() {
        let builder = FfmpegCommandBuilder::default();
        let request = RemuxRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("same.mkv"),
            output_path: PathBuf::from("same.mkv"),
            output_container: RemuxContainer::Mkv,
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        assert!(builder.remux(&request).is_err());
    }

    #[test]
    fn ffmpeg_builder_plans_hls_single_variant() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let command = builder.hls(&request).unwrap();

        assert_eq!(
            command.argv_lossy(),
            vec![
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "warning",
                "-nostats",
                "-progress",
                "pipe:1",
                "-y",
                "-i",
                "input.mkv",
                "-map",
                "0:v:0",
                "-map",
                "0:a:0?",
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "-f",
                "hls",
                "-hls_time",
                "6",
                "-hls_playlist_type",
                "vod",
                "-hls_segment_filename",
                "hls/segment_%05d.ts",
                "hls/playlist.m3u8",
            ]
        );
    }

    #[test]
    fn ffmpeg_builder_plans_hls_fmp4_single_variant() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.m4s",
                HlsOutputRequirement {
                    variant_policy: HlsVariantPolicy::SingleVariant,
                    segment_container: HlsSegmentContainer::Fmp4,
                },
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-hls_segment_type" && args[1] == "fmp4")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-hls_fmp4_init_filename" && args[1] == "init.mp4")
        );
        assert!(argv.windows(2).any(|args| {
            args[0] == "-hls_segment_filename" && args[1] == "hls/segment_%05d.m4s"
        }));
    }

    #[test]
    fn ffmpeg_builder_rejects_hls_segment_container_mismatch() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let artifacts = HlsArtifactManifest {
            output_dir: PathBuf::from("hls"),
            primary_playlist_path: PathBuf::from("hls/playlist.m3u8"),
            media_segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
            variant_playlist_pattern: None,
            renditions: Vec::new(),
            has_audio: true,
            output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::SingleVariant,
                segment_container: HlsSegmentContainer::Fmp4,
            },
        };
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts,
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let err = builder.hls(&request).unwrap_err();

        assert!(err.to_string().contains("segment pattern extension"));
    }

    #[test]
    fn ffmpeg_builder_plans_adaptive_hls_fmp4_ladder() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let artifacts = HlsArtifactManifest::adaptive_fmp4(
            "hls",
            "hls/master.m3u8",
            HlsRendition::default_adaptive_ladder(),
        )
        .unwrap();
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts,
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(argv.windows(2).any(|args| {
            args[0] == "-hls_segment_filename" && args[1] == "hls/variant_%v_segment_%05d.m4s"
        }));
        assert!(argv.windows(2).any(|args| {
            args[0] == "-hls_fmp4_init_filename" && args[1] == "variant_%v_init.mp4"
        }));
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-master_pl_name" && args[1] == "master.m3u8")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-var_stream_map" && args[1] == "v:0,a:0 v:1,a:1")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-s:v:0" && args[1] == "1280x720")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-s:v:1" && args[1] == "854x480")
        );
        assert!(argv.contains(&"hls/variant_%v.m3u8".to_owned()));
    }

    #[test]
    fn ffmpeg_builder_plans_adaptive_hls_without_audio_streams() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let artifacts = HlsArtifactManifest::adaptive_fmp4_with_audio(
            "hls",
            "hls/master.m3u8",
            HlsRendition::default_adaptive_ladder(),
            false,
        )
        .unwrap();
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts,
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-var_stream_map" && args[1] == "v:0 v:1")
        );
        assert!(!argv.iter().any(|arg| arg == "0:a:0?"));
        assert!(!argv.windows(2).any(|args| args[0] == "-c:a"));
    }

    #[test]
    fn hls_adaptive_ladder_plan_respects_source_and_client_caps() {
        let source = source_video_with_shape_and_audio(1920, 1080, Some(4_000_000), true);
        let plan = HlsAdaptiveLadderPlan::from_source_facts(
            Some(&source),
            TranscodeOutputConstraints {
                max_video_bitrate: Some(2_000_000),
                max_width: Some(1280),
                max_height: Some(720),
                prefer_hdr: None,
            },
        );

        assert!(plan.has_audio());
        assert_eq!(
            plan.renditions()[0],
            HlsRendition::new(0, 1280, 720, 2_000_000, 128_000)
        );
        assert!(
            plan.renditions()
                .iter()
                .all(|rendition| rendition.width <= 1280
                    && rendition.height <= 720
                    && rendition.video_bitrate <= 2_000_000)
        );
        assert!(
            !plan
                .renditions()
                .iter()
                .any(|rendition| rendition.height > 720)
        );
        assert_eq!(
            HlsAdaptiveLadderPlan::from_identity_key(&plan.identity_key()).unwrap(),
            plan
        );
    }

    #[test]
    fn hls_adaptive_ladder_plan_avoids_upscale_and_records_no_audio() {
        let source = source_video_with_shape_and_audio(640, 360, Some(700_000), false);
        let plan = HlsAdaptiveLadderPlan::from_source_facts(
            Some(&source),
            TranscodeOutputConstraints::default(),
        );

        assert!(!plan.has_audio());
        assert_eq!(
            plan.renditions(),
            &[HlsRendition::new(0, 640, 360, 700_000, 128_000)]
        );
        assert!(plan.identity_key().contains("audio=false"));
    }

    #[test]
    fn ffmpeg_builder_rejects_hls_outputs_outside_layout() {
        let builder = FfmpegCommandBuilder::default();
        let artifacts = HlsArtifactManifest {
            output_dir: PathBuf::from("hls"),
            primary_playlist_path: PathBuf::from("outside/playlist.m3u8"),
            media_segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
            variant_playlist_pattern: None,
            renditions: Vec::new(),
            has_audio: true,
            output: HlsOutputRequirement::default(),
        };
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts,
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        assert!(builder.hls(&request).is_err());
    }

    #[test]
    fn ffmpeg_builder_plans_hls_with_nvenc_policy() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::Nvenc),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let command = builder.hls(&request).unwrap();
        let argv = command.argv_lossy();

        assert!(argv.contains(&"h264_nvenc".to_owned()));
        assert!(!argv.contains(&"libx264".to_owned()));
        assert!(!argv.windows(2).any(|args| args[0] == "-hwaccel"));
    }

    #[test]
    fn ffmpeg_builder_plans_hls_with_stage_aware_vaapi_policy() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::Vaapi),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-hwaccel" && args[1] == "vaapi")
        );
        assert!(
            argv.iter().position(|arg| arg == "-hwaccel").unwrap()
                < argv.iter().position(|arg| arg == "-i").unwrap()
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-vf" && args[1] == "format=nv12,hwupload")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-c:v" && args[1] == "h264_vaapi")
        );
    }

    #[test]
    fn ffmpeg_builder_plans_hls_with_stage_aware_quicksync_policy() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::QuickSync),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert_eq!(
            argv.windows(2)
                .filter(|args| args[0] == "-hwaccel" && args[1] == "qsv")
                .count(),
            1
        );
        assert!(
            argv.iter().position(|arg| arg == "-hwaccel").unwrap()
                < argv.iter().position(|arg| arg == "-i").unwrap()
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-c:v" && args[1] == "h264_qsv")
        );
    }

    #[test]
    fn ffmpeg_builder_plans_hls_with_platform_encoder_policies() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");

        for (acceleration, encoder) in [
            (HardwareAcceleration::Amf, "h264_amf"),
            (HardwareAcceleration::VideoToolbox, "h264_videotoolbox"),
        ] {
            let request = HlsRequest {
                source_id: MediaSourceId::new(),
                input_path: PathBuf::from("input.mkv"),
                artifacts: hls_artifacts(
                    "hls",
                    "hls/playlist.m3u8",
                    "hls/segment_%05d.ts",
                    HlsOutputRequirement::default(),
                ),
                segment_time_seconds: 6,
                execution_policy: hls_policy(acceleration),
                overwrite: FfmpegOverwritePolicy::Allow,
            };

            let argv = builder.hls(&request).unwrap().argv_lossy();

            assert!(
                argv.windows(2)
                    .any(|args| args[0] == "-c:v" && args[1] == encoder)
            );
        }
    }

    #[test]
    fn ffmpeg_builder_applies_hls_output_constraints_from_policy() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut execution_policy = hls_policy(HardwareAcceleration::None);
        execution_policy.output_constraints.max_video_bitrate = Some(8_000_000);
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy,
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-maxrate" && args[1] == "8000000")
        );
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-bufsize" && args[1] == "16000000")
        );
    }

    #[test]
    fn ffmpeg_builder_accepts_hls_omitted_subtitle_strategy_without_subtitle_maps() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut execution_policy = hls_policy(HardwareAcceleration::None);
        execution_policy.subtitle_strategy = TranscodeSubtitleStrategy::OmitSelected;
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy,
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(!argv.windows(2).any(|args| {
            args[0] == "-map" && (args[1].starts_with("0:s") || args[1].starts_with("0:2"))
        }));
        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-c:a" && args[1] == "aac")
        );
    }

    #[test]
    fn ffmpeg_builder_plans_hls_muxer_with_minimum_segment_time() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 0,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let argv = builder.hls(&request).unwrap().argv_lossy();

        assert!(
            argv.windows(2)
                .any(|args| args[0] == "-hls_time" && args[1] == "1")
        );
        assert!(argv.windows(2).any(|args| {
            args[0] == "-hls_segment_filename" && args[1] == "hls/segment_%05d.ts"
        }));
    }

    #[test]
    fn ffmpeg_builder_rejects_unimplemented_hls_subtitle_strategies() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut execution_policy = hls_policy(HardwareAcceleration::None);
        execution_policy.subtitle_strategy = TranscodeSubtitleStrategy::SidecarSelected;
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy,
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let err = builder.hls(&request).unwrap_err();

        assert!(err.to_string().contains("subtitle strategy"));
    }

    #[test]
    fn transcode_profile_identity_changes_when_hls_acceleration_plan_changes() {
        let cpu = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();
        let nvenc = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::Nvenc),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();

        assert_ne!(cpu.persisted_request_key(), nvenc.persisted_request_key());
        assert_ne!(cpu.storage_slug(), nvenc.storage_slug());
        assert!(
            cpu.persisted_request_key()
                .contains("kind=hls_single_variant")
        );
        assert!(
            cpu.persisted_request_key()
                .contains("acceleration=accel:v1:decode=none")
        );
        assert!(nvenc.persisted_request_key().contains("encode=nvenc"));
        assert!(cpu.storage_slug().starts_with("hls_single_variant-v1-"));
    }

    #[test]
    fn transcode_profile_identity_changes_when_hls_segment_container_changes() {
        let ts = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();
        let fmp4 = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::SingleVariant,
                segment_container: HlsSegmentContainer::Fmp4,
            },
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();

        assert_ne!(ts.persisted_request_key(), fmp4.persisted_request_key());
        assert_ne!(ts.storage_slug(), fmp4.storage_slug());
        assert!(ts.persisted_request_key().contains("hls_segment=mpeg_ts"));
        assert!(fmp4.persisted_request_key().contains("hls_segment=fmp4"));
    }

    #[test]
    fn transcode_profile_output_shape_separates_remux_and_hls_state() {
        let remux = TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container: RemuxContainer::Mp4,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });
        let hls_output = HlsOutputRequirement {
            variant_policy: HlsVariantPolicy::SingleVariant,
            segment_container: HlsSegmentContainer::Fmp4,
        };
        let hls = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });
        let adaptive_output = HlsOutputRequirement {
            variant_policy: HlsVariantPolicy::Adaptive,
            segment_container: HlsSegmentContainer::Fmp4,
        };
        let adaptive_hls = TranscodeProfile::hls(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: adaptive_output,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });

        assert_eq!(remux.kind(), TranscodeProfileKind::Remux);
        assert_eq!(
            remux.output,
            TranscodeOutputShape::Remux {
                container: RemuxContainer::Mp4
            }
        );
        assert_eq!(remux.hls_output_requirement(), None);
        assert_eq!(hls.kind(), TranscodeProfileKind::HlsSingleVariant);
        assert_eq!(
            hls.output,
            TranscodeOutputShape::Hls {
                requirement: hls_output
            }
        );
        assert_eq!(hls.hls_output_requirement(), Some(hls_output));
        assert_eq!(adaptive_hls.kind(), TranscodeProfileKind::HlsAdaptive);
        assert_eq!(adaptive_hls.hls_output_requirement(), Some(adaptive_output));
        assert!(
            adaptive_hls
                .identity()
                .persisted_request_key()
                .contains("kind=hls_adaptive")
        );
    }

    #[test]
    fn transcode_request_identity_includes_source_revision_and_profile() {
        let source = nako_core::MediaSource {
            id: MediaSourceId::new(),
            library_id: nako_core::LibraryId::new(),
            item_id: nako_core::MediaItemId::new(),
            locator: "local:///Movies/Demo.mkv".to_owned(),
            file_name: "Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256:demo".to_owned()),
        };
        let changed_source = nako_core::MediaSource {
            fingerprint: Some("sha256:changed".to_owned()),
            ..source.clone()
        };
        let playback_profile_key = "playback-profile:v1;client=default".to_owned();
        let remux = TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container: RemuxContainer::Mp4,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: playback_profile_key.clone(),
        })
        .identity();
        let hls = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key,
        })
        .identity();

        let remux_request = remux.bind_source(&TranscodeSourceIdentity::from_media_source(&source));
        let same_remux_request =
            remux.bind_source(&TranscodeSourceIdentity::from_media_source(&source));
        let changed_source_request =
            remux.bind_source(&TranscodeSourceIdentity::from_media_source(&changed_source));
        let hls_request = hls.bind_source(&TranscodeSourceIdentity::from_media_source(&source));

        assert_eq!(remux_request, same_remux_request);
        assert_ne!(
            remux_request.persisted_request_key(),
            changed_source_request.persisted_request_key()
        );
        assert_ne!(
            remux_request.persisted_request_key(),
            hls_request.persisted_request_key()
        );
        assert_ne!(
            remux_request.storage_slug(),
            changed_source_request.storage_slug()
        );
        assert_ne!(remux_request.storage_slug(), hls_request.storage_slug());
        assert!(
            remux_request
                .persisted_request_key()
                .starts_with("transcode-request:v1;source=source-revision:v1;")
        );
        assert!(
            remux_request
                .persisted_request_key()
                .contains(";profile=transcode-profile:v1")
        );
    }

    #[test]
    fn transcode_profile_validation_rejects_remux_with_video_codec() {
        let mut profile = TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container: RemuxContainer::Mp4,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });
        profile.video_codec = Some("h264".to_owned());

        let err = profile.validate().unwrap_err();

        assert_eq!(
            err.reason,
            TranscodeProfileValidationReason::RemuxMustNotTranscodeVideo
        );
        assert!(err.operator_message.contains("remux"));
        assert!(!err.operator_message.contains("input"));
    }

    #[test]
    #[should_panic(expected = "transcode profile must be valid before identity")]
    fn transcode_profile_identity_rejects_invalid_profile() {
        let mut profile = TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container: RemuxContainer::Mp4,
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });
        profile.execution_policy.acceleration =
            TranscodeAccelerationPlan::for_selected_hardware(HardwareAcceleration::Nvenc);

        let _ = profile.identity();
    }

    #[test]
    fn transcode_profile_validation_rejects_hls_with_unsupported_codecs() {
        let mut profile = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("vp9".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });

        let err = profile.validate().unwrap_err();
        assert_eq!(
            err.reason,
            TranscodeProfileValidationReason::HlsVideoCodecUnsupported
        );

        profile.video_codec = Some("h264".to_owned());
        profile.audio_codec = Some("opus".to_owned());
        let err = profile.validate().unwrap_err();
        assert_eq!(
            err.reason,
            TranscodeProfileValidationReason::HlsAudioCodecUnsupported
        );
        assert!(!err.operator_message.contains("local:///"));
    }

    #[test]
    fn transcode_profile_validation_accepts_adaptive_fmp4_profile_identity() {
        let profile = TranscodeProfile::hls(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::Fmp4,
            },
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });

        profile.validate().unwrap();
        let identity = profile.identity();

        assert!(
            identity
                .persisted_request_key()
                .contains("hls_variant=adaptive")
        );
        assert!(identity.storage_slug().starts_with("hls_adaptive-v1-"));
    }

    #[test]
    fn transcode_profile_validation_rejects_adaptive_mpeg_ts_profile() {
        let profile = TranscodeProfile::hls(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::None),
            hls_output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::MpegTs,
            },
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });

        let err = profile.validate().unwrap_err();

        assert_eq!(
            err.reason,
            TranscodeProfileValidationReason::HlsAdaptiveRequiresFmp4
        );
    }

    #[test]
    fn transcode_profile_validation_accepts_current_hls_playback_profile() {
        let profile = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("H264".to_owned()),
            audio_codec: Some(" AAC ".to_owned()),
            execution_policy: TranscodeExecutionPolicy::hls_single_variant(
                TranscodeAccelerationPlan::for_selected_hardware(HardwareAcceleration::Nvenc),
                TranscodeTrackSelection {
                    audio_stream: Some(1),
                    subtitle_stream: Some(2),
                },
                TranscodeOutputConstraints {
                    max_video_bitrate: Some(8_000_000),
                    max_width: None,
                    max_height: None,
                    prefer_hdr: Some(true),
                },
            ),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: Some(2),
            },
            remote_input: true,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        });

        profile.validate().unwrap();
        assert_eq!(profile.video_codec.as_deref(), Some("h264"));
        assert_eq!(profile.audio_codec.as_deref(), Some("aac"));
    }

    #[test]
    fn playback_transcode_plan_validation_rejects_unsupported_hls_codecs() {
        let mut plan = TranscodePlan {
            input_locator: "local:///demo.mkv".to_owned(),
            output_container: OutputContainer::Hls,
            video_codec: Some("vp9".to_owned()),
            audio_codec: Some("aac".to_owned()),
        };

        let err = plan.validate_for_playback_request().unwrap_err();
        assert_eq!(
            err.reason,
            TranscodePlanValidationReason::HlsMustUseSupportedVideoCodec
        );

        plan.video_codec = Some("h264".to_owned());
        plan.audio_codec = Some("opus".to_owned());
        let err = plan.validate_for_playback_request().unwrap_err();
        assert_eq!(
            err.reason,
            TranscodePlanValidationReason::HlsMustUseSupportedAudioCodec
        );
    }

    #[test]
    fn ffmpeg_progress_parser_keeps_latest_redaction_safe_metrics() {
        let metrics = parse_ffmpeg_progress_report(
            b"frame=12\nfps=24.98\nbitrate=1234.5kbits/s\ntotal_size=4096\nout_time_us=1500000\ndup_frames=1\ndrop_frames=2\nspeed=1.25x\nprogress=continue\nframe=24\nout_time=00:00:02.500000\nprogress=end\n",
        );

        assert_eq!(metrics.frame_count, Some(24));
        assert_eq!(metrics.fps_millis, None);
        assert_eq!(metrics.bitrate_kbps, None);
        assert_eq!(metrics.total_size_bytes, None);
        assert_eq!(metrics.output_time_ms, Some(2_500));
        assert_eq!(
            metrics.progress,
            Some(nako_core::TranscodeSessionRuntimeProgress::End)
        );
    }

    #[test]
    fn ffmpeg_progress_parser_reads_single_snapshot_metrics() {
        let metrics = parse_ffmpeg_progress_report(
            b"frame=12\nfps=24.98\nbitrate=1234.5kbits/s\ntotal_size=4096\nout_time_us=1500000\ndup_frames=1\ndrop_frames=2\nspeed=1.25x\nprogress=continue\n",
        );

        assert_eq!(metrics.frame_count, Some(12));
        assert_eq!(metrics.fps_millis, Some(24_980));
        assert_eq!(metrics.bitrate_kbps, Some(1_234));
        assert_eq!(metrics.total_size_bytes, Some(4_096));
        assert_eq!(metrics.output_time_ms, Some(1_500));
        assert_eq!(metrics.dup_frames, Some(1));
        assert_eq!(metrics.drop_frames, Some(2));
        assert_eq!(metrics.speed_millis, Some(1_250));
        assert_eq!(
            metrics.progress,
            Some(nako_core::TranscodeSessionRuntimeProgress::Continue)
        );
    }

    #[test]
    fn pipeline_planner_selects_available_and_falls_back_to_cpu() {
        let report = HardwareAccelerationReport::with_available([
            HardwareAcceleration::None,
            HardwareAcceleration::Nvenc,
        ]);
        let planner = TranscodePipelinePlanner::new();
        let nvenc = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    HardwareAccelerationPolicy {
                        requested: HardwareAcceleration::Nvenc,
                        fallback: HardwareAccelerationFallback::Cpu,
                    },
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                ),
                &report,
            )
            .unwrap();
        let fallback = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    HardwareAccelerationPolicy {
                        requested: HardwareAcceleration::Vaapi,
                        fallback: HardwareAccelerationFallback::Cpu,
                    },
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                ),
                &report,
            )
            .unwrap();

        assert_eq!(nvenc.selected_acceleration(), HardwareAcceleration::Nvenc);
        assert!(!nvenc.fallback_used());
        assert_eq!(fallback.selected_acceleration(), HardwareAcceleration::None);
        assert!(fallback.fallback_used());
        assert_eq!(
            fallback.acceleration.encode.accelerator,
            HardwareAcceleration::None
        );
        assert_eq!(
            fallback.acceleration.fallback.requested,
            HardwareAcceleration::Vaapi
        );
        assert!(fallback.acceleration.fallback.fallback_used);
    }

    #[test]
    fn pipeline_planner_falls_back_when_source_codec_does_not_match_hardware_decode_path() {
        let report = HardwareAccelerationReport::with_available([
            HardwareAcceleration::None,
            HardwareAcceleration::Vaapi,
        ]);
        let planner = TranscodePipelinePlanner::new();

        let plan = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    HardwareAccelerationPolicy {
                        requested: HardwareAcceleration::Vaapi,
                        fallback: HardwareAccelerationFallback::Cpu,
                    },
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                )
                .with_source(source_video("hevc", None)),
                &report,
            )
            .unwrap();

        assert_eq!(plan.selected_acceleration(), HardwareAcceleration::None);
        assert!(plan.fallback_used());
        assert_eq!(
            plan.readiness.reason,
            TranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline
        );
    }

    #[test]
    fn pipeline_planner_rejects_source_incompatible_hardware_when_fallback_is_fail() {
        let report = HardwareAccelerationReport::with_available([
            HardwareAcceleration::None,
            HardwareAcceleration::QuickSync,
        ]);
        let planner = TranscodePipelinePlanner::new();

        let err = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    HardwareAccelerationPolicy {
                        requested: HardwareAcceleration::QuickSync,
                        fallback: HardwareAccelerationFallback::Fail,
                    },
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                )
                .with_source(source_video("h264", Some(10))),
                &report,
            )
            .unwrap_err();

        assert!(err.to_string().contains("incompatible with source media"));
    }

    #[test]
    fn ffmpeg_encoder_report_detects_hardware_accelerators() {
        let report = hls_probe_report(
            r#"
 V..... libx264
 A..... aac
 V..... h264_nvenc
 V..... h264_vaapi
 V..... h264_qsv
"#,
        );

        assert!(report.is_available(HardwareAcceleration::None));
        assert!(report.is_available(HardwareAcceleration::Nvenc));
        assert!(report.is_available(HardwareAcceleration::Vaapi));
        assert!(report.is_available(HardwareAcceleration::QuickSync));
        assert!(!report.is_available(HardwareAcceleration::Amf));
        assert!(!report.is_available(HardwareAcceleration::VideoToolbox));
        assert!(
            report
                .capability_for(HardwareAcceleration::Nvenc)
                .unwrap()
                .reason
                .as_deref()
                .unwrap()
                .contains("h264_nvenc")
        );
    }

    #[test]
    fn ffmpeg_probe_inventory_parses_stage_lists_without_headers() {
        let inventory = FfmpegProbeInventory::from_outputs(
            r#"
Encoders:
 V..... h264_nvenc        NVIDIA NVENC H.264 encoder
 A..... aac               AAC encoder
"#,
            r#"
Decoders:
 VFS..D h264              H.264 decoder
 V..... h264_qsv          H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10
"#,
            r#"
Hardware acceleration methods:
vaapi
qsv
videotoolbox
"#,
            r#"
Filters:
 T.. = Timeline support
 ... hwupload          Upload a normal frame to a hardware frame
 ... scale_vaapi       Scale to/from VAAPI surfaces
"#,
            r#"
Bitstream filters:
h264_mp4toannexb
hevc_metadata
"#,
        );

        assert!(inventory.has_encoder("h264_nvenc"));
        assert!(inventory.has_decoder("h264"));
        assert!(inventory.has_decoder("h264_qsv"));
        assert!(inventory.has_hwaccel("vaapi"));
        assert!(inventory.has_filter("hwupload"));
        assert!(inventory.has_bitstream_filter("h264_mp4toannexb"));
        assert!(!inventory.encoders.contains("Encoders:"));
    }

    #[test]
    fn ffmpeg_probe_detector_runs_stage_inventory_commands() {
        let temp = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_probe_ffmpeg_script(temp.path(), "probe_success", false);
        let detector = FfmpegHardwareAccelerationDetector::new(ffmpeg_path);

        let report = detector.detect_result().unwrap();
        let vaapi = report.capability_for(HardwareAcceleration::Vaapi).unwrap();

        assert!(report.is_available(HardwareAcceleration::Vaapi));
        assert!(vaapi.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Hwaccel
                && stage.required
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::Listed
                && stage.feature.as_deref() == Some("vaapi")
        }));
        assert!(vaapi.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Filter
                && stage.required
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::Listed
                && stage.feature.as_deref() == Some("hwupload")
        }));
        assert!(vaapi.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::BitstreamFilter
                && !stage.required
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::Listed
                && stage.feature.as_deref() == Some("h264_mp4toannexb")
        }));
    }

    #[test]
    fn ffmpeg_probe_detector_degrades_when_stage_command_fails() {
        let temp = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_probe_ffmpeg_script(temp.path(), "probe_failure", true);
        let detector = FfmpegHardwareAccelerationDetector::new(ffmpeg_path);

        let report = detector.detect();
        let vaapi = report.capability_for(HardwareAcceleration::Vaapi).unwrap();

        assert!(!report.is_available(HardwareAcceleration::Vaapi));
        assert!(vaapi.has_probe_error());
        assert!(vaapi.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Filter
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::ProbeError
        }));
    }

    #[test]
    fn transcode_runtime_inventory_summarizes_ffmpeg_probe_without_paths() {
        let report = hls_probe_report(
            r#"
 V..... libx264
 A..... aac
 V..... h264_nvenc
"#,
        );

        let inventory = TranscodeRuntimeInventory::ffmpeg_cli(&report);

        assert_eq!(inventory.engine, TranscodeEngineAdapterKind::FfmpegCli);
        assert_eq!(
            inventory.probe_status,
            TranscodeRuntimeInventoryStatus::Ready
        );
        assert!(!inventory.has_probe_error);
        assert_eq!(inventory.hardware_capability_count, 6);
        assert_eq!(inventory.available_gpu_capabilities, 1);
    }

    #[test]
    fn transcode_runtime_inventory_degrades_on_probe_error() {
        let report = HardwareAccelerationReport {
            capabilities: vec![HardwareAccelerationCapability {
                accelerator: HardwareAcceleration::Vaapi,
                available: false,
                device: None,
                reason: Some("ffmpeg hardware capability probe failed".to_owned()),
                stage_capabilities: vec![HardwareStageCapability::probe_error(
                    HardwarePipelineStage::Encode,
                    "failed to run ffmpeg hardware capability probe: denied",
                )],
                encoder_discovery: HardwareEncoderDiscovery::probe_error(
                    "failed to run ffmpeg hardware capability probe: denied",
                ),
                device_initialization: HardwareDeviceInitialization::not_run(
                    HardwareAcceleration::Vaapi,
                ),
                smoke_probe: HardwareSmokeProbe::not_run(HardwareAcceleration::Vaapi),
            }],
        };

        let inventory = TranscodeRuntimeInventory::ffmpeg_cli(&report);

        assert_eq!(inventory.engine, TranscodeEngineAdapterKind::FfmpegCli);
        assert_eq!(
            inventory.probe_status,
            TranscodeRuntimeInventoryStatus::Degraded
        );
        assert!(inventory.has_probe_error);
        assert_eq!(inventory.hardware_capability_count, 1);
        assert_eq!(inventory.available_gpu_capabilities, 0);
    }

    #[test]
    fn ffmpeg_encoder_report_records_safe_evidence_and_operator_smoke_checks() {
        let report = hls_probe_report(" V..... libx264\n A..... aac\n V..... h264_nvenc\n");
        let nvenc = report.capability_for(HardwareAcceleration::Nvenc).unwrap();
        let cpu = report.capability_for(HardwareAcceleration::None).unwrap();

        assert_eq!(
            nvenc.encoder_discovery.status,
            HardwareEncoderDiscoveryStatus::Listed
        );
        assert_eq!(
            nvenc.device_initialization.status,
            HardwareDeviceInitializationStatus::NotRun
        );
        assert_eq!(nvenc.smoke_probe.status, HardwareSmokeProbeStatus::NotRun);
        assert!(nvenc.smoke_probe.operator_check.contains("NVENC"));
        assert!(!nvenc.smoke_probe.operator_check.contains('\\'));
        assert_eq!(
            cpu.encoder_discovery.status,
            HardwareEncoderDiscoveryStatus::Listed
        );
        assert_eq!(
            cpu.encoder_discovery.encoder.as_deref(),
            Some("libx264,aac")
        );
        assert_eq!(
            cpu.device_initialization.status,
            HardwareDeviceInitializationStatus::NotRequired
        );
        assert_eq!(
            cpu.smoke_probe.status,
            HardwareSmokeProbeStatus::NotRequired
        );
    }

    #[test]
    fn ffmpeg_encoder_report_accepts_fake_smoke_probe_results() {
        let smoke_probe = StaticHardwareSmokeProbe::new([
            (
                HardwareAcceleration::Nvenc,
                HardwareSmokeProbe::passed(HardwareAcceleration::Nvenc),
            ),
            (
                HardwareAcceleration::Vaapi,
                HardwareSmokeProbe::failed(HardwareAcceleration::Vaapi, "device smoke failed"),
            ),
        ]);
        let inventory = hls_probe_inventory(" V..... h264_nvenc\n V..... h264_vaapi\n");
        let report = report_from_ffmpeg_probe_inventory_with_smoke_probe(&inventory, &smoke_probe);
        let nvenc = report.capability_for(HardwareAcceleration::Nvenc).unwrap();
        let vaapi = report.capability_for(HardwareAcceleration::Vaapi).unwrap();

        assert_eq!(nvenc.smoke_probe.status, HardwareSmokeProbeStatus::Passed);
        assert!(nvenc.available);
        assert_eq!(vaapi.smoke_probe.status, HardwareSmokeProbeStatus::Failed);
        assert!(!vaapi.available);
        assert!(vaapi.smoke_probe.detail.is_some());
        assert!(vaapi.smoke_probe.operator_check.contains("VAAPI"));
    }

    #[test]
    fn hardware_diagnostics_separate_encoder_device_initialization_and_smoke_probe() {
        let initialization = StaticHardwareDeviceInitialization::new([
            (
                HardwareAcceleration::Nvenc,
                HardwareDeviceInitialization::passed(HardwareAcceleration::Nvenc),
            ),
            (
                HardwareAcceleration::Vaapi,
                HardwareDeviceInitialization::failed(
                    HardwareAcceleration::Vaapi,
                    "device initialization failed",
                ),
            ),
        ]);
        let smoke_probe = StaticHardwareSmokeProbe::new([
            (
                HardwareAcceleration::Nvenc,
                HardwareSmokeProbe::passed(HardwareAcceleration::Nvenc),
            ),
            (
                HardwareAcceleration::Vaapi,
                HardwareSmokeProbe::passed(HardwareAcceleration::Vaapi),
            ),
        ]);
        let inventory = hls_probe_inventory(" V..... h264_nvenc\n V..... h264_vaapi\n");
        let report = report_from_ffmpeg_probe_inventory_with_diagnostics(
            &inventory,
            &initialization,
            &smoke_probe,
        );
        let nvenc = report.capability_for(HardwareAcceleration::Nvenc).unwrap();
        let vaapi = report.capability_for(HardwareAcceleration::Vaapi).unwrap();

        assert!(nvenc.available);
        assert_eq!(
            nvenc.encoder_discovery.status,
            HardwareEncoderDiscoveryStatus::Listed
        );
        assert_eq!(
            nvenc.encoder_discovery.encoder.as_deref(),
            Some("h264_nvenc")
        );
        assert!(nvenc.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Encode
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::Listed
                && stage.feature.as_deref() == Some("h264_nvenc")
        }));
        assert!(nvenc.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Decode
                && stage.discovery_status == HardwareEncoderDiscoveryStatus::Static
                && stage.feature.as_deref() == Some("software")
        }));
        assert_eq!(
            nvenc.device_initialization.status,
            HardwareDeviceInitializationStatus::Passed
        );
        assert_eq!(nvenc.smoke_probe.status, HardwareSmokeProbeStatus::Passed);

        assert!(!vaapi.available);
        assert_eq!(
            vaapi.encoder_discovery.status,
            HardwareEncoderDiscoveryStatus::Listed
        );
        assert_eq!(
            vaapi.device_initialization.status,
            HardwareDeviceInitializationStatus::Failed
        );
        assert_eq!(vaapi.smoke_probe.status, HardwareSmokeProbeStatus::Passed);
        assert!(
            vaapi
                .reason
                .as_deref()
                .unwrap()
                .contains("device initialization failed")
        );
    }

    #[test]
    fn ffmpeg_encoder_report_marks_missing_hardware_unavailable() {
        let report = hls_probe_report(" V..... libx264\n A..... aac\n");

        assert!(report.is_available(HardwareAcceleration::None));
        assert!(!report.is_available(HardwareAcceleration::Nvenc));
        assert!(!report.is_available(HardwareAcceleration::Vaapi));
        assert!(!report.is_available(HardwareAcceleration::QuickSync));
        assert!(!report.is_available(HardwareAcceleration::Amf));
        assert!(!report.is_available(HardwareAcceleration::VideoToolbox));
        assert!(
            report
                .capability_for(HardwareAcceleration::QuickSync)
                .unwrap()
                .encoder_discovery
                .status
                == HardwareEncoderDiscoveryStatus::Missing
        );
        assert!(
            report
                .capability_for(HardwareAcceleration::QuickSync)
                .unwrap()
                .reason
                .as_deref()
                .unwrap()
                .contains("not listed")
        );
    }

    #[test]
    fn ffmpeg_encoder_report_marks_cpu_unavailable_without_required_software_encoders() {
        let report = hls_probe_report(" V..... libx264\n");
        let cpu = report.capability_for(HardwareAcceleration::None).unwrap();

        assert!(!report.is_available(HardwareAcceleration::None));
        assert!(!cpu.available);
        assert_eq!(
            cpu.encoder_discovery.status,
            HardwareEncoderDiscoveryStatus::Missing
        );
        assert_eq!(cpu.encoder_discovery.encoder.as_deref(), Some("aac"));
        assert!(cpu.stage_capabilities.iter().any(|stage| {
            stage.stage == HardwarePipelineStage::Encode
                && stage.required
                && !stage.available
                && stage.feature.as_deref() == Some("aac")
        }));
        assert!(cpu.reason.as_deref().unwrap().contains("aac"));
    }

    #[test]
    fn hardware_policy_can_fail_when_requested_acceleration_is_unavailable() {
        let report = HardwareAccelerationReport::cpu_only();
        let err = TranscodePipelinePlanner::new()
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    HardwareAccelerationPolicy {
                        requested: HardwareAcceleration::QuickSync,
                        fallback: HardwareAccelerationFallback::Fail,
                    },
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                ),
                &report,
            )
            .unwrap_err();

        assert!(err.to_string().contains("unavailable"));
    }

    #[test]
    fn hardware_readiness_classifies_cpu_fallback_without_probe_details() {
        let report = HardwareAccelerationReport::cpu_only();
        let policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::Nvenc,
            fallback: HardwareAccelerationFallback::Cpu,
        };

        let readiness = transcode_pipeline_readiness_without_selection(policy, &report);

        assert_eq!(readiness.status, TranscodePipelineReadinessStatus::Degraded);
        assert_eq!(
            readiness.reason,
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu
        );
        assert_eq!(readiness.requested, HardwareAcceleration::Nvenc);
        assert_eq!(readiness.selected, HardwareAcceleration::None);
        assert!(readiness.fallback_used);
    }

    #[test]
    fn hardware_readiness_classifies_fail_policy_without_selection() {
        let report = HardwareAccelerationReport::cpu_only();
        let policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::QuickSync,
            fallback: HardwareAccelerationFallback::Fail,
        };

        let readiness = transcode_pipeline_readiness_without_selection(policy, &report);

        assert_eq!(
            readiness.status,
            TranscodePipelineReadinessStatus::Unavailable
        );
        assert_eq!(
            readiness.reason,
            TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy
        );
        assert_eq!(readiness.requested, HardwareAcceleration::QuickSync);
        assert_eq!(readiness.selected, HardwareAcceleration::QuickSync);
        assert!(!readiness.fallback_used);
    }

    #[test]
    fn pipeline_planner_rejects_unavailable_software_pipeline() {
        let report = hls_probe_report(" V..... h264_nvenc\n");
        let planner = TranscodePipelinePlanner::new();
        let cpu_policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::None,
            fallback: HardwareAccelerationFallback::Cpu,
        };
        let fallback_policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::Vaapi,
            fallback: HardwareAccelerationFallback::Cpu,
        };

        let cpu_err = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    cpu_policy,
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                ),
                &report,
            )
            .unwrap_err();
        let fallback_err = planner
            .plan_hls_single_variant(
                TranscodePipelineRequest::hls_single_variant(
                    fallback_policy,
                    TranscodeTrackSelection::default(),
                    TranscodeOutputConstraints::default(),
                ),
                &report,
            )
            .unwrap_err();
        let cpu_readiness = transcode_pipeline_readiness_without_selection(cpu_policy, &report);
        let fallback_readiness =
            transcode_pipeline_readiness_without_selection(fallback_policy, &report);

        assert!(cpu_err.to_string().contains("software transcode"));
        assert!(fallback_err.to_string().contains("cpu fallback"));
        assert_eq!(
            cpu_readiness.status,
            TranscodePipelineReadinessStatus::Unavailable
        );
        assert_eq!(
            cpu_readiness.reason,
            TranscodePipelineReadinessReason::SoftwarePipelineUnavailable
        );
        assert_eq!(
            fallback_readiness.status,
            TranscodePipelineReadinessStatus::Unavailable
        );
        assert_eq!(
            fallback_readiness.reason,
            TranscodePipelineReadinessReason::CpuFallbackUnavailable
        );
        assert!(!fallback_readiness.fallback_used);
    }

    #[test]
    fn hardware_readiness_preserves_probe_failure_reason_for_cpu_fallback() {
        let mut report = HardwareAccelerationReport::cpu_only();
        report.capabilities.push(HardwareAccelerationCapability {
            accelerator: HardwareAcceleration::Vaapi,
            available: false,
            device: None,
            reason: Some("ffmpeg hardware capability probe failed".to_owned()),
            stage_capabilities: vec![HardwareStageCapability::probe_error(
                HardwarePipelineStage::Encode,
                "failed to run ffmpeg hardware capability probe: denied",
            )],
            encoder_discovery: HardwareEncoderDiscovery::probe_error(
                "failed to run ffmpeg hardware capability probe: denied",
            ),
            device_initialization: HardwareDeviceInitialization::not_run(
                HardwareAcceleration::Vaapi,
            ),
            smoke_probe: HardwareSmokeProbe::not_run(HardwareAcceleration::Vaapi),
        });
        let policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::Vaapi,
            fallback: HardwareAccelerationFallback::Cpu,
        };

        let readiness = transcode_pipeline_readiness_without_selection(policy, &report);

        assert_eq!(readiness.status, TranscodePipelineReadinessStatus::Degraded);
        assert_eq!(
            readiness.reason,
            TranscodePipelineReadinessReason::ProbeError
        );
        assert_eq!(readiness.selected, HardwareAcceleration::None);
        assert!(readiness.fallback_used);
    }

    #[test]
    fn transcode_resource_budget_is_bounded_by_resource_class() {
        let budget = TranscodeResourceBudget::new(0, 2);

        assert_eq!(budget.slots_for(HardwareAcceleration::None), 1);
        assert_eq!(budget.slots_for(HardwareAcceleration::Vaapi), 2);
    }

    #[test]
    fn remux_session_manager_tracks_lifecycle_without_spawning_ffmpeg() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut manager = TranscodeSessionManager::new();
        let request = RemuxRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_path: PathBuf::from("output.mp4"),
            output_container: RemuxContainer::Mp4,
            overwrite: FfmpegOverwritePolicy::Never,
        };

        let session = manager.plan_remux(request, &builder).unwrap();
        assert_eq!(session.kind, TranscodeSessionKind::Remux);
        assert_eq!(session.state, TranscodeSessionState::Planned);
        assert_eq!(
            session
                .command
                .args
                .iter()
                .filter(|arg| *arg == &FfmpegArg::raw("-c"))
                .count(),
            1
        );

        let starting = manager.mark_starting(session.id).unwrap();
        assert_eq!(starting.state, TranscodeSessionState::Starting);

        let running = manager.mark_running(session.id).unwrap();
        assert_eq!(running.state, TranscodeSessionState::Running);

        let cancelling = manager.request_cancel(session.id).unwrap();
        assert_eq!(cancelling.state, TranscodeSessionState::CancelRequested);

        let cancelled = manager.mark_cancelled(session.id).unwrap();
        assert_eq!(cancelled.state, TranscodeSessionState::Cancelled);
    }

    #[test]
    fn hls_session_manager_tracks_lifecycle_without_spawning_ffmpeg() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut manager = TranscodeSessionManager::new();
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            artifacts: hls_artifacts(
                "hls",
                "hls/playlist.m3u8",
                "hls/segment_%05d.ts",
                HlsOutputRequirement::default(),
            ),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::None),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let session = manager.plan_hls(request, &builder).unwrap();

        assert_eq!(session.kind, TranscodeSessionKind::HlsTranscode);
        assert_eq!(session.state, TranscodeSessionState::Planned);
        assert_eq!(session.output_path, PathBuf::from("hls/playlist.m3u8"));

        let running = manager.mark_running(session.id).unwrap();
        assert_eq!(running.state, TranscodeSessionState::Running);
        let finished = manager.mark_finished(session.id).unwrap();
        assert_eq!(finished.state, TranscodeSessionState::Finished);
    }

    #[test]
    fn remux_session_manager_rejects_invalid_transitions() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut manager = TranscodeSessionManager::new();
        let request = RemuxRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_path: PathBuf::from("output.mp4"),
            output_container: RemuxContainer::Mp4,
            overwrite: FfmpegOverwritePolicy::Never,
        };

        let session = manager.plan_remux(request, &builder).unwrap();
        let err = manager.mark_finished(session.id).unwrap_err();

        assert!(err.to_string().contains("cannot transition"));
    }

    #[test]
    fn remux_session_manager_can_cancel_while_starting() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut manager = TranscodeSessionManager::new();
        let request = RemuxRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_path: PathBuf::from("output.mp4"),
            output_container: RemuxContainer::Mp4,
            overwrite: FfmpegOverwritePolicy::Never,
        };

        let session = manager.plan_remux(request, &builder).unwrap();
        manager.mark_starting(session.id).unwrap();
        let cancelling = manager.request_cancel(session.id).unwrap();

        assert_eq!(cancelling.state, TranscodeSessionState::CancelRequested);
    }

    #[tokio::test]
    async fn remux_runner_promotes_temp_output_on_success() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "success",
            &["printf remuxed > \"$out\"", "exit 0"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let runner = FfmpegRemuxRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 5_000,
        }));

        let outcome = runner
            .run(&mut manager, session.id, CancellationToken::new())
            .await
            .unwrap();

        assert_eq!(
            outcome,
            RemuxRunOutcome::Finished {
                session_id: session.id,
                output_path: output_path.clone()
            }
        );
        assert_eq!(fs::read_to_string(&output_path).unwrap(), "remuxed");
        assert_eq!(
            manager.get(session.id).unwrap().state,
            TranscodeSessionState::Finished
        );
        assert!(temp_files_for(&output_path).is_empty());
    }

    #[tokio::test]
    async fn ffmpeg_remux_engine_adapter_returns_typed_artifact_outcome() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "success",
            &["printf remuxed > \"$out\"", "exit 0"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let engine = FfmpegRemuxRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 5_000,
        }));

        let outcome = engine
            .start(
                &mut manager,
                TranscodeEngineStartCommand {
                    session_id: session.id,
                    cancel: CancellationToken::new(),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            outcome,
            TranscodeEngineStartOutcome::Finished {
                session_id: session.id,
                artifact_kind: TranscodeEngineArtifactKind::RemuxFile,
                output_path: output_path.clone()
            }
        );
        assert_eq!(
            engine.progress(&manager, session.id).unwrap(),
            TranscodeEngineProgress {
                session_id: session.id,
                adapter_kind: TranscodeEngineAdapterKind::FfmpegCli,
                artifact_kind: TranscodeEngineArtifactKind::RemuxFile,
                state: TranscodeSessionState::Finished,
                output_path: output_path.clone(),
                failure_message: None,
                runtime_metrics: Default::default()
            }
        );
        assert_eq!(fs::read_to_string(&output_path).unwrap(), "remuxed");
    }

    #[tokio::test]
    async fn remux_runner_cleans_temp_output_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "failure",
            &["printf partial > \"$out\"", "printf failed >&2", "exit 42"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let runner = FfmpegRemuxRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 5_000,
        }));

        let err = runner
            .run(&mut manager, session.id, CancellationToken::new())
            .await
            .unwrap_err();

        assert!(err.to_string().contains("failed"));
        assert!(!output_path.exists());
        assert!(temp_files_for(&output_path).is_empty());
        assert_eq!(
            manager.get(session.id).unwrap().state,
            TranscodeSessionState::Failed
        );
    }

    #[tokio::test]
    async fn hls_runner_promotes_temp_output_on_success() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_hls_ffmpeg_script(temp.path(), "hls_success");
        let output_dir = temp.path().join("hls");
        let playlist_path = output_dir.join("playlist.m3u8");
        let segment_pattern = output_dir.join("segment_%05d.ts");
        let (mut manager, session) =
            planned_hls_session(&script, &output_dir, &playlist_path, &segment_pattern);
        let runner = FfmpegHlsRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 5_000,
        }));

        let outcome = runner
            .run(&mut manager, session.id, CancellationToken::new())
            .await
            .unwrap();

        assert_eq!(
            outcome,
            HlsRunOutcome::Finished {
                session_id: session.id,
                playlist_path: playlist_path.clone()
            }
        );
        assert!(
            fs::read_to_string(&playlist_path)
                .unwrap()
                .contains("#EXTM3U")
        );
        assert_eq!(
            fs::read_to_string(output_dir.join("segment_00000.ts")).unwrap(),
            "segment"
        );
        assert_eq!(
            manager.get(session.id).unwrap().state,
            TranscodeSessionState::Finished
        );
        assert_eq!(
            manager.get(session.id).unwrap().runtime_metrics.frame_count,
            Some(12)
        );
        assert_eq!(
            manager
                .get(session.id)
                .unwrap()
                .runtime_metrics
                .output_time_ms,
            Some(1_500)
        );
        assert!(temp_hls_dirs_for(&output_dir).is_empty());
    }

    #[tokio::test]
    async fn remux_runner_kills_and_cleans_temp_output_on_cancel() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "cancel",
            &["printf partial > \"$out\"", "sleep 5", "exit 0"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let cancel = CancellationToken::new();
        let cancel_handle = cancel.clone();
        let runner = FfmpegRemuxRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 5_000,
        }));

        tokio::spawn(async move {
            time::sleep(Duration::from_millis(100)).await;
            cancel_handle.cancel();
        });

        let outcome = time::timeout(
            Duration::from_millis(800),
            runner.run(&mut manager, session.id, cancel),
        )
        .await
        .expect("remux cancellation should not wait for inherited stderr pipes")
        .unwrap();

        assert!(matches!(outcome, RemuxRunOutcome::Cancelled { .. }));
        assert!(!output_path.exists());
        assert!(temp_files_for(&output_path).is_empty());
        assert_eq!(
            manager.get(session.id).unwrap().state,
            TranscodeSessionState::Cancelled
        );
    }

    #[tokio::test]
    async fn remux_runner_times_out_and_cleans_temp_output() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "timeout",
            &["printf partial > \"$out\"", "sleep 5", "exit 0"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let runner = FfmpegRemuxRunner::new(TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 100,
        }));

        let err = runner
            .run(&mut manager, session.id, CancellationToken::new())
            .await
            .unwrap_err();

        assert!(err.to_string().contains("timed out"));
        assert!(!output_path.exists());
        assert!(temp_files_for(&output_path).is_empty());
        assert_eq!(
            manager.get(session.id).unwrap().state,
            TranscodeSessionState::Failed
        );
    }

    #[tokio::test]
    async fn remux_runtime_guard_bounds_concurrent_sessions() {
        let guard = TranscodeRuntimeGuard::new(TranscodeRuntimeLimits {
            max_concurrent_sessions: 1,
            timeout_ms: 1_000,
        });
        let first = guard.acquire().await.unwrap();
        let blocked = time::timeout(Duration::from_millis(50), guard.acquire()).await;

        assert!(blocked.is_err());

        drop(first);
        let second = time::timeout(Duration::from_millis(500), guard.acquire())
            .await
            .unwrap()
            .unwrap();
        drop(second);
    }

    #[test]
    fn remux_runner_rejects_command_plan_without_expected_output_path() {
        let command = FfmpegCommandPlan::new(
            "ffmpeg",
            vec![
                FfmpegArg::raw("-i"),
                FfmpegArg::path("input.mkv"),
                FfmpegArg::path("other-output.mp4"),
            ],
        );
        let err = command_with_output_path(
            &command,
            Path::new("expected-output.mp4"),
            Path::new("temp-output.mp4"),
        )
        .unwrap_err();

        assert!(err.to_string().contains("expected output path"));
    }

    fn planned_remux_session(
        ffmpeg_path: &Path,
        output_path: &Path,
    ) -> (TranscodeSessionManager, TranscodeSession) {
        let builder = FfmpegCommandBuilder::new(ffmpeg_path);
        let mut manager = TranscodeSessionManager::new();
        let session = manager
            .plan_remux(
                RemuxRequest {
                    source_id: MediaSourceId::new(),
                    input_path: PathBuf::from("input.mkv"),
                    output_path: output_path.to_path_buf(),
                    output_container: RemuxContainer::Mp4,
                    overwrite: FfmpegOverwritePolicy::Allow,
                },
                &builder,
            )
            .unwrap();

        (manager, session)
    }

    fn hls_policy(acceleration: HardwareAcceleration) -> TranscodeExecutionPolicy {
        TranscodeExecutionPolicy::hls_single_variant(
            TranscodeAccelerationPlan::for_selected_hardware(acceleration),
            TranscodeTrackSelection::default(),
            TranscodeOutputConstraints::default(),
        )
    }

    fn planned_hls_session(
        ffmpeg_path: &Path,
        output_dir: &Path,
        playlist_path: &Path,
        segment_pattern: &Path,
    ) -> (TranscodeSessionManager, TranscodeSession) {
        let builder = FfmpegCommandBuilder::new(ffmpeg_path);
        let mut manager = TranscodeSessionManager::new();
        let session = manager
            .plan_hls(
                HlsRequest {
                    source_id: MediaSourceId::new(),
                    input_path: PathBuf::from("input.mkv"),
                    artifacts: hls_artifacts(
                        output_dir,
                        playlist_path,
                        segment_pattern,
                        HlsOutputRequirement::default(),
                    ),
                    segment_time_seconds: 6,
                    execution_policy: hls_policy(HardwareAcceleration::None),
                    overwrite: FfmpegOverwritePolicy::Allow,
                },
                &builder,
            )
            .unwrap();

        (manager, session)
    }

    fn hls_probe_report(encoders: &str) -> HardwareAccelerationReport {
        report_from_ffmpeg_probe_inventory(&hls_probe_inventory(encoders))
    }

    fn hls_probe_inventory(encoders: &str) -> FfmpegProbeInventory {
        FfmpegProbeInventory::from_outputs(
            encoders,
            r#"
 VFS..D h264
 V..... h264_qsv
"#,
            r#"
vaapi
qsv
videotoolbox
"#,
            r#"
 ... hwupload
 ... scale_vaapi
"#,
            r#"
h264_mp4toannexb
"#,
        )
    }

    fn fake_probe_ffmpeg_script(root: &Path, name: &str, fail_filters: bool) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("for arg do\n");
            content.push_str("case \"$arg\" in\n");
            content.push_str("-encoders)\n");
            content.push_str("cat <<'EOF'\n V..... libx264\n A..... aac\n V..... h264_vaapi\n V..... h264_nvenc\nEOF\n");
            content.push_str("exit 0\n;;\n");
            content.push_str("-decoders)\n");
            content.push_str("cat <<'EOF'\n VFS..D h264\nEOF\n");
            content.push_str("exit 0\n;;\n");
            content.push_str("-hwaccels)\n");
            content.push_str("cat <<'EOF'\nvaapi\nqsv\nEOF\n");
            content.push_str("exit 0\n;;\n");
            content.push_str("-filters)\n");
            if fail_filters {
                content.push_str("echo filters denied 1>&2\nexit 42\n;;\n");
            } else {
                content.push_str("cat <<'EOF'\n ... hwupload\n ... scale_vaapi\nEOF\n");
                content.push_str("exit 0\n;;\n");
            }
            content.push_str("-bsfs)\n");
            content.push_str("cat <<'EOF'\nh264_mp4toannexb\nEOF\n");
            content.push_str("exit 0\n;;\n");
            content.push_str("esac\ndone\n");
            content.push_str("echo missing probe argument 1>&2\nexit 64\n");
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto missing\r\n");
            content.push_str("if \"%~1\"==\"-encoders\" goto encoders\r\n");
            content.push_str("if \"%~1\"==\"-decoders\" goto decoders\r\n");
            content.push_str("if \"%~1\"==\"-hwaccels\" goto hwaccels\r\n");
            content.push_str("if \"%~1\"==\"-filters\" goto filters\r\n");
            content.push_str("if \"%~1\"==\"-bsfs\" goto bsfs\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":encoders\r\n");
            content.push_str("echo  V..... libx264\r\n");
            content.push_str("echo  A..... aac\r\n");
            content.push_str("echo  V..... h264_vaapi\r\n");
            content.push_str("echo  V..... h264_nvenc\r\n");
            content.push_str("exit /b 0\r\n");
            content.push_str(":decoders\r\n");
            content.push_str("echo  VFS..D h264\r\n");
            content.push_str("exit /b 0\r\n");
            content.push_str(":hwaccels\r\n");
            content.push_str("echo vaapi\r\n");
            content.push_str("echo qsv\r\n");
            content.push_str("exit /b 0\r\n");
            content.push_str(":filters\r\n");
            if fail_filters {
                content.push_str("echo filters denied 1>&2\r\n");
                content.push_str("exit /b 42\r\n");
            } else {
                content.push_str("echo  ... hwupload\r\n");
                content.push_str("echo  ... scale_vaapi\r\n");
                content.push_str("exit /b 0\r\n");
            }
            content.push_str(":bsfs\r\n");
            content.push_str("echo h264_mp4toannexb\r\n");
            content.push_str("exit /b 0\r\n");
            content.push_str(":missing\r\n");
            content.push_str("echo missing probe argument 1>&2\r\n");
            content.push_str("exit /b 64\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    fn fake_ffmpeg_script(root: &Path, name: &str, lines: &[&str]) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("for arg do out=\"$arg\"; done\n");
            content.push_str(&lines.join("\n"));
            content.push('\n');
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let _ = lines;
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");

            match name {
                "success" => {
                    content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
                    content.push_str("exit /b 0\r\n");
                }
                "failure" => {
                    content.push_str("<nul set /p dummy=partial>\"%out%\"\r\n");
                    content.push_str("echo failed 1>&2\r\n");
                    content.push_str("exit /b 42\r\n");
                }
                "cancel" | "timeout" => {
                    content.push_str("<nul set /p dummy=partial>\"%out%\"\r\n");
                    content.push_str("ping -n 6 127.0.0.1 > nul\r\n");
                    content.push_str("exit /b 0\r\n");
                }
                _ => unreachable!("unknown fake ffmpeg script"),
            }

            fs::write(&path, content).unwrap();
            path
        }
    }

    fn fake_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("segment_pattern=\n");
            content.push_str("init_pattern=\n");
            content.push_str("master_name=\n");
            content.push_str("prev=\n");
            content.push_str("for arg do\n");
            content.push_str("  if [ \"$prev\" = \"-hls_segment_filename\" ]; then segment_pattern=\"$arg\"; fi\n");
            content.push_str("  if [ \"$prev\" = \"-hls_fmp4_init_filename\" ]; then init_pattern=\"$arg\"; fi\n");
            content.push_str(
                "  if [ \"$prev\" = \"-master_pl_name\" ]; then master_name=\"$arg\"; fi\n",
            );
            content.push_str("  out=\"$arg\"\n");
            content.push_str("  prev=\"$arg\"\n");
            content.push_str("done\n");
            content.push_str("dir=$(dirname \"$out\")\n");
            content.push_str("mkdir -p \"$dir\"\n");
            content.push_str("if [ -n \"$master_name\" ]; then\n");
            content.push_str("  variant0=$(printf '%s' \"$out\" | sed 's/%v/0/g')\n");
            content.push_str("  variant1=$(printf '%s' \"$out\" | sed 's/%v/1/g')\n");
            content.push_str(
                "  segment0=$(printf '%s' \"$segment_pattern\" | sed 's/%v/0/g;s/%05d/00000/g')\n",
            );
            content.push_str(
                "  segment1=$(printf '%s' \"$segment_pattern\" | sed 's/%v/1/g;s/%05d/00000/g')\n",
            );
            content.push_str("  init0=$(printf '%s' \"$init_pattern\" | sed 's/%v/0/g')\n");
            content.push_str("  init1=$(printf '%s' \"$init_pattern\" | sed 's/%v/1/g')\n");
            content.push_str("  variant0_name=$(basename \"$variant0\")\n");
            content.push_str("  variant1_name=$(basename \"$variant1\")\n");
            content.push_str("  printf '#EXTM3U\\n#EXT-X-STREAM-INF:BANDWIDTH=3128000,RESOLUTION=1280x720\\n%s\\n#EXT-X-STREAM-INF:BANDWIDTH=1328000,RESOLUTION=854x480\\n%s\\n' \"$variant0_name\" \"$variant1_name\" > \"$dir/$master_name\"\n");
            content.push_str("  printf '#EXTM3U\\n#EXT-X-MAP:URI=\"%s\"\\n#EXTINF:1,\\n%s\\n#EXT-X-ENDLIST\\n' \"$init0\" \"$(basename \"$segment0\")\" > \"$variant0\"\n");
            content.push_str("  printf '#EXTM3U\\n#EXT-X-MAP:URI=\"%s\"\\n#EXTINF:1,\\n%s\\n#EXT-X-ENDLIST\\n' \"$init1\" \"$(basename \"$segment1\")\" > \"$variant1\"\n");
            content.push_str("  printf init > \"$dir/$init0\"\n");
            content.push_str("  printf init > \"$dir/$init1\"\n");
            content.push_str("  printf segment > \"$segment0\"\n");
            content.push_str("  printf segment > \"$segment1\"\n");
            content.push_str("else\n");
            content.push_str(
                "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
            );
            content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
            content.push_str("fi\n");
            content.push_str(
                "printf 'frame=12\\nout_time_us=1500000\\nspeed=1.25x\\nprogress=end\\n'\n",
            );
            content.push_str("exit 0\n");
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");
            content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
            content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
            content.push_str(">\"%out%\" echo #EXTM3U\r\n");
            content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
            content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
            content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
            content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
            content.push_str("echo frame=12\r\n");
            content.push_str("echo out_time_us=1500000\r\n");
            content.push_str("echo speed=1.25x\r\n");
            content.push_str("echo progress=end\r\n");
            content.push_str("exit /b 0\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    fn source_video(codec: &str, bits_per_raw_sample: Option<u32>) -> TranscodePipelineSourceFacts {
        TranscodePipelineSourceFacts {
            video: Some(MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some(codec.to_owned()),
                language: None,
                duration_ms: None,
                bit_rate: None,
                width: Some(1920),
                height: Some(1080),
                channels: None,
                sample_rate: None,
                technical: MediaStreamTechnicalFacts {
                    bits_per_raw_sample,
                    ..MediaStreamTechnicalFacts::default()
                },
            }),
            audio: None,
            subtitle: None,
        }
    }

    fn source_video_with_shape_and_audio(
        width: u32,
        height: u32,
        bit_rate: Option<u64>,
        has_audio: bool,
    ) -> TranscodePipelineSourceFacts {
        TranscodePipelineSourceFacts {
            video: Some(MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some("h264".to_owned()),
                language: None,
                duration_ms: None,
                bit_rate,
                width: Some(width),
                height: Some(height),
                channels: None,
                sample_rate: None,
                technical: MediaStreamTechnicalFacts::default(),
            }),
            audio: has_audio.then(|| MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: None,
                duration_ms: None,
                bit_rate: Some(128_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: MediaStreamTechnicalFacts::default(),
            }),
            subtitle: None,
        }
    }

    fn temp_files_for(output_path: &Path) -> Vec<PathBuf> {
        let parent = output_path.parent().unwrap();
        let output_name = output_path.file_name().unwrap().to_string_lossy();
        fs::read_dir(parent)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(output_name.as_ref())
                    && path.file_name().unwrap().to_string_lossy().contains(".tmp")
            })
            .collect()
    }

    fn temp_hls_dirs_for(output_dir: &Path) -> Vec<PathBuf> {
        let parent = output_dir.parent().unwrap();
        let output_name = output_dir.file_name().unwrap().to_string_lossy();
        fs::read_dir(parent)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with(output_name.as_ref())
                    && path.file_name().unwrap().to_string_lossy().contains(".tmp")
            })
            .collect()
    }
}
