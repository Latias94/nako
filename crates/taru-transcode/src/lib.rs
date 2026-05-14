use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use serde::{Deserialize, Serialize};
use taru_core::{MediaSourceId, Result, TaruError};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{OwnedSemaphorePermit, Semaphore},
    time,
};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePlan {
    pub input_locator: String,
    pub output_container: OutputContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub hardware_acceleration: HardwareAcceleration,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputContainer {
    Hls,
    Mp4,
    Mkv,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAcceleration {
    #[default]
    None,
    Vaapi,
    Nvenc,
    QuickSync,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FfmpegCommandPlan {
    pub program: PathBuf,
    pub args: Vec<FfmpegArg>,
}

impl FfmpegCommandPlan {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>, args: Vec<FfmpegArg>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    #[must_use]
    pub fn args_as_os_strings(&self) -> Vec<OsString> {
        self.args.iter().map(FfmpegArg::to_os_string).collect()
    }

    #[must_use]
    pub fn argv_lossy(&self) -> Vec<String> {
        let mut argv = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.program.display().to_string());
        argv.extend(self.args.iter().map(FfmpegArg::to_string_lossy));
        argv
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxRuntimeLimits {
    pub max_concurrent_sessions: usize,
    pub timeout_ms: u64,
}

impl Default for RemuxRuntimeLimits {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 1,
            timeout_ms: 30 * 60 * 1_000,
        }
    }
}

impl RemuxRuntimeLimits {
    #[must_use]
    pub fn max_concurrent_sessions(self) -> usize {
        self.max_concurrent_sessions.max(1)
    }

    #[must_use]
    pub fn timeout(self) -> Duration {
        Duration::from_millis(self.timeout_ms.max(1))
    }
}

#[derive(Clone, Debug)]
pub struct RemuxRuntimeGuard {
    semaphore: Arc<Semaphore>,
    timeout: Duration,
}

impl RemuxRuntimeGuard {
    #[must_use]
    pub fn new(limits: RemuxRuntimeLimits) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(limits.max_concurrent_sessions())),
            timeout: limits.timeout(),
        }
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn acquire(&self) -> Result<RemuxRuntimePermit> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("remux runtime guard closed: {err}"),
            })?;

        Ok(RemuxRuntimePermit { permit })
    }
}

#[derive(Debug)]
pub struct RemuxRuntimePermit {
    #[allow(dead_code)]
    permit: OwnedSemaphorePermit,
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        loop {
            if self.is_cancelled() {
                return;
            }

            time::sleep(Duration::from_millis(10)).await;
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum FfmpegArg {
    Raw(String),
    Path(PathBuf),
}

impl FfmpegArg {
    #[must_use]
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    #[must_use]
    pub fn path(value: impl Into<PathBuf>) -> Self {
        Self::Path(value.into())
    }

    #[must_use]
    pub fn to_os_string(&self) -> OsString {
        match self {
            Self::Raw(value) => OsString::from(value),
            Self::Path(value) => value.as_os_str().to_os_string(),
        }
    }

    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        match self {
            Self::Raw(value) => value.clone(),
            Self::Path(value) => value.display().to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegOverwritePolicy {
    Allow,
    #[default]
    Never,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub output_container: RemuxContainer,
    pub overwrite: FfmpegOverwritePolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemuxContainer {
    Mp4,
    Mkv,
}

impl RemuxContainer {
    #[must_use]
    pub const fn ffmpeg_format(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "matroska",
        }
    }

    #[must_use]
    pub const fn file_extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FfmpegCommandBuilder {
    ffmpeg_path: PathBuf,
}

impl Default for FfmpegCommandBuilder {
    fn default() -> Self {
        Self::new("ffmpeg")
    }
}

impl FfmpegCommandBuilder {
    #[must_use]
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
        }
    }

    #[must_use]
    pub fn ffmpeg_path(&self) -> &Path {
        &self.ffmpeg_path
    }

    pub fn remux(&self, request: &RemuxRequest) -> Result<FfmpegCommandPlan> {
        if request.input_path.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "remux input path cannot be empty".to_owned(),
            });
        }

        if request.output_path.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "remux output path cannot be empty".to_owned(),
            });
        }

        if request.input_path == request.output_path {
            return Err(TaruError::InvalidInput {
                message: "remux input and output paths must differ".to_owned(),
            });
        }

        let overwrite_arg = match request.overwrite {
            FfmpegOverwritePolicy::Allow => "-y",
            FfmpegOverwritePolicy::Never => "-n",
        };

        Ok(FfmpegCommandPlan::new(
            self.ffmpeg_path.clone(),
            vec![
                FfmpegArg::raw("-hide_banner"),
                FfmpegArg::raw("-loglevel"),
                FfmpegArg::raw("warning"),
                FfmpegArg::raw(overwrite_arg),
                FfmpegArg::raw("-i"),
                FfmpegArg::path(request.input_path.clone()),
                FfmpegArg::raw("-map"),
                FfmpegArg::raw("0"),
                FfmpegArg::raw("-c"),
                FfmpegArg::raw("copy"),
                FfmpegArg::raw("-f"),
                FfmpegArg::raw(request.output_container.ffmpeg_format()),
                FfmpegArg::path(request.output_path.clone()),
            ],
        ))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TranscodeSessionId(Uuid);

