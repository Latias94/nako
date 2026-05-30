use std::{
    io::ErrorKind,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use nako_core::{
    NakoError, PlaybackSessionId, Result, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionState,
};
use nako_streaming::DirectPlayRangeRequest;
use nako_transcode::{HlsArtifactManifest, HlsArtifactSpec};

use crate::config::PlaybackConfig;

use super::{
    HlsSegmentPlan,
    paths::path_exists,
    playlist::{HlsPlaylistSessionBinding, HlsPlaylistUrlDecoration, author_hls_session_playlist},
};

#[derive(Clone, Copy, Debug)]
pub(super) struct HlsArtifactService {
    config: PlaybackConfig,
}

impl HlsArtifactService {
    pub(super) const fn new(config: PlaybackConfig) -> Self {
        Self { config }
    }

    pub(super) async fn read_playback_playlist(
        &self,
        transcode: &TranscodeSessionRecord,
        playback_session_id: PlaybackSessionId,
        transport_query: Option<&str>,
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
        if transcode.state == TranscodeSessionState::Running
            && !hls_playlist_contains_artifact_uri(&body)
        {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls playlist for session {} is not ready; current state is {:?}",
                    transcode.id, transcode.state
                ),
            });
        }

        author_hls_session_playlist(
            &body,
            &manifest,
            HlsPlaylistSessionBinding::Playback(playback_session_id),
            HlsPlaylistUrlDecoration::optional_query(transport_query),
        )
    }

    pub(super) async fn playlist_is_ready(
        &self,
        transcode: &TranscodeSessionRecord,
    ) -> Result<bool> {
        let manifest = hls_artifact_manifest_for_session(transcode)?;
        let playlist_path = manifest.primary_playlist_path();
        if !path_exists(playlist_path)? {
            return Ok(false);
        }

        let body = match tokio::fs::read_to_string(playlist_path).await {
            Ok(body) => body,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(NakoError::storage_io(
                    playlist_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                ));
            }
        };

        Ok(hls_playlist_contains_artifact_uri(&body))
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

        let Some(total_len) = hls_segment_len(&artifact.path).await? else {
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
        };
        if total_len == 0 && session.state == TranscodeSessionState::Running {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls segment {segment_name} for session {} is not ready; current state is {:?}",
                    session.id, session.state
                ),
            });
        }
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
    HlsArtifactSpec::from_persisted_request_key(&session.request_key)?
        .manifest_for_primary_playlist(session.output_path.clone())
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

fn hls_session_can_serve_artifacts(state: TranscodeSessionState) -> bool {
    matches!(
        state,
        TranscodeSessionState::Running | TranscodeSessionState::Finished
    )
}

fn hls_playlist_contains_artifact_uri(body: &str) -> bool {
    body.lines()
        .map(str::trim)
        .any(|line| !line.is_empty() && !line.starts_with('#'))
}

async fn wait_for_hls_segment_if_configured(
    config: &PlaybackConfig,
    state: TranscodeSessionState,
    path: &Path,
) -> Result<()> {
    if state != TranscodeSessionState::Running || !config.transcode_throttle_enabled {
        return Ok(());
    }

    if let Some(len) = hls_segment_len(path).await? {
        if len > 0 {
            return Ok(());
        }
    }

    tokio::time::sleep(Duration::from_millis(config.transcode_throttle_delay_ms)).await;
    Ok(())
}

