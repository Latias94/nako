use std::{path::Path, process::Stdio};

use taru_core::{Result, TaruError};
use tokio::{io::AsyncReadExt, process::Command};

use super::ffmpeg::{FfmpegArg, FfmpegCommandPlan};

pub(crate) fn command_with_output_path(
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

pub(crate) fn command_with_hls_output_dir(
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

pub(crate) async fn promote_temp_output(temp_output: &Path, final_output: &Path) -> Result<()> {
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

pub(crate) async fn promote_temp_hls_output(
    temp_output_dir: &Path,
    final_output_dir: &Path,
) -> Result<()> {
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

pub(crate) async fn remove_file_if_exists(path: &Path) -> Result<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(TaruError::Storage {
            uri: path.display().to_string(),
            message: format!("failed to remove temporary remux output: {err}"),
        }),
    }
}

pub(crate) async fn remove_dir_if_exists(path: &Path) -> Result<()> {
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(TaruError::Storage {
            uri: path.display().to_string(),
            message: format!("failed to remove temporary hls output directory: {err}"),
        }),
    }
}

pub(crate) async fn kill_child(child: &mut tokio::process::Child) -> Result<()> {
    match child.kill().await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
        Err(err) => Err(TaruError::Provider {
            provider: "ffmpeg".to_owned(),
            message: format!("failed to kill ffmpeg process: {err}"),
        }),
    }
}

pub(crate) async fn read_child_stderr(
    stderr: Option<tokio::process::ChildStderr>,
) -> Result<Vec<u8>> {
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

pub(crate) async fn join_stderr_task(
    task: tokio::task::JoinHandle<Result<Vec<u8>>>,
) -> Result<Vec<u8>> {
    task.await.map_err(|err| TaruError::Provider {
        provider: "ffmpeg".to_owned(),
        message: format!("failed to join ffmpeg stderr reader: {err}"),
    })?
}

pub(crate) fn ffmpeg_command(command: &FfmpegCommandPlan) -> Command {
    let mut child = Command::new(&command.program);
    child
        .args(command.args_as_os_strings())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    child
}
