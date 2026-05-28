use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use nako_core::{
    NakoError, PlaybackSessionId, Result, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionState,
};
use nako_streaming::DirectPlayRangeRequest;
use nako_transcode::{
    HlsArtifactManifest, HlsOutputRequirement, HlsRendition, HlsSegmentContainer, HlsVariantPolicy,
};

use crate::config::PlaybackConfig;

use super::{
    HlsSegmentPlan, paths::path_exists, playlist::rewrite_hls_playlist_for_playback_session,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct HlsArtifactService {
    config: PlaybackConfig,
}

impl HlsArtifactService {
    pub(super) const fn new(config: PlaybackConfig) -> Self {
        Self { config }
    }

    pub(super) fn rewrite_playlist_for_playback_session(
        &self,
        body: &str,
        session_id: PlaybackSessionId,
    ) -> String {
        rewrite_hls_playlist_for_playback_session(body, session_id)
    }

    pub(super) async fn read_playback_playlist(
        &self,
        transcode: &TranscodeSessionRecord,
        playback_session_id: PlaybackSessionId,
    ) -> Result<String> {
        let manifest = hls_artifact_manifest_for_session(transcode)?;
        let playlist_path = manifest.primary_playlist_path();
        if !path_exists(playlist_path)? && transcode.state == TranscodeSessionState::Running {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls playlist for session {} is not ready; current state is {:?}",
                    transcode.id, transcode.state
                ),
            });
        }

        let body = tokio::fs::read_to_string(playlist_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    playlist_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                )
            })?;

        Ok(rewrite_hls_playlist_for_playback_session(
            &body,
            playback_session_id,
        ))
    }

    pub(super) async fn plan_segment(
        &self,
        session: &TranscodeSessionRecord,
        segment_name: &str,
    ) -> Result<HlsSegmentPlan> {
        let manifest = hls_artifact_manifest_for_session(session)?;
        let artifact = manifest.artifact_for_name(segment_name)?;

        cleanup_hls_segment_dir_if_enabled(&self.config, &manifest, segment_name).await?;

        wait_for_hls_segment_if_configured(&self.config, session.state, &artifact.path).await?;
        if !path_exists(&artifact.path)? {
            if session.state == TranscodeSessionState::Running {
                return Err(NakoError::Conflict {
                    message: format!(
                        "hls segment {segment_name} for session {} is not ready; current state is {:?}",
                        session.id, session.state
                    ),
                });
            }

            return Err(NakoError::NotFound {
                entity: "hls_segment",
                id: segment_name.to_owned(),
            });
        }

        let total_len = tokio::fs::metadata(&artifact.path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    artifact.path.display().to_string(),
                    format!("failed to read hls segment length: {err}"),
                )
            })?
            .len();
        let response = nako_streaming::plan_direct_play_response(
            total_len,
            artifact.content_type,
            DirectPlayRangeRequest::None,
        );

        Ok(HlsSegmentPlan {
            path: artifact.path,
            response,
        })
    }
}

pub(super) fn hls_artifact_manifest_for_session(
    session: &TranscodeSessionRecord,
) -> Result<HlsArtifactManifest> {
    ensure_hls_session_artifacts_are_servable(session)?;
    let output = hls_output_requirement_from_request_key(&session.request_key);
    let output_dir = hls_output_dir_for_primary_playlist(&session.output_path)?;

    if output.variant_policy == HlsVariantPolicy::Adaptive {
        if output.segment_container != HlsSegmentContainer::Fmp4 {
            return Err(NakoError::Unsupported(
                "adaptive hls artifacts currently require fmp4 segments",
            ));
        }

        return HlsArtifactManifest::adaptive_fmp4(
            output_dir,
            session.output_path.clone(),
            HlsRendition::default_adaptive_ladder(),
        );
    }

    let segment_pattern = output_dir.join(format!(
        "segment_%05d.{}",
        output.segment_container.segment_extension()
    ));

    HlsArtifactManifest::single_variant(
        output_dir,
        session.output_path.clone(),
        segment_pattern,
        output,
    )
}

fn ensure_hls_session_artifacts_are_servable(session: &TranscodeSessionRecord) -> Result<()> {
    if session.kind != TranscodeSessionKind::HlsTranscode {
        return Err(NakoError::InvalidInput {
            message: format!("session {} is not an hls transcode session", session.id),
        });
    }

    if !hls_session_can_serve_artifacts(session.state) {
        return Err(NakoError::Conflict {
            message: format!(
                "hls session {} is not ready; current state is {:?}",
                session.id, session.state
            ),
        });
    }

    Ok(())
}

