use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use nako_core::{MediaSource, NakoError, Result};

use super::{
    HlsOutputRequirement, HlsVariantPolicy, OutputContainer, RemuxContainer,
    TranscodeExecutionPolicy,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeProfileKind {
    Remux,
    HlsSingleVariant,
}

impl TranscodeProfileKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Remux => "remux",
            Self::HlsSingleVariant => "hls_single_variant",
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
    pub kind: TranscodeProfileKind,
    pub output_container: String,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub execution_policy: TranscodeExecutionPolicy,
    pub hls_output: Option<HlsOutputRequirement>,
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
    RemuxMustNotSetHlsOutput,
    HlsMustUseHlsContainer,
    HlsOutputRequired,
    HlsVariantPolicyUnsupported,
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
            kind: TranscodeProfileKind::Remux,
            output_container: profile.output_container.file_extension().to_owned(),
            video_codec: None,
            audio_codec: None,
            execution_policy: TranscodeExecutionPolicy::remux(),
            hls_output: None,
            track_selection: profile.track_selection,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn hls_single_variant(profile: HlsTranscodeProfile) -> Self {
        Self {
            kind: TranscodeProfileKind::HlsSingleVariant,
            output_container: OutputContainer::Hls.as_str().to_owned(),
            video_codec: normalized_optional(profile.video_codec),
            audio_codec: normalized_optional(profile.audio_codec),
            execution_policy: profile.execution_policy,
            hls_output: Some(profile.hls_output),
            track_selection: profile.track_selection,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn identity(&self) -> TranscodeProfileIdentity {
        validate_transcode_profile(self).expect("transcode profile must be valid before identity");
        let request_key = self.persisted_request_key();
        let digest = Sha256::digest(request_key.as_bytes());
        let digest = lowercase_hex(&digest);

        TranscodeProfileIdentity {
            request_key,
            storage_slug: format!("{}-v1-{}", self.kind.as_str(), &digest[..16]),
        }
    }

    pub fn validate(&self) -> std::result::Result<(), TranscodeProfileValidationError> {
        if self.playback_profile_key.trim().is_empty() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::PlaybackProfileKeyRequired,
                "transcode profile requires a playback profile key",
            ));
        }

        match self.kind {
            TranscodeProfileKind::Remux => self.validate_remux(),
            TranscodeProfileKind::HlsSingleVariant => self.validate_hls_single_variant(),
        }
    }

    fn persisted_request_key(&self) -> String {
        format!(
            "transcode-profile:v1;kind={};container={};vcodec={};acodec={};hls_variant={};hls_segment={};acceleration={};audio={};subtitle={};subtitle_strategy={};max_video_bitrate={};prefer_hdr={};remote_input={};reuse={};playback={}",
            self.kind.as_str(),
            canonical_value(&self.output_container),
            optional_str(self.video_codec.as_deref()),
            optional_str(self.audio_codec.as_deref()),
            hls_variant_key(self.hls_output),
            hls_segment_key(self.hls_output),
            self.execution_policy.acceleration.identity_key(),
            optional_u32(self.track_selection.audio_stream),
            optional_u32(self.track_selection.subtitle_stream),
            self.execution_policy.subtitle_strategy.as_str(),
            optional_u64(self.execution_policy.output_constraints.max_video_bitrate),
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
        if self.hls_output.is_some() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::RemuxMustNotSetHlsOutput,
                "remux profile must not set HLS output requirements",
            ));
        }
        Ok(())
    }

    fn validate_hls_single_variant(
        &self,
    ) -> std::result::Result<(), TranscodeProfileValidationError> {
        if canonical_value(&self.output_container) != OutputContainer::Hls.as_str() {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::HlsMustUseHlsContainer,
                "hls transcode profile must use the hls output container",
            ));
        }
        let Some(hls_output) = self.hls_output else {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::HlsOutputRequired,
                "hls transcode profile requires HLS output requirements",
            ));
        };
        if hls_output.variant_policy != HlsVariantPolicy::SingleVariant {
            return Err(TranscodeProfileValidationError::new(
                TranscodeProfileValidationReason::HlsVariantPolicyUnsupported,
                "hls single-variant profile cannot execute adaptive output yet",
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
        let request_key = format!(
            "transcode-request:v1;source={};profile={}",
            source_identity.revision_key(),
            escaped_component(profile_identity.persisted_request_key()),
        );

        Self {
            storage_slug: format!(
                "{}-{}",
                profile_identity.storage_slug(),
                source_identity.storage_slug()
            ),
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

fn hls_variant_key(value: Option<HlsOutputRequirement>) -> &'static str {
    value.map_or("none", |value| value.variant_policy.as_str())
}

fn hls_segment_key(value: Option<HlsOutputRequirement>) -> &'static str {
    value.map_or("none", |value| value.segment_container.as_str())
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
