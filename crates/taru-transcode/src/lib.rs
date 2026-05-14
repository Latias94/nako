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
pub use taru_core::{TranscodeSessionId, TranscodeSessionKind, TranscodeSessionState};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{OwnedSemaphorePermit, Semaphore},
    time,
};

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub playlist_path: PathBuf,
    pub segment_pattern: PathBuf,
    pub segment_time_seconds: u32,
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

    pub fn hls(&self, request: &HlsRequest) -> Result<FfmpegCommandPlan> {
        if request.input_path.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "hls input path cannot be empty".to_owned(),
            });
        }

        if request.output_dir.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "hls output directory cannot be empty".to_owned(),
            });
        }

        if request.playlist_path.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "hls playlist path cannot be empty".to_owned(),
            });
        }

        if request.segment_pattern.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "hls segment pattern cannot be empty".to_owned(),
            });
        }

        if !request.playlist_path.starts_with(&request.output_dir) {
            return Err(TaruError::InvalidInput {
                message: "hls playlist path must be inside the output directory".to_owned(),
            });
        }

        if !request.segment_pattern.starts_with(&request.output_dir) {
            return Err(TaruError::InvalidInput {
                message: "hls segment pattern must be inside the output directory".to_owned(),
            });
        }

        if !request
            .segment_pattern
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.contains('%'))
        {
            return Err(TaruError::InvalidInput {
                message: "hls segment pattern must contain a printf-style segment placeholder"
                    .to_owned(),
            });
        }

        if request.input_path == request.playlist_path {
            return Err(TaruError::InvalidInput {
                message: "hls input and playlist paths must differ".to_owned(),
            });
        }

        let overwrite_arg = match request.overwrite {
            FfmpegOverwritePolicy::Allow => "-y",
            FfmpegOverwritePolicy::Never => "-n",
        };
        let segment_time = request.segment_time_seconds.max(1).to_string();

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
                FfmpegArg::raw("0:v:0"),
                FfmpegArg::raw("-map"),
                FfmpegArg::raw("0:a:0?"),
                FfmpegArg::raw("-c:v"),
                FfmpegArg::raw("libx264"),
                FfmpegArg::raw("-c:a"),
                FfmpegArg::raw("aac"),
                FfmpegArg::raw("-f"),
                FfmpegArg::raw("hls"),
                FfmpegArg::raw("-hls_time"),
                FfmpegArg::raw(segment_time),
                FfmpegArg::raw("-hls_playlist_type"),
                FfmpegArg::raw("vod"),
                FfmpegArg::raw("-hls_segment_filename"),
                FfmpegArg::path(request.segment_pattern.clone()),
                FfmpegArg::path(request.playlist_path.clone()),
            ],
        ))
    }
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
        self.plan_remux_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_remux_with_id(
        &mut self,
        session_id: TranscodeSessionId,
        request: RemuxRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        let command = builder.remux(&request)?;
        let session = TranscodeSession {
            id: session_id,
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

    pub fn plan_hls(
        &mut self,
        request: HlsRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        self.plan_hls_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_hls_with_id(
        &mut self,
        session_id: TranscodeSessionId,
        request: HlsRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        let command = builder.hls(&request)?;
        let session = TranscodeSession {
            id: session_id,
            source_id: request.source_id,
            kind: TranscodeSessionKind::HlsTranscode,
            state: TranscodeSessionState::Planned,
            command,
            output_path: request.playlist_path,
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
            .map_err(|err| TaruError::Storage {
                uri: temp_output_dir.display().to_string(),
                message: format!("failed to create temporary hls output directory: {err}"),
            })?;

        let command = command_with_hls_output_dir(
            &session.command,
            &output_dir,
            &temp_output_dir,
            &session.output_path,
        )?;
        let mut child = Command::new(&command.program)
            .args(command.args_as_os_strings())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
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
                let _ = join_stderr_task(stderr_task).await?;
                manager.request_cancel(session_id)?;
                remove_dir_if_exists(&temp_output_dir).await?;
                manager.mark_cancelled(session_id)?;
                return Ok(HlsRunOutcome::Cancelled {
                    session_id,
                    temp_output_dir,
                });
            }
            () = time::sleep(self.guard.timeout()) => {
                kill_child(&mut child).await?;
                let _ = join_stderr_task(stderr_task).await?;
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

fn temporary_hls_output_dir(output_dir: &Path, session_id: TranscodeSessionId) -> PathBuf {
    let dir_name = output_dir
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "hls-output".to_owned());

    output_dir.with_file_name(format!("{dir_name}.{session_id}.tmp"))
}

fn command_with_hls_output_dir(
    command: &FfmpegCommandPlan,
    output_dir: &Path,
    temp_output_dir: &Path,
    playlist_path: &Path,
) -> Result<FfmpegCommandPlan> {
    let mut rewritten = command.clone();
    let mut rewrote_playlist = false;
    let mut rewrote_segment_pattern = false;

    for arg in &mut rewritten.args {
        let FfmpegArg::Path(path) = arg else {
            continue;
        };

        if path == playlist_path {
            let relative =
                path.strip_prefix(output_dir)
                    .map_err(|err| TaruError::InvalidInput {
                        message: format!("hls playlist path is not inside output directory: {err}"),
                    })?;
            *path = temp_output_dir.join(relative);
            rewrote_playlist = true;
            continue;
        }

        if path.starts_with(output_dir) {
            let relative =
                path.strip_prefix(output_dir)
                    .map_err(|err| TaruError::InvalidInput {
                        message: format!("hls output path is not inside output directory: {err}"),
                    })?;
            *path = temp_output_dir.join(relative);
            rewrote_segment_pattern = true;
        }
    }

    if !rewrote_playlist {
        return Err(TaruError::InvalidInput {
            message: format!(
                "hls command plan does not contain expected playlist path: {}",
                playlist_path.display()
            ),
        });
    }

    if !rewrote_segment_pattern {
        return Err(TaruError::InvalidInput {
            message: format!(
                "hls command plan does not contain output paths under {}",
                output_dir.display()
            ),
        });
    }

    Ok(rewritten)
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

async fn promote_temp_hls_output(temp_output_dir: &Path, final_output_dir: &Path) -> Result<()> {
    if let Some(parent) = final_output_dir.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| TaruError::Storage {
                uri: final_output_dir.display().to_string(),
                message: format!("failed to create hls output directory: {err}"),
            })?;
    }

    remove_dir_if_exists(final_output_dir).await?;

    tokio::fs::rename(temp_output_dir, final_output_dir)
        .await
        .map_err(|err| TaruError::Storage {
            uri: final_output_dir.display().to_string(),
            message: format!("failed to promote hls output directory: {err}"),
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

async fn remove_dir_if_exists(path: &Path) -> Result<()> {
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(TaruError::Storage {
            uri: path.display().to_string(),
            message: format!("failed to remove temporary hls output directory: {err}"),
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
    fn ffmpeg_builder_plans_hls_single_variant() {
        let builder = FfmpegCommandBuilder::new("ffmpeg");
        let request = HlsRequest {
            source_id: MediaSourceId::new(),
            input_path: PathBuf::from("input.mkv"),
            output_dir: PathBuf::from("hls"),
            playlist_path: PathBuf::from("hls/playlist.m3u8"),
            segment_pattern: PathBuf::from("hls/segment_%05d.ts"),
            segment_time_seconds: 6,
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
            overwrite: FfmpegOverwritePolicy::Allow,
        };

        assert!(builder.hls(&request).is_err());
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
