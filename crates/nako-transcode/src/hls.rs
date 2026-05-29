use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};

use super::{
    TranscodeSessionId, TranscodeSessionKind,
    engine::{
        TranscodeEngineAdapter, TranscodeEngineAdapterKind, TranscodeEngineArtifactKind,
        TranscodeEngineStartCommand, TranscodeEngineStartOutcome,
    },
    execution::TranscodeExecutionRequest,
    ffmpeg::stderr_message,
    progress::parse_ffmpeg_progress_report,
    runner_util::{
        abort_stderr_task, abort_stdout_task, command_with_hls_output_dir,
        ffmpeg_command_with_progress, join_stderr_task, join_stdout_task, kill_child,
        promote_temp_hls_output, read_child_stderr, read_child_stdout, remove_dir_if_exists,
    },
    runtime::{CancellationToken, TranscodeRuntimeGuard},
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
        command: TranscodeEngineStartCommand,
    ) -> Result<TranscodeEngineStartOutcome> {
        self.run(command.execution, command.cancel)
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
        execution: TranscodeExecutionRequest,
        cancel: CancellationToken,
    ) -> Result<HlsRunOutcome> {
        let _permit = self.guard.acquire().await?;

        if execution.kind != TranscodeSessionKind::HlsTranscode {
            let message = "ffmpeg hls runner only accepts hls transcode sessions".to_owned();
            return Err(NakoError::InvalidInput { message });
        }

        let output_dir = execution
            .output_path
            .parent()
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!(
                    "hls session {} playlist path does not have a parent directory",
                    execution.session_id
                ),
            })?
            .to_path_buf();
        let temp_output_dir = temporary_hls_output_dir(&output_dir, execution.session_id);

        remove_dir_if_exists(&temp_output_dir).await?;
        tokio::fs::create_dir_all(&temp_output_dir)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    temp_output_dir.display().to_string(),
                    format!("failed to create temporary hls output directory: {err}"),
                )
            })?;

        let command =
            command_with_hls_output_dir(&execution.command, &output_dir, &temp_output_dir)?;
        let mut child = ffmpeg_command_with_progress(&command)
            .spawn()
            .map_err(|err| NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!(
                    "failed to start hls session {}: {err}",
                    execution.session_id
                ),
            })?;

        let stderr_task = tokio::spawn(read_child_stderr(child.stderr.take()));
        let stdout_task = tokio::spawn(read_child_stdout(child.stdout.take()));
        let status = tokio::select! {
            status = child.wait() => {
                status.map_err(|err| NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("failed to wait for hls session {}: {err}", execution.session_id),
                })?
            }
            () = cancel.cancelled() => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                abort_stdout_task(stdout_task);
                remove_dir_if_exists(&temp_output_dir).await?;
                return Ok(HlsRunOutcome::Cancelled {
                    session_id: execution.session_id,
                    temp_output_dir,
                });
            }
            () = tokio::time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                abort_stdout_task(stdout_task);
                remove_dir_if_exists(&temp_output_dir).await?;
                let message = format!(
                    "hls session {} timed out after {} ms",
                    execution.session_id,
                    self.guard.timeout().as_millis()
                );
                return Err(NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message,
                });
            }
        };
        let stderr = join_stderr_task(stderr_task).await?;
        let stdout = join_stdout_task(stdout_task).await?;
        let metrics = parse_ffmpeg_progress_report(&stdout);

        if status.success() {
            promote_temp_hls_output(&temp_output_dir, &output_dir).await?;
            Ok(HlsRunOutcome::Finished {
                session_id: execution.session_id,
                playlist_path: execution.output_path,
                runtime_metrics: metrics,
            })
        } else {
            remove_dir_if_exists(&temp_output_dir).await?;
            let message = stderr_message(&stderr);
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
                runtime_metrics,
            } => Self::Finished {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::HlsPlaylist,
                output_path: playlist_path,
                runtime_metrics,
            },
            HlsRunOutcome::Cancelled {
                session_id,
                temp_output_dir,
            } => Self::Cancelled {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::HlsPlaylist,
                temporary_output_path: temp_output_dir,
                runtime_metrics: Default::default(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HlsRunOutcome {
    Finished {
        session_id: TranscodeSessionId,
        playlist_path: PathBuf,
        runtime_metrics: nako_core::TranscodeSessionRuntimeMetrics,
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
