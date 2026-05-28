use std::path::PathBuf;

use nako_core::{MediaSourceId, NakoError, Result};
use nako_transcode::{
    HlsArtifactManifest, HlsOutputRequirement, HlsVariantPolicy, RemuxContainer,
    TranscodeRequestIdentity,
};

#[derive(Clone, Debug)]
pub struct RemuxStagingPolicy {
    root: PathBuf,
}

impl RemuxStagingPolicy {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        if root.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "remux staging root cannot be empty".to_owned(),
            });
        }

        if root
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(NakoError::InvalidInput {
                message: "remux staging root must not contain relative path components".to_owned(),
            });
        }

        Ok(Self { root })
    }

    pub fn output_path(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        container: RemuxContainer,
    ) -> Result<PathBuf> {
        let output = self
            .root
            .join(source_id.to_string())
            .join(request_identity.storage_slug())
            .join(format!("stream.{}", container.file_extension()));

        if !output.starts_with(&self.root) {
            return Err(NakoError::storage_security_violation(
                self.root.display().to_string(),
                "remux staging output escaped the staging root",
            ));
        }

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct HlsStagingPolicy {
    root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsOutputLayout {
    pub output_dir: PathBuf,
    pub playlist_path: PathBuf,
    pub segment_pattern: PathBuf,
    pub output: HlsOutputRequirement,
    pub artifacts: HlsArtifactManifest,
}

impl HlsStagingPolicy {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        if root.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls staging root cannot be empty".to_owned(),
            });
        }

        if root
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(NakoError::InvalidInput {
                message: "hls staging root must not contain relative path components".to_owned(),
            });
        }

        Ok(Self { root })
    }

    pub fn single_variant_layout(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        output: HlsOutputRequirement,
    ) -> Result<HlsOutputLayout> {
        if output.variant_policy != HlsVariantPolicy::SingleVariant {
            return Err(NakoError::Unsupported(
                "adaptive hls output is not implemented by the staging policy",
            ));
        }

        let output_dir = self
            .root
            .join(source_id.to_string())
            .join(request_identity.storage_slug());
        let playlist_path = output_dir.join("playlist.m3u8");
        let segment_pattern = output_dir.join(format!(
            "segment_%05d.{}",
            output.segment_container.segment_extension()
        ));

        for path in [&output_dir, &playlist_path, &segment_pattern] {
            if !path.starts_with(&self.root) {
                return Err(NakoError::storage_security_violation(
                    self.root.display().to_string(),
                    "hls staging output escaped the staging root",
                ));
            }
        }

        let artifacts = HlsArtifactManifest::single_variant(
            output_dir.clone(),
            playlist_path.clone(),
            segment_pattern.clone(),
            output,
        )?;

        Ok(HlsOutputLayout {
            output_dir,
            playlist_path,
            segment_pattern,
            output,
            artifacts,
        })
    }
}
