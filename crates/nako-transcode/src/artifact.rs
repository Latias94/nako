use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

use crate::policy::{HlsOutputRequirement, HlsVariantPolicy};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TranscodeArtifactSet {
    Remux { output_path: PathBuf },
    Hls { manifest: HlsArtifactManifest },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsArtifactManifest {
    pub output_dir: PathBuf,
    pub primary_playlist_path: PathBuf,
    pub media_segment_pattern: PathBuf,
    pub output: HlsOutputRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsArtifactDescriptor {
    pub path: PathBuf,
    pub content_type: &'static str,
    pub cleanup_candidate: bool,
}

impl HlsArtifactManifest {
    pub fn single_variant(
        output_dir: impl Into<PathBuf>,
        primary_playlist_path: impl Into<PathBuf>,
        media_segment_pattern: impl Into<PathBuf>,
        output: HlsOutputRequirement,
    ) -> Result<Self> {
        if output.variant_policy != HlsVariantPolicy::SingleVariant {
            return Err(NakoError::Unsupported(
                "adaptive hls artifact manifests are not implemented yet",
            ));
        }

        let manifest = Self {
            output_dir: output_dir.into(),
            primary_playlist_path: primary_playlist_path.into(),
            media_segment_pattern: media_segment_pattern.into(),
            output,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    #[must_use]
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    #[must_use]
    pub fn primary_playlist_path(&self) -> &Path {
        &self.primary_playlist_path
    }

    #[must_use]
    pub fn media_segment_pattern(&self) -> &Path {
        &self.media_segment_pattern
    }

    #[must_use]
    pub const fn output(&self) -> HlsOutputRequirement {
        self.output
    }

    #[must_use]
    pub fn init_segment_path(&self) -> Option<PathBuf> {
        self.output
            .segment_container
            .init_segment_file_name()
            .map(|file_name| self.output_dir.join(file_name))
    }

    pub fn artifact_for_name(&self, artifact_name: &str) -> Result<HlsArtifactDescriptor> {
        validate_hls_artifact_name(artifact_name)?;

        let path = self.output_dir.join(artifact_name);
        if !path.starts_with(&self.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls artifact path escaped the manifest directory".to_owned(),
            });
        }

        if self
            .primary_playlist_path
            .file_name()
            .and_then(|value| value.to_str())
            == Some(artifact_name)
        {
            return Ok(HlsArtifactDescriptor {
                path,
                content_type: "application/vnd.apple.mpegurl",
                cleanup_candidate: false,
            });
        }

        if self
            .output
            .segment_container
            .init_segment_file_name()
            .is_some_and(|file_name| file_name == artifact_name)
        {
            return Ok(HlsArtifactDescriptor {
                path,
                content_type: self.output.segment_container.segment_content_type(),
                cleanup_candidate: false,
            });
        }

        if Path::new(artifact_name)
            .extension()
            .and_then(|value| value.to_str())
            == Some(self.output.segment_container.segment_extension())
        {
            return Ok(HlsArtifactDescriptor {
                path,
                content_type: self.output.segment_container.segment_content_type(),
                cleanup_candidate: true,
            });
        }

        Err(NakoError::InvalidInput {
            message: "hls artifact is not part of the manifest".to_owned(),
        })
    }

    pub fn cleanup_candidate_for_name(&self, artifact_name: &str) -> bool {
        self.artifact_for_name(artifact_name)
            .is_ok_and(|artifact| artifact.cleanup_candidate)
    }

    pub fn validate(&self) -> Result<()> {
        if self.output_dir.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls output directory cannot be empty".to_owned(),
            });
        }

        if self.primary_playlist_path.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls playlist path cannot be empty".to_owned(),
            });
        }

        if self.media_segment_pattern.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern cannot be empty".to_owned(),
            });
        }

        if !self.primary_playlist_path.starts_with(&self.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls playlist path must be inside the output directory".to_owned(),
            });
        }

        if !self.media_segment_pattern.starts_with(&self.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern must be inside the output directory".to_owned(),
            });
        }

        if !self
            .media_segment_pattern
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.contains('%'))
        {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern must contain a printf-style segment placeholder"
                    .to_owned(),
            });
        }

        if self
            .media_segment_pattern
            .extension()
            .and_then(|value| value.to_str())
            != Some(self.output.segment_container.segment_extension())
        {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern extension must match the requested segment container"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

impl TranscodeArtifactSet {
    #[must_use]
    pub fn remux(output_path: impl Into<PathBuf>) -> Self {
        Self::Remux {
            output_path: output_path.into(),
        }
    }

    #[must_use]
    pub fn hls(manifest: HlsArtifactManifest) -> Self {
        Self::Hls { manifest }
    }

    #[must_use]
    pub fn primary_output_path(&self) -> &Path {
        match self {
            Self::Remux { output_path } => output_path,
            Self::Hls { manifest } => manifest.primary_playlist_path(),
        }
    }

    #[must_use]
    pub const fn hls_manifest(&self) -> Option<&HlsArtifactManifest> {
        match self {
            Self::Remux { .. } => None,
            Self::Hls { manifest } => Some(manifest),
        }
    }
}

fn validate_hls_artifact_name(artifact_name: &str) -> Result<()> {
    if artifact_name.is_empty()
        || artifact_name.contains('/')
        || artifact_name.contains('\\')
        || artifact_name.contains("..")
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls artifact name".to_owned(),
        });
    }

    let path = Path::new(artifact_name);
    if !path
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls artifact name".to_owned(),
        });
    }

    Ok(())
}