fn hls_output_dir_for_primary_playlist(playlist_path: &Path) -> Result<PathBuf> {
    playlist_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            NakoError::storage_security_violation(
                playlist_path.display().to_string(),
                "hls playlist path does not have a parent directory",
            )
        })
}

fn hls_output_requirement_from_request_key(request_key: &str) -> HlsOutputRequirement {
    let variant_policy = if request_key_contains_component(request_key, "hls_variant", "adaptive") {
        HlsVariantPolicy::Adaptive
    } else {
        HlsVariantPolicy::SingleVariant
    };
    let segment_container = if request_key_contains_component(request_key, "hls_segment", "fmp4") {
        HlsSegmentContainer::Fmp4
    } else {
        HlsSegmentContainer::MpegTs
    };

    HlsOutputRequirement {
        variant_policy,
        segment_container,
    }
}

fn request_key_contains_component(request_key: &str, name: &str, value: &str) -> bool {
    request_key.contains(&format!("{name}={value}"))
        || request_key.contains(&format!("{name}%3D{value}"))
        || request_key.contains(&format!("{name}%3d{value}"))
}

fn hls_session_can_serve_artifacts(state: TranscodeSessionState) -> bool {
    matches!(
        state,
        TranscodeSessionState::Running | TranscodeSessionState::Finished
    )
}

async fn wait_for_hls_segment_if_configured(
    config: &PlaybackConfig,
    state: TranscodeSessionState,
    path: &Path,
) -> Result<()> {
    if path_exists(path)?
        || state != TranscodeSessionState::Running
        || !config.transcode_throttle_enabled
    {
        return Ok(());
    }

    tokio::time::sleep(Duration::from_millis(config.transcode_throttle_delay_ms)).await;
    Ok(())
}

async fn cleanup_hls_segment_dir_if_enabled(
    config: &PlaybackConfig,
    manifest: &HlsArtifactManifest,
    requested_artifact: &str,
) -> Result<()> {
    if !config.hls_segment_cleanup_enabled {
        return Ok(());
    }

    cleanup_hls_segment_dir_at(
        manifest,
        requested_artifact,
        config.hls_segment_keep_ms,
        current_time_ms(),
    )
    .await
}

async fn cleanup_hls_segment_dir_at(
    manifest: &HlsArtifactManifest,
    requested_artifact: &str,
    keep_ms: u64,
    now_ms: i64,
) -> Result<()> {
    let mut entries = match tokio::fs::read_dir(manifest.output_dir()).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(NakoError::storage_io(
                manifest.output_dir().display().to_string(),
                format!("failed to read hls segment directory: {err}"),
            ));
        }
    };
    let keep_ms = i64::try_from(keep_ms).unwrap_or(i64::MAX);

    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(err) => {
                return Err(NakoError::storage_io(
                    manifest.output_dir().display().to_string(),
                    format!("failed to iterate hls segment directory: {err}"),
                ));
            }
        };
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name == requested_artifact || !manifest.cleanup_candidate_for_name(file_name) {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => {
                return Err(NakoError::storage_io(
                    path.display().to_string(),
                    format!("failed to read hls segment metadata: {err}"),
                ));
            }
        };
        if !metadata.is_file() {
            continue;
        }
        let Some(modified_ms) = metadata.modified().ok().and_then(system_time_ms) else {
            continue;
        };
        if now_ms.saturating_sub(modified_ms) < keep_ms {
            continue;
        }

        if let Err(err) = tokio::fs::remove_file(&path).await {
            if err.kind() == std::io::ErrorKind::NotFound {
                continue;
            }

            return Err(NakoError::storage_io(
                path.display().to_string(),
                format!("failed to remove stale hls segment: {err}"),
            ));
        }
    }

    Ok(())
}

fn current_time_ms() -> i64 {
    system_time_ms(SystemTime::now()).unwrap_or(i64::MAX)
}