impl TranscodeSessionId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for TranscodeSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TranscodeSessionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSessionKind {
    Remux,
    HlsTranscode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeSessionState {
    Planned,
    Starting,
    Running,
    CancelRequested,
    Cancelled,
    Failed,
    Finished,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSession {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub state: TranscodeSessionState,
    pub command: FfmpegCommandPlan,
    pub output_path: PathBuf,
    pub failure_message: Option<String>,
}

#[derive(Debug, Default)]
pub struct TranscodeSessionManager {
    sessions: HashMap<TranscodeSessionId, TranscodeSession>,
}

impl TranscodeSessionManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plan_remux(
        &mut self,
        request: RemuxRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        let command = builder.remux(&request)?;
        let session = TranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: request.source_id,
            kind: TranscodeSessionKind::Remux,
            state: TranscodeSessionState::Planned,
            command,
            output_path: request.output_path,
            failure_message: None,
        };

        self.sessions.insert(session.id, session.clone());
        Ok(session)
    }

    pub fn mark_starting(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::Planned],
            TranscodeSessionState::Starting,
            None,
        )
    }

    pub fn mark_running(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Planned,
                TranscodeSessionState::Starting,
            ],
            TranscodeSessionState::Running,
            None,
        )
    }

    pub fn request_cancel(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Planned,
                TranscodeSessionState::Starting,
                TranscodeSessionState::Running,
            ],
            TranscodeSessionState::CancelRequested,
            None,
        )
    }

    pub fn mark_cancelled(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::CancelRequested],
            TranscodeSessionState::Cancelled,
            None,
        )
    }

    pub fn mark_finished(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::Running],
            TranscodeSessionState::Finished,
            None,
        )
    }

    pub fn mark_failed(
        &mut self,
        session_id: TranscodeSessionId,
        message: impl Into<String>,
    ) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Starting,
                TranscodeSessionState::Running,
                TranscodeSessionState::CancelRequested,
            ],
            TranscodeSessionState::Failed,
            Some(message.into()),
        )
    }

    #[must_use]
    pub fn get(&self, session_id: TranscodeSessionId) -> Option<&TranscodeSession> {
        self.sessions.get(&session_id)
    }

    fn transition(
        &mut self,
        session_id: TranscodeSessionId,
        allowed_from: &[TranscodeSessionState],
        target: TranscodeSessionState,
        failure_message: Option<String>,
    ) -> Result<TranscodeSession> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if !allowed_from.contains(&session.state) {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "cannot transition transcode session {session_id} from {:?} to {:?}",
                    session.state, target
                ),
            });
        }

        session.state = target;
        session.failure_message = failure_message;
        Ok(session.clone())
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegRemuxRunner {
    guard: RemuxRuntimeGuard,
}

impl FfmpegRemuxRunner {
    #[must_use]
    pub fn new(guard: RemuxRuntimeGuard) -> Self {
        Self { guard }
    }

