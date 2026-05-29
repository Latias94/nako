use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};

use super::{
    TranscodeSessionId, TranscodeSessionKind,
    engine::{
        TranscodeEngineAdapter, TranscodeEngineAdapterKind, TranscodeEngineArtifactKind,
        TranscodeEngineStartCommand, TranscodeEngineStartOutcome,
    },
    execution::TranscodeExecutionRequest,
    ffmpeg::{FfmpegCommandPlan, stderr_message},
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
    output_publication_policy: HlsOutputPublicationPolicy,
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
        Self::new_with_output_publication_policy(
            guard,
            HlsOutputPublicationPolicy::AtomicOnCompletion,
        )
    }

    #[must_use]
    pub fn new_with_output_publication_policy(
        guard: TranscodeRuntimeGuard,
        output_publication_policy: HlsOutputPublicationPolicy,
    ) -> Self {
        Self {
            guard,
            output_publication_policy,
        }
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
        let output_publication = prepare_hls_output_publication(
            self.output_publication_policy,
            &execution.command,
            &output_dir,
            execution.session_id,
        )
        .await?;
        let mut child = match ffmpeg_command_with_progress(output_publication.command()).spawn() {
            Ok(child) => child,
            Err(err) => {
                output_publication.cleanup_after_abort().await?;
                return Err(NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!(
                        "failed to start hls session {}: {err}",
                        execution.session_id
                    ),
                });
            }
        };

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
                output_publication.cleanup_after_abort().await?;
                return Ok(HlsRunOutcome::Cancelled {
                    session_id: execution.session_id,
                    discarded_output_dir: output_publication.discarded_output_dir().to_path_buf(),
                });
            }
            () = tokio::time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                abort_stderr_task(stderr_task);
                abort_stdout_task(stdout_task);
                output_publication.cleanup_after_abort().await?;
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
            output_publication.complete().await?;
            Ok(HlsRunOutcome::Finished {
                session_id: execution.session_id,
                playlist_path: execution.output_path,
                runtime_metrics: metrics,
            })
        } else {
            output_publication.cleanup_after_abort().await?;
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
                discarded_output_dir,
            } => Self::Cancelled {
                session_id,
                artifact_kind: TranscodeEngineArtifactKind::HlsPlaylist,
                temporary_output_path: discarded_output_dir,
                runtime_metrics: Default::default(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HlsOutputPublicationPolicy {
    #[default]
    AtomicOnCompletion,
    ServeWhileRunning,
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
        discarded_output_dir: PathBuf,
    },
}

#[derive(Clone, Debug)]
struct HlsOutputPublication {
    command: FfmpegCommandPlan,
    discarded_output_dir: PathBuf,
    completion: HlsOutputPublicationCompletion,
}

impl HlsOutputPublication {
    fn command(&self) -> &FfmpegCommandPlan {
        &self.command
    }

    fn discarded_output_dir(&self) -> &Path {
        &self.discarded_output_dir
    }

    async fn cleanup_after_abort(&self) -> Result<()> {
        remove_dir_if_exists(&self.discarded_output_dir).await
    }

    async fn complete(&self) -> Result<()> {
        match &self.completion {
            HlsOutputPublicationCompletion::PromoteTempDir {
                temp_output_dir,
                final_output_dir,
            } => promote_temp_hls_output(temp_output_dir, final_output_dir).await,
            HlsOutputPublicationCompletion::AlreadyVisible => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
enum HlsOutputPublicationCompletion {
    PromoteTempDir {
        temp_output_dir: PathBuf,
        final_output_dir: PathBuf,
    },
    AlreadyVisible,
}

async fn prepare_hls_output_publication(
    policy: HlsOutputPublicationPolicy,
    command: &FfmpegCommandPlan,
    output_dir: &Path,
    session_id: TranscodeSessionId,
) -> Result<HlsOutputPublication> {
    match policy {
        HlsOutputPublicationPolicy::AtomicOnCompletion => {
            let temp_output_dir = temporary_hls_output_dir(output_dir, session_id);
            let command = command_with_hls_output_dir(command, output_dir, &temp_output_dir)?;
            remove_dir_if_exists(&temp_output_dir).await?;
            create_hls_output_dir(&temp_output_dir, "temporary hls output directory").await?;
            Ok(HlsOutputPublication {
                command,
                discarded_output_dir: temp_output_dir.clone(),
                completion: HlsOutputPublicationCompletion::PromoteTempDir {
                    temp_output_dir,
                    final_output_dir: output_dir.to_path_buf(),
                },
            })
        }
        HlsOutputPublicationPolicy::ServeWhileRunning => {
            let command = command_with_hls_output_dir(command, output_dir, output_dir)?;
            remove_dir_if_exists(output_dir).await?;
            create_hls_output_dir(output_dir, "serve-visible hls output directory").await?;
            Ok(HlsOutputPublication {
                command,
                discarded_output_dir: output_dir.to_path_buf(),
                completion: HlsOutputPublicationCompletion::AlreadyVisible,
            })
        }
    }
}

async fn create_hls_output_dir(path: &Path, description: &str) -> Result<()> {
    tokio::fs::create_dir_all(path).await.map_err(|err| {
        NakoError::storage_io(
            path.display().to_string(),
            format!("failed to create {description}: {err}"),
        )
    })
}

fn temporary_hls_output_dir(output_dir: &Path, session_id: TranscodeSessionId) -> PathBuf {
    let dir_name = output_dir
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "hls-output".to_owned());

    output_dir.with_file_name(format!("{dir_name}.{session_id}.tmp"))
}
