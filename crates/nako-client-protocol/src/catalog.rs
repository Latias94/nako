use serde::{Deserialize, Serialize};

use crate::PageInfo;

macro_rules! public_string_value {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $wire:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $($variant,)+
            Other(String),
        }

        impl $name {
            #[must_use]
            pub fn from_wire_value(value: &str) -> Self {
                match value {
                    $($wire => Self::$variant,)+
                    other => Self::Other(other.to_owned()),
                }
            }

            #[must_use]
            pub fn wire_value(&self) -> &str {
                match self {
                    $(Self::$variant => $wire,)+
                    Self::Other(value) => value,
                }
            }

            #[must_use]
            pub const fn is_known(&self) -> bool {
                !matches!(self, Self::Other(_))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.wire_value())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Ok(Self::from_wire_value(&value))
            }
        }
    };
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryListResponse {
    pub libraries: Vec<LibraryDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryResponse {
    pub library: LibraryDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourcesResponse {
    pub library: LibraryDto,
    pub sources: Vec<LibrarySourceResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryItemsResponse {
    pub library: LibraryDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibrarySourceResponse {
    pub source: MediaSourceDto,
    pub item: Option<MediaItemDto>,
    pub probe: Option<MediaProbeDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryDto {
    pub id: String,
    pub name: String,
    pub roots: Vec<String>,
    pub options: LibraryOptionsDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryOptionsDto {
    pub domain: ClientMediaDomain,
    pub preset: ClientLibraryPreset,
    pub scan: LibraryScanOptionsDto,
    pub naming_strategy: ClientNamingStrategy,
    pub metadata_profile: MetadataProfileDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanOptionsDto {
    pub realtime_monitor: bool,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProfileDto {
    pub item_kinds: Vec<ClientMediaKind>,
    pub local_readers: Vec<ClientLocalMetadataReader>,
    pub metadata_providers: Vec<ClientExternalProvider>,
    pub image_providers: Vec<ClientExternalProvider>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub refresh_mode: ClientMetadataRefreshMode,
    pub local_metadata_policy: ClientLocalMetadataPolicy,
    pub scan: MetadataScanPolicyDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataScanPolicyDto {
    pub enabled: bool,
    pub addon_scrape: bool,
    pub addon_writeback: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemsResponse {
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

public_string_value! {
    pub enum ClientBrowseSortKey {
        Title => "title",
        ReleaseDate => "release_date",
        DateAdded => "date_added",
        LastPlayed => "last_played",
    }
}

public_string_value! {
    pub enum ClientSortOrder {
        Asc => "asc",
        Desc => "desc",
    }
}

public_string_value! {
    pub enum ClientWatchStateFilter {
        Any => "any",
        Watched => "watched",
        Unwatched => "unwatched",
        InProgress => "in_progress",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemDetailResponse {
    pub item: MediaItemDto,
    pub sources: Vec<MediaSourceDto>,
    pub credits: Vec<ItemCreditDto>,
    pub genres: Vec<ItemGenreDto>,
    pub tags: Vec<ItemTagDto>,
    pub collections: Vec<CollectionItemDto>,
    pub studios: Vec<ItemStudioDto>,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditsResponse {
    pub item_id: String,
    pub credits: Vec<ItemCreditDto>,
    pub people: Vec<PersonDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImagesResponse {
    pub item_id: String,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PeopleResponse {
    pub people: Vec<PersonDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonResponse {
    pub person: PersonDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonItemsResponse {
    pub person: PersonDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<TagDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagItemsResponse {
    pub tag: TagDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreListResponse {
    pub genres: Vec<GenreDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreItemsResponse {
    pub genre: GenreDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchItemHit>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchItemHit {
    pub item: MediaItemDto,
    pub score: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagementContextLinksResponse {
    pub context: ManagementContextDto,
    pub links: Vec<ManagementContextLinkDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagementContextDto {
    pub library_id: Option<String>,
    pub item_id: Option<String>,
    pub source_id: Option<String>,
    pub playback_session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagementContextLinkDto {
    pub route_name: String,
    pub method: ClientManagementHttpMethod,
    pub surface: ClientManagementSurface,
    pub action: ClientManagementAction,
    pub target: ManagementContextDto,
    pub enabled: bool,
    pub required_access: ClientManagementRequiredAccess,
    pub disabled_reason: Option<ClientManagementDisabledReason>,
}

public_string_value! {
    pub enum ClientManagementHttpMethod {
        Get => "GET",
        Post => "POST",
        Put => "PUT",
        Delete => "DELETE",
    }
}

public_string_value! {
    pub enum ClientManagementSurface {
        Management => "management",
        Media => "media",
    }
}

public_string_value! {
    pub enum ClientManagementAction {
        ScanLibrary => "scan_library",
        UpdateLibraryMetadataProfile => "update_library_metadata_profile",
        RefreshItemMetadata => "refresh_item_metadata",
        ViewJobs => "view_jobs",
        ViewPlaybackDiagnostics => "view_playback_diagnostics",
        ViewPlaybackRuntime => "view_playback_runtime",
        ManageLibraryAccess => "manage_library_access",
    }
}

public_string_value! {
    pub enum ClientManagementRequiredAccess {
        LibraryManage => "library_manage",
        Administrator => "administrator",
    }
}

public_string_value! {
    pub enum ClientManagementDisabledReason {
        MissingContext => "missing_context",
        InsufficientPermission => "insufficient_permission",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceProbeResponse {
    pub source_id: String,
    pub probe: MediaProbeDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaItemDto {
    pub id: String,
    pub kind: ClientMediaKind,
    pub parent_id: Option<String>,
    pub metadata: CanonicalMetadataDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CanonicalMetadataDto {
    pub title: String,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime_minutes: Option<u32>,
    pub tagline: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: Vec<ContentRatingDto>,
    pub credits: Vec<CreditDto>,
    pub collections: Vec<CollectionRefDto>,
    pub studios: Vec<StudioRefDto>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentRatingDto {
    pub source: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreditDto {
    pub name: String,
    pub role: ClientCreditRole,
    pub character: Option<String>,
    pub order: Option<u32>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionRefDto {
    pub name: String,
    pub overview: Option<String>,
    pub sort_order: Option<u32>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StudioRefDto {
    pub name: String,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExternalIdDto {
    pub provider: ClientExternalProvider,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSourceDto {
    pub id: String,
    pub library_id: String,
    pub item_id: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeDto {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<MediaStreamDto>,
}

public_string_value! {
pub enum BrowserPlaybackMode {
        Direct => "direct",
        Remux => "remux",
        Hls => "hls",
        Subtitle => "subtitle",
    }
}

public_string_value! {
    pub enum BrowserPlaybackUrlKind {
        Stream => "stream",
        Playlist => "playlist",
        Subtitle => "subtitle",
    }
}

public_string_value! {
    pub enum BrowserPlaybackOutputContainer {
        Mp4 => "mp4",
        Mkv => "mkv",
    }
}

public_string_value! {
    pub enum ClientPlaybackTargetKind {
        Browser => "browser",
        NativeDesktop => "native_desktop",
        NativeMobile => "native_mobile",
        NakoRemoteClient => "nako_remote_client",
        Chromecast => "chromecast",
        DlnaRenderer => "dlna_renderer",
        Airplay => "airplay",
    }
}

public_string_value! {
    pub enum ClientPlaybackTargetNetworkScope {
        Local => "local",
        Remote => "remote",
        Unknown => "unknown",
    }
}

public_string_value! {
    pub enum ClientPlaybackTargetTransportAuth {
        Bearer => "bearer",
        BrowserTicket => "browser_ticket",
        CastTicket => "cast_ticket",
        None => "none",
    }
}

public_string_value! {
    pub enum ClientRendererControlCommand {
        ShowItem => "show_item",
        Play => "play",
        Pause => "pause",
        Resume => "resume",
        Seek => "seek",
        Stop => "stop",
        SetVolume => "set_volume",
    }
}

public_string_value! {
    pub enum ClientPlaybackPermission {
        MediaPlayback => "media_playback",
        DirectPlay => "direct_play",
        Remux => "remux",
        AudioTranscode => "audio_transcode",
        VideoTranscode => "video_transcode",
        RemotePlayback => "remote_playback",
        RemoteControl => "remote_control",
        Cast => "cast",
    }
}

public_string_value! {
    pub enum ClientPlaybackPermissionDecisionReason {
        Allowed => "allowed",
        LibraryAccessDoesNotAllowPlay => "library_access_does_not_allow_play",
        MediaPlaybackDisabled => "media_playback_disabled",
        DirectPlayDisabled => "direct_play_disabled",
        RemuxDisabled => "remux_disabled",
        AudioTranscodeDisabled => "audio_transcode_disabled",
        VideoTranscodeDisabled => "video_transcode_disabled",
        RemotePlaybackDisabled => "remote_playback_disabled",
        RemoteControlDisabled => "remote_control_disabled",
        CastDisabled => "cast_disabled",
    }
}

public_string_value! {
    pub enum ClientPlaybackProfileFamily {
        BrowserChromium => "browser_chromium",
        BrowserFirefox => "browser_firefox",
        BrowserSafari => "browser_safari",
        AndroidMedia3 => "android_media3",
        DesktopNative => "desktop_native",
        TvWebos => "tv_webos",
        TvTizen => "tv_tizen",
        Chromecast => "chromecast",
        DlnaRenderer => "dlna_renderer",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackProfilePresetsResponse {
    pub presets: Vec<PlaybackProfilePresetDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackProfilePresetDto {
    pub family: ClientPlaybackProfileFamily,
    pub device_family: String,
    pub profile_version: u32,
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_video_bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_channels: Option<u32>,
    pub supports_hdr: bool,
    pub supports_subtitles: bool,
    pub hls_variant_policy: ClientHlsVariantPolicy,
    pub hls_segment_container: ClientHlsSegmentContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BrowserPlaybackTicketRequest {
    pub mode: BrowserPlaybackMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<BrowserPlaybackCapabilitiesDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle_stream_index: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BrowserPlaybackCapabilitiesDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_play: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_codec: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_codec: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_video_bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_channels: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_hdr: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_subtitles: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hls_variant_policy: Option<ClientHlsVariantPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hls_segment_container: Option<ClientHlsSegmentContainer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_container: Option<BrowserPlaybackOutputContainer>,
}

impl Default for BrowserPlaybackCapabilitiesDto {
    fn default() -> Self {
        Self {
            direct_play: None,
            device_family: None,
            profile_version: None,
            container: None,
            video_codec: None,
            audio_codec: None,
            max_video_bitrate: None,
            max_width: None,
            max_height: None,
            max_audio_channels: None,
            supports_hdr: None,
            supports_subtitles: None,
            hls_variant_policy: None,
            hls_segment_container: None,
            output_container: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BrowserPlaybackTicketResponse {
    pub source_id: String,
    pub item_id: Option<String>,
    pub playback_session_id: Option<String>,
    pub mode: BrowserPlaybackMode,
    pub expires_at: String,
    pub urls: Vec<BrowserPlaybackUrlDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BrowserPlaybackUrlDto {
    pub kind: BrowserPlaybackUrlKind,
    pub url: String,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecisionResponse {
    pub source: MediaSourceDto,
    pub probe: Option<MediaProbeDto>,
    pub target: ClientPlaybackTargetDto,
    pub decision: ClientPlaybackDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackDecision {
    pub mode: ClientPlaybackMode,
    pub reason: ClientPlaybackDecisionReason,
    pub report: ClientPlaybackDecisionReport,
    pub denial: Option<ClientPlaybackDenialDto>,
    pub direct_play: Option<ClientDirectPlayPlan>,
    pub transcode_plan: Option<ClientTranscodePlan>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackDecisionReport {
    pub selected_mode: ClientPlaybackMode,
    #[serde(default)]
    pub selection_reasons: Vec<ClientPlaybackCompatibilityCondition>,
    #[serde(default)]
    pub selection_reason_details: Vec<ClientPlaybackCompatibilityConditionDetail>,
    pub direct_play: ClientPlaybackCapabilityEvaluation,
    pub remux: ClientPlaybackCapabilityEvaluation,
    pub transcode: ClientPlaybackCapabilityEvaluation,
    pub denial: Option<ClientPlaybackDenialDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCapabilityEvaluation {
    pub supported: bool,
    pub reasons: Vec<ClientPlaybackCompatibilityCondition>,
    #[serde(default)]
    pub reason_details: Vec<ClientPlaybackCompatibilityConditionDetail>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCompatibilityConditionDetail {
    pub condition: ClientPlaybackCompatibilityCondition,
    pub summary: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackTargetDto {
    pub kind: ClientPlaybackTargetKind,
    pub network_scope: ClientPlaybackTargetNetworkScope,
    pub transport_auth: ClientPlaybackTargetTransportAuth,
    pub media_capabilities: ClientPlaybackCapabilitiesDto,
    pub control_capabilities: ClientRendererControlCapabilitiesDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientRendererControlCapabilitiesDto {
    pub commands: Vec<ClientRendererControlCommand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackDenialDto {
    pub permission: ClientPlaybackPermission,
    pub reason: ClientPlaybackPermissionDecisionReason,
}

public_string_value! {
    pub enum ClientPlaybackDecisionReason {
        Compatible => "compatible",
        RequestedTranscodeOutput => "requested_transcode_output",
        ClientDisabledDirectPlay => "client_disabled_direct_play",
        SourceContainerUnknown => "source_container_unknown",
        ClientContainerUnsupported => "client_container_unsupported",
        SourceCodecsUnsupported => "source_codecs_unsupported",
        PolicyDenied => "policy_denied",
    }
}

public_string_value! {
    pub enum ClientPlaybackCompatibilityCondition {
        Compatible => "compatible",
        DirectPlayDisabled => "direct_play_disabled",
        MediaTechnicalFactsMissing => "media_technical_facts_missing",
        ContainerUnknown => "container_unknown",
        ContainerUnsupported => "container_unsupported",
        RemuxContainerUnsupported => "remux_container_unsupported",
        VideoCodecUnsupported => "video_codec_unsupported",
        AudioCodecUnsupported => "audio_codec_unsupported",
        VideoBitrateUnsupported => "video_bitrate_unsupported",
        VideoResolutionUnsupported => "video_resolution_unsupported",
        VideoHdrUnsupported => "video_hdr_unsupported",
        AudioChannelsUnsupported => "audio_channels_unsupported",
        SubtitleDeliveryUnsupported => "subtitle_delivery_unsupported",
        RequestedTranscodeOutput => "requested_transcode_output",
        TranscodeProfileUnsupported => "transcode_profile_unsupported",
        PolicyDenied => "policy_denied",
    }
}

impl ClientPlaybackCompatibilityCondition {
    #[must_use]
    pub fn detail(&self) -> ClientPlaybackCompatibilityConditionDetail {
        let (summary, detail) = playback_compatibility_condition_copy(self);
        ClientPlaybackCompatibilityConditionDetail {
            condition: self.clone(),
            summary: summary.to_owned(),
            detail: detail.to_owned(),
        }
    }
}

#[must_use]
pub fn playback_compatibility_condition_details(
    reasons: &[ClientPlaybackCompatibilityCondition],
) -> Vec<ClientPlaybackCompatibilityConditionDetail> {
    reasons
        .iter()
        .map(ClientPlaybackCompatibilityCondition::detail)
        .collect()
}

fn playback_compatibility_condition_copy(
    condition: &ClientPlaybackCompatibilityCondition,
) -> (&'static str, &'static str) {
    match condition {
        ClientPlaybackCompatibilityCondition::Compatible => (
            "Compatible",
            "The selected source matches the advertised playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::DirectPlayDisabled => (
            "Direct Play disabled",
            "The client request or capability profile disabled Direct Play.",
        ),
        ClientPlaybackCompatibilityCondition::MediaTechnicalFactsMissing => (
            "Media facts missing",
            "Nako does not have enough media technical facts to safely choose this playback mode.",
        ),
        ClientPlaybackCompatibilityCondition::ContainerUnknown => (
            "Container unknown",
            "Nako could not infer the source container from the available source facts.",
        ),
        ClientPlaybackCompatibilityCondition::ContainerUnsupported => (
            "Container unsupported",
            "The source container is not listed in the client playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::RemuxContainerUnsupported => (
            "Remux container unsupported",
            "The requested remux output container is not available for this playback target.",
        ),
        ClientPlaybackCompatibilityCondition::VideoCodecUnsupported => (
            "Video codec unsupported",
            "The selected source video codec is not listed in the client playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::AudioCodecUnsupported => (
            "Audio codec unsupported",
            "The selected source audio codec is not listed in the client playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::VideoBitrateUnsupported => (
            "Video bitrate unsupported",
            "The selected source video bitrate exceeds the client or request playback limit.",
        ),
        ClientPlaybackCompatibilityCondition::VideoResolutionUnsupported => (
            "Video resolution unsupported",
            "The selected source resolution exceeds the client playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::VideoHdrUnsupported => (
            "HDR unsupported",
            "The selected source uses HDR video that the client capability profile cannot present directly.",
        ),
        ClientPlaybackCompatibilityCondition::AudioChannelsUnsupported => (
            "Audio channels unsupported",
            "The selected source audio channel count exceeds the client playback capability profile.",
        ),
        ClientPlaybackCompatibilityCondition::SubtitleDeliveryUnsupported => (
            "Subtitle delivery unsupported",
            "The selected subtitle track cannot be delivered by the current direct or remux playback path.",
        ),
        ClientPlaybackCompatibilityCondition::RequestedTranscodeOutput => (
            "Transcode requested",
            "The playback request explicitly selected a transcode output.",
        ),
        ClientPlaybackCompatibilityCondition::TranscodeProfileUnsupported => (
            "Transcode profile unsupported",
            "The requested transcode output is not available for this playback target.",
        ),
        ClientPlaybackCompatibilityCondition::PolicyDenied => (
            "Playback policy denied",
            "The effective playback policy blocked the selected playback mode.",
        ),
        ClientPlaybackCompatibilityCondition::Other(_) => (
            "Unknown compatibility reason",
            "The server returned a newer playback compatibility reason that this client does not recognize yet.",
        ),
    }
}

public_string_value! {
    pub enum ClientPlaybackMode {
        DirectPlay => "direct_play",
        Remux => "remux",
        Transcode => "transcode",
        Denied => "denied",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientDirectPlayPlan {
    pub source_id: String,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientTranscodePlan {
    pub output_container: ClientOutputContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSessionResponse {
    pub session: PlaybackSessionDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSessionDto {
    pub id: String,
    pub source_id: String,
    pub item_id: String,
    pub mode: ClientPlaybackSessionMode,
    pub state: ClientPlaybackSessionState,
    pub transcode_session_id: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub client_capabilities: Option<ClientPlaybackCapabilitiesDto>,
    pub last_heartbeat_at: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCapabilitiesDto {
    pub direct_play: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_version: Option<u32>,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_video_bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_channels: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_hdr: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_subtitles: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hls_variant_policy: Option<ClientHlsVariantPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hls_segment_container: Option<ClientHlsSegmentContainer>,
}

impl Default for ClientPlaybackCapabilitiesDto {
    fn default() -> Self {
        Self {
            direct_play: true,
            device_family: None,
            profile_version: None,
            containers: Vec::new(),
            video_codecs: Vec::new(),
            audio_codecs: Vec::new(),
            max_video_bitrate: None,
            max_width: None,
            max_height: None,
            max_audio_channels: None,
            supports_hdr: None,
            supports_subtitles: None,
            hls_variant_policy: None,
            hls_segment_container: None,
        }
    }
}

public_string_value! {
    pub enum ClientHlsVariantPolicy {
        SingleVariant => "single_variant",
        Adaptive => "adaptive",
    }
}

public_string_value! {
    pub enum ClientHlsSegmentContainer {
        MpegTs => "mpeg_ts",
        Fmp4 => "fmp4",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSessionHeartbeatRequest {
    pub state: ClientPlaybackSessionState,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererRegistrationRequest {
    pub display_name: String,
    pub target_kind: ClientPlaybackTargetKind,
    pub network_scope: ClientPlaybackTargetNetworkScope,
    pub transport_auth: ClientPlaybackTargetTransportAuth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_capabilities: Option<ClientPlaybackCapabilitiesDto>,
    pub control_capabilities: ClientRendererControlCapabilitiesDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererHeartbeatRequest {
    pub state: ClientRendererSessionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_capabilities: Option<ClientPlaybackCapabilitiesDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_capabilities: Option<ClientRendererControlCapabilitiesDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererSessionResponse {
    pub renderer: RendererSessionDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererSessionsResponse {
    pub renderers: Vec<RendererSessionDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererSessionDto {
    pub id: String,
    pub target_kind: ClientPlaybackTargetKind,
    pub display_name: String,
    pub network_scope: ClientPlaybackTargetNetworkScope,
    pub transport_auth: ClientPlaybackTargetTransportAuth,
    pub media_capabilities: Option<ClientPlaybackCapabilitiesDto>,
    pub control_capabilities: ClientRendererControlCapabilitiesDto,
    pub state: ClientRendererSessionState,
    pub active_playback_session_id: Option<String>,
    pub last_seen_at: Option<String>,
    pub expires_at: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandPollResponse {
    pub command: Option<RendererCommandDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandResponse {
    pub command: RendererCommandDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandDto {
    pub id: String,
    pub renderer_session_id: String,
    pub command: ClientRendererControlCommand,
    pub state: ClientRendererCommandState,
    pub item_id: Option<String>,
    pub source_id: Option<String>,
    pub playback_session_id: Option<String>,
    pub position_ms: Option<u64>,
    pub volume_percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<RendererCommandTransportDto>,
    pub created_at: String,
    pub updated_at: String,
}

public_string_value! {
    pub enum RendererTransportMode {
        Direct => "direct",
        Remux => "remux",
        Hls => "hls",
    }
}

public_string_value! {
    pub enum RendererTransportUrlKind {
        Stream => "stream",
        Playlist => "playlist",
        SegmentBase => "segment_base",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandTransportDto {
    pub mode: RendererTransportMode,
    pub expires_at: String,
    pub urls: Vec<RendererCommandTransportUrlDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandTransportUrlDto {
    pub kind: RendererTransportUrlKind,
    pub url: String,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandCompletionRequest {
    pub state: ClientRendererCommandState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererPlayCommandRequest {
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererPlayCommandResponse {
    pub command: RendererCommandDto,
    pub session: PlaybackSessionDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionResponse {
    pub session: TranscodeSessionDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSessionDto {
    pub id: String,
    pub source_id: String,
    pub kind: ClientTranscodeSessionKind,
    pub request_key: String,
    pub state: ClientTranscodeSessionState,
    pub failure_category: Option<ClientTranscodeFailureCategory>,
    pub failure_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPlaybackStateResponse {
    pub state: UserPlaybackStateDto,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueWatchingItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContinueWatchingItemDto {
    pub item: MediaItemDto,
    pub state: UserPlaybackStateDto,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistsResponse {
    pub playlists: Vec<UserPlaylistDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistResponse {
    pub playlist: UserPlaylistDto,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistItemsResponse {
    pub playlist: UserPlaylistDto,
    pub items: Vec<UserPlaylistItemDto>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistDeleteResponse {
    pub playlist_id: String,
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistDto {
    pub id: String,
    pub name: String,
    pub visibility: ClientUserPlaylistVisibility,
    pub item_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistItemDto {
    pub playlist_id: String,
    pub item_id: String,
    pub position: u32,
    pub added_at: String,
    pub item: MediaItemDto,
    pub images: Vec<PublicImageRefDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateUserPlaylistRequest {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdateUserPlaylistRequest {
    pub name: String,
    pub expected_version: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddUserPlaylistItemRequest {
    pub position: Option<u32>,
    pub expected_version: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReorderUserPlaylistItemsRequest {
    pub item_ids: Vec<String>,
    pub expected_version: Option<u64>,
}

public_string_value! {
    pub enum ClientUserPlaylistVisibility {
        Private => "private",
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPlaybackStateDto {
    pub item_id: String,
    pub source_id: Option<String>,
    pub resume_position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub progress_percent: Option<f32>,
    pub watched: bool,
    pub watched_at: Option<String>,
    pub last_played_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdatePlaybackProgressRequest {
    pub source_id: Option<String>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub reported_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetWatchedStateRequest {
    pub watched: bool,
    pub source_id: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub marked_at: Option<String>,
}

public_string_value! {
    pub enum ClientPlaybackSessionMode {
        Direct => "direct",
        Remux => "remux",
        Hls => "hls",
    }
}

public_string_value! {
    pub enum ClientPlaybackSessionState {
        Active => "active",
        Paused => "paused",
        CancelRequested => "cancel_requested",
        Cancelled => "cancelled",
        Ended => "ended",
        Failed => "failed",
    }
}

public_string_value! {
    pub enum ClientRendererSessionState {
        Online => "online",
        Offline => "offline",
        Revoked => "revoked",
    }
}

public_string_value! {
    pub enum ClientRendererCommandState {
        Queued => "queued",
        Delivered => "delivered",
        Acknowledged => "acknowledged",
        Failed => "failed",
        Cancelled => "cancelled",
    }
}

public_string_value! {
    pub enum ClientTranscodeSessionKind {
        Remux => "remux",
        HlsTranscode => "hls_transcode",
    }
}

public_string_value! {
    pub enum ClientTranscodeSessionState {
        Planned => "planned",
        Starting => "starting",
        Running => "running",
        CancelRequested => "cancel_requested",
        Cancelled => "cancelled",
        Failed => "failed",
        Finished => "finished",
    }
}

public_string_value! {
    pub enum ClientTranscodeFailureCategory {
        InvalidRequest => "invalid_request",
        Runner => "runner",
        Timeout => "timeout",
        Storage => "storage",
        Stale => "stale",
        Cancelled => "cancelled",
        Unknown => "unknown",
    }
}

public_string_value! {
    pub enum ClientOutputContainer {
        Hls => "hls",
        Mp4 => "mp4",
        Mkv => "mkv",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamDto {
    pub index: u32,
    pub kind: ClientMediaStreamKind,
    pub origin: Option<ClientMediaStreamOrigin>,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub disposition: MediaStreamDispositionDto,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamDispositionDto {
    pub default: bool,
    pub forced: bool,
    pub hearing_impaired: bool,
    pub visual_impaired: bool,
    pub commentary: bool,
    pub attached_pic: bool,
    pub captions: bool,
    pub descriptions: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonDto {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub overview: Option<String>,
    pub external_ids: Vec<ExternalIdDto>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCreditDto {
    pub item_id: String,
    pub person_id: String,
    pub role: ClientCreditRole,
    pub character: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenreDto {
    pub id: String,
    pub name: String,
    pub source: ClientMetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemGenreDto {
    pub item_id: String,
    pub genre_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TagDto {
    pub id: String,
    pub name: String,
    pub source: ClientMetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemTagDto {
    pub item_id: String,
    pub tag_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionItemDto {
    pub collection_id: String,
    pub item_id: String,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemStudioDto {
    pub item_id: String,
    pub studio_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicImageRefDto {
    pub id: String,
    pub owner: ClientImageOwner,
    pub kind: ClientImageKind,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub media_type: Option<String>,
    pub etag: Option<String>,
}

public_string_value! {
    pub enum ClientMediaKind {
        Movie => "movie",
        Series => "series",
        Season => "season",
        Episode => "episode",
        Collection => "collection",
        Extra => "extra",
        Unknown => "unknown",
    }
}

public_string_value! {
    pub enum ClientMediaDomain {
        Video => "video",
        Audio => "audio",
        Image => "image",
        Document => "document",
        Mixed => "mixed",
        Online => "online",
    }
}

public_string_value! {
    pub enum ClientLibraryPreset {
        Movies => "movies",
        Tv => "tv",
        Anime => "anime",
        Music => "music",
        Podcast => "podcast",
        Photos => "photos",
        HomeVideo => "home_video",
        MixedVideo => "mixed_video",
        OnlineCatalog => "online_catalog",
        Custom => "custom",
    }
}

public_string_value! {
    pub enum ClientNamingStrategy {
        Movie => "movie",
        Series => "series",
        Anime => "anime",
        Music => "music",
        Podcast => "podcast",
        Photo => "photo",
        HomeVideo => "home_video",
        Mixed => "mixed",
        OnlineCatalog => "online_catalog",
    }
}

public_string_value! {
    pub enum ClientLocalMetadataReader {
        Nfo => "nfo",
        Embedded => "embedded",
        Sidecar => "sidecar",
    }
}

public_string_value! {
    pub enum ClientMetadataRefreshMode {
        None => "none",
        ValidationOnly => "validation_only",
        Default => "default",
        MissingOnly => "missing_only",
        FullRefresh => "full_refresh",
    }
}

public_string_value! {
    pub enum ClientLocalMetadataPolicy {
        Disabled => "disabled",
        ReadOnly => "read_only",
        LocalFirst => "local_first",
        RemoteFirst => "remote_first",
        WriteSidecar => "write_sidecar",
    }
}

public_string_value! {
    pub enum ClientExternalProvider {
        Tmdb => "tmdb",
        Douban => "douban",
        Bangumi => "bangumi",
        Imdb => "imdb",
        Local => "local",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMetadataSource {
    Local,
    Nfo,
    Provider(ClientExternalProvider),
    User,
    Addon(String),
}

public_string_value! {
    pub enum ClientImageKind {
        Poster => "poster",
        Backdrop => "backdrop",
        Logo => "logo",
        Thumbnail => "thumbnail",
        Banner => "banner",
    }
}

public_string_value! {
    pub enum ClientCreditRole {
        Actor => "actor",
        Director => "director",
        Writer => "writer",
        Producer => "producer",
        Creator => "creator",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientImageOwner {
    Item(String),
    Person(String),
    Collection(String),
    Studio(String),
}

public_string_value! {
    pub enum ClientMediaStreamKind {
        Video => "video",
        Audio => "audio",
        Subtitle => "subtitle",
        Data => "data",
        Attachment => "attachment",
    }
}

public_string_value! {
    pub enum ClientMediaStreamOrigin {
        Embedded => "embedded",
        Sidecar => "sidecar",
        External => "external",
    }
}
