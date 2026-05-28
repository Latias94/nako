use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use nako_core::{MediaSource, NakoError, Result};

use super::{
    HlsOutputRequirement, HlsSegmentContainer, HlsVariantPolicy, RemuxContainer,
    TranscodeExecutionPolicy,
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
        format!(
            "transcode-profile:v1;kind={};container={};vcodec={};acodec={};hls_variant={};hls_segment={};acceleration={};audio={};subtitle={};subtitle_strategy={};max_video_bitrate={};max_width={};max_height={};prefer_hdr={};remote_input={};reuse={};playback={}",
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
