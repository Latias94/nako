use std::path::PathBuf;

use nako_core::{MediaSourceId, NakoError, Result};
use nako_transcode::{
    HLS_ADAPTIVE_MASTER_PLAYLIST_FILE, HlsAdaptiveLadderPlan, HlsArtifactManifest,
    HlsMediaRenditionPlan, HlsOutputRequirement, HlsRequestVariantPlan, HlsSegmentContainer,
    HlsVariantPolicy, RemuxContainer, TranscodeRequestIdentity,
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

    pub fn layout_for_output(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        output: HlsOutputRequirement,
    ) -> Result<HlsOutputLayout> {
        match output.variant_policy {
            HlsVariantPolicy::SingleVariant => {
                self.single_variant_layout(source_id, request_identity, output)
            }
            HlsVariantPolicy::Adaptive => self.layout_for_output_with_request_variant_plan(
                source_id,
                request_identity,
                output,
                &HlsRequestVariantPlan::new(
                    Some(HlsAdaptiveLadderPlan::default()),
                    HlsMediaRenditionPlan::default(),
                ),
            ),
        }
    }

    pub fn layout_for_output_with_request_variant_plan(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        output: HlsOutputRequirement,
        request_variant: &HlsRequestVariantPlan,
    ) -> Result<HlsOutputLayout> {
        match output.variant_policy {
            HlsVariantPolicy::SingleVariant => {
                let layout = self.single_variant_layout(source_id, request_identity, output)?;
                apply_media_renditions(layout, request_variant.media_renditions.clone())
            }
            HlsVariantPolicy::Adaptive => {
                let default_ladder;
                let adaptive_ladder =
                    if let Some(adaptive_ladder) = request_variant.adaptive_ladder.as_ref() {
                        adaptive_ladder
                    } else {
                        default_ladder = HlsAdaptiveLadderPlan::default();
                        &default_ladder
                    };

                self.adaptive_fmp4_layout_with_media(
                    source_id,
                    request_identity,
                    output,
                    adaptive_ladder,
                    &request_variant.media_renditions,
                )
            }
        }
    }

    pub fn layout_for_output_with_adaptive_plan(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        output: HlsOutputRequirement,
        adaptive_plan: &HlsAdaptiveLadderPlan,
    ) -> Result<HlsOutputLayout> {
        match output.variant_policy {
            HlsVariantPolicy::SingleVariant => {
                self.single_variant_layout(source_id, request_identity, output)
            }
            HlsVariantPolicy::Adaptive => self.adaptive_fmp4_layout_with_media(
                source_id,
                request_identity,
                output,
                adaptive_plan,
                &HlsMediaRenditionPlan::default(),
            ),
        }
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

    fn adaptive_fmp4_layout_with_media(
        &self,
        source_id: MediaSourceId,
        request_identity: &TranscodeRequestIdentity,
        output: HlsOutputRequirement,
        adaptive_plan: &HlsAdaptiveLadderPlan,
        media_renditions: &HlsMediaRenditionPlan,
    ) -> Result<HlsOutputLayout> {
        if output.variant_policy != HlsVariantPolicy::Adaptive {
            return Err(NakoError::InvalidInput {
                message: "adaptive hls layout requires adaptive variant policy".to_owned(),
            });
        }

        if output.segment_container != HlsSegmentContainer::Fmp4 {
            return Err(NakoError::Unsupported(
                "adaptive hls layout currently requires fmp4 segments",
            ));
        }

        let output_dir = self
            .root
            .join(source_id.to_string())
            .join(request_identity.storage_slug());
        let playlist_path = output_dir.join(HLS_ADAPTIVE_MASTER_PLAYLIST_FILE);

        for path in [&output_dir, &playlist_path] {
            if !path.starts_with(&self.root) {
                return Err(NakoError::storage_security_violation(
                    self.root.display().to_string(),
                    "hls staging output escaped the staging root",
                ));
            }
        }

        let artifacts = HlsArtifactManifest::adaptive_fmp4_with_audio(
            output_dir.clone(),
            playlist_path.clone(),
            adaptive_plan.renditions().to_vec(),
            adaptive_plan.has_audio(),
        )?
        .with_media_renditions(media_renditions.clone())?;
        let segment_pattern = artifacts.media_segment_pattern().to_path_buf();

        Ok(HlsOutputLayout {
            output_dir,
            playlist_path,
            segment_pattern,
            output: artifacts.output(),
            artifacts,
        })
    }
}

fn apply_media_renditions(
    mut layout: HlsOutputLayout,
    media_renditions: HlsMediaRenditionPlan,
) -> Result<HlsOutputLayout> {
    layout.artifacts = layout.artifacts.with_media_renditions(media_renditions)?;
    Ok(layout)
}
