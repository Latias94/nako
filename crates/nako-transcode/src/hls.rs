use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};

use super::{
    engine::{
        TranscodeEngineAdapter, TranscodeEngineAdapterKind, TranscodeEngineArtifactKind,
        TranscodeEngineStartCommand, TranscodeEngineStartOutcome,
    },
    ffmpeg::stderr_message,
    progress::parse_ffmpeg_progress_report,
    runner_util::{
        abort_stderr_task, abort_stdout_task, command_with_hls_output_dir,
        ffmpeg_command_with_progress, join_stderr_task, join_stdout_task, kill_child,
        promote_temp_hls_output, read_child_stderr, read_child_stdout, remove_dir_if_exists,
    },
    runtime::{CancellationToken, TranscodeRuntimeGuard},
    session::{TranscodeSessionId, TranscodeSessionKind, TranscodeSessionManager},
};

#[derive(Clone, Debug)]
pub struct FfmpegHlsRunner {
    guard: TranscodeRuntimeGuard,
}

impl TranscodeEngineAdapter for FfmpegHlsRunner {
    fn adapter_kind(&self) -> TranscodeEngineAdapterKind {
        TranscodeEngineAdapterKind::FfmpegCli
    }

    async fn start(
        &self,
        manager: &mut TranscodeSessionManager,
        command: TranscodeEngineStartCommand,
    ) -> Result<TranscodeEngineStartOutcome> {
        self.run(manager, command.session_id, command.cancel)
            .await
            .map(TranscodeEngineStartOutcome::from)
    }
}

impl FfmpegHlsRunner {
    #[must_use]
    pub fn new(guard: TranscodeRuntimeGuard) -> Self {
        Self { guard }
    }

    pub(crate) async fn run(
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
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if session.kind != TranscodeSessionKind::HlsTranscode {
            let message = "ffmpeg hls runner only accepts hls transcode sessions".to_owned();
            let _ = manager.mark_failed(session_id, message.clone());
            return Err(NakoError::InvalidInput { message });
        }

        let output_dir = session
            .output_path
            .parent()
            .ok_or_else(|| NakoError::InvalidInput {
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
                NakoError::storage_io(
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
        let mut child = ffmpeg_command_with_progress(&command)
            .spawn()
            .map_err(|err| NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to start hls session {session_id}: {err}"),
            })?;

        manager.mark_running(session_id)?;

        let stderr_task = tokio::spawn(read_child_stderr(child.stderr.take()));
        let stdout_task = tokio::spawn(read_child_stdout(child.stdout.take()));
        let status = tokio::select! {
            status = child.wait() => {
                status.map_err(|err| NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("failed to wait for hls session {session_id}: {err}"),
                })?
            }
            () = cancel.cancelled() => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                abort_stdout_task(stdout_task);
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
                abort_stdout_task(stdout_task);
                manager.request_cancel(session_id)?;
                remove_dir_if_exists(&temp_output_dir).await?;
                let message = format!(
                    "hls session {session_id} timed out after {} ms",
                    self.guard.timeout().as_millis()
                );
                manager.mark_failed(session_id, message.clone())?;
                return Err(NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message,
                });
            }
        };
        let stderr = join_stderr_task(stderr_task).await?;
        let stdout = join_stdout_task(stdout_task).await?;
        let metrics = parse_ffmpeg_progress_report(&stdout);
        if !metrics.is_empty() {
            manager.update_runtime_metrics(session_id, metrics)?;
        }

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
            Err(NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message,
            })
        }
    }
}

impl From<HlsRunOutcome> for TranscodeEngineStartOutcome {
    fn from(outcome: HlsRunOutcome) -> Self {
        match outcome {
            HlsRunOutcome::Finished {
                session_id,
                playlist_path,
            } => Self::Finished {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::HlsPlaylist,
                output_path: playlist_path,
            },
            HlsRunOutcome::Cancelled {
                session_id,
                temp_output_dir,
            } => Self::Cancelled {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::HlsPlaylist,
                temporary_output_path: temp_output_dir,
            },
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
