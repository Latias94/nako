use std::path::{Path, PathBuf};

use nako_core::{
    MediaProbeResult, MediaStreamInfo, MediaStreamKind, MediaStreamOrigin, NakoError, Result,
};
use serde::{Deserialize, Serialize};

use crate::{
    TranscodeOutputConstraints, TranscodePipelineSourceFacts, TranscodeTrackSelection,
    policy::{HlsOutputRequirement, HlsVariantPolicy},
};

pub const HLS_ADAPTIVE_MASTER_PLAYLIST_FILE: &str = "master.m3u8";
pub const HLS_ADAPTIVE_VARIANT_PLAYLIST_PATTERN: &str = "variant_%v.m3u8";
pub const HLS_ADAPTIVE_FMP4_SEGMENT_PATTERN: &str = "variant_%v_segment_%05d.m4s";
pub const HLS_ADAPTIVE_FMP4_INIT_PATTERN: &str = "variant_%v_init.mp4";

const HLS_ADAPTIVE_LADDER_IDENTITY_VERSION: &str = "hls-adaptive-ladder:v1";
const HLS_MAIN_OUTPUT_IDENTITY_VERSION: &str = "hls-main-output:v1";
const HLS_MEDIA_RENDITIONS_IDENTITY_VERSION: &str = "hls-media-renditions:v1";
const HLS_PLAYBACK_GENERATION_IDENTITY_VERSION: &str = "hls-playback-generation:v1";
const HLS_REQUEST_VARIANT_IDENTITY_VERSION: &str = "hls-request-variant:v1";
const TRANSCODE_REQUEST_IDENTITY_VERSION: &str = "transcode-request:v1";
const TRANSCODE_PROFILE_IDENTITY_VERSION: &str = "transcode-profile:v1";
const HLS_ADAPTIVE_AUDIO_BITRATE: u64 = 128_000;
const HLS_ADAPTIVE_LADDER_CANDIDATES: &[(u32, u32, u64)] = &[
    (3840, 2160, 16_000_000),
    (2560, 1440, 10_000_000),
    (1920, 1080, 6_000_000),
    (1280, 720, 3_000_000),
    (854, 480, 1_200_000),
    (640, 360, 800_000),
];
const HLS_ADAPTIVE_DEFAULT_LADDER_CANDIDATES: &[(u32, u32, u64)] =
    &[(1280, 720, 3_000_000), (854, 480, 1_200_000)];
const HLS_BURN_IN_TEXT_SUBTITLE_CODECS: &[&str] =
    &["ass", "mov_text", "ssa", "srt", "subrip", "text", "webvtt"];

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
            Self::new(0, 1280, 720, 3_000_000, HLS_ADAPTIVE_AUDIO_BITRATE),
            Self::new(1, 854, 480, 1_200_000, HLS_ADAPTIVE_AUDIO_BITRATE),
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
pub struct HlsAudioRendition {
    pub index: usize,
    pub source_stream_index: u32,
    pub language: Option<String>,
    pub default: bool,
}

impl HlsAudioRendition {
    #[must_use]
    pub fn new(
        index: usize,
        source_stream_index: u32,
        language: Option<String>,
        default: bool,
    ) -> Self {
        Self {
            index,
            source_stream_index,
            language: language
                .map(|value| canonical_value(&value))
                .filter(|value| !value.is_empty()),
            default,
        }
    }

    #[must_use]
    pub fn playlist_file_name(&self) -> String {
        format!("audio_{}.m3u8", self.index)
    }

    #[must_use]
    pub fn segment_file_prefix(&self) -> String {
        format!("audio_{}_", self.index)
    }

    #[must_use]
    pub fn segment_pattern_file_name(&self) -> String {
        format!("audio_{}_%05d.aac", self.index)
    }

