use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FfmpegCommandPlan {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<FfmpegArg>,
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

    #[cfg(test)]
    #[must_use]
    pub fn argv_lossy(&self) -> Vec<String> {
        let mut argv = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.program.display().to_string());
        argv.extend(self.args.iter().map(FfmpegArg::to_string_lossy));
        argv
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub(crate) enum FfmpegArg {
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

    #[cfg(test)]
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
pub(crate) enum FfmpegOverwritePolicy {
    Allow,
    #[default]
    Never,
}

pub(in crate::ffmpeg) fn overwrite_arg(policy: FfmpegOverwritePolicy) -> &'static str {
    match policy {
        FfmpegOverwritePolicy::Allow => "-y",
        FfmpegOverwritePolicy::Never => "-n",
    }
}

pub(in crate::ffmpeg) fn command_plan(
    ffmpeg_path: &Path,
    args: Vec<FfmpegArg>,
) -> FfmpegCommandPlan {
    FfmpegCommandPlan::new(ffmpeg_path.to_path_buf(), args)
}
