use std::path::{Path, PathBuf};

use taru_core::{Result, TaruError};

use super::{
    ffmpeg::stderr_message,
    runner_util::{
        abort_stderr_task, command_with_hls_output_dir, ffmpeg_command, join_stderr_task,
        kill_child, promote_temp_hls_output, read_child_stderr, remove_dir_if_exists,
    },
    runtime::{CancellationToken, RemuxRuntimeGuard},
    session::{TranscodeSessionId, TranscodeSessionKind, TranscodeSessionManager},
};

#[derive(Clone, Debug)]
pub struct FfmpegHlsRunner {
    guard: RemuxRuntimeGuard,
}

impl FfmpegHlsRunner {
    #[must_use]
    pub fn new(guard: RemuxRuntimeGuard) -> Self {
        Self { guard }
    }

    pub async fn run(
        &self,
        manager: &mut TranscodeSessionManager,
        session_id: TranscodeSessionId,
        cancel: CancellationToken,
    ) -> Result<HlsRunOutcome> {
        let _permit = self.guard.acquire().await?;
        manager.mark_starting(session_id)?;

        let session = manager
            .get(session_id)
            .cloned()
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if session.kind != TranscodeSessionKind::HlsTranscode {
            let message = "ffmpeg hls runner only accepts hls transcode sessions".to_owned();
            let _ = manager.mark_failed(session_id, message.clone());
            return Err(TaruError::InvalidInput { message });
        }

        let output_dir = session
            .output_path
            .parent()
            .ok_or_else(|| TaruError::InvalidInput {
                message: format!(
                    "hls session {} playlist path does not have a parent directory",
                    session.id
                ),
            })?
            .to_path_buf();
        let temp_output_dir = temporary_hls_output_dir(&output_dir, session.id);

        remove_dir_if_exists(&temp_output_dir).await?;
        tokio::fs::create_dir_all(&temp_output_dir)
            .await
            .map_err(|err| {
                TaruError::storage_io(
                    temp_output_dir.display().to_string(),
                    format!("failed to create temporary hls output directory: {err}"),
                )
            })?;

        let command = command_with_hls_output_dir(
            &session.command,
            &output_dir,
            &temp_output_dir,
            &session.output_path,
        )?;
        let mut child = ffmpeg_command(&command)
            .spawn()
            .map_err(|err| TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to start hls session {session_id}: {err}"),
            })?;

        manager.mark_running(session_id)?;

        let stderr_task = tokio::spawn(read_child_stderr(child.stderr.take()));
        let status = tokio::select! {
            status = child.wait() => {
                status.map_err(|err| TaruError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("failed to wait for hls session {session_id}: {err}"),
                })?
            }
            () = cancel.cancelled() => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                manager.request_cancel(session_id)?;
                remove_dir_if_exists(&temp_output_dir).await?;
                manager.mark_cancelled(session_id)?;
                return Ok(HlsRunOutcome::Cancelled {
                    session_id,
                    temp_output_dir,
                });
            }
            () = tokio::time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                manager.request_cancel(session_id)?;
                remove_dir_if_exists(&temp_output_dir).await?;
                let message = format!(
                    "hls session {session_id} timed out after {} ms",
                    self.guard.timeout().as_millis()
                );
                manager.mark_failed(session_id, message.clone())?;
                return Err(TaruError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message,
                });
            }
        };
        let stderr = join_stderr_task(stderr_task).await?;

        if status.success() {
            promote_temp_hls_output(&temp_output_dir, &output_dir).await?;
            manager.mark_finished(session_id)?;
            Ok(HlsRunOutcome::Finished {
                session_id,
                playlist_path: session.output_path,
            })
        } else {
            remove_dir_if_exists(&temp_output_dir).await?;
            let message = stderr_message(&stderr);
            manager.mark_failed(session_id, message.clone())?;
            Err(TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message,
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HlsRunOutcome {
    Finished {
        session_id: TranscodeSessionId,
        playlist_path: PathBuf,
    },
    Cancelled {
        session_id: TranscodeSessionId,
        temp_output_dir: PathBuf,
    },
}

fn temporary_hls_output_dir(output_dir: &Path, session_id: TranscodeSessionId) -> PathBuf {
    let dir_name = output_dir
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "hls-output".to_owned());

    output_dir.with_file_name(format!("{dir_name}.{session_id}.tmp"))
}
