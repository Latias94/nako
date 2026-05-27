use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};

use super::{
    engine::{
        TranscodeEngineAdapter, TranscodeEngineAdapterKind, TranscodeEngineArtifactKind,
        TranscodeEngineStartCommand, TranscodeEngineStartOutcome,
    },
    ffmpeg::stderr_message,
    runner_util::{
        abort_stderr_task, command_with_output_path, ffmpeg_command, join_stderr_task, kill_child,
        promote_temp_output, read_child_stderr, remove_file_if_exists,
    },
    runtime::{CancellationToken, TranscodeRuntimeGuard},
    session::{TranscodeSessionId, TranscodeSessionKind, TranscodeSessionManager},
};

#[derive(Clone, Debug)]
pub struct FfmpegRemuxRunner {
    guard: TranscodeRuntimeGuard,
}

impl TranscodeEngineAdapter for FfmpegRemuxRunner {
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

impl FfmpegRemuxRunner {
    #[must_use]
    pub fn new(guard: TranscodeRuntimeGuard) -> Self {
        Self { guard }
    }

    pub(crate) async fn run(
        &self,
        manager: &mut TranscodeSessionManager,
        session_id: TranscodeSessionId,
        cancel: CancellationToken,
    ) -> Result<RemuxRunOutcome> {
        let _permit = self.guard.acquire().await?;
        manager.mark_starting(session_id)?;

        let session = manager
            .get(session_id)
            .cloned()
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if session.kind != TranscodeSessionKind::Remux {
            let message = "ffmpeg remux runner only accepts remux sessions".to_owned();
            let _ = manager.mark_failed(session_id, message.clone());
            return Err(NakoError::InvalidInput { message });
        }

        let temp_output = temporary_output_path(&session.output_path, session.id);
        if temp_output.exists() {
            remove_file_if_exists(&temp_output).await?;
        }

        let command =
            command_with_output_path(&session.command, &session.output_path, &temp_output)?;
        let mut child = ffmpeg_command(&command)
            .spawn()
            .map_err(|err| NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to start remux session {session_id}: {err}"),
            })?;

        manager.mark_running(session_id)?;

        let stderr_task = tokio::spawn(read_child_stderr(child.stderr.take()));
        let status = tokio::select! {
            status = child.wait() => {
                status.map_err(|err| NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("failed to wait for remux session {session_id}: {err}"),
                })?
            }
            () = cancel.cancelled() => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                manager.request_cancel(session_id)?;
                remove_file_if_exists(&temp_output).await?;
                manager.mark_cancelled(session_id)?;
                return Ok(RemuxRunOutcome::Cancelled {
                    session_id,
                    temp_output,
                });
            }
            () = tokio::time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                manager.request_cancel(session_id)?;
                remove_file_if_exists(&temp_output).await?;
                let message = format!(
                    "remux session {session_id} timed out after {} ms",
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

        if status.success() {
            promote_temp_output(&temp_output, &session.output_path).await?;
            manager.mark_finished(session_id)?;
            Ok(RemuxRunOutcome::Finished {
                session_id,
                output_path: session.output_path,
            })
        } else {
            remove_file_if_exists(&temp_output).await?;
            let message = stderr_message(&stderr);
            manager.mark_failed(session_id, message.clone())?;
            Err(NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message,
            })
        }
    }
}

impl From<RemuxRunOutcome> for TranscodeEngineStartOutcome {
    fn from(outcome: RemuxRunOutcome) -> Self {
        match outcome {
            RemuxRunOutcome::Finished {
                session_id,
                output_path,
            } => Self::Finished {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::RemuxFile,
                output_path,
            },
            RemuxRunOutcome::Cancelled {
                session_id,
                temp_output,
            } => Self::Cancelled {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::RemuxFile,
                temporary_output_path: temp_output,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemuxRunOutcome {
    Finished {
        session_id: TranscodeSessionId,
        output_path: PathBuf,
    },
    Cancelled {
        session_id: TranscodeSessionId,
        temp_output: PathBuf,
    },
}

fn temporary_output_path(output_path: &Path, session_id: TranscodeSessionId) -> PathBuf {
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty());
    let mut file_name = output_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "remux-output".to_owned());
    file_name.push_str(&format!(".{}.tmp", session_id));
    if let Some(extension) = extension {
        file_name.push('.');
        file_name.push_str(extension);
    }

    output_path.with_file_name(file_name)
}
