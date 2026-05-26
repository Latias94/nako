mod ffmpeg;
mod hardware;
mod hls;
mod plan;
mod policy;
mod profile;
mod remux;
mod runner_util;
mod runtime;
mod session;

pub use ffmpeg::*;
pub use hardware::*;
pub use hls::*;
pub use plan::*;
pub use policy::*;
pub use profile::*;
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

    use nako_core::MediaSourceId;
    use tokio::time;

    use super::*;

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
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
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
    fn ffmpeg_builder_rejects_hls_outputs_outside_layout() {
        let builder = FfmpegCommandBuilder::default();
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("outside/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
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
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
            segment_time_seconds: 6,
            execution_policy: hls_policy(HardwareAcceleration::Nvenc),
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let command = builder.hls(&request).unwrap();
        let argv = command.argv_lossy();

        assert!(argv.contains(&"h264_nvenc".to_owned()));
        assert!(!argv.contains(&"libx264".to_owned()));
    }

    #[test]
    fn ffmpeg_builder_applies_hls_output_constraints_from_policy() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut execution_policy = hls_policy(HardwareAcceleration::None);
        execution_policy.output_constraints.max_video_bitrate = Some(8_000_000);
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
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
    fn ffmpeg_builder_rejects_unimplemented_hls_subtitle_strategies() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let mut execution_policy = hls_policy(HardwareAcceleration::None);
        execution_policy.subtitle_strategy = TranscodeSubtitleStrategy::SidecarSelected;
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
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
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();
        let nvenc = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            execution_policy: hls_policy(HardwareAcceleration::Nvenc),
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
                    prefer_hdr: Some(true),
                },
            ),
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
    fn hardware_policy_selects_available_and_falls_back_to_cpu() {
        let report = HardwareAccelerationReport::with_available([HardwareAcceleration::Nvenc]);
        let nvenc = select_hardware_acceleration(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Nvenc,
                fallback: HardwareAccelerationFallback::Cpu,
            },
            &report,
        )
        .unwrap();
        let fallback = select_hardware_acceleration(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Vaapi,
                fallback: HardwareAccelerationFallback::Cpu,
            },
            &report,
        )
        .unwrap();

        assert_eq!(nvenc.acceleration, HardwareAcceleration::Nvenc);
        assert!(!nvenc.fallback_used);
        assert_eq!(fallback.acceleration, HardwareAcceleration::None);
        assert!(fallback.fallback_used);

        let fallback_plan = TranscodeAccelerationPlan::from_hardware_selection(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Vaapi,
                fallback: HardwareAccelerationFallback::Cpu,
            },
            &fallback,
        );
        assert_eq!(fallback_plan.encode.accelerator, HardwareAcceleration::None);
        assert_eq!(
            fallback_plan.fallback.requested,
            HardwareAcceleration::Vaapi
        );
        assert!(fallback_plan.fallback.fallback_used);
    }

    #[test]
    fn ffmpeg_encoder_report_detects_hardware_accelerators() {
        let report = report_from_ffmpeg_encoders(
            r#"
 V..... libx264
 V..... h264_nvenc
 V..... h264_vaapi
 V..... h264_qsv
"#,
        );

        assert!(report.is_available(HardwareAcceleration::None));
        assert!(report.is_available(HardwareAcceleration::Nvenc));
        assert!(report.is_available(HardwareAcceleration::Vaapi));
        assert!(report.is_available(HardwareAcceleration::QuickSync));
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
    fn ffmpeg_encoder_report_records_safe_evidence_and_operator_smoke_checks() {
        let report = report_from_ffmpeg_encoders(" V..... h264_nvenc\n");
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
            HardwareEncoderDiscoveryStatus::NotRequired
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
        let report = report_from_ffmpeg_encoders_with_smoke_probe(
            " V..... h264_nvenc\n V..... h264_vaapi\n",
            &smoke_probe,
        );
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
        let report = report_from_ffmpeg_encoders_with_diagnostics(
            " V..... h264_nvenc\n V..... h264_vaapi\n",
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
        let report = report_from_ffmpeg_encoders(" V..... libx264\n");

        assert!(report.is_available(HardwareAcceleration::None));
        assert!(!report.is_available(HardwareAcceleration::Nvenc));
        assert!(!report.is_available(HardwareAcceleration::Vaapi));
        assert!(!report.is_available(HardwareAcceleration::QuickSync));
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
    fn hardware_policy_can_fail_when_requested_acceleration_is_unavailable() {
        let report = HardwareAccelerationReport::cpu_only();
        let err = select_hardware_acceleration(
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::QuickSync,
                fallback: HardwareAccelerationFallback::Fail,
            },
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
        let selection = select_hardware_acceleration(policy, &report).unwrap();

        let readiness = hardware_acceleration_readiness(policy, &selection, &report);

        assert_eq!(
            readiness.status,
            HardwareAccelerationReadinessStatus::Degraded
        );
        assert_eq!(
            readiness.reason,
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu
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

        let readiness = hardware_acceleration_readiness_without_selection(policy, &report);

        assert_eq!(
            readiness.status,
            HardwareAccelerationReadinessStatus::Unavailable
        );
        assert_eq!(
            readiness.reason,
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy
        );
        assert_eq!(readiness.requested, HardwareAcceleration::QuickSync);
        assert_eq!(readiness.selected, HardwareAcceleration::QuickSync);
        assert!(!readiness.fallback_used);
    }

    #[test]
    fn hardware_readiness_preserves_probe_failure_reason_for_cpu_fallback() {
        let report = HardwareAccelerationReport {
            capabilities: vec![HardwareAccelerationCapability {
                accelerator: HardwareAcceleration::Vaapi,
                available: false,
                device: None,
                reason: Some("ffmpeg hardware capability probe failed".to_owned()),
                encoder_discovery: HardwareEncoderDiscovery::probe_error(
                    "failed to run ffmpeg hardware capability probe: denied",
                ),
                device_initialization: HardwareDeviceInitialization::not_run(
                    HardwareAcceleration::Vaapi,
                ),
                smoke_probe: HardwareSmokeProbe::not_run(HardwareAcceleration::Vaapi),
            }],
        };
        let policy = HardwareAccelerationPolicy {
            requested: HardwareAcceleration::Vaapi,
            fallback: HardwareAccelerationFallback::Cpu,
        };
        let selection = select_hardware_acceleration(policy, &report).unwrap();

        let readiness = hardware_acceleration_readiness(policy, &selection, &report);

        assert_eq!(
            readiness.status,
            HardwareAccelerationReadinessStatus::Degraded
        );
        assert_eq!(
            readiness.reason,
            HardwareAccelerationReadinessReason::ProbeError
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
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
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
        let runner = FfmpegRemuxRunner::new(RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
    async fn remux_runner_cleans_temp_output_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let script = fake_ffmpeg_script(
            temp.path(),
            "failure",
            &["printf partial > \"$out\"", "printf failed >&2", "exit 42"],
        );
        let output_path = temp.path().join("output.mp4");
        let (mut manager, session) = planned_remux_session(&script, &output_path);
        let runner = FfmpegRemuxRunner::new(RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
        let runner = FfmpegHlsRunner::new(RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
        let runner = FfmpegRemuxRunner::new(RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
        let runner = FfmpegRemuxRunner::new(RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
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
                    output_dir: output_dir.to_path_buf(),
                    playlist_path: playlist_path.to_path_buf(),
                    segment_pattern: segment_pattern.to_path_buf(),
                    segment_time_seconds: 6,
                    execution_policy: hls_policy(HardwareAcceleration::None),
                    overwrite: FfmpegOverwritePolicy::Allow,
                },
                &builder,
            )
            .unwrap();

        (manager, session)
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
            content.push_str("for arg do out=\"$arg\"; done\n");
            content.push_str("dir=$(dirname \"$out\")\n");
            content.push_str("mkdir -p \"$dir\"\n");
            content.push_str(
                "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
            );
            content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
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
            content.push_str("exit /b 0\r\n");
            fs::write(&path, content).unwrap();
            path
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