    #[must_use]
    pub fn playlist_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(self.playlist_file_name())
    }

    #[must_use]
    pub fn segment_pattern_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(self.segment_pattern_file_name())
    }

    #[must_use]
    pub fn identity_component(&self) -> String {
        let default = if self.default { "1" } else { "0" };
        format!(
            "{}:{}:{}:{}",
            self.index,
            self.source_stream_index,
            default,
            self.language.as_deref().unwrap_or("und")
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsSubtitleRendition {
    pub index: usize,
    pub source_stream_index: u32,
    pub language: Option<String>,
}

impl HlsSubtitleRendition {
    #[must_use]
    pub fn new(index: usize, source_stream_index: u32, language: Option<String>) -> Self {
        Self {
            index,
            source_stream_index,
            language: language
                .map(|value| canonical_value(&value))
                .filter(|value| !value.is_empty()),
        }
    }

    #[must_use]
    pub fn playlist_file_name(&self) -> String {
        format!("subtitle_{}.m3u8", self.index)
    }

    #[must_use]
    pub fn segment_file_prefix(&self) -> String {
        format!("subtitle_{}_", self.index)
    }

    #[must_use]
    pub fn segment_pattern_file_name(&self) -> String {
        format!("subtitle_{}_%05d.vtt", self.index)
    }

    #[must_use]
    pub fn playlist_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(self.playlist_file_name())
    }

    #[must_use]
    pub fn segment_pattern_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(self.segment_pattern_file_name())
    }

    #[must_use]
    pub fn identity_component(&self) -> String {
        format!(
            "{}:{}:{}",
            self.index,
            self.source_stream_index,
            self.language.as_deref().unwrap_or("und")
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsSubtitleBurnInPlan {
    pub source_stream_index: u32,
    pub filter_stream_index: u32,
}

impl HlsSubtitleBurnInPlan {
    #[must_use]
    pub const fn new(source_stream_index: u32, filter_stream_index: u32) -> Self {
        Self {
            source_stream_index,
            filter_stream_index,
        }
    }

    pub fn selected_from_probe(
        probe: Option<&MediaProbeResult>,
        track_selection: TranscodeTrackSelection,
    ) -> Result<Option<Self>> {
        let Some(selected_subtitle_stream) = track_selection.subtitle_stream else {
            return Ok(None);
        };
        let Some(probe) = probe else {
            return Err(NakoError::InvalidInput {
                message: "hls subtitle burn-in requires media probe subtitle stream facts"
                    .to_owned(),
            });
        };

        for (filter_stream_index, stream) in probe
            .streams
            .iter()
            .filter(|stream| matches!(stream.kind, MediaStreamKind::Subtitle))
            .enumerate()
        {
            if stream.index == selected_subtitle_stream {
                validate_hls_burn_in_subtitle_stream(stream)?;
                let filter_stream_index =
                    u32::try_from(filter_stream_index).map_err(|_| NakoError::InvalidInput {
                        message: "hls subtitle burn-in stream index exceeds ffmpeg filter range"
                            .to_owned(),
                    })?;
                return Ok(Some(Self::new(stream.index, filter_stream_index)));
            }
        }

        Err(NakoError::InvalidInput {
            message: format!(
                "selected subtitle stream {selected_subtitle_stream} was not found in media probe"
            ),
        })
    }
}

fn validate_hls_burn_in_subtitle_stream(stream: &MediaStreamInfo) -> Result<()> {
    if !matches!(stream.kind, MediaStreamKind::Subtitle) {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle burn-in requires a subtitle stream".to_owned(),
        });
    }
    if stream
        .technical
        .origin
        .as_ref()
        .is_some_and(|origin| !matches!(origin, MediaStreamOrigin::Embedded))
    {
        return Err(NakoError::Unsupported(
            "hls subtitle burn-in supports only embedded subtitle streams",
        ));
    }

    let Some(codec) = stream
        .codec
        .as_deref()
        .map(str::trim)
        .filter(|codec| !codec.is_empty())
    else {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle burn-in requires subtitle codec facts".to_owned(),
        });
    };
    if HLS_BURN_IN_TEXT_SUBTITLE_CODECS
        .iter()
        .any(|supported| codec.eq_ignore_ascii_case(supported))
    {
        Ok(())
    } else {
        Err(NakoError::Unsupported(
            "hls subtitle burn-in supports only text subtitle codecs",
        ))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsMediaRenditionPlan {
    audios: Vec<HlsAudioRendition>,
    subtitles: Vec<HlsSubtitleRendition>,
}

impl HlsMediaRenditionPlan {
    pub fn from_audios(audios: Vec<HlsAudioRendition>) -> Result<Self> {
        Self::from_audio_and_subtitles(audios, Vec::new())
    }

    pub fn from_subtitles(subtitles: Vec<HlsSubtitleRendition>) -> Result<Self> {
        Self::from_audio_and_subtitles(Vec::new(), subtitles)
    }

    pub fn from_audio_and_subtitles(
        audios: Vec<HlsAudioRendition>,
        subtitles: Vec<HlsSubtitleRendition>,
    ) -> Result<Self> {
        let plan = Self { audios, subtitles };
        plan.validate_identity()?;
        Ok(plan)
    }

    pub fn with_audio_renditions(mut self, audios: Vec<HlsAudioRendition>) -> Result<Self> {
        self.audios = audios;
        self.validate_identity()?;
        Ok(self)
    }

    pub fn selected_from_probe(
        probe: Option<&MediaProbeResult>,
        source: Option<&TranscodePipelineSourceFacts>,
        track_selection: TranscodeTrackSelection,
    ) -> Result<Self> {
        Self::selected_from_source_facts(source, track_selection)?
            .with_audio_renditions(hls_audio_renditions_from_probe(probe, source))
    }

    pub fn selected_from_source_facts(
        source: Option<&TranscodePipelineSourceFacts>,
        track_selection: TranscodeTrackSelection,
    ) -> Result<Self> {
        let Some(source) = source else {
            return Ok(Self::default());
        };
        if track_selection.subtitle_stream.is_none() {
            return Ok(Self::default());
        }
        let Some(subtitle) = source.subtitle.as_ref() else {
            return Ok(Self::default());
        };

        Self::from_subtitles(vec![HlsSubtitleRendition::new(
            0,
            subtitle.index,
            subtitle.language.clone(),
        )])
    }

    pub fn from_identity_key(value: &str) -> Result<Self> {
        let Some(rest) = value.strip_prefix(HLS_MEDIA_RENDITIONS_IDENTITY_VERSION) else {
            return Err(NakoError::InvalidInput {
                message: "hls media rendition identity version is unsupported".to_owned(),
            });
        };
        let rest = rest
            .strip_prefix(';')
            .ok_or_else(|| NakoError::InvalidInput {
                message: "hls media rendition identity is missing components".to_owned(),
            })?;
        let mut audios = Vec::new();
        let mut subtitles = Vec::new();

        for component in rest.split(';') {
            if let Some(value) = component.strip_prefix("audios=") {
                audios = parse_audio_renditions(value)?;
            } else if let Some(value) = component.strip_prefix("subtitles=") {
                subtitles = parse_subtitle_renditions(value)?;
            }
        }

        let plan = Self { audios, subtitles };
        plan.validate_identity()?;
        Ok(plan)
    }

    #[must_use]
    pub fn identity_key(&self) -> Option<String> {
        if self.audios.is_empty() && self.subtitles.is_empty() {
            return None;
        }

        let mut components = Vec::new();
        if !self.audios.is_empty() {
            let audios = self
                .audios
                .iter()
                .map(HlsAudioRendition::identity_component)
                .collect::<Vec<_>>()
                .join("|");
            components.push(format!("audios={audios}"));
        }
        if !self.subtitles.is_empty() {
            let subtitles = self
                .subtitles
                .iter()
                .map(HlsSubtitleRendition::identity_component)
                .collect::<Vec<_>>()
                .join("|");
            components.push(format!("subtitles={subtitles}"));
        }

        Some(format!(
            "{HLS_MEDIA_RENDITIONS_IDENTITY_VERSION};{}",
            components.join(";")
        ))
    }

    #[must_use]
    pub fn audios(&self) -> &[HlsAudioRendition] {
        &self.audios
    }

    #[must_use]
    pub fn subtitles(&self) -> &[HlsSubtitleRendition] {
        &self.subtitles
    }

    #[must_use]
    pub fn has_audios(&self) -> bool {
        !self.audios.is_empty()
    }

    #[must_use]
    pub fn has_subtitles(&self) -> bool {
        !self.subtitles.is_empty()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.audios.is_empty() && self.subtitles.is_empty()
    }

    fn validate_identity(&self) -> Result<()> {
        let mut default_audio_count = 0;
        for (expected_index, audio) in self.audios.iter().enumerate() {
            if audio.index != expected_index {
                return Err(NakoError::InvalidInput {
                    message: "hls media rendition audio indexes must be dense".to_owned(),
                });
            }
            if audio.default {
                default_audio_count += 1;
            }
            if audio
                .language
                .as_deref()
                .is_some_and(invalid_media_rendition_language)
            {
                return Err(NakoError::InvalidInput {
                    message: "hls media rendition audio language is invalid".to_owned(),
                });
            }
        }
        if !self.audios.is_empty() && default_audio_count != 1 {
            return Err(NakoError::InvalidInput {
                message: "hls media rendition audios require exactly one default".to_owned(),
            });
        }

        for (expected_index, subtitle) in self.subtitles.iter().enumerate() {
            if subtitle.index != expected_index {
                return Err(NakoError::InvalidInput {
                    message: "hls media rendition subtitle indexes must be dense".to_owned(),
                });
            }
            if subtitle
                .language
                .as_deref()
                .is_some_and(invalid_media_rendition_language)
            {
                return Err(NakoError::InvalidInput {
                    message: "hls media rendition subtitle language is invalid".to_owned(),
                });
            }
        }

        Ok(())
    }
}

fn hls_audio_renditions_from_probe(
    probe: Option<&MediaProbeResult>,
    source: Option<&TranscodePipelineSourceFacts>,
) -> Vec<HlsAudioRendition> {
    let Some(probe) = probe else {
        return Vec::new();
    };
    let audio_streams = probe
        .streams
        .iter()
        .filter(|stream| matches!(stream.kind, MediaStreamKind::Audio))
        .collect::<Vec<_>>();
    if audio_streams.len() < 2 {
        return Vec::new();
    }

    let default_stream_index = source
        .and_then(|facts| facts.audio.as_ref())
        .map(|stream| stream.index)
        .unwrap_or(audio_streams[0].index);

    audio_streams
        .into_iter()
        .enumerate()
        .map(|(index, stream)| {
            HlsAudioRendition::new(
                index,
                stream.index,
                stream.language.clone(),
                stream.index == default_stream_index,
            )
        })
        .collect()
}

fn invalid_media_rendition_language(language: &str) -> bool {
    language.contains('|')
        || language.contains(';')
        || language.contains('~')
        || language.contains(':')
        || language.contains('=')
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsPlaybackGeneration {
    start_position_ms: u64,
}

impl HlsPlaybackGeneration {
    #[must_use]
    pub const fn from_start_position_ms(start_position_ms: u64) -> Self {
        Self { start_position_ms }
    }

    #[must_use]
    pub const fn start_position_ms(self) -> u64 {
        self.start_position_ms
    }

    #[must_use]
    pub const fn is_default_start(self) -> bool {
        self.start_position_ms == 0
    }

    pub fn from_identity_key(value: &str) -> Result<Self> {
        let Some(rest) = value.strip_prefix(HLS_PLAYBACK_GENERATION_IDENTITY_VERSION) else {
            return Err(NakoError::InvalidInput {
                message: "hls playback generation identity version is unsupported".to_owned(),
            });
        };
        let rest = rest
            .strip_prefix(";start_ms=")
            .ok_or_else(|| NakoError::InvalidInput {
                message: "hls playback generation identity is missing start position".to_owned(),
            })?;
        let start_position_ms = rest.parse::<u64>().map_err(|_| NakoError::InvalidInput {
            message: "hls playback generation identity has invalid start position".to_owned(),
        })?;

        Ok(Self { start_position_ms })
    }

    #[must_use]
    pub fn identity_key(self) -> Option<String> {
        (!self.is_default_start()).then(|| {
            format!(
                "{HLS_PLAYBACK_GENERATION_IDENTITY_VERSION};start_ms={}",
                self.start_position_ms
            )
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRequestVariantPlan {
    pub adaptive_ladder: Option<HlsAdaptiveLadderPlan>,
    pub media_renditions: HlsMediaRenditionPlan,
    pub playback_generation: HlsPlaybackGeneration,
}

impl HlsRequestVariantPlan {
    #[must_use]
    pub fn new(
        adaptive_ladder: Option<HlsAdaptiveLadderPlan>,
        media_renditions: HlsMediaRenditionPlan,
    ) -> Self {
        Self {
            adaptive_ladder,
            media_renditions,
            playback_generation: HlsPlaybackGeneration::default(),
        }
    }

    #[must_use]
    pub const fn with_playback_generation(
        mut self,
        playback_generation: HlsPlaybackGeneration,
    ) -> Self {
        self.playback_generation = playback_generation;
        self
    }

    pub fn from_identity_key(value: &str) -> Result<Self> {
        if value.starts_with(HLS_ADAPTIVE_LADDER_IDENTITY_VERSION) {
            return Ok(Self {
                adaptive_ladder: Some(HlsAdaptiveLadderPlan::from_identity_key(value)?),
                media_renditions: HlsMediaRenditionPlan::default(),
                playback_generation: HlsPlaybackGeneration::default(),
            });
        }
        if value.starts_with(HLS_MEDIA_RENDITIONS_IDENTITY_VERSION) {
            return Ok(Self {
                adaptive_ladder: None,
                media_renditions: HlsMediaRenditionPlan::from_identity_key(value)?,
                playback_generation: HlsPlaybackGeneration::default(),
            });
        }
        if value.starts_with(HLS_PLAYBACK_GENERATION_IDENTITY_VERSION) {
            return Ok(Self {
                adaptive_ladder: None,
                media_renditions: HlsMediaRenditionPlan::default(),
                playback_generation: HlsPlaybackGeneration::from_identity_key(value)?,
            });
        }

        let Some(rest) = value.strip_prefix(HLS_REQUEST_VARIANT_IDENTITY_VERSION) else {
            return Err(NakoError::InvalidInput {
                message: "hls request variant identity version is unsupported".to_owned(),
            });
        };
        let rest = rest
            .strip_prefix(";components=")
            .ok_or_else(|| NakoError::InvalidInput {
                message: "hls request variant identity is missing components".to_owned(),
            })?;
        let mut plan = Self::default();
        for component in rest.split('~') {
            plan.apply_identity_component(component)?;
        }
        Ok(plan)
    }

    #[must_use]
    pub fn identity_key(&self) -> Option<String> {
        let mut components = Vec::new();
        if let Some(ladder) = self.adaptive_ladder.as_ref() {
            components.push(ladder.identity_key());
        }
        if let Some(media) = self.media_renditions.identity_key() {
            components.push(media);
        }
        if let Some(main_output) = hls_main_output_identity_key(&self.media_renditions) {
            components.push(main_output);
        }
        if let Some(generation) = self.playback_generation.identity_key() {
            components.push(generation);
        }

        match components.len() {
            0 => None,
            1 => components.into_iter().next(),
            _ => Some(format!(
                "{HLS_REQUEST_VARIANT_IDENTITY_VERSION};components={}",
                components.join("~")
            )),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.adaptive_ladder.is_none()
            && self.media_renditions.is_empty()
            && self.playback_generation.is_default_start()
    }

    fn apply_identity_component(&mut self, component: &str) -> Result<()> {
        if component.starts_with(HLS_ADAPTIVE_LADDER_IDENTITY_VERSION) {
            self.adaptive_ladder = Some(HlsAdaptiveLadderPlan::from_identity_key(component)?);
            return Ok(());
        }
        if component.starts_with(HLS_MEDIA_RENDITIONS_IDENTITY_VERSION) {
            self.media_renditions = HlsMediaRenditionPlan::from_identity_key(component)?;
            return Ok(());
        }
        if component.starts_with(HLS_MAIN_OUTPUT_IDENTITY_VERSION) {
            validate_hls_main_output_identity(component)?;
            return Ok(());
        }
        if component.starts_with(HLS_PLAYBACK_GENERATION_IDENTITY_VERSION) {
            self.playback_generation = HlsPlaybackGeneration::from_identity_key(component)?;
            return Ok(());
        }

        Err(NakoError::InvalidInput {
            message: "hls request variant identity contains an unknown component".to_owned(),
        })
    }
}

fn hls_main_output_identity_key(media_renditions: &HlsMediaRenditionPlan) -> Option<String> {
    media_renditions
        .has_audios()
        .then(|| format!("{HLS_MAIN_OUTPUT_IDENTITY_VERSION};main_audio=false"))
}

fn validate_hls_main_output_identity(value: &str) -> Result<()> {
    let Some(rest) = value.strip_prefix(HLS_MAIN_OUTPUT_IDENTITY_VERSION) else {
        return Err(NakoError::InvalidInput {
            message: "hls main output identity version is unsupported".to_owned(),
        });
    };
    let rest = rest
        .strip_prefix(";main_audio=")
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls main output identity is missing audio shape".to_owned(),
        })?;
    if rest == "false" {
        return Ok(());
    }

    Err(NakoError::InvalidInput {
        message: "hls main output identity has unsupported audio shape".to_owned(),
    })
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsAdaptiveLadderSource {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_bitrate: Option<u64>,
    pub has_audio: Option<bool>,
}

impl HlsAdaptiveLadderSource {
    #[must_use]
    pub fn from_source_facts(source: Option<&TranscodePipelineSourceFacts>) -> Self {
        let Some(source) = source else {
            return Self {
                has_audio: None,
                ..Self::default()
            };
        };
        let video = source.video.as_ref();

        Self {
            width: video.and_then(|stream| stream.width),
            height: video.and_then(|stream| stream.height),
            video_bitrate: video.and_then(|stream| stream.bit_rate),
            has_audio: Some(source.audio.is_some()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsAdaptiveLadderPlan {
    renditions: Vec<HlsRendition>,
    has_audio: bool,
}

impl Default for HlsAdaptiveLadderPlan {
    fn default() -> Self {
        Self {
            renditions: HlsRendition::default_adaptive_ladder(),
            has_audio: true,
        }
    }
}

impl HlsAdaptiveLadderPlan {
    #[must_use]
    pub fn from_source_facts(
        source: Option<&TranscodePipelineSourceFacts>,
        constraints: TranscodeOutputConstraints,
    ) -> Self {
        Self::from_source(
            HlsAdaptiveLadderSource::from_source_facts(source),
            constraints,
        )
    }

    #[must_use]
    pub fn from_source(
        source: HlsAdaptiveLadderSource,
        constraints: TranscodeOutputConstraints,
    ) -> Self {
        let has_audio = source.has_audio.unwrap_or(true);
        let candidates = if source.width.is_none()
            && source.height.is_none()
            && constraints.max_width.is_none()
            && constraints.max_height.is_none()
        {
            HLS_ADAPTIVE_DEFAULT_LADDER_CANDIDATES
        } else {
            HLS_ADAPTIVE_LADDER_CANDIDATES
        };
        let width_limit = min_optional_u32(source.width, constraints.max_width);
        let height_limit = min_optional_u32(source.height, constraints.max_height);
        let bitrate_cap = min_optional_u64(source.video_bitrate, constraints.max_video_bitrate);
        let mut renditions = Vec::new();

        for &(width, height, base_video_bitrate) in candidates {
            if width_limit.is_some_and(|limit| width > limit)
                || height_limit.is_some_and(|limit| height > limit)
            {
                continue;
            }

            renditions.push(HlsRendition::new(
                renditions.len(),
                width,
                height,
                capped_video_bitrate(base_video_bitrate, bitrate_cap),
                HLS_ADAPTIVE_AUDIO_BITRATE,
            ));
        }

        if renditions.is_empty() {
            let (width, height) = fallback_ladder_dimensions(source, constraints);
            renditions.push(HlsRendition::new(
                0,
                width,
                height,
                capped_video_bitrate(fallback_video_bitrate(height), bitrate_cap),
                HLS_ADAPTIVE_AUDIO_BITRATE,
            ));
        }

        Self {
            renditions,
            has_audio,
        }
    }

    pub fn from_identity_key(value: &str) -> Result<Self> {
        let Some(rest) = value.strip_prefix(HLS_ADAPTIVE_LADDER_IDENTITY_VERSION) else {
            return Err(NakoError::InvalidInput {
                message: "hls adaptive ladder identity version is unsupported".to_owned(),
            });
        };
        let rest = rest
            .strip_prefix(';')
            .ok_or_else(|| NakoError::InvalidInput {
                message: "hls adaptive ladder identity is missing components".to_owned(),
            })?;
        let mut has_audio = None;
        let mut renditions = None;

        for component in rest.split(';') {
            if let Some(value) = component.strip_prefix("audio=") {
                has_audio = Some(match value {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(NakoError::InvalidInput {
                            message: "hls adaptive ladder identity has invalid audio flag"
                                .to_owned(),
                        });
                    }
                });
            } else if let Some(value) = component.strip_prefix("renditions=") {
                renditions = Some(parse_ladder_renditions(value)?);
            }
        }

        let plan = Self {
            renditions: renditions.ok_or_else(|| NakoError::InvalidInput {
                message: "hls adaptive ladder identity is missing renditions".to_owned(),
            })?,
            has_audio: has_audio.ok_or_else(|| NakoError::InvalidInput {
                message: "hls adaptive ladder identity is missing audio presence".to_owned(),
            })?,
        };
        plan.validate_identity()?;
        Ok(plan)
    }

    #[must_use]
    pub fn identity_key(&self) -> String {
        let renditions = self
            .renditions
            .iter()
            .map(|rendition| {
                format!(
                    "{}:{}x{}@{}+{}",
                    rendition.index,
                    rendition.width,
                    rendition.height,
                    rendition.video_bitrate,
                    rendition.audio_bitrate
                )
            })
            .collect::<Vec<_>>()
            .join("|");

        format!(
            "{HLS_ADAPTIVE_LADDER_IDENTITY_VERSION};audio={};renditions={renditions}",
            self.has_audio
        )
    }

    #[must_use]
    pub fn renditions(&self) -> &[HlsRendition] {
        &self.renditions
    }

    #[must_use]
    pub const fn has_audio(&self) -> bool {
        self.has_audio
    }

    fn validate_identity(&self) -> Result<()> {
        if self.renditions.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls adaptive ladder identity requires at least one rendition".to_owned(),
            });
        }

        for (expected_index, rendition) in self.renditions.iter().enumerate() {
            if rendition.index != expected_index
                || rendition.width == 0
                || rendition.height == 0
                || rendition.video_bitrate == 0
                || rendition.audio_bitrate == 0
            {
                return Err(NakoError::InvalidInput {
                    message: "hls adaptive ladder identity has invalid renditions".to_owned(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsArtifactManifest {
    pub output_dir: PathBuf,
    pub primary_playlist_path: PathBuf,
    pub media_segment_pattern: PathBuf,
    pub variant_playlist_pattern: Option<PathBuf>,
    pub renditions: Vec<HlsRendition>,
    pub has_audio: bool,
    pub media_renditions: HlsMediaRenditionPlan,
    pub output: HlsOutputRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsArtifactDescriptor {
    pub path: PathBuf,
    pub content_type: &'static str,
    pub cleanup_candidate: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsArtifactSpec {
    output: HlsOutputRequirement,
    request_variant: HlsRequestVariantPlan,
}

impl HlsArtifactSpec {
    pub fn from_persisted_request_key(request_key: &str) -> Result<Self> {
        let profile_key = profile_identity_key_from_request_key(request_key)?;
        let output = hls_output_requirement_from_profile_identity_key(&profile_key)?;
        let request_variant = hls_request_variant_from_request_key(request_key)?;

        Ok(Self {
            output,
            request_variant,
        })
    }

    #[must_use]
    pub const fn output(&self) -> HlsOutputRequirement {
        self.output
    }

    #[must_use]
    pub const fn request_variant(&self) -> &HlsRequestVariantPlan {
        &self.request_variant
    }

    pub fn manifest_for_primary_playlist(
        &self,
        primary_playlist_path: impl Into<PathBuf>,
    ) -> Result<HlsArtifactManifest> {
        let primary_playlist_path = primary_playlist_path.into();
        let output_dir = hls_output_dir_for_primary_playlist(&primary_playlist_path)?;

        let manifest = if self.output.variant_policy == HlsVariantPolicy::Adaptive {
            let ladder_plan = self
                .request_variant
                .adaptive_ladder
                .clone()
                .unwrap_or_default();
            HlsArtifactManifest::adaptive_fmp4_with_audio(
                output_dir,
                primary_playlist_path,
                ladder_plan.renditions().to_vec(),
                ladder_plan.has_audio(),
            )?
        } else {
            let segment_pattern = output_dir.join(format!(
                "segment_%05d.{}",
                self.output.segment_container.segment_extension()
            ));
            HlsArtifactManifest::single_variant(
                output_dir,
                primary_playlist_path,
                segment_pattern,
                self.output,
            )?
        };

        manifest.with_media_renditions(self.request_variant.media_renditions.clone())
    }
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
            has_audio: true,
            media_renditions: HlsMediaRenditionPlan::default(),
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
        Self::adaptive_fmp4_with_audio(output_dir, primary_playlist_path, renditions, true)
    }

    pub fn adaptive_fmp4_with_audio(
        output_dir: impl Into<PathBuf>,
        primary_playlist_path: impl Into<PathBuf>,
        renditions: Vec<HlsRendition>,
        has_audio: bool,
    ) -> Result<Self> {
        let output_dir = output_dir.into();
        let manifest = Self {
            primary_playlist_path: primary_playlist_path.into(),
            media_segment_pattern: output_dir.join(HLS_ADAPTIVE_FMP4_SEGMENT_PATTERN),
            variant_playlist_pattern: Some(output_dir.join(HLS_ADAPTIVE_VARIANT_PLAYLIST_PATTERN)),
            renditions,
            has_audio,
            media_renditions: HlsMediaRenditionPlan::default(),
            output: HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: crate::policy::HlsSegmentContainer::Fmp4,
            },
            output_dir,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn with_media_renditions(
        mut self,
        media_renditions: HlsMediaRenditionPlan,
    ) -> Result<Self> {
        self.media_renditions = media_renditions;
        self.validate()?;
        Ok(self)
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
    pub const fn has_audio(&self) -> bool {
        self.has_audio
    }

    #[must_use]
    pub fn main_output_has_audio(&self) -> bool {
        self.has_audio && !self.media_renditions.has_audios()
    }

    #[must_use]
    pub fn media_renditions(&self) -> &HlsMediaRenditionPlan {
        &self.media_renditions
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

        if let Some(artifact) = self.audio_artifact_for_name(artifact_name, &path) {
            return Ok(artifact);
        }

        if let Some(artifact) = self.subtitle_artifact_for_name(artifact_name, &path) {
            return Ok(artifact);
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

        if hls_artifact_name_matches_sequence_pattern(&self.media_segment_pattern, artifact_name) {
            return Ok(HlsArtifactDescriptor {
                path,
                content_type: self.output.segment_container.segment_content_type(),
                cleanup_candidate: true,
            });
        }

        Err(NakoError::NotFound {
            entity: "hls_artifact",
            id: artifact_name.to_owned(),
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

            let segment_pattern = format!(
                "{}%05d.{}",
                rendition.segment_file_prefix(),
                self.output.segment_container.segment_extension()
            );
            if hls_artifact_name_matches_sequence_pattern(&segment_pattern, artifact_name) {
                return Ok(HlsArtifactDescriptor {
                    path,
                    content_type: self.output.segment_container.segment_content_type(),
                    cleanup_candidate: true,
                });
            }
        }

        Err(NakoError::NotFound {
            entity: "hls_artifact",
            id: artifact_name.to_owned(),
        })
    }

    fn audio_artifact_for_name(
        &self,
        artifact_name: &str,
        path: &Path,
    ) -> Option<HlsArtifactDescriptor> {
        for audio in self.media_renditions.audios() {
            if audio.playlist_file_name() == artifact_name {
                return Some(HlsArtifactDescriptor {
                    path: path.to_path_buf(),
                    content_type: "application/vnd.apple.mpegurl",
                    cleanup_candidate: false,
                });
            }

            if hls_artifact_name_matches_sequence_pattern(
                audio.segment_pattern_file_name(),
                artifact_name,
            ) {
                return Some(HlsArtifactDescriptor {
                    path: path.to_path_buf(),
                    content_type: "audio/aac",
                    cleanup_candidate: true,
                });
            }
        }

        None
    }

    fn subtitle_artifact_for_name(
        &self,
        artifact_name: &str,
        path: &Path,
    ) -> Option<HlsArtifactDescriptor> {
        for subtitle in self.media_renditions.subtitles() {
            if subtitle.playlist_file_name() == artifact_name {
                return Some(HlsArtifactDescriptor {
                    path: path.to_path_buf(),
                    content_type: "application/vnd.apple.mpegurl",
                    cleanup_candidate: false,
                });
            }

            if hls_artifact_name_matches_sequence_pattern(
                subtitle.segment_pattern_file_name(),
                artifact_name,
            ) {
                return Some(HlsArtifactDescriptor {
                    path: path.to_path_buf(),
                    content_type: "text/vtt",
                    cleanup_candidate: true,
                });
            }
        }

        None
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
        self.media_renditions.validate_identity()?;

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

fn hls_output_dir_for_primary_playlist(playlist_path: &Path) -> Result<PathBuf> {
    playlist_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            NakoError::storage_security_violation(
                playlist_path.display().to_string(),
                "hls playlist path does not have a parent directory",
            )
        })
}

fn hls_artifact_name_matches_sequence_pattern(
    sequence_pattern: impl AsRef<Path>,
    artifact_name: &str,
) -> bool {
    let Some(pattern_name) = sequence_pattern
        .as_ref()
        .file_name()
        .and_then(|value| value.to_str())
    else {
        return false;
    };
    let Some((prefix, suffix, min_width)) = hls_sequence_pattern_parts(pattern_name) else {
        return false;
    };

    if artifact_name.len() < prefix.len().saturating_add(suffix.len())
        || !artifact_name.starts_with(prefix)
        || !artifact_name.ends_with(suffix)
    {
        return false;
    }

    let sequence_end = artifact_name.len() - suffix.len();
    let sequence = &artifact_name[prefix.len()..sequence_end];
    sequence.len() >= min_width && sequence.bytes().all(|value| value.is_ascii_digit())
}

fn hls_sequence_pattern_parts(pattern_name: &str) -> Option<(&str, &str, usize)> {
    let bytes = pattern_name.as_bytes();
    let mut search_at = 0;

    while let Some(relative_start) = pattern_name[search_at..].find('%') {
        let placeholder_start = search_at + relative_start;
        let mut cursor = placeholder_start + 1;
        let width_start = cursor;

        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }

        if bytes.get(cursor) == Some(&b'd') {
            let min_width = if cursor == width_start {
                1
            } else {
                pattern_name[width_start..cursor]
                    .parse::<usize>()
                    .unwrap_or(1)
                    .max(1)
            };
            return Some((
                &pattern_name[..placeholder_start],
                &pattern_name[cursor + 1..],
                min_width,
            ));
        }

        search_at = placeholder_start + 1;
    }

    None
}

fn profile_identity_key_from_request_key(request_key: &str) -> Result<String> {
    if !request_key.starts_with(TRANSCODE_REQUEST_IDENTITY_VERSION) {
        return Err(NakoError::InvalidInput {
            message: "transcode request identity version is unsupported".to_owned(),
        });
    }

    let value = request_identity_component(request_key, "profile").ok_or_else(|| {
        NakoError::InvalidInput {
            message: "transcode request identity is missing profile component".to_owned(),
        }
    })?;
    percent_decode_identity_component(value).ok_or_else(|| NakoError::InvalidInput {
        message: "transcode request profile identity is not valid percent encoding".to_owned(),
    })
}

fn hls_request_variant_from_request_key(request_key: &str) -> Result<HlsRequestVariantPlan> {
    let Some(value) = request_identity_component(request_key, "request_variant") else {
        return Ok(HlsRequestVariantPlan::default());
    };
    let value =
        percent_decode_identity_component(value).ok_or_else(|| NakoError::InvalidInput {
            message: "hls request variant identity is not valid percent encoding".to_owned(),
        })?;

    HlsRequestVariantPlan::from_identity_key(&value)
}

fn request_identity_component<'a>(request_key: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(";{name}=");
    let start = request_key.find(&needle)? + needle.len();
    let rest = &request_key[start..];

    if name == "profile" {
        return rest
            .find(";request_variant=")
            .map_or(Some(rest), |end| Some(&rest[..end]));
    }

    rest.find(';').map_or(Some(rest), |end| Some(&rest[..end]))
}

fn hls_output_requirement_from_profile_identity_key(
    profile_key: &str,
) -> Result<HlsOutputRequirement> {
    if !profile_key.starts_with(TRANSCODE_PROFILE_IDENTITY_VERSION) {
        return Err(NakoError::InvalidInput {
            message: "transcode profile identity version is unsupported".to_owned(),
        });
    }

    let kind =
        profile_identity_component(profile_key, "kind").ok_or_else(|| NakoError::InvalidInput {
            message: "transcode profile identity is missing kind component".to_owned(),
        })?;
    if !matches!(kind, "hls_single_variant" | "hls_adaptive") {
        return Err(NakoError::InvalidInput {
            message: "transcode profile identity is not an hls profile".to_owned(),
        });
    }

    let variant_policy = match profile_identity_component(profile_key, "hls_variant") {
        Some("single_variant") => HlsVariantPolicy::SingleVariant,
        Some("adaptive") => HlsVariantPolicy::Adaptive,
        Some(_) => {
            return Err(NakoError::InvalidInput {
                message: "transcode profile identity has unsupported hls variant policy".to_owned(),
            });
        }
        None => {
            return Err(NakoError::InvalidInput {
                message: "transcode profile identity is missing hls variant policy".to_owned(),
            });
        }
    };
    let segment_container = match profile_identity_component(profile_key, "hls_segment") {
        Some("mpeg_ts") => crate::policy::HlsSegmentContainer::MpegTs,
        Some("fmp4") => crate::policy::HlsSegmentContainer::Fmp4,
        Some(_) => {
            return Err(NakoError::InvalidInput {
                message: "transcode profile identity has unsupported hls segment container"
                    .to_owned(),
            });
        }
        None => {
            return Err(NakoError::InvalidInput {
                message: "transcode profile identity is missing hls segment container".to_owned(),
            });
        }
    };

    Ok(HlsOutputRequirement {
        variant_policy,
        segment_container,
    })
}

fn profile_identity_component<'a>(profile_key: &'a str, name: &str) -> Option<&'a str> {
    profile_key
        .split(';')
        .find_map(|component| component.strip_prefix(&format!("{name}=")))
}

fn percent_decode_identity_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = String::with_capacity(value.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).copied().and_then(hex_value)?;
            let low = bytes.get(index + 2).copied().and_then(hex_value)?;
            decoded.push((high << 4 | low) as char);
            index += 3;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }

    Some(decoded)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn capped_video_bitrate(base: u64, cap: Option<u64>) -> u64 {
    cap.map_or(base, |cap| base.min(cap)).max(1)
}

fn fallback_ladder_dimensions(
    source: HlsAdaptiveLadderSource,
    constraints: TranscodeOutputConstraints,
) -> (u32, u32) {
    (
        normalized_dimension(min_optional_u32(source.width, constraints.max_width).unwrap_or(1280)),
        normalized_dimension(
            min_optional_u32(source.height, constraints.max_height).unwrap_or(720),
        ),
    )
}

fn fallback_video_bitrate(height: u32) -> u64 {
    if height >= 1080 {
        6_000_000
    } else if height >= 720 {
        3_000_000
    } else if height >= 480 {
        1_200_000
    } else {
        800_000
    }
}

fn min_optional_u32(left: Option<u32>, right: Option<u32>) -> Option<u32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn min_optional_u64(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn normalized_dimension(value: u32) -> u32 {
    value.max(2) & !1
}

fn parse_ladder_renditions(value: &str) -> Result<Vec<HlsRendition>> {
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "hls adaptive ladder identity has no renditions".to_owned(),
        });
    }

    value
        .split('|')
        .map(parse_ladder_rendition)
        .collect::<Result<Vec<_>>>()
}

fn parse_ladder_rendition(value: &str) -> Result<HlsRendition> {
    let (index, rest) = value
        .split_once(':')
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls adaptive ladder rendition is missing index".to_owned(),
        })?;
    let (dimensions, bitrates) = rest
        .split_once('@')
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls adaptive ladder rendition is missing bitrate".to_owned(),
        })?;
    let (width, height) = dimensions
        .split_once('x')
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls adaptive ladder rendition is missing dimensions".to_owned(),
        })?;
    let (video_bitrate, audio_bitrate) =
        bitrates
            .split_once('+')
            .ok_or_else(|| NakoError::InvalidInput {
                message: "hls adaptive ladder rendition is missing audio bitrate".to_owned(),
            })?;

    Ok(HlsRendition::new(
        parse_usize_component(index, "index")?,
        parse_u32_component(width, "width")?,
        parse_u32_component(height, "height")?,
        parse_u64_component(video_bitrate, "video bitrate")?,
        parse_u64_component(audio_bitrate, "audio bitrate")?,
    ))
}

fn parse_audio_renditions(value: &str) -> Result<Vec<HlsAudioRendition>> {
    if value.is_empty() || value == "none" {
        return Ok(Vec::new());
    }

    value
        .split('|')
        .map(parse_audio_rendition)
        .collect::<Result<Vec<_>>>()
}

fn parse_audio_rendition(value: &str) -> Result<HlsAudioRendition> {
    let mut parts = value.split(':');
    let Some(index) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls audio rendition is missing index".to_owned(),
        });
    };
    let Some(source_stream_index) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls audio rendition is missing source stream".to_owned(),
        });
    };
    let Some(default) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls audio rendition is missing default flag".to_owned(),
        });
    };
    let Some(language) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls audio rendition is missing language".to_owned(),
        });
    };
    if parts.next().is_some() {
        return Err(NakoError::InvalidInput {
            message: "hls audio rendition has too many components".to_owned(),
        });
    }

    let default = match default {
        "1" | "true" => true,
        "0" | "false" => false,
        _ => {
            return Err(NakoError::InvalidInput {
                message: "hls audio rendition default flag is invalid".to_owned(),
            });
        }
    };

    Ok(HlsAudioRendition::new(
        parse_usize_component(index, "audio index")?,
        parse_u32_component(source_stream_index, "audio source stream")?,
        (language != "und").then(|| language.to_owned()),
        default,
    ))
}

