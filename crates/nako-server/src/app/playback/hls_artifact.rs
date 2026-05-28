use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use nako_core::{
    NakoError, PlaybackSessionId, Result, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionState,
};
use nako_streaming::DirectPlayRangeRequest;

use crate::config::PlaybackConfig;

use super::{HlsSegmentPlan, paths::path_exists, playlist::validate_hls_segment_name};

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
        rewrite_playlist_segments_for_playback_session(body, session_id)
    }

    pub(super) async fn read_playback_playlist(
        &self,
        transcode: &TranscodeSessionRecord,
        playback_session_id: PlaybackSessionId,
    ) -> Result<String> {
        ensure_hls_session_artifacts_are_servable(transcode)?;
        if !path_exists(&transcode.output_path)?
            && transcode.state == TranscodeSessionState::Running
        {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls playlist for session {} is not ready; current state is {:?}",
                    transcode.id, transcode.state
                ),
            });
        }

        let body = tokio::fs::read_to_string(&transcode.output_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    transcode.output_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                )
            })?;

        Ok(rewrite_playlist_segments_for_playback_session(
            &body,
            playback_session_id,
        ))
    }

    pub(super) async fn plan_segment(
        &self,
        session: &TranscodeSessionRecord,
        segment_name: &str,
    ) -> Result<HlsSegmentPlan> {
        validate_hls_segment_name(segment_name)?;
        ensure_hls_session_artifacts_are_servable(session)?;

        let segment_dir = session.output_path.parent().ok_or_else(|| {
            NakoError::storage_security_violation(
                session.output_path.display().to_string(),
                "hls playlist path does not have a parent directory",
            )
        })?;
        let path = segment_dir.join(segment_name);

        if !path.starts_with(segment_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls segment path escaped the session directory".to_owned(),
            });
        }

        cleanup_hls_segment_dir_if_enabled(&self.config, segment_dir, segment_name).await?;

        wait_for_hls_segment_if_configured(&self.config, session.state, &path).await?;
        if !path_exists(&path)? {
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

        let total_len = tokio::fs::metadata(&path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    path.display().to_string(),
                    format!("failed to read hls segment length: {err}"),
                )
            })?
            .len();
        let response = nako_streaming::plan_direct_play_response(
            total_len,
            "video/mp2t",
            DirectPlayRangeRequest::None,
        );

        Ok(HlsSegmentPlan { path, response })
    }
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

fn rewrite_playlist_segments_for_playback_session(
    body: &str,
    session_id: PlaybackSessionId,
) -> String {
    let mut rewritten = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return line.to_owned();
            }
            let Some(rest) = line.strip_prefix("/playback/sessions/") else {
                return format!("/playback/sessions/{session_id}/hls/segments/{trimmed}");
            };
            let Some((_old_session_id, segment_path)) = rest.split_once("/hls/segments/") else {
                return line.to_owned();
            };

            format!("/playback/sessions/{session_id}/hls/segments/{segment_path}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if body.ends_with('\n') {
        rewritten.push('\n');
    }

    rewritten
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
    segment_dir: &Path,
    requested_segment: &str,
) -> Result<()> {
    if !config.hls_segment_cleanup_enabled {
        return Ok(());
    }

    cleanup_hls_segment_dir_at(
        segment_dir,
        requested_segment,
        config.hls_segment_keep_ms,
        current_time_ms(),
    )
    .await
}

async fn cleanup_hls_segment_dir_at(
    segment_dir: &Path,
    requested_segment: &str,
    keep_ms: u64,
    now_ms: i64,
) -> Result<()> {
    let mut entries = match tokio::fs::read_dir(segment_dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(NakoError::storage_io(
                segment_dir.display().to_string(),
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
                    segment_dir.display().to_string(),
                    format!("failed to iterate hls segment directory: {err}"),
                ));
            }
        };
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name == requested_segment
            || path.extension().and_then(|value| value.to_str()) != Some("ts")
        {
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
    async fn hls_segment_cleanup_removes_stale_siblings_and_keeps_requested() {
        let temp = tempfile::tempdir().unwrap();
        let segment_dir = temp.path();
        let requested = segment_dir.join("segment_00001.ts");
        let stale = segment_dir.join("segment_00000.ts");
        let playlist = segment_dir.join("playlist.m3u8");
        let subtitle = segment_dir.join("segment_00002.vtt");
        tokio::fs::write(&requested, b"requested").await.unwrap();
        tokio::fs::write(&stale, b"stale").await.unwrap();
        tokio::fs::write(&playlist, b"playlist").await.unwrap();
        tokio::fs::write(&subtitle, b"subtitle").await.unwrap();

        cleanup_hls_segment_dir_at(segment_dir, "segment_00001.ts", 60_000, i64::MAX / 2)
            .await
            .unwrap();

        assert!(path_exists(&requested).unwrap());
        assert!(!path_exists(&stale).unwrap());
        assert!(path_exists(&playlist).unwrap());
        assert!(path_exists(&subtitle).unwrap());
    }
}
