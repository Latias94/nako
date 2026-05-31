use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use nako_core::{MediaSource, NakoError, Result};

use super::{
    HlsOutputRequirement, HlsSegmentContainer, HlsVariantPolicy, OutputContainer, RemuxContainer,
    TranscodeExecutionPolicy, TranscodePlan, validate_playback_transcode_plan,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeProfileKind {
    Remux,
    HlsSingleVariant,
    HlsAdaptive,
}

impl TranscodeProfileKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Remux => "remux",
            Self::HlsSingleVariant => "hls_single_variant",
            Self::HlsAdaptive => "hls_adaptive",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeReusePolicy {
    #[default]
    FinishedOutput,
}

impl TranscodeReusePolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FinishedOutput => "finished_output",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeTrackSelection {
    pub audio_stream: Option<u32>,
    pub subtitle_stream: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TranscodeOutputShape {
    Remux { container: RemuxContainer },
    Hls { requirement: HlsOutputRequirement },
}

impl TranscodeOutputShape {
    #[must_use]
    pub const fn profile_kind(self) -> TranscodeProfileKind {
        match self {
            Self::Remux { .. } => TranscodeProfileKind::Remux,
            Self::Hls { requirement } => match requirement.variant_policy {
                HlsVariantPolicy::SingleVariant => TranscodeProfileKind::HlsSingleVariant,
                HlsVariantPolicy::Adaptive => TranscodeProfileKind::HlsAdaptive,
            },
        }
    }

    #[must_use]
    pub const fn container_key(self) -> &'static str {
        match self {
            Self::Remux { container } => container.file_extension(),
            Self::Hls { .. } => "hls",
        }
    }

    #[must_use]
    pub const fn hls_requirement(self) -> Option<HlsOutputRequirement> {
        match self {
            Self::Remux { .. } => None,
            Self::Hls { requirement } => Some(requirement),
        }
    }

    #[must_use]
    pub const fn hls_variant_key(self) -> &'static str {
        match self {
            Self::Remux { .. } => "none",
            Self::Hls { requirement } => requirement.variant_policy.as_str(),
        }
    }

    #[must_use]
    pub const fn hls_segment_key(self) -> &'static str {
        match self {
            Self::Remux { .. } => "none",
            Self::Hls { requirement } => requirement.segment_container.as_str(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxTranscodeProfile {
    pub output_container: RemuxContainer,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsTranscodeProfile {
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub execution_policy: TranscodeExecutionPolicy,
    pub hls_output: HlsOutputRequirement,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackRemuxProfileRequest {
    pub output_container: RemuxContainer,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackHlsProfileRequest {
    pub plan: TranscodePlan,
    pub execution_policy: TranscodeExecutionPolicy,
    pub hls_output: HlsOutputRequirement,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeProfile {
    pub output: TranscodeOutputShape,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub execution_policy: TranscodeExecutionPolicy,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub reuse_policy: TranscodeReusePolicy,
    pub playback_profile_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeProfileValidationReason {
    RemuxMustNotTranscodeVideo,
    RemuxMustNotTranscodeAudio,
    RemuxMustUseCpuPath,
    RemuxMustNotSetVideoBitrate,
    RemuxMustNotSetHdrPreference,
    HlsAdaptiveRequiresFmp4,
    HlsVideoCodecUnsupported,
    HlsAudioCodecUnsupported,
    HlsVideoBitrateMustBePositive,
    PlaybackProfileKeyRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeProfileValidationError {
    pub reason: TranscodeProfileValidationReason,
    pub operator_message: String,
}

impl TranscodeProfileValidationError {
    fn new(reason: TranscodeProfileValidationReason, operator_message: &'static str) -> Self {
        Self {
            reason,
            operator_message: operator_message.to_owned(),
        }
    }
}

impl TranscodeProfile {
    #[must_use]
    pub fn remux(profile: RemuxTranscodeProfile) -> Self {
        Self {
            output: TranscodeOutputShape::Remux {
                container: profile.output_container,
            },
            video_codec: None,
            audio_codec: None,
            execution_policy: TranscodeExecutionPolicy::remux(),
            track_selection: profile.track_selection,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn hls(profile: HlsTranscodeProfile) -> Self {
        Self {
            output: TranscodeOutputShape::Hls {
                requirement: profile.hls_output,
            },
            video_codec: normalized_optional(profile.video_codec),
            audio_codec: normalized_optional(profile.audio_codec),
            execution_policy: profile.execution_policy,
            track_selection: profile.track_selection,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn hls_single_variant(profile: HlsTranscodeProfile) -> Self {
        Self::hls(profile)
    }

    #[must_use]
    pub const fn kind(&self) -> TranscodeProfileKind {
        self.output.profile_kind()
    }

    #[must_use]
    pub const fn hls_output_requirement(&self) -> Option<HlsOutputRequirement> {
        self.output.hls_requirement()
    }

    #[must_use]
    pub fn identity(&self) -> TranscodeProfileIdentity {
        validate_transcode_profile(self).expect("transcode profile must be valid before identity");
        let request_key = self.persisted_request_key();
        let digest = Sha256::digest(request_key.as_bytes());
        let digest = lowercase_hex(&digest);

        TranscodeProfileIdentity {
            request_key,
            storage_slug: format!("{}-v1-{}", self.kind().as_str(), &digest[..16]),
        }
    }

    pub fn validate(&self) -> std::result::Result<(), TranscodeProfileValidationError> {
        if self.playback_profile_key.trim().is_empty() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::PlaybackProfileKeyRequired,
                "transcode profile requires a playback profile key",
            ));
        }

        match self.output {
            TranscodeOutputShape::Remux { .. } => self.validate_remux(),
            TranscodeOutputShape::Hls { requirement } => self.validate_hls(requirement),
        }
    }

    fn persisted_request_key(&self) -> String {
        let output = self.output;
        let audio_output = self.execution_policy.audio_output;
        let audio_output_identity = if audio_output == Default::default() {
            String::new()
        } else {
            format!(";audio_output={}", audio_output.persisted_identity_key())
        };
        let color_pipeline = self.execution_policy.color_pipeline;
        let color_pipeline_identity = if color_pipeline == Default::default() {
            String::new()
        } else {
            format!(
                ";color_pipeline={}",
                color_pipeline.persisted_identity_key()
            )
        };
        format!(
            "transcode-profile:v1;kind={};container={};vcodec={};acodec={};hls_variant={};hls_segment={};acceleration={};audio={};subtitle={};subtitle_strategy={}{}{};max_video_bitrate={};max_width={};max_height={};prefer_hdr={};remote_input={};reuse={};playback={}",
            output.profile_kind().as_str(),
            output.container_key(),
            optional_str(self.video_codec.as_deref()),
            optional_str(self.audio_codec.as_deref()),
            output.hls_variant_key(),
            output.hls_segment_key(),
            self.execution_policy.acceleration.identity_key(),
            optional_u32(self.track_selection.audio_stream),
            optional_u32(self.track_selection.subtitle_stream),
            self.execution_policy.subtitle_strategy.as_str(),
            audio_output_identity,
            color_pipeline_identity,
            optional_u64(self.execution_policy.output_constraints.max_video_bitrate),
            optional_u32(self.execution_policy.output_constraints.max_width),
            optional_u32(self.execution_policy.output_constraints.max_height),
            optional_bool(self.execution_policy.output_constraints.prefer_hdr),
            self.remote_input,
            self.reuse_policy.as_str(),
            escaped_component(&self.playback_profile_key),
        )
    }

    fn validate_remux(&self) -> std::result::Result<(), TranscodeProfileValidationError> {
        if self.video_codec.is_some() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustNotTranscodeVideo,
                "remux profile must not request video transcoding",
            ));
        }
        if self.audio_codec.is_some() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustNotTranscodeAudio,
                "remux profile must not request audio transcoding",
            ));
        }
        if !self.execution_policy.acceleration.is_software_only() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustUseCpuPath,
                "remux profile must not request hardware acceleration",
            ));
        }
        if self
            .execution_policy
            .output_constraints
            .max_video_bitrate
            .is_some()
        {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustNotSetVideoBitrate,
                "remux profile must not set a video bitrate limit",
            ));
        }
        if self
            .execution_policy
            .output_constraints
            .prefer_hdr
            .is_some()
        {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustNotSetHdrPreference,
                "remux profile must not set an HDR preference",
            ));
        }
        Ok(())
    }

    fn validate_hls(
        &self,
        hls_output: HlsOutputRequirement,
    ) -> std::result::Result<(), TranscodeProfileValidationError> {
        if hls_output.variant_policy == HlsVariantPolicy::Adaptive
            && hls_output.segment_container != HlsSegmentContainer::Fmp4
        {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::HlsAdaptiveRequiresFmp4,
                "adaptive hls profile currently requires fmp4 segment output",
            ));
        }
        if let Some(codec) = self.video_codec.as_deref() {
            if !matches!(codec, "h264") {
                return Err(TranscodeProfileValidationError::new(
                    TranscodeProfileValidationReason::HlsVideoCodecUnsupported,
                    "hls transcode profile currently supports h264 video output",
                ));
            }
        }
        if let Some(codec) = self.audio_codec.as_deref() {
            if !matches!(codec, "aac") {
                return Err(TranscodeProfileValidationError::new(
                    TranscodeProfileValidationReason::HlsAudioCodecUnsupported,
                    "hls transcode profile currently supports aac audio output",
                ));
            }
        }
        if self
            .execution_policy
            .output_constraints
            .max_video_bitrate
            .is_some_and(|value| value == 0)
        {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::HlsVideoBitrateMustBePositive,
                "hls transcode profile video bitrate limit must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeProfileIdentity {
    request_key: String,
    storage_slug: String,
}