fn parse_subtitle_renditions(value: &str) -> Result<Vec<HlsSubtitleRendition>> {
    if value.is_empty() || value == "none" {
        return Ok(Vec::new());
    }

    value
        .split('|')
        .map(parse_subtitle_rendition)
        .collect::<Result<Vec<_>>>()
}

fn parse_subtitle_rendition(value: &str) -> Result<HlsSubtitleRendition> {
    let mut parts = value.split(':');
    let Some(index) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle rendition is missing index".to_owned(),
        });
    };
    let Some(source_stream_index) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle rendition is missing source stream".to_owned(),
        });
    };
    let Some(language) = parts.next() else {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle rendition is missing language".to_owned(),
        });
    };
    if parts.next().is_some() {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle rendition has too many components".to_owned(),
        });
    }

    Ok(HlsSubtitleRendition::new(
        parse_usize_component(index, "subtitle index")?,
        parse_u32_component(source_stream_index, "subtitle source stream")?,
        (language != "und").then(|| language.to_owned()),
    ))
}

fn parse_usize_component(value: &str, name: &'static str) -> Result<usize> {
    value.parse::<usize>().map_err(|_| NakoError::InvalidInput {
        message: format!("hls adaptive ladder rendition has invalid {name}"),
    })
}

fn parse_u32_component(value: &str, name: &'static str) -> Result<u32> {
    value.parse::<u32>().map_err(|_| NakoError::InvalidInput {
        message: format!("hls adaptive ladder rendition has invalid {name}"),
    })
}