async fn hls_segment_len(path: &Path) -> Result<Option<u64>> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Ok(Some(metadata.len())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(NakoError::storage_io(
            path.display().to_string(),
            format!("failed to read hls segment length: {err}"),
        )),
    }
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
    use std::{path::PathBuf, time::Duration};

    use nako_transcode::{
        HlsAdaptiveLadderPlan, HlsAudioRendition, HlsMediaRenditionPlan, HlsOutputRequirement,
        HlsRendition, HlsRequestVariantPlan, HlsSegmentContainer, HlsSubtitleRendition,
        HlsVariantPolicy,
    };

    use super::*;

    #[test]
    fn hls_playlist_readiness_requires_a_media_uri_line() {
        assert!(!hls_playlist_contains_artifact_uri("#EXTM3U\n#EXTINF:1,\n"));
        assert!(hls_playlist_contains_artifact_uri(
            "#EXTM3U\n#EXTINF:1,\nsegment_00000.ts\n"
        ));
        assert!(hls_playlist_contains_artifact_uri(
            "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nvariant_0.m3u8\n"
        ));
    }

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

        let started = std::time::Instant::now();
        wait_for_hls_segment_if_configured(&config, TranscodeSessionState::Running, &segment_path)
            .await
            .unwrap();

        writer.await.unwrap();
        assert!(started.elapsed() >= Duration::from_millis(50));
        assert!(path_exists(&segment_path).unwrap());
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

    #[test]
    fn hls_artifact_manifest_reconstructs_adaptive_ladder_from_request_variant() {
        let plan = HlsAdaptiveLadderPlan::from_source(
            nako_transcode::HlsAdaptiveLadderSource {
                width: Some(640),
                height: Some(360),
                video_bitrate: Some(700_000),
                has_audio: Some(false),
            },
            Default::default(),
        );
        let request_key = format!(
            "transcode-request:v1;source=source-revision:v1;profile=transcode-profile:v1%3Bkind%3Dhls_adaptive%3Bhls_variant%3Dadaptive%3Bhls_segment%3Dfmp4;request_variant={}",
            escape_request_key_component(&plan.identity_key())
        );
        let session = TranscodeSessionRecord {
            id: nako_core::TranscodeSessionId::new(),
            source_id: nako_core::MediaSourceId::new(),
            kind: TranscodeSessionKind::HlsTranscode,
            request_key,
            output_path: PathBuf::from("hls/master.m3u8"),
            state: TranscodeSessionState::Finished,
            failure_category: None,
            failure_message: None,
            runtime_metrics: Default::default(),
            created_at: "2026-05-28T00:00:00Z".to_owned(),
            updated_at: "2026-05-28T00:00:00Z".to_owned(),
            started_at: None,
            completed_at: None,
        };

        let manifest = hls_artifact_manifest_for_session(&session).unwrap();

        assert!(!manifest.has_audio());
        assert_eq!(manifest.renditions(), plan.renditions());
        assert!(
            manifest
                .artifact_for_name("variant_0_segment_00000.m4s")
                .is_ok()
        );
        assert!(
            manifest
                .artifact_for_name("variant_1_segment_00000.m4s")
                .is_err()
        );
    }

    #[test]
    fn hls_artifact_manifest_reconstructs_subtitle_renditions_from_request_variant() {
        let media = HlsMediaRenditionPlan::from_subtitles(vec![HlsSubtitleRendition::new(
            0,
            2,
            Some("eng".to_owned()),
        )])
        .unwrap();
        let request_variant = HlsRequestVariantPlan::new(None, media.clone());
        let request_key = format!(
            "transcode-request:v1;source=source-revision:v1;profile=transcode-profile:v1%3Bkind%3Dhls_single_variant%3Bhls_variant%3Dsingle_variant%3Bhls_segment%3Dmpeg_ts;request_variant={}",
            escape_request_key_component(&request_variant.identity_key().unwrap())
        );
        let session = TranscodeSessionRecord {
            id: nako_core::TranscodeSessionId::new(),
            source_id: nako_core::MediaSourceId::new(),
            kind: TranscodeSessionKind::HlsTranscode,
            request_key,
            output_path: PathBuf::from("hls/playlist.m3u8"),
            state: TranscodeSessionState::Finished,
            failure_category: None,
            failure_message: None,
            runtime_metrics: Default::default(),
            created_at: "2026-05-28T00:00:00Z".to_owned(),
            updated_at: "2026-05-28T00:00:00Z".to_owned(),
            started_at: None,
            completed_at: None,
        };

        let manifest = hls_artifact_manifest_for_session(&session).unwrap();

        assert_eq!(manifest.media_renditions(), &media);
        assert_eq!(
            manifest
                .artifact_for_name("subtitle_0.m3u8")
                .unwrap()
                .content_type,
            "application/vnd.apple.mpegurl"
        );
        assert_eq!(
            manifest
                .artifact_for_name("subtitle_0_00000.vtt")
                .unwrap()
                .content_type,
            "text/vtt"
        );
    }

    #[test]
    fn hls_artifact_manifest_reconstructs_audio_renditions_from_request_variant() {
        let media = HlsMediaRenditionPlan::from_audios(vec![
            HlsAudioRendition::new(0, 1, Some("eng".to_owned()), false),
            HlsAudioRendition::new(1, 2, Some("jpn".to_owned()), true),
        ])
        .unwrap();
        let request_variant = HlsRequestVariantPlan::new(None, media.clone());
        let request_key = format!(
            "transcode-request:v1;source=source-revision:v1;profile=transcode-profile:v1%3Bkind%3Dhls_single_variant%3Bhls_variant%3Dsingle_variant%3Bhls_segment%3Dmpeg_ts;request_variant={}",
            escape_request_key_component(&request_variant.identity_key().unwrap())
        );
        let session = TranscodeSessionRecord {
            id: nako_core::TranscodeSessionId::new(),
            source_id: nako_core::MediaSourceId::new(),
            kind: TranscodeSessionKind::HlsTranscode,
            request_key,
            output_path: PathBuf::from("hls/playlist.m3u8"),
            state: TranscodeSessionState::Finished,
            failure_category: None,
            failure_message: None,
            runtime_metrics: Default::default(),
            created_at: "2026-05-28T00:00:00Z".to_owned(),
            updated_at: "2026-05-28T00:00:00Z".to_owned(),
            started_at: None,
            completed_at: None,
        };

        let manifest = hls_artifact_manifest_for_session(&session).unwrap();

        assert_eq!(manifest.media_renditions(), &media);
        assert_eq!(
            manifest
                .artifact_for_name("audio_0.m3u8")
                .unwrap()
                .content_type,
            "application/vnd.apple.mpegurl"
        );
        assert_eq!(
            manifest
                .artifact_for_name("audio_1_00000.aac")
                .unwrap()
                .content_type,
            "audio/aac"
        );
    }

    fn escape_request_key_component(value: &str) -> String {
        value
            .replace('%', "%25")
            .replace(';', "%3B")
            .replace('=', "%3D")
    }
}