impl TranscodeProfileIdentity {
    #[must_use]
    pub fn persisted_request_key(&self) -> &str {
        &self.request_key
    }

    #[must_use]
    pub fn storage_slug(&self) -> &str {
        &self.storage_slug
    }

    #[must_use]
    pub fn bind_source(
        &self,
        source_identity: &TranscodeSourceIdentity,
    ) -> TranscodeRequestIdentity {
        TranscodeRequestIdentity::new(source_identity.clone(), self.clone())
    }

    #[must_use]
    pub fn bind_source_with_request_variant(
        &self,
        source_identity: &TranscodeSourceIdentity,
        request_variant_key: impl Into<String>,
    ) -> TranscodeRequestIdentity {
        TranscodeRequestIdentity::new_with_request_variant(
            source_identity.clone(),
            self.clone(),
            request_variant_key.into(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeSourceIdentity {
    revision_key: String,
    storage_slug: String,
}

impl TranscodeSourceIdentity {
    #[must_use]
    pub fn from_media_source(source: &MediaSource) -> Self {
        let material = format!(
            "transcode-source-revision-input:v1;library={};source={};locator={};file_name={};size={};fingerprint={}",
            source.library_id,
            source.id,
            source.locator,
            source.file_name,
            optional_u64(source.size_bytes),
            source.fingerprint.as_deref().unwrap_or("unknown"),
        );
        let digest = lowercase_hex(&Sha256::digest(material.as_bytes()));

        Self {
            revision_key: format!("source-revision:v1;digest={}", &digest[..32]),
            storage_slug: format!("source-v1-{}", &digest[..16]),
        }
    }

    #[must_use]
    pub fn revision_key(&self) -> &str {
        &self.revision_key
    }

    #[must_use]
    pub fn storage_slug(&self) -> &str {
        &self.storage_slug
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeRequestIdentity {
    source_identity: TranscodeSourceIdentity,
    profile_identity: TranscodeProfileIdentity,
    request_key: String,
    storage_slug: String,
}

impl TranscodeRequestIdentity {
    #[must_use]
    pub fn new(
        source_identity: TranscodeSourceIdentity,
        profile_identity: TranscodeProfileIdentity,
    ) -> Self {
        Self::new_inner(source_identity, profile_identity, None)
    }

    #[must_use]
    pub fn new_with_request_variant(
        source_identity: TranscodeSourceIdentity,
        profile_identity: TranscodeProfileIdentity,
        request_variant_key: String,
    ) -> Self {
        Self::new_inner(source_identity, profile_identity, Some(request_variant_key))
    }

    fn new_inner(
        source_identity: TranscodeSourceIdentity,
        profile_identity: TranscodeProfileIdentity,
        request_variant_key: Option<String>,
    ) -> Self {
        let request_key = format!(
            "transcode-request:v1;source={};profile={}",
            source_identity.revision_key(),
            escaped_component(profile_identity.persisted_request_key()),
        );
        let (request_key, storage_slug) =
            if let Some(request_variant_key) = request_variant_key.as_deref() {
                let variant_digest = lowercase_hex(&Sha256::digest(request_variant_key.as_bytes()));
                (
                    format!(
                        "{request_key};request_variant={}",
                        escaped_component(request_variant_key)
                    ),
                    format!(
                        "{}-{}-variant-v1-{}",
                        profile_identity.storage_slug(),
                        source_identity.storage_slug(),
                        &variant_digest[..16]
                    ),
                )
            } else {
                (
                    request_key,
                    format!(
                        "{}-{}",
                        profile_identity.storage_slug(),
                        source_identity.storage_slug()
                    ),
                )
            };

        Self {
            storage_slug,
            source_identity,
            profile_identity,
            request_key,
        }
    }

    #[must_use]
    pub fn persisted_request_key(&self) -> &str {
        &self.request_key
    }

    #[must_use]
    pub fn storage_slug(&self) -> &str {
        &self.storage_slug
    }

    #[must_use]
    pub fn source_identity(&self) -> &TranscodeSourceIdentity {
        &self.source_identity
    }

    #[must_use]
    pub fn profile_identity(&self) -> &TranscodeProfileIdentity {
        &self.profile_identity
    }
}

pub fn validate_transcode_profile(profile: &TranscodeProfile) -> Result<()> {
    profile.validate().map_err(|error| NakoError::InvalidInput {
        message: error.operator_message,
    })
}

pub fn build_playback_remux_profile(
    request: PlaybackRemuxProfileRequest,
) -> Result<TranscodeProfile> {
    let profile = TranscodeProfile::remux(RemuxTranscodeProfile {
        output_container: request.output_container,
        track_selection: request.track_selection,
        remote_input: request.remote_input,
        playback_profile_key: request.playback_profile_key,
    });
    validate_transcode_profile(&profile)?;
    Ok(profile)
}

pub fn build_playback_hls_profile(request: PlaybackHlsProfileRequest) -> Result<TranscodeProfile> {
    if request.plan.output_container != OutputContainer::Hls {
        return Err(NakoError::InvalidInput {
            message: "hls playback profile requires an hls transcode plan".to_owned(),
        });
    }
    validate_playback_transcode_plan(&request.plan)?;
    let profile = TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
        video_codec: request.plan.video_codec,
        audio_codec: request.plan.audio_codec,
        execution_policy: request.execution_policy,
        hls_output: request.hls_output,
        track_selection: request.track_selection,
        remote_input: request.remote_input,
        playback_profile_key: request.playback_profile_key,
    });
    validate_transcode_profile(&profile)?;
    Ok(profile)
}

fn optional_str(value: Option<&str>) -> String {
    value.map_or_else(|| "auto".to_owned(), canonical_value)
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "default".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| canonical_value(&value))
        .filter(|value| !value.is_empty())
}

fn canonical_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn escaped_component(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'%' => escaped.push_str("%25"),
            b';' => escaped.push_str("%3B"),
            b'=' => escaped.push_str("%3D"),
            _ => escaped.push(*byte as char),
        }
    }
    escaped
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HardwareAcceleration, TranscodeAccelerationPlan, TranscodeAudioCompatibilityReasons,
        TranscodeAudioDownmixRequirement, TranscodeAudioNormalizationRequirement,
        TranscodeAudioOutputRequirement, TranscodeOutputConstraints,
    };