    pub async fn run(
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
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if session.kind != TranscodeSessionKind::Remux {
            let message = "ffmpeg remux runner only accepts remux sessions".to_owned();
            let _ = manager.mark_failed(session_id, message.clone());
            return Err(TaruError::InvalidInput { message });
        }

        let temp_output = temporary_output_path(&session.output_path, session.id);
        if temp_output.exists() {
            remove_file_if_exists(&temp_output).await?;
        }

        let command =
            command_with_output_path(&session.command, &session.output_path, &temp_output)?;
        let mut child = Command::new(&command.program)
            .args(command.args_as_os_strings())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|err| TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("failed to start remux session {session_id}: {err}"),
            })?;

        manager.mark_running(session_id)?;

        let stderr_task = tokio::spawn(read_child_stderr(child.stderr.take()));
        let status = tokio::select! {
            status = child.wait() => {
                status.map_err(|err| TaruError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("failed to wait for remux session {session_id}: {err}"),
                })?
            }
            () = cancel.cancelled() => {
                kill_child(&mut child).await?;
                let _ = join_stderr_task(stderr_task).await?;
                manager.request_cancel(session_id)?;
                remove_file_if_exists(&temp_output).await?;
                manager.mark_cancelled(session_id)?;
                return Ok(RemuxRunOutcome::Cancelled {
                    session_id,
                    temp_output,
                });
            }
            () = time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                let _ = join_stderr_task(stderr_task).await?;
                manager.request_cancel(session_id)?;
                remove_file_if_exists(&temp_output).await?;
                let message = format!(
                    "remux session {session_id} timed out after {} ms",
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
            Err(TaruError::Provider {
                provider: "ffmpeg".to_owned(),
                message,
            })
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

fn command_with_output_path(
    command: &FfmpegCommandPlan,
    output_path: &Path,
    temp_output: &Path,
) -> Result<FfmpegCommandPlan> {
    let mut rewritten = command.clone();

    for arg in rewritten.args.iter_mut().rev() {
        if *arg == FfmpegArg::path(output_path) {
            *arg = FfmpegArg::path(temp_output);
            return Ok(rewritten);
        }
    }

    Err(TaruError::InvalidInput {
        message: format!(
            "remux command plan does not contain expected output path: {}",
            output_path.display()
        ),
    })
}

async fn promote_temp_output(temp_output: &Path, final_output: &Path) -> Result<()> {
    if let Some(parent) = final_output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| TaruError::Storage {
                uri: final_output.display().to_string(),
                message: format!("failed to create remux output directory: {err}"),
            })?;
    }

    if final_output.exists() {
        remove_file_if_exists(final_output).await?;
    }

    tokio::fs::rename(temp_output, final_output)
        .await
        .map_err(|err| TaruError::Storage {
            uri: final_output.display().to_string(),
            message: format!("failed to promote remux output: {err}"),
        })
}

async fn remove_file_if_exists(path: &Path) -> Result<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(TaruError::Storage {
            uri: path.display().to_string(),
            message: format!("failed to remove temporary remux output: {err}"),
        }),
    }
}

async fn kill_child(child: &mut tokio::process::Child) -> Result<()> {
    match child.kill().await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
        Err(err) => Err(TaruError::Provider {
            provider: "ffmpeg".to_owned(),
            message: format!("failed to kill ffmpeg process: {err}"),
        }),
    }
}

async fn read_child_stderr(stderr: Option<tokio::process::ChildStderr>) -> Result<Vec<u8>> {
    let Some(mut stderr) = stderr else {
        return Ok(Vec::new());
    };

    let mut bytes = Vec::new();
    stderr
        .read_to_end(&mut bytes)
        .await
        .map_err(|err| TaruError::Provider {
            provider: "ffmpeg".to_owned(),
            message: format!("failed to read ffmpeg stderr: {err}"),
        })?;

    Ok(bytes)
}

async fn join_stderr_task(task: tokio::task::JoinHandle<Result<Vec<u8>>>) -> Result<Vec<u8>> {
    task.await.map_err(|err| TaruError::Provider {
        provider: "ffmpeg".to_owned(),
        message: format!("failed to join ffmpeg stderr reader: {err}"),
    })?
}

fn stderr_message(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_owned();

    if message.is_empty() {
        "ffmpeg remux process failed".to_owned()
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::Duration};

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

        let outcome = runner.run(&mut manager, session.id, cancel).await.unwrap();

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
}
