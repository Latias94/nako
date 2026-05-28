use std::path::{Path, PathBuf};

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

use crate::policy::{HlsOutputRequirement, HlsVariantPolicy};

pub const HLS_ADAPTIVE_MASTER_PLAYLIST_FILE: &str = "master.m3u8";
pub const HLS_ADAPTIVE_VARIANT_PLAYLIST_PATTERN: &str = "variant_%v.m3u8";
pub const HLS_ADAPTIVE_FMP4_SEGMENT_PATTERN: &str = "variant_%v_segment_%05d.m4s";
pub const HLS_ADAPTIVE_FMP4_INIT_PATTERN: &str = "variant_%v_init.mp4";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TranscodeArtifactSet {
    Remux { output_path: PathBuf },
    Hls { manifest: HlsArtifactManifest },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRendition {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub video_bitrate: u64,
    pub audio_bitrate: u64,
}

impl HlsRendition {
    #[must_use]
    pub const fn new(
        index: usize,
        width: u32,
        height: u32,
        video_bitrate: u64,
        audio_bitrate: u64,
    ) -> Self {
        Self {
            index,
            width,
            height,
            video_bitrate,
            audio_bitrate,
        }
    }

    #[must_use]
    pub fn default_adaptive_ladder() -> Vec<Self> {
        vec![
            Self::new(0, 1280, 720, 3_000_000, 128_000),
            Self::new(1, 854, 480, 1_200_000, 128_000),
        ]
    }

    #[must_use]
    pub fn playlist_file_name(self) -> String {
        format!("variant_{}.m3u8", self.index)
    }

    #[must_use]
    pub fn segment_file_prefix(self) -> String {
        format!("variant_{}_segment_", self.index)
    }

    #[must_use]
    pub fn init_segment_file_name(self) -> String {
        format!("variant_{}_init.mp4", self.index)
    }

    #[must_use]
    pub const fn bandwidth(self) -> u64 {
        self.video_bitrate.saturating_add(self.audio_bitrate)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsArtifactManifest {
    pub output_dir: PathBuf,
    pub primary_playlist_path: PathBuf,
    pub media_segment_pattern: PathBuf,
    pub variant_playlist_pattern: Option<PathBuf>,
    pub renditions: Vec<HlsRendition>,
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
                "single-variant hls artifact manifest requires single-variant output",
            ));
        }

        let manifest = Self {
            output_dir: output_dir.into(),
            primary_playlist_path: primary_playlist_path.into(),
            media_segment_pattern: media_segment_pattern.into(),
            variant_playlist_pattern: None,
            renditions: Vec::new(),
            output,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn adaptive_fmp4(
        output_dir: impl Into<PathBuf>,
        primary_playlist_path: impl Into<PathBuf>,
        renditions: Vec<HlsRendition>,
    ) -> Result<Self> {
        let output_dir = output_dir.into();
        let manifest = Self {
            primary_playlist_path: primary_playlist_path.into(),
            media_segment_pattern: output_dir.join(HLS_ADAPTIVE_FMP4_SEGMENT_PATTERN),
            variant_playlist_pattern: Some(output_dir.join(HLS_ADAPTIVE_VARIANT_PLAYLIST_PATTERN)),
            renditions,
            output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: crate::policy::HlsSegmentContainer::Fmp4,
            },
            output_dir,
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
    pub fn variant_playlist_pattern(&self) -> Option<&Path> {
        self.variant_playlist_pattern.as_deref()
    }

    #[must_use]
    pub fn renditions(&self) -> &[HlsRendition] {
        &self.renditions
    }

    #[must_use]
    pub const fn output(&self) -> HlsOutputRequirement {
        self.output
    }

    #[must_use]
    pub fn init_segment_path(&self) -> Option<PathBuf> {
        if self.output.variant_policy != HlsVariantPolicy::SingleVariant {
            return None;
        }

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

        if self.output.variant_policy == HlsVariantPolicy::Adaptive {
            return self.adaptive_artifact_for_name(artifact_name, path);
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

    fn adaptive_artifact_for_name(
        &self,
        artifact_name: &str,
        path: PathBuf,
    ) -> Result<HlsArtifactDescriptor> {
        for rendition in &self.renditions {
            if rendition.playlist_file_name() == artifact_name {
                return Ok(HlsArtifactDescriptor {
                    path,
                    content_type: "application/vnd.apple.mpegurl",
                    cleanup_candidate: false,
                });
            }

            if rendition.init_segment_file_name() == artifact_name {
                return Ok(HlsArtifactDescriptor {
                    path,
                    content_type: self.output.segment_container.segment_content_type(),
                    cleanup_candidate: false,
                });
            }

            if artifact_name.starts_with(&rendition.segment_file_prefix())
                && Path::new(artifact_name)
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

        if self
            .primary_playlist_path
            .extension()
            .and_then(|value| value.to_str())
            != Some("m3u8")
        {
            return Err(NakoError::InvalidInput {
                message: "hls playlist path must use the m3u8 extension".to_owned(),
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

        match self.output.variant_policy {
            HlsVariantPolicy::SingleVariant => self.validate_single_variant()?,
            HlsVariantPolicy::Adaptive => self.validate_adaptive()?,
        }

        Ok(())
    }

    fn validate_single_variant(&self) -> Result<()> {
        if self.variant_playlist_pattern.is_some() || !self.renditions.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "single-variant hls manifest must not carry an adaptive ladder".to_owned(),
            });
        }

        Ok(())
    }

    fn validate_adaptive(&self) -> Result<()> {
        if self.output.segment_container != crate::policy::HlsSegmentContainer::Fmp4 {
            return Err(NakoError::Unsupported(
                "adaptive hls manifests currently require fmp4 segments",
            ));
        }

        let Some(variant_playlist_pattern) = self.variant_playlist_pattern.as_ref() else {
            return Err(NakoError::InvalidInput {
                message: "adaptive hls manifest requires a variant playlist pattern".to_owned(),
            });
        };

        if !variant_playlist_pattern.starts_with(&self.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls variant playlist pattern must be inside the output directory"
                    .to_owned(),
            });
        }

        if variant_playlist_pattern
            .extension()
            .and_then(|value| value.to_str())
            != Some("m3u8")
        {
            return Err(NakoError::InvalidInput {
                message: "hls variant playlist pattern must use the m3u8 extension".to_owned(),
            });
        }

        if !variant_playlist_pattern
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.contains("%v"))
        {
            return Err(NakoError::InvalidInput {
                message: "adaptive hls variant playlist pattern must contain %v".to_owned(),
            });
        }

        if !self
            .media_segment_pattern
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.contains("%v"))
        {
            return Err(NakoError::InvalidInput {
                message: "adaptive hls segment pattern must contain %v".to_owned(),
            });
        }

        if self.renditions.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "adaptive hls manifest requires at least one rendition".to_owned(),
            });
        }

        let mut indexes = Vec::with_capacity(self.renditions.len());
        for rendition in &self.renditions {
            if rendition.width == 0
                || rendition.height == 0
                || rendition.video_bitrate == 0
                || rendition.audio_bitrate == 0
            {
                return Err(NakoError::InvalidInput {
                    message: "adaptive hls renditions require positive dimensions and bitrates"
                        .to_owned(),
                });
            }

            if indexes.contains(&rendition.index) {
                return Err(NakoError::InvalidInput {
                    message: "adaptive hls rendition indexes must be unique".to_owned(),
                });
            }
            indexes.push(rendition.index);
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
