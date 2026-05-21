use serde::{Deserialize, Serialize};
use taru_core::{Result, TaruError};

use super::hardware::HardwareAcceleration;

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

impl OutputContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hls => "hls",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodePlanValidationReason {
    InputLocatorRequired,
    HlsMustUseSupportedVideoCodec,
    HlsMustUseSupportedAudioCodec,
    HardwareAccelerationMustBeSelectedByRuntime,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePlanValidationError {
    pub reason: TranscodePlanValidationReason,
    pub operator_message: String,
}

impl TranscodePlanValidationError {
    fn new(reason: TranscodePlanValidationReason, operator_message: &'static str) -> Self {
        Self {
            reason,
            operator_message: operator_message.to_owned(),
        }
    }
}

impl TranscodePlan {
    pub fn validate_for_playback_request(
        &self,
    ) -> std::result::Result<(), TranscodePlanValidationError> {
        if self.input_locator.trim().is_empty() {
            return Err(TranscodePlanValidationError::new(
                TranscodePlanValidationReason::InputLocatorRequired,
                "playback transcode plan requires an input locator",
            ));
        }

        if self.hardware_acceleration != HardwareAcceleration::None {
            return Err(TranscodePlanValidationError::new(
                TranscodePlanValidationReason::HardwareAccelerationMustBeSelectedByRuntime,
                "playback transcode plan must leave hardware acceleration selection to the runtime",
            ));
        }

        if self.output_container == OutputContainer::Hls {
            if self
                .video_codec
                .as_deref()
                .is_some_and(|codec| !codec.eq_ignore_ascii_case("h264"))
            {
                return Err(TranscodePlanValidationError::new(
                    TranscodePlanValidationReason::HlsMustUseSupportedVideoCodec,
                    "hls playback transcode plan currently supports h264 video output",
                ));
            }

            if self
                .audio_codec
                .as_deref()
                .is_some_and(|codec| !codec.eq_ignore_ascii_case("aac"))
            {
                return Err(TranscodePlanValidationError::new(
                    TranscodePlanValidationReason::HlsMustUseSupportedAudioCodec,
                    "hls playback transcode plan currently supports aac audio output",
                ));
            }
        }

        Ok(())
    }
}

pub fn validate_playback_transcode_plan(plan: &TranscodePlan) -> Result<()> {
    plan.validate_for_playback_request()
        .map_err(|error| TaruError::InvalidInput {
            message: error.operator_message,
        })
}