fn system_time_ms(value: SystemTime) -> Option<i64> {
    let duration = value.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(duration.as_millis()).ok()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn hls_segment_waits_once_for_running_segment_when_throttle_enabled() {
        let temp = tempfile::tempdir().unwrap();
        let segment_path = temp.path().join("segment_00000.ts");
        let writer_path = segment_path.clone();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            tokio::fs::write(writer_path, b"segment").await.unwrap();
        });
        let config = PlaybackConfig {
            transcode_throttle_enabled: true,
            transcode_throttle_delay_ms: 50,
            ..PlaybackConfig::default()
        };

        wait_for_hls_segment_if_configured(&config, TranscodeSessionState::Running, &segment_path)
            .await
            .unwrap();

        assert!(path_exists(&segment_path).unwrap());
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn hls_segment_cleanup_uses_manifest_segment_container() {
        let temp = tempfile::tempdir().unwrap();
        let segment_dir = temp.path();
        let manifest = HlsArtifactManifest::single_variant(
            segment_dir,
            segment_dir.join("playlist.m3u8"),
            segment_dir.join("segment_%05d.ts"),
            HlsOutputRequirement::default(),
        )
        .unwrap();
        let requested = segment_dir.join("segment_00001.ts");
        let stale = segment_dir.join("segment_00000.ts");
        let stale_fmp4 = segment_dir.join("segment_00000.m4s");
        let init = segment_dir.join("init.mp4");
        let playlist = segment_dir.join("playlist.m3u8");
        let subtitle = segment_dir.join("segment_00002.vtt");
        tokio::fs::write(&requested, b"requested").await.unwrap();
        tokio::fs::write(&stale, b"stale").await.unwrap();
        tokio::fs::write(&stale_fmp4, b"stale").await.unwrap();
        tokio::fs::write(&init, b"init").await.unwrap();
        tokio::fs::write(&playlist, b"playlist").await.unwrap();
        tokio::fs::write(&subtitle, b"subtitle").await.unwrap();

        cleanup_hls_segment_dir_at(&manifest, "segment_00001.ts", 60_000, i64::MAX / 2)
            .await
            .unwrap();

        assert!(path_exists(&requested).unwrap());
        assert!(!path_exists(&stale).unwrap());
        assert!(path_exists(&stale_fmp4).unwrap());
        assert!(path_exists(&init).unwrap());
        assert!(path_exists(&playlist).unwrap());
        assert!(path_exists(&subtitle).unwrap());
    }

    #[test]
    fn hls_artifact_manifest_covers_ts_fmp4_segments_and_init() {
        let ts = HlsArtifactManifest::single_variant(
            "hls",
            "hls/playlist.m3u8",
            "hls/segment_%05d.ts",
            HlsOutputRequirement::default(),
        )
        .unwrap();
        let fmp4 = HlsArtifactManifest::single_variant(
            "hls",
            "hls/playlist.m3u8",
            "hls/segment_%05d.m4s",
            HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::SingleVariant,
                segment_container: HlsSegmentContainer::Fmp4,
            },
        )
        .unwrap();

        assert_eq!(
            ts.artifact_for_name("segment_00000.ts")
                .unwrap()
                .content_type,
            "video/mp2t"
        );
        assert_eq!(
            fmp4.artifact_for_name("segment_00000.m4s")
                .unwrap()
                .content_type,
            "video/mp4"
        );
        assert_eq!(
            fmp4.artifact_for_name("init.mp4").unwrap().content_type,
            "video/mp4"
        );
        assert!(ts.artifact_for_name("init.mp4").is_err());
        assert!(ts.artifact_for_name("segment_00000.vtt").is_err());
    }

    #[test]
    fn hls_artifact_manifest_covers_adaptive_master_variants_and_fmp4_segments() {
        let manifest = HlsArtifactManifest::adaptive_fmp4(
            "hls",
            "hls/master.m3u8",
            HlsRendition::default_adaptive_ladder(),
        )
        .unwrap();

        assert_eq!(
            manifest
                .artifact_for_name("master.m3u8")
                .unwrap()
                .content_type,
            "application/vnd.apple.mpegurl"
        );
        assert_eq!(
            manifest
                .artifact_for_name("variant_0.m3u8")
                .unwrap()
                .content_type,
            "application/vnd.apple.mpegurl"
        );
        assert_eq!(
            manifest
                .artifact_for_name("variant_0_init.mp4")
                .unwrap()
                .content_type,
            "video/mp4"
        );
        assert_eq!(
            manifest
                .artifact_for_name("variant_0_segment_00000.m4s")
                .unwrap()
                .content_type,
            "video/mp4"
        );
        assert!(!manifest.cleanup_candidate_for_name("variant_0.m3u8"));
        assert!(!manifest.cleanup_candidate_for_name("variant_0_init.mp4"));
        assert!(manifest.cleanup_candidate_for_name("variant_0_segment_00000.m4s"));
        assert!(manifest.artifact_for_name("init.mp4").is_err());
        assert!(
            manifest
                .artifact_for_name("variant_2_segment_00000.m4s")
                .is_err()
        );
    }
}