    #[test]
    fn playback_remux_profile_builder_creates_copy_profile_identity() {
        let profile = build_playback_remux_profile(PlaybackRemuxProfileRequest {
            output_container: RemuxContainer::Mp4,
            track_selection: TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: Some(2),
            },
            remote_input: true,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        })
        .unwrap();

        assert_eq!(
            profile.output,
            TranscodeOutputShape::Remux {
                container: RemuxContainer::Mp4
            }
        );
        assert!(profile.video_codec.is_none());
        assert!(profile.audio_codec.is_none());
        assert!(profile.execution_policy.acceleration.is_software_only());
        assert!(profile.remote_input);
        assert!(
            profile
                .identity()
                .persisted_request_key()
                .contains("playback=playback-target-profile:v1%3Bdemo%3Dtrue")
        );
    }

    #[test]
    fn playback_hls_profile_builder_preserves_runtime_policy() {
        let execution_policy = TranscodeExecutionPolicy::hls_single_variant(
            TranscodeAccelerationPlan::for_selected_hardware(HardwareAcceleration::Nvenc),
            TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: Some(3),
            },
            TranscodeOutputConstraints {
                max_video_bitrate: Some(8_000_000),
                max_width: Some(1920),
                max_height: Some(1080),
                prefer_hdr: Some(false),
            },
        );

        let profile = build_playback_hls_profile(PlaybackHlsProfileRequest {
            plan: TranscodePlan {
                input_locator: "local:///movie.mkv".to_owned(),
                output_container: crate::OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            execution_policy,
            hls_output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::Fmp4,
            },
            track_selection: TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: Some(3),
            },
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        })
        .unwrap();

        assert_eq!(profile.kind(), TranscodeProfileKind::HlsAdaptive);
        assert_eq!(profile.video_codec.as_deref(), Some("h264"));
        assert_eq!(profile.audio_codec.as_deref(), Some("aac"));
        assert_eq!(
            profile.execution_policy.acceleration.encode.accelerator,
            HardwareAcceleration::Nvenc
        );
        assert_eq!(
            profile
                .execution_policy
                .output_constraints
                .max_video_bitrate,
            Some(8_000_000)
        );
        assert_eq!(
            profile.hls_output_requirement(),
            Some(HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::Fmp4,
            })
        );
    }

    #[test]
    fn playback_hls_profile_identity_carries_audio_output_requirement() {
        let audio_output = TranscodeAudioOutputRequirement {
            source_channels: Some(6),
            max_supported_channels: Some(2),
            target_channels: Some(2),
            downmix: TranscodeAudioDownmixRequirement::Required,
            normalization: TranscodeAudioNormalizationRequirement::None,
            reasons: TranscodeAudioCompatibilityReasons {
                channel_limit_exceeded: true,
                downmix_required: true,
                normalization_requested: false,
            },
        };
        let execution_policy = TranscodeExecutionPolicy::hls_single_variant_with_audio_output(
            TranscodeAccelerationPlan::software(),
            TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: None,
            },
            TranscodeOutputConstraints::default(),
            audio_output,
        );

        let profile = build_playback_hls_profile(PlaybackHlsProfileRequest {
            plan: TranscodePlan {
                input_locator: "local:///movie.mkv".to_owned(),
                output_container: crate::OutputContainer::Hls,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            execution_policy,
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection {
                audio_stream: Some(1),
                subtitle_stream: None,
            },
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        })
        .unwrap();

        assert_eq!(profile.execution_policy.audio_output, audio_output);
        assert!(
            profile
                .identity()
                .persisted_request_key()
                .contains("audio_output=source:6,max:2,target:2,downmix:required,normalization:none,reasons:channel_limit_exceeded|downmix_required")
        );
    }

    #[test]
    fn playback_hls_profile_builder_validates_playback_plan() {
        let err = build_playback_hls_profile(PlaybackHlsProfileRequest {
            plan: TranscodePlan {
                input_locator: "local:///movie.mp4".to_owned(),
                output_container: crate::OutputContainer::Mp4,
                video_codec: Some("h264".to_owned()),
                audio_codec: Some("aac".to_owned()),
            },
            execution_policy: TranscodeExecutionPolicy::hls_single_variant(
                TranscodeAccelerationPlan::software(),
                TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
            ),
            hls_output: HlsOutputRequirement::default(),
            track_selection: TranscodeTrackSelection::default(),
            remote_input: false,
            playback_profile_key: "playback-target-profile:v1;demo=true".to_owned(),
        })
        .unwrap_err();

        assert!(matches!(err, NakoError::InvalidInput { .. }));
    }
}
