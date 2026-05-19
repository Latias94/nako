mod ffmpeg;
mod hardware;
mod hls;
mod plan;
mod profile;
mod remux;
mod runner_util;
mod runtime;
mod session;

pub use ffmpeg::*;
pub use hardware::*;
pub use hls::*;
pub use plan::*;
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

    use taru_core::MediaSourceId;
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
            hardware_acceleration: HardwareAcceleration::None,
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
            hardware_acceleration: HardwareAcceleration::None,
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
            hardware_acceleration: HardwareAcceleration::Nvenc,
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        let command = builder.hls(&request).unwrap();
        let argv = command.argv_lossy();

        assert!(argv.contains(&"h264_nvenc".to_owned()));
        assert!(!argv.contains(&"libx264".to_owned()));
    }

    #[test]
    fn transcode_profile_identity_changes_when_hls_hardware_policy_changes() {
        let cpu = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            hardware_acceleration: HardwareAcceleration::None,
            track_selection: TranscodeTrackSelection::default(),
            max_video_bitrate: None,
            prefer_hdr: None,
            remote_input: false,
            playback_profile_key: "playback-profile:v1;client=default".to_owned(),
        })
        .identity();
        let nvenc = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
            hardware_acceleration: HardwareAcceleration::Nvenc,
            track_selection: TranscodeTrackSelection::default(),
            max_video_bitrate: None,
            prefer_hdr: None,
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
        assert!(cpu.persisted_request_key().contains("hw=none"));
        assert!(nvenc.persisted_request_key().contains("hw=nvenc"));
        assert!(cpu.storage_slug().starts_with("hls_single_variant-v1-"));
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
            nvenc.evidence,
            HardwareCapabilityEvidence::FfmpegEncoderListed
        );
        assert_eq!(nvenc.smoke_probe.status, HardwareSmokeProbeStatus::NotRun);
        assert!(nvenc.smoke_probe.operator_check.contains("NVENC"));
        assert!(!nvenc.smoke_probe.operator_check.contains('\\'));
        assert_eq!(cpu.evidence, HardwareCapabilityEvidence::CpuAlwaysAvailable);
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
        assert_eq!(vaapi.smoke_probe.status, HardwareSmokeProbeStatus::Failed);
        assert!(vaapi.smoke_probe.detail.is_some());
        assert!(vaapi.smoke_probe.operator_check.contains("VAAPI"));
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
            hardware_acceleration: HardwareAcceleration::None,
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
                    hardware_acceleration: HardwareAcceleration::None,
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