fn parse_u64_component(value: &str, name: &'static str) -> Result<u64> {
    value.parse::<u64>().map_err(|_| NakoError::InvalidInput {
        message: format!("hls adaptive ladder rendition has invalid {name}"),
    })
}

fn canonical_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nako_core::MediaStreamTechnicalFacts;

    use crate::policy::HlsSegmentContainer;

    use super::*;

    #[test]
    fn hls_subtitle_burn_in_plan_uses_subtitle_ordinal_not_source_stream_index() {
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                media_stream(0, MediaStreamKind::Video, "h264"),
                media_stream(2, MediaStreamKind::Subtitle, "subrip"),
                media_stream(3, MediaStreamKind::Audio, "aac"),
                media_stream(5, MediaStreamKind::Subtitle, "webvtt"),
            ],
        };

        let plan = HlsSubtitleBurnInPlan::selected_from_probe(
            Some(&probe),
            TranscodeTrackSelection {
                audio_stream: None,
                subtitle_stream: Some(5),
            },
        )
        .unwrap();

        assert_eq!(plan, Some(HlsSubtitleBurnInPlan::new(5, 1)));
    }

    #[test]
    fn hls_subtitle_burn_in_plan_rejects_image_subtitle_codec() {
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![media_stream(
                2,
                MediaStreamKind::Subtitle,
                "hdmv_pgs_subtitle",
            )],
        };

        let err = HlsSubtitleBurnInPlan::selected_from_probe(
            Some(&probe),
            TranscodeTrackSelection {
                audio_stream: None,
                subtitle_stream: Some(2),
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("text subtitle codecs"));
    }

    #[test]
    fn hls_subtitle_burn_in_plan_rejects_external_subtitle_origin() {
        let mut subtitle = media_stream(2, MediaStreamKind::Subtitle, "subrip");
        subtitle.technical.origin = Some(MediaStreamOrigin::External);
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![subtitle],
        };

        let err = HlsSubtitleBurnInPlan::selected_from_probe(
            Some(&probe),
            TranscodeTrackSelection {
                audio_stream: None,
                subtitle_stream: Some(2),
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("embedded subtitle streams"));
    }

    #[test]
    fn hls_request_variant_identity_round_trips_full_artifact_authority_components() {
        let ladder = single_rendition_ladder(false);
        let media = media_renditions();
        let plan = HlsRequestVariantPlan::new(Some(ladder), media)
            .with_playback_generation(HlsPlaybackGeneration::from_start_position_ms(90_000));

        let identity = plan.identity_key().unwrap();

        assert!(identity.starts_with("hls-request-variant:v1;components="));
        assert!(identity.contains("hls-adaptive-ladder:v1"));
        assert!(identity.contains("hls-media-renditions:v1"));
        assert!(identity.contains("hls-main-output:v1;main_audio=false"));
        assert!(identity.contains("hls-playback-generation:v1;start_ms=90000"));

        let reconstructed = HlsRequestVariantPlan::from_identity_key(&identity).unwrap();

        assert_eq!(reconstructed, plan);
        assert_eq!(
            reconstructed.identity_key().as_deref(),
            Some(identity.as_str())
        );
    }

    #[test]
    fn hls_artifact_spec_reconstructs_manifest_allow_list_from_request_variant_identity() {
        let ladder = single_rendition_ladder(false);
        let media = media_renditions();
        let request_variant = HlsRequestVariantPlan::new(Some(ladder.clone()), media.clone())
            .with_playback_generation(HlsPlaybackGeneration::from_start_position_ms(90_000));
        let request_key = hls_request_key(
            "hls_adaptive",
            "adaptive",
            "fmp4",
            Some(&request_variant.identity_key().unwrap()),
        );

        let spec = HlsArtifactSpec::from_persisted_request_key(&request_key).unwrap();

        assert_eq!(
            spec.output(),
            HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::Adaptive,
                segment_container: HlsSegmentContainer::Fmp4,
            }
        );
        assert_eq!(spec.request_variant(), &request_variant);
        assert_eq!(
            spec.request_variant()
                .playback_generation
                .start_position_ms(),
            90_000
        );

        let manifest = spec
            .manifest_for_primary_playlist(PathBuf::from("hls/master.m3u8"))
            .unwrap();

        assert_eq!(manifest.renditions(), ladder.renditions());
        assert_eq!(manifest.media_renditions(), &media);
        assert!(!manifest.has_audio());
        assert!(!manifest.main_output_has_audio());
        assert_artifact(
            &manifest,
            "master.m3u8",
            "application/vnd.apple.mpegurl",
            false,
        );
        assert_artifact(
            &manifest,
            "variant_0.m3u8",
            "application/vnd.apple.mpegurl",
            false,
        );
        assert_artifact(&manifest, "variant_0_init.mp4", "video/mp4", false);
        assert_artifact(&manifest, "variant_0_segment_00042.m4s", "video/mp4", true);
        assert_artifact(
            &manifest,
            "audio_0.m3u8",
            "application/vnd.apple.mpegurl",
            false,
        );
        assert_artifact(&manifest, "audio_0_00042.aac", "audio/aac", true);
        assert_artifact(
            &manifest,
            "subtitle_0.m3u8",
            "application/vnd.apple.mpegurl",
            false,
        );
        assert_artifact(&manifest, "subtitle_0_00042.vtt", "text/vtt", true);
        assert_not_manifest_artifact(&manifest, "variant_1_segment_00042.m4s");
        assert_not_manifest_artifact(&manifest, "variant_0_segment_final.m4s");
        assert_not_manifest_artifact(&manifest, "audio_0_final.aac");
        assert_not_manifest_artifact(&manifest, "subtitle_0_final.vtt");
        assert_not_manifest_artifact(&manifest, "init.mp4");
    }

    #[test]
    fn hls_artifact_manifest_serves_only_sequence_instances_from_manifest_patterns() {
        let ts = HlsArtifactManifest::single_variant(
            "hls",
            "hls/playlist.m3u8",
            "hls/segment_%05d.ts",
            HlsOutputRequirement::default(),
        )
        .unwrap();
        let fmp4 = HlsArtifactManifest::single_variant(
            "hls",
            "hls/playlist.m3u8",
            "hls/segment_%05d.m4s",
            HlsOutputRequirement {
                variant_policy: HlsVariantPolicy::SingleVariant,
                segment_container: HlsSegmentContainer::Fmp4,
            },
        )
        .unwrap()
        .with_media_renditions(media_renditions())
        .unwrap();

        assert_artifact(&ts, "playlist.m3u8", "application/vnd.apple.mpegurl", false);
        assert_artifact(&ts, "segment_00000.ts", "video/mp2t", true);
        assert_artifact(&ts, "segment_100000.ts", "video/mp2t", true);
        assert_not_manifest_artifact(&ts, "segment_1.ts");
        assert_not_manifest_artifact(&ts, "segment_final.ts");
        assert_not_manifest_artifact(&ts, "movie.ts");

        assert_artifact(&fmp4, "init.mp4", "video/mp4", false);
        assert_artifact(&fmp4, "segment_00000.m4s", "video/mp4", true);
        assert_artifact(&fmp4, "audio_0_00000.aac", "audio/aac", true);
        assert_artifact(&fmp4, "subtitle_0_00000.vtt", "text/vtt", true);
        assert_not_manifest_artifact(&fmp4, "segment_1.m4s");
        assert_not_manifest_artifact(&fmp4, "movie.m4s");
        assert_not_manifest_artifact(&fmp4, "audio_0_final.aac");
        assert_not_manifest_artifact(&fmp4, "subtitle_0_final.vtt");
        assert_not_manifest_artifact(&fmp4, "../segment_00000.m4s");
    }

    fn media_stream(index: u32, kind: MediaStreamKind, codec: &str) -> MediaStreamInfo {
        MediaStreamInfo {
            index,
            kind,
            codec: Some(codec.to_owned()),
            language: None,
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
            technical: MediaStreamTechnicalFacts::default(),
        }
    }

    fn single_rendition_ladder(has_audio: bool) -> HlsAdaptiveLadderPlan {
        HlsAdaptiveLadderPlan::from_source(
            HlsAdaptiveLadderSource {
                width: Some(640),
                height: Some(360),
                video_bitrate: Some(700_000),
                has_audio: Some(has_audio),
            },
            TranscodeOutputConstraints::default(),
        )
    }

    fn media_renditions() -> HlsMediaRenditionPlan {
        HlsMediaRenditionPlan::from_audio_and_subtitles(
            vec![HlsAudioRendition::new(0, 1, Some("eng".to_owned()), true)],
            vec![HlsSubtitleRendition::new(0, 2, Some("jpn".to_owned()))],
        )
        .unwrap()
    }

    fn hls_request_key(
        profile_kind: &str,
        variant_policy: &str,
        segment_container: &str,
        request_variant: Option<&str>,
    ) -> String {
        let profile = format!(
            "transcode-profile:v1;kind={profile_kind};hls_variant={variant_policy};hls_segment={segment_container}",
        );
        let mut request_key = format!(
            "transcode-request:v1;source=source-revision:v1;profile={}",
            escape_request_key_component(&profile)
        );
        if let Some(request_variant) = request_variant {
            request_key.push_str(";request_variant=");
            request_key.push_str(&escape_request_key_component(request_variant));
        }

        request_key
    }

    fn escape_request_key_component(value: &str) -> String {
        value
            .replace('%', "%25")
            .replace(';', "%3B")
            .replace('=', "%3D")
    }

    fn assert_artifact(
        manifest: &HlsArtifactManifest,
        artifact_name: &str,
        content_type: &'static str,
        cleanup_candidate: bool,
    ) {
        let artifact = manifest.artifact_for_name(artifact_name).unwrap();

        assert_eq!(artifact.path, manifest.output_dir().join(artifact_name));
        assert_eq!(artifact.content_type, content_type);
        assert_eq!(artifact.cleanup_candidate, cleanup_candidate);
        assert_eq!(
            manifest.cleanup_candidate_for_name(artifact_name),
            cleanup_candidate
        );
    }

    fn assert_not_manifest_artifact(manifest: &HlsArtifactManifest, artifact_name: &str) {
        assert!(
            manifest.artifact_for_name(artifact_name).is_err(),
            "{artifact_name} should not be serveable from the HLS artifact manifest"
        );
        assert!(!manifest.cleanup_candidate_for_name(artifact_name));
    }
}
