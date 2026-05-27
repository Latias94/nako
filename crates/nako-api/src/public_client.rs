use nako_core::{
    CanonicalMetadata, CollectionItem, Credit, CreditRole, ExternalId, ExternalProvider, Genre,
    ImageKind, ItemCredit, ItemGenre, ItemStudio, ItemTag, Library, LibraryOptions, LibraryPreset,
    LocalMetadataPolicy, LocalMetadataReader, ManagedArtworkArtifactRecord, MediaDomain, MediaItem,
    MediaKind, MediaProbeResult, MediaSource, MediaStreamInfo, MediaStreamKind, MetadataProfile,
    MetadataRefreshMode, MetadataSource, NamingStrategy, PageRequest, Person, PlaybackSessionMode,
    PlaybackSessionRecord, PlaybackSessionState, RendererCommandRecord, RendererCommandState,
    RendererControlCapabilities, RendererSessionRecord, RendererSessionState,
    SelectedArtworkRecord, Tag, TranscodeFailureCategory, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState, UserPlaybackState,
};
use nako_playback::{
    ClientPlaybackCapabilities, DirectPlayPlan, PlaybackDecision, PlaybackDecisionReason,
    PlaybackDenial, PlaybackMode, PlaybackPermission, PlaybackPermissionDecisionReason,
    PlaybackTarget, PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth,
    RendererControlCommand,
};
use nako_transcode::{OutputContainer, TranscodePlan};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub use nako_client_protocol::{
    API_VERSION_HEADER, BrowserPlaybackCapabilitiesDto, BrowserPlaybackMode,
    BrowserPlaybackOutputContainer, BrowserPlaybackTicketRequest, BrowserPlaybackTicketResponse,
    BrowserPlaybackUrlDto, BrowserPlaybackUrlKind, CLIENT_PROTOCOL_VERSION as API_VERSION,
    CanonicalMetadataDto, ClientCreditRole, ClientDirectPlayPlan, ClientErrorCode,
    ClientExternalProvider, ClientImageKind, ClientImageOwner, ClientLibraryPreset,
    ClientLocalMetadataPolicy, ClientLocalMetadataReader, ClientManagementAction,
    ClientManagementDisabledReason, ClientManagementHttpMethod, ClientManagementRequiredAccess,
    ClientManagementSurface, ClientMediaDomain, ClientMediaKind, ClientMediaStreamKind,
    ClientMetadataRefreshMode, ClientMetadataSource, ClientNamingStrategy, ClientOutputContainer,
    ClientPlaybackCapabilitiesDto, ClientPlaybackDecision, ClientPlaybackDecisionReason,
    ClientPlaybackDenialDto, ClientPlaybackMode, ClientPlaybackPermission,
    ClientPlaybackPermissionDecisionReason, ClientPlaybackSessionMode, ClientPlaybackSessionState,
    ClientPlaybackTargetDto, ClientPlaybackTargetKind, ClientPlaybackTargetNetworkScope,
    ClientPlaybackTargetTransportAuth, ClientRendererCommandState,
    ClientRendererControlCapabilitiesDto, ClientRendererControlCommand, ClientRendererSessionState,
    ClientTranscodeFailureCategory, ClientTranscodePlan, ClientTranscodeSessionKind,
    ClientTranscodeSessionState, CollectionItemDto, CollectionRefDto, ContentRatingDto,
    ContinueWatchingItemDto, ContinueWatchingResponse, CreditDto, CurrentUserDto,
    CurrentUserResponse, ErrorResponse, ExternalIdDto, GenreDto, GenreItemsResponse,
    GenreListResponse, HealthResponse, ImagesResponse, ItemCreditDto, ItemCreditsResponse,
    ItemDetailResponse, ItemGenreDto, ItemStudioDto, ItemTagDto, ItemsResponse, LibraryDto,
    LibraryListResponse, LibraryOptionsDto, LibraryResponse, LibraryScanOptionsDto,
    LibrarySourceResponse, LibrarySourcesResponse, LoginRequest, LoginResponse, LogoutResponse,
    ManagementContextDto, ManagementContextLinkDto, ManagementContextLinksResponse, MediaItemDto,
    MediaProbeDto, MediaSourceDto, MediaStreamDto, MetadataProfileDto, MetadataScanPolicyDto,
    PLAYBACK_SESSION_ID_HEADER, PageInfo, PeopleResponse, PersonDto, PersonItemsResponse,
    PersonResponse, PlaybackDecisionResponse, PlaybackSessionDto, PlaybackSessionHeartbeatRequest,
    PlaybackSessionResponse, PublicImageRefDto, RedeemInvitationRequest,
    RendererCommandCompletionRequest, RendererCommandDto, RendererCommandPollResponse,
    RendererCommandResponse, RendererCommandTransportDto, RendererCommandTransportUrlDto,
    RendererHeartbeatRequest, RendererPlayCommandRequest, RendererPlayCommandResponse,
    RendererRegistrationRequest, RendererSessionDto, RendererSessionResponse,
    RendererSessionsResponse, RendererTransportMode, RendererTransportUrlKind, SearchItemHit,
    SearchResponse, SetWatchedStateRequest, SourceProbeResponse, StudioRefDto, TagDto,
    TagItemsResponse, TagsResponse, TranscodeSessionDto, TranscodeSessionResponse,
    UpdatePlaybackProgressRequest, UserPlaybackStateDto, UserPlaybackStateResponse, UserSessionDto,
};

#[must_use]
pub fn page_info_from_request(page: PageRequest, returned: usize) -> PageInfo {
    let page = page.clamped();

    PageInfo::new(
        page.limit,
        page.offset,
        u32::try_from(returned).unwrap_or(u32::MAX),
    )
}

#[must_use]
pub fn library_to_dto(library: Library) -> LibraryDto {
    LibraryDto {
        id: library.id.to_string(),
        name: library.name,
        roots: library.roots,
        options: library_options_to_dto(library.options),
    }
}

#[must_use]
pub fn library_options_to_dto(options: LibraryOptions) -> LibraryOptionsDto {
    LibraryOptionsDto {
        domain: media_domain_to_dto(options.domain),
        preset: library_preset_to_dto(options.preset),
        scan: LibraryScanOptionsDto {
            realtime_monitor: options.scan.realtime_monitor,
            max_depth: options.scan.max_depth,
        },
        naming_strategy: naming_strategy_to_dto(options.naming_strategy),
        metadata_profile: metadata_profile_to_dto(options.metadata_profile),
    }
}

#[must_use]
pub fn metadata_profile_to_dto(profile: MetadataProfile) -> MetadataProfileDto {
    MetadataProfileDto {
        item_kinds: profile
            .item_kinds
            .into_iter()
            .map(media_kind_to_dto)
            .collect(),
        local_readers: profile
            .local_readers
            .into_iter()
            .map(local_metadata_reader_to_dto)
            .collect(),
        metadata_providers: profile
            .metadata_providers
            .into_iter()
            .map(external_provider_to_dto)
            .collect(),
        image_providers: profile
            .image_providers
            .into_iter()
            .map(external_provider_to_dto)
            .collect(),
        language: profile.language,
        country: profile.country,
        refresh_mode: metadata_refresh_mode_to_dto(profile.refresh_mode),
        local_metadata_policy: local_metadata_policy_to_dto(profile.local_metadata_policy),
        scan: MetadataScanPolicyDto {
            enabled: profile.scan.enabled,
            addon_scrape: profile.scan.addon_scrape,
            addon_writeback: profile.scan.addon_writeback,
        },
    }
}

#[must_use]
pub fn media_item_to_dto(item: MediaItem) -> MediaItemDto {
    MediaItemDto {
        id: item.id.to_string(),
        kind: media_kind_to_dto(item.kind),
        parent_id: item.parent_id.map(|id| id.to_string()),
        metadata: canonical_metadata_to_dto(item.metadata),
    }
}

#[must_use]
pub fn canonical_metadata_to_dto(metadata: CanonicalMetadata) -> CanonicalMetadataDto {
    CanonicalMetadataDto {
        title: metadata.title,
        original_title: metadata.original_title,
        sort_title: metadata.sort_title,
        overview: metadata.overview,
        release_date: metadata.release_date,
        runtime_minutes: metadata.runtime_minutes,
        tagline: metadata.tagline,
        genres: metadata.genres,
        tags: metadata.tags,
        ratings: metadata
            .ratings
            .into_iter()
            .map(|rating| ContentRatingDto {
                source: rating.source,
                value: rating.value,
            })
            .collect(),
        credits: metadata.credits.into_iter().map(credit_to_dto).collect(),
        collections: metadata
            .collections
            .into_iter()
            .map(|collection| CollectionRefDto {
                name: collection.name,
                overview: collection.overview,
                sort_order: collection.sort_order,
                external_ids: collection
                    .external_ids
                    .into_iter()
                    .map(external_id_to_dto)
                    .collect(),
            })
            .collect(),
        studios: metadata
            .studios
            .into_iter()
            .map(|studio| StudioRefDto {
                name: studio.name,
                external_ids: studio
                    .external_ids
                    .into_iter()
                    .map(external_id_to_dto)
                    .collect(),
            })
            .collect(),
        external_ids: metadata
            .external_ids
            .into_iter()
            .map(external_id_to_dto)
            .collect(),
    }
}

#[must_use]
pub fn media_source_to_dto(source: MediaSource) -> MediaSourceDto {
    MediaSourceDto {
        id: source.id.to_string(),
        library_id: source.library_id.to_string(),
        item_id: source.item_id.to_string(),
        file_name: source.file_name,
        size_bytes: source.size_bytes,
        fingerprint: source.fingerprint,
    }
}

#[must_use]
pub fn media_probe_to_dto(probe: MediaProbeResult) -> MediaProbeDto {
    MediaProbeDto {
        duration_ms: probe.duration_ms,
        container: probe.container,
        bit_rate: probe.bit_rate,
        streams: probe.streams.into_iter().map(media_stream_to_dto).collect(),
    }
}

#[must_use]
pub fn playback_decision_response_to_dto(
    source: MediaSource,
    probe: Option<MediaProbeResult>,
    target: PlaybackTarget,
    decision: PlaybackDecision,
) -> PlaybackDecisionResponse {
    PlaybackDecisionResponse {
        source: media_source_to_dto(source),
        probe: probe.map(media_probe_to_dto),
        target: playback_target_to_dto(target),
        decision: playback_decision_to_dto(decision),
    }
}

#[must_use]
pub fn playback_decision_to_dto(decision: PlaybackDecision) -> ClientPlaybackDecision {
    ClientPlaybackDecision {
        mode: playback_mode_to_dto(decision.mode),
        reason: playback_decision_reason_to_dto(decision.reason),
        denial: decision.denial.map(playback_denial_to_dto),
        direct_play: decision.direct_play.map(direct_play_plan_to_dto),
        transcode_plan: decision.transcode_plan.map(transcode_plan_to_dto),
    }
}

#[must_use]
pub fn playback_target_to_dto(target: PlaybackTarget) -> ClientPlaybackTargetDto {
    ClientPlaybackTargetDto {
        kind: playback_target_kind_to_dto(target.kind),
        network_scope: playback_target_network_scope_to_dto(target.network_scope),
        transport_auth: playback_target_transport_auth_to_dto(target.transport_auth),
        media_capabilities: playback_capabilities_to_dto(target.media_capabilities),
        control_capabilities: ClientRendererControlCapabilitiesDto {
            commands: target
                .control_capabilities
                .commands
                .into_iter()
                .map(renderer_control_command_to_dto)
                .collect(),
        },
    }
}

fn direct_play_plan_to_dto(plan: DirectPlayPlan) -> ClientDirectPlayPlan {
    ClientDirectPlayPlan {
        source_id: plan.source_id.to_string(),
        content_type: plan.content_type,
        supports_range_requests: plan.supports_range_requests,
    }
}

fn transcode_plan_to_dto(plan: TranscodePlan) -> ClientTranscodePlan {
    ClientTranscodePlan {
        output_container: output_container_to_dto(plan.output_container),
        video_codec: plan.video_codec,
        audio_codec: plan.audio_codec,
    }
}

#[must_use]
pub fn transcode_session_response_from_record(
    session: TranscodeSessionRecord,
) -> TranscodeSessionResponse {
    TranscodeSessionResponse {
        session: transcode_session_to_dto(session),
    }
}

#[must_use]
pub fn playback_session_response_from_record(
    session: PlaybackSessionRecord,
) -> PlaybackSessionResponse {
    PlaybackSessionResponse {
        session: playback_session_to_dto(session),
    }
}

#[must_use]
pub fn playback_session_to_dto(session: PlaybackSessionRecord) -> PlaybackSessionDto {
    let client_capabilities = session
        .client_capabilities_json
        .as_deref()
        .and_then(playback_session_client_capabilities_from_json);

    PlaybackSessionDto {
        id: session.id.to_string(),
        source_id: session.source_id.to_string(),
        item_id: session.item_id.to_string(),
        mode: playback_session_mode_to_dto(session.mode),
        state: playback_session_state_to_dto(session.state),
        transcode_session_id: session.transcode_session_id.map(|id| id.to_string()),
        position_ms: session.position_ms,
        duration_ms: session.duration_ms,
        client_capabilities,
        last_heartbeat_at: timestamp_ms_to_rfc3339(session.last_heartbeat_at_ms),
        started_at: timestamp_ms_to_rfc3339(Some(session.started_at_ms)),
        ended_at: timestamp_ms_to_rfc3339(session.ended_at_ms),
        updated_at: session.updated_at,
    }
}

#[must_use]
pub fn renderer_session_response_from_record(
    session: RendererSessionRecord,
) -> RendererSessionResponse {
    RendererSessionResponse {
        renderer: renderer_session_to_dto(session),
    }
}

#[must_use]
pub fn renderer_session_to_dto(session: RendererSessionRecord) -> RendererSessionDto {
    let media_capabilities = session
        .media_capabilities_json
        .as_deref()
        .and_then(playback_session_client_capabilities_from_json);

    RendererSessionDto {
        id: session.id.to_string(),
        target_kind: playback_target_kind_to_dto(session.target_kind),
        display_name: session.display_name,
        network_scope: playback_target_network_scope_to_dto(session.network_scope),
        transport_auth: playback_target_transport_auth_to_dto(session.transport_auth),
        media_capabilities,
        control_capabilities: renderer_control_capabilities_to_dto(session.control_capabilities),
        state: renderer_session_state_to_dto(session.state),
        active_playback_session_id: session.active_playback_session_id.map(|id| id.to_string()),
        last_seen_at: timestamp_ms_to_rfc3339(Some(session.last_seen_at_ms)),
        expires_at: timestamp_ms_to_rfc3339(session.expires_at_ms),
        updated_at: session.updated_at,
    }
}

#[must_use]
pub fn renderer_command_poll_response_from_record(
    command: Option<RendererCommandRecord>,
) -> RendererCommandPollResponse {
    RendererCommandPollResponse {
        command: command.map(renderer_command_to_dto),
    }
}

#[must_use]
pub fn renderer_command_response_from_record(
    command: RendererCommandRecord,
) -> RendererCommandResponse {
    RendererCommandResponse {
        command: renderer_command_to_dto(command),
    }
}

#[must_use]
pub fn renderer_play_command_response_from_records(
    command: RendererCommandRecord,
    session: PlaybackSessionRecord,
) -> RendererPlayCommandResponse {
    RendererPlayCommandResponse {
        command: renderer_command_to_dto(command),
        session: playback_session_to_dto(session),
    }
}

#[must_use]
pub fn renderer_command_to_dto(command: RendererCommandRecord) -> RendererCommandDto {
    RendererCommandDto {
        id: command.id.to_string(),
        renderer_session_id: command.renderer_session_id.to_string(),
        command: renderer_control_command_to_dto(command.command),
        state: renderer_command_state_to_dto(command.state),
        item_id: command.item_id.map(|id| id.to_string()),
        source_id: command.source_id.map(|id| id.to_string()),
        playback_session_id: command.playback_session_id.map(|id| id.to_string()),
        position_ms: command.position_ms,
        volume_percent: command.volume_percent,
        transport: None,
        created_at: command.created_at,
        updated_at: command.updated_at,
    }
}

#[must_use]
pub fn transcode_session_to_dto(session: TranscodeSessionRecord) -> TranscodeSessionDto {
    let failure_message = session
        .failure_message
        .as_ref()
        .map(|_| public_transcode_failure_message(session.failure_category));

    TranscodeSessionDto {
        id: session.id.to_string(),
        source_id: session.source_id.to_string(),
        kind: transcode_session_kind_to_dto(session.kind),
        request_key: session.request_key,
        state: transcode_session_state_to_dto(session.state),
        failure_category: session
            .failure_category
            .map(transcode_failure_category_to_dto),
        failure_message,
        created_at: session.created_at,
        updated_at: session.updated_at,
        started_at: session.started_at,
        completed_at: session.completed_at,
    }
}

#[must_use]
pub fn user_playback_state_response_from_state(
    state: UserPlaybackState,
) -> UserPlaybackStateResponse {
    UserPlaybackStateResponse {
        state: user_playback_state_to_dto(state),
    }
}

#[must_use]
pub fn user_playback_state_to_dto(state: UserPlaybackState) -> UserPlaybackStateDto {
    UserPlaybackStateDto {
        item_id: state.item_id.to_string(),
        source_id: state.source_id.map(|id| id.to_string()),
        resume_position_ms: state.resume_position_ms,
        duration_ms: state.duration_ms,
        progress_percent: state.progress_percent(),
        watched: state.watched,
        watched_at: timestamp_ms_to_rfc3339(state.watched_at_ms),
        last_played_at: timestamp_ms_to_rfc3339(state.last_played_at_ms),
        updated_at: timestamp_ms_to_rfc3339(Some(state.updated_at_ms)),
        version: state.version,
    }
}

#[must_use]
pub fn timestamp_ms_to_rfc3339(timestamp_ms: Option<i64>) -> Option<String> {
    let timestamp_ms = timestamp_ms.filter(|value| *value > 0)?;
    let nanos = i128::from(timestamp_ms).checked_mul(1_000_000)?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .ok()?
        .format(&Rfc3339)
        .ok()
}

#[must_use]
pub fn media_stream_to_dto(stream: MediaStreamInfo) -> MediaStreamDto {
    MediaStreamDto {
        index: stream.index,
        kind: media_stream_kind_to_dto(stream.kind),
        codec: stream.codec,
        language: stream.language,
        duration_ms: stream.duration_ms,
        bit_rate: stream.bit_rate,
        width: stream.width,
        height: stream.height,
        channels: stream.channels,
        sample_rate: stream.sample_rate,
    }
}

#[must_use]
pub fn person_to_dto(person: Person) -> PersonDto {
    PersonDto {
        id: person.id.to_string(),
        name: person.name,
        sort_name: person.sort_name,
        overview: person.overview,
        external_ids: person
            .external_ids
            .into_iter()
            .map(external_id_to_dto)
            .collect(),
    }
}

#[must_use]
pub fn item_credit_to_dto(credit: ItemCredit) -> ItemCreditDto {
    ItemCreditDto {
        item_id: credit.item_id.to_string(),
        person_id: credit.person_id.to_string(),
        role: credit_role_to_dto(credit.role),
        character: credit.character,
        sort_order: credit.sort_order,
    }
}

#[must_use]
pub fn genre_to_dto(genre: Genre) -> GenreDto {
    GenreDto {
        id: genre.id.to_string(),
        name: genre.name,
        source: metadata_source_to_dto(genre.source),
    }
}

#[must_use]
pub fn item_genre_to_dto(genre: ItemGenre) -> ItemGenreDto {
    ItemGenreDto {
        item_id: genre.item_id.to_string(),
        genre_id: genre.genre_id.to_string(),
    }
}

#[must_use]
pub fn tag_to_dto(tag: Tag) -> TagDto {
    TagDto {
        id: tag.id.to_string(),
        name: tag.name,
        source: metadata_source_to_dto(tag.source),
    }
}

#[must_use]
pub fn item_tag_to_dto(tag: ItemTag) -> ItemTagDto {
    ItemTagDto {
        item_id: tag.item_id.to_string(),
        tag_id: tag.tag_id.to_string(),
    }
}

#[must_use]
pub fn collection_item_to_dto(collection: CollectionItem) -> CollectionItemDto {
    CollectionItemDto {
        collection_id: collection.collection_id.to_string(),
        item_id: collection.item_id.to_string(),
        sort_order: collection.sort_order,
    }
}

#[must_use]
pub fn item_studio_to_dto(studio: ItemStudio) -> ItemStudioDto {
    ItemStudioDto {
        item_id: studio.item_id.to_string(),
        studio_id: studio.studio_id.to_string(),
    }
}

#[must_use]
pub fn selected_artwork_to_public_image_ref(
    selected: SelectedArtworkRecord,
    artifact: ManagedArtworkArtifactRecord,
) -> PublicImageRefDto {
    PublicImageRefDto {
        id: selected.id.to_string(),
        owner: ClientImageOwner::Item(selected.item_id.to_string()),
        kind: image_kind_to_dto(selected.kind),
        url: format!("/images/{}", selected.id),
        width: artifact.width,
        height: artifact.height,
        language: None,
        media_type: artifact.media_type,
        etag: None,
    }
}

fn external_id_to_dto(id: ExternalId) -> ExternalIdDto {
    ExternalIdDto {
        provider: external_provider_to_dto(id.provider),
        value: id.value,
    }
}

fn credit_to_dto(credit: Credit) -> CreditDto {
    CreditDto {
        name: credit.name,
        role: credit_role_to_dto(credit.role),
        character: credit.character,
        order: credit.order,
        external_ids: credit
            .external_ids
            .into_iter()
            .map(external_id_to_dto)
            .collect(),
    }
}

fn media_kind_to_dto(kind: MediaKind) -> ClientMediaKind {
    match kind {
        MediaKind::Movie => ClientMediaKind::Movie,
        MediaKind::Series => ClientMediaKind::Series,
        MediaKind::Season => ClientMediaKind::Season,
        MediaKind::Episode => ClientMediaKind::Episode,
        MediaKind::Collection => ClientMediaKind::Collection,
        MediaKind::Extra => ClientMediaKind::Extra,
        MediaKind::Unknown => ClientMediaKind::Unknown,
    }
}

fn media_domain_to_dto(domain: MediaDomain) -> ClientMediaDomain {
    match domain {
        MediaDomain::Video => ClientMediaDomain::Video,
        MediaDomain::Audio => ClientMediaDomain::Audio,
        MediaDomain::Image => ClientMediaDomain::Image,
        MediaDomain::Document => ClientMediaDomain::Document,
        MediaDomain::Mixed => ClientMediaDomain::Mixed,
        MediaDomain::Online => ClientMediaDomain::Online,
    }
}

fn library_preset_to_dto(preset: LibraryPreset) -> ClientLibraryPreset {
    match preset {
        LibraryPreset::Movies => ClientLibraryPreset::Movies,
        LibraryPreset::Tv => ClientLibraryPreset::Tv,
        LibraryPreset::Anime => ClientLibraryPreset::Anime,
        LibraryPreset::Music => ClientLibraryPreset::Music,
        LibraryPreset::Podcast => ClientLibraryPreset::Podcast,
        LibraryPreset::Photos => ClientLibraryPreset::Photos,
        LibraryPreset::HomeVideo => ClientLibraryPreset::HomeVideo,
        LibraryPreset::MixedVideo => ClientLibraryPreset::MixedVideo,
        LibraryPreset::OnlineCatalog => ClientLibraryPreset::OnlineCatalog,
        LibraryPreset::Custom => ClientLibraryPreset::Custom,
    }
}

fn naming_strategy_to_dto(strategy: NamingStrategy) -> ClientNamingStrategy {
    match strategy {
        NamingStrategy::Movie => ClientNamingStrategy::Movie,
        NamingStrategy::Series => ClientNamingStrategy::Series,
        NamingStrategy::Anime => ClientNamingStrategy::Anime,
        NamingStrategy::Music => ClientNamingStrategy::Music,
        NamingStrategy::Podcast => ClientNamingStrategy::Podcast,
        NamingStrategy::Photo => ClientNamingStrategy::Photo,
        NamingStrategy::HomeVideo => ClientNamingStrategy::HomeVideo,
        NamingStrategy::Mixed => ClientNamingStrategy::Mixed,
        NamingStrategy::OnlineCatalog => ClientNamingStrategy::OnlineCatalog,
    }
}

fn local_metadata_reader_to_dto(reader: LocalMetadataReader) -> ClientLocalMetadataReader {
    match reader {
        LocalMetadataReader::Nfo => ClientLocalMetadataReader::Nfo,
        LocalMetadataReader::Embedded => ClientLocalMetadataReader::Embedded,
        LocalMetadataReader::Sidecar => ClientLocalMetadataReader::Sidecar,
        LocalMetadataReader::Other(value) => ClientLocalMetadataReader::Other(value.to_string()),
    }
}

fn metadata_refresh_mode_to_dto(mode: MetadataRefreshMode) -> ClientMetadataRefreshMode {
    match mode {
        MetadataRefreshMode::None => ClientMetadataRefreshMode::None,
        MetadataRefreshMode::ValidationOnly => ClientMetadataRefreshMode::ValidationOnly,
        MetadataRefreshMode::Default => ClientMetadataRefreshMode::Default,
        MetadataRefreshMode::MissingOnly => ClientMetadataRefreshMode::MissingOnly,
        MetadataRefreshMode::FullRefresh => ClientMetadataRefreshMode::FullRefresh,
    }
}

fn local_metadata_policy_to_dto(policy: LocalMetadataPolicy) -> ClientLocalMetadataPolicy {
    match policy {
        LocalMetadataPolicy::Disabled => ClientLocalMetadataPolicy::Disabled,
        LocalMetadataPolicy::ReadOnly => ClientLocalMetadataPolicy::ReadOnly,
        LocalMetadataPolicy::LocalFirst => ClientLocalMetadataPolicy::LocalFirst,
        LocalMetadataPolicy::RemoteFirst => ClientLocalMetadataPolicy::RemoteFirst,
        LocalMetadataPolicy::WriteSidecar => ClientLocalMetadataPolicy::WriteSidecar,
    }
}

fn external_provider_to_dto(provider: ExternalProvider) -> ClientExternalProvider {
    match provider {
        ExternalProvider::Tmdb => ClientExternalProvider::Tmdb,
        ExternalProvider::Douban => ClientExternalProvider::Douban,
        ExternalProvider::Bangumi => ClientExternalProvider::Bangumi,
        ExternalProvider::Imdb => ClientExternalProvider::Imdb,
        ExternalProvider::Local => ClientExternalProvider::Local,
        ExternalProvider::Other(value) => ClientExternalProvider::Other(value.to_string()),
    }
}

fn metadata_source_to_dto(source: MetadataSource) -> ClientMetadataSource {
    match source {
        MetadataSource::Local => ClientMetadataSource::Local,
        MetadataSource::Nfo => ClientMetadataSource::Nfo,
        MetadataSource::Provider(provider) => {
            ClientMetadataSource::Provider(external_provider_to_dto(provider))
        }
        MetadataSource::User => ClientMetadataSource::User,
        MetadataSource::Addon(addon_id) => ClientMetadataSource::Addon(addon_id.to_string()),
    }
}

pub fn image_kind_to_dto(kind: ImageKind) -> ClientImageKind {
    match kind {
        ImageKind::Poster => ClientImageKind::Poster,
        ImageKind::Backdrop => ClientImageKind::Backdrop,
        ImageKind::Logo => ClientImageKind::Logo,
        ImageKind::Thumbnail => ClientImageKind::Thumbnail,
        ImageKind::Banner => ClientImageKind::Banner,
        ImageKind::Other(value) => ClientImageKind::Other(value.to_string()),
    }
}

fn credit_role_to_dto(role: CreditRole) -> ClientCreditRole {
    match role {
        CreditRole::Actor => ClientCreditRole::Actor,
        CreditRole::Director => ClientCreditRole::Director,
        CreditRole::Writer => ClientCreditRole::Writer,
        CreditRole::Producer => ClientCreditRole::Producer,
        CreditRole::Creator => ClientCreditRole::Creator,
        CreditRole::Other(value) => ClientCreditRole::Other(value.to_string()),
    }
}

fn media_stream_kind_to_dto(kind: MediaStreamKind) -> ClientMediaStreamKind {
    match kind {
        MediaStreamKind::Video => ClientMediaStreamKind::Video,
        MediaStreamKind::Audio => ClientMediaStreamKind::Audio,
        MediaStreamKind::Subtitle => ClientMediaStreamKind::Subtitle,
        MediaStreamKind::Data => ClientMediaStreamKind::Data,
        MediaStreamKind::Attachment => ClientMediaStreamKind::Attachment,
        MediaStreamKind::Other(value) => ClientMediaStreamKind::Other(value.to_string()),
    }
}

fn playback_mode_to_dto(mode: PlaybackMode) -> ClientPlaybackMode {
    match mode {
        PlaybackMode::DirectPlay => ClientPlaybackMode::DirectPlay,
        PlaybackMode::Remux => ClientPlaybackMode::Remux,
        PlaybackMode::Transcode => ClientPlaybackMode::Transcode,
        PlaybackMode::Denied => ClientPlaybackMode::Denied,
    }
}

fn playback_decision_reason_to_dto(reason: PlaybackDecisionReason) -> ClientPlaybackDecisionReason {
    match reason {
        PlaybackDecisionReason::Compatible => ClientPlaybackDecisionReason::Compatible,
        PlaybackDecisionReason::RequestedTranscodeOutput => {
            ClientPlaybackDecisionReason::RequestedTranscodeOutput
        }
        PlaybackDecisionReason::ClientDisabledDirectPlay => {
            ClientPlaybackDecisionReason::ClientDisabledDirectPlay
        }
        PlaybackDecisionReason::SourceContainerUnknown => {
            ClientPlaybackDecisionReason::SourceContainerUnknown
        }
        PlaybackDecisionReason::ClientContainerUnsupported => {
            ClientPlaybackDecisionReason::ClientContainerUnsupported
        }
        PlaybackDecisionReason::SourceCodecsUnsupported => {
            ClientPlaybackDecisionReason::SourceCodecsUnsupported
        }
        PlaybackDecisionReason::PolicyDenied => ClientPlaybackDecisionReason::PolicyDenied,
    }
}

fn playback_denial_to_dto(denial: PlaybackDenial) -> ClientPlaybackDenialDto {
    ClientPlaybackDenialDto {
        permission: playback_permission_to_dto(denial.permission),
        reason: playback_permission_decision_reason_to_dto(denial.reason),
    }
}

fn playback_permission_to_dto(permission: PlaybackPermission) -> ClientPlaybackPermission {
    match permission {
        PlaybackPermission::MediaPlayback => ClientPlaybackPermission::MediaPlayback,
        PlaybackPermission::DirectPlay => ClientPlaybackPermission::DirectPlay,
        PlaybackPermission::Remux => ClientPlaybackPermission::Remux,
        PlaybackPermission::AudioTranscode => ClientPlaybackPermission::AudioTranscode,
        PlaybackPermission::VideoTranscode => ClientPlaybackPermission::VideoTranscode,
        PlaybackPermission::RemotePlayback => ClientPlaybackPermission::RemotePlayback,
        PlaybackPermission::RemoteControl => ClientPlaybackPermission::RemoteControl,
        PlaybackPermission::Cast => ClientPlaybackPermission::Cast,
    }
}

fn playback_permission_decision_reason_to_dto(
    reason: PlaybackPermissionDecisionReason,
) -> ClientPlaybackPermissionDecisionReason {
    match reason {
        PlaybackPermissionDecisionReason::Allowed => {
            ClientPlaybackPermissionDecisionReason::Allowed
        }
        PlaybackPermissionDecisionReason::LibraryAccessDoesNotAllowPlay => {
            ClientPlaybackPermissionDecisionReason::LibraryAccessDoesNotAllowPlay
        }
        PlaybackPermissionDecisionReason::MediaPlaybackDisabled => {
            ClientPlaybackPermissionDecisionReason::MediaPlaybackDisabled
        }
        PlaybackPermissionDecisionReason::DirectPlayDisabled => {
            ClientPlaybackPermissionDecisionReason::DirectPlayDisabled
        }
        PlaybackPermissionDecisionReason::RemuxDisabled => {
            ClientPlaybackPermissionDecisionReason::RemuxDisabled
        }
        PlaybackPermissionDecisionReason::AudioTranscodeDisabled => {
            ClientPlaybackPermissionDecisionReason::AudioTranscodeDisabled
        }
        PlaybackPermissionDecisionReason::VideoTranscodeDisabled => {
            ClientPlaybackPermissionDecisionReason::VideoTranscodeDisabled
        }
        PlaybackPermissionDecisionReason::RemotePlaybackDisabled => {
            ClientPlaybackPermissionDecisionReason::RemotePlaybackDisabled
        }
        PlaybackPermissionDecisionReason::RemoteControlDisabled => {
            ClientPlaybackPermissionDecisionReason::RemoteControlDisabled
        }
        PlaybackPermissionDecisionReason::CastDisabled => {
            ClientPlaybackPermissionDecisionReason::CastDisabled
        }
    }
}

fn playback_target_kind_to_dto(kind: PlaybackTargetKind) -> ClientPlaybackTargetKind {
    match kind {
        PlaybackTargetKind::Browser => ClientPlaybackTargetKind::Browser,
        PlaybackTargetKind::NativeDesktop => ClientPlaybackTargetKind::NativeDesktop,
        PlaybackTargetKind::NativeMobile => ClientPlaybackTargetKind::NativeMobile,
        PlaybackTargetKind::NakoRemoteClient => ClientPlaybackTargetKind::NakoRemoteClient,
        PlaybackTargetKind::Chromecast => ClientPlaybackTargetKind::Chromecast,
        PlaybackTargetKind::DlnaRenderer => ClientPlaybackTargetKind::DlnaRenderer,
        PlaybackTargetKind::Airplay => ClientPlaybackTargetKind::Airplay,
    }
}

fn playback_target_network_scope_to_dto(
    scope: PlaybackTargetNetworkScope,
) -> ClientPlaybackTargetNetworkScope {
    match scope {
        PlaybackTargetNetworkScope::Local => ClientPlaybackTargetNetworkScope::Local,
        PlaybackTargetNetworkScope::Remote => ClientPlaybackTargetNetworkScope::Remote,
        PlaybackTargetNetworkScope::Unknown => ClientPlaybackTargetNetworkScope::Unknown,
    }
}

fn playback_target_transport_auth_to_dto(
    auth: PlaybackTargetTransportAuth,
) -> ClientPlaybackTargetTransportAuth {
    match auth {
        PlaybackTargetTransportAuth::Bearer => ClientPlaybackTargetTransportAuth::Bearer,
        PlaybackTargetTransportAuth::BrowserTicket => {
            ClientPlaybackTargetTransportAuth::BrowserTicket
        }
        PlaybackTargetTransportAuth::CastTicket => ClientPlaybackTargetTransportAuth::CastTicket,
        PlaybackTargetTransportAuth::None => ClientPlaybackTargetTransportAuth::None,
    }
}

fn renderer_control_capabilities_to_dto(
    capabilities: RendererControlCapabilities,
) -> ClientRendererControlCapabilitiesDto {
    ClientRendererControlCapabilitiesDto {
        commands: capabilities
            .commands
            .into_iter()
            .map(renderer_control_command_to_dto)
            .collect(),
    }
}

fn renderer_control_command_to_dto(
    command: RendererControlCommand,
) -> ClientRendererControlCommand {
    match command {
        RendererControlCommand::ShowItem => ClientRendererControlCommand::ShowItem,
        RendererControlCommand::Play => ClientRendererControlCommand::Play,
        RendererControlCommand::Pause => ClientRendererControlCommand::Pause,
        RendererControlCommand::Resume => ClientRendererControlCommand::Resume,
        RendererControlCommand::Seek => ClientRendererControlCommand::Seek,
        RendererControlCommand::Stop => ClientRendererControlCommand::Stop,
        RendererControlCommand::SetVolume => ClientRendererControlCommand::SetVolume,
    }
}

fn renderer_session_state_to_dto(state: RendererSessionState) -> ClientRendererSessionState {
    match state {
        RendererSessionState::Online => ClientRendererSessionState::Online,
        RendererSessionState::Offline => ClientRendererSessionState::Offline,
        RendererSessionState::Revoked => ClientRendererSessionState::Revoked,
    }
}

fn renderer_command_state_to_dto(state: RendererCommandState) -> ClientRendererCommandState {
    match state {
        RendererCommandState::Queued => ClientRendererCommandState::Queued,
        RendererCommandState::Delivered => ClientRendererCommandState::Delivered,
        RendererCommandState::Acknowledged => ClientRendererCommandState::Acknowledged,
        RendererCommandState::Failed => ClientRendererCommandState::Failed,
        RendererCommandState::Cancelled => ClientRendererCommandState::Cancelled,
    }
}

fn playback_session_mode_to_dto(mode: PlaybackSessionMode) -> ClientPlaybackSessionMode {
    match mode {
        PlaybackSessionMode::Direct => ClientPlaybackSessionMode::Direct,
        PlaybackSessionMode::Remux => ClientPlaybackSessionMode::Remux,
        PlaybackSessionMode::Hls => ClientPlaybackSessionMode::Hls,
    }
}

fn playback_session_state_to_dto(state: PlaybackSessionState) -> ClientPlaybackSessionState {
    match state {
        PlaybackSessionState::Active => ClientPlaybackSessionState::Active,
        PlaybackSessionState::Paused => ClientPlaybackSessionState::Paused,
        PlaybackSessionState::CancelRequested => ClientPlaybackSessionState::CancelRequested,
        PlaybackSessionState::Cancelled => ClientPlaybackSessionState::Cancelled,
        PlaybackSessionState::Ended => ClientPlaybackSessionState::Ended,
        PlaybackSessionState::Failed => ClientPlaybackSessionState::Failed,
    }
}

fn playback_session_client_capabilities_from_json(
    value: &str,
) -> Option<ClientPlaybackCapabilitiesDto> {
    let capabilities = serde_json::from_str::<ClientPlaybackCapabilities>(value).ok()?;

    Some(ClientPlaybackCapabilitiesDto {
        direct_play: capabilities.direct_play,
        containers: capabilities.containers,
        video_codecs: capabilities.video_codecs,
        audio_codecs: capabilities.audio_codecs,
    })
}

fn playback_capabilities_to_dto(
    capabilities: ClientPlaybackCapabilities,
) -> ClientPlaybackCapabilitiesDto {
    ClientPlaybackCapabilitiesDto {
        direct_play: capabilities.direct_play,
        containers: capabilities.containers,
        video_codecs: capabilities.video_codecs,
        audio_codecs: capabilities.audio_codecs,
    }
}

fn output_container_to_dto(container: OutputContainer) -> ClientOutputContainer {
    match container {
        OutputContainer::Hls => ClientOutputContainer::Hls,
        OutputContainer::Mp4 => ClientOutputContainer::Mp4,
        OutputContainer::Mkv => ClientOutputContainer::Mkv,
    }
}

fn transcode_session_kind_to_dto(kind: TranscodeSessionKind) -> ClientTranscodeSessionKind {
    match kind {
        TranscodeSessionKind::Remux => ClientTranscodeSessionKind::Remux,
        TranscodeSessionKind::HlsTranscode => ClientTranscodeSessionKind::HlsTranscode,
    }
}

fn transcode_session_state_to_dto(state: TranscodeSessionState) -> ClientTranscodeSessionState {
    match state {
        TranscodeSessionState::Planned => ClientTranscodeSessionState::Planned,
        TranscodeSessionState::Starting => ClientTranscodeSessionState::Starting,
        TranscodeSessionState::Running => ClientTranscodeSessionState::Running,
        TranscodeSessionState::CancelRequested => ClientTranscodeSessionState::CancelRequested,
        TranscodeSessionState::Cancelled => ClientTranscodeSessionState::Cancelled,
        TranscodeSessionState::Failed => ClientTranscodeSessionState::Failed,
        TranscodeSessionState::Finished => ClientTranscodeSessionState::Finished,
    }
}

fn transcode_failure_category_to_dto(
    category: TranscodeFailureCategory,
) -> ClientTranscodeFailureCategory {
    match category {
        TranscodeFailureCategory::InvalidRequest => ClientTranscodeFailureCategory::InvalidRequest,
        TranscodeFailureCategory::Plan => ClientTranscodeFailureCategory::InvalidRequest,
        TranscodeFailureCategory::Runner => ClientTranscodeFailureCategory::Runner,
        TranscodeFailureCategory::Probe | TranscodeFailureCategory::HardwareFallback => {
            ClientTranscodeFailureCategory::Runner
        }
        TranscodeFailureCategory::Timeout => ClientTranscodeFailureCategory::Timeout,
        TranscodeFailureCategory::Storage => ClientTranscodeFailureCategory::Storage,
        TranscodeFailureCategory::Staging | TranscodeFailureCategory::Budget => {
            ClientTranscodeFailureCategory::Storage
        }
        TranscodeFailureCategory::Stale => ClientTranscodeFailureCategory::Stale,
        TranscodeFailureCategory::Cancelled => ClientTranscodeFailureCategory::Cancelled,
        TranscodeFailureCategory::Unknown => ClientTranscodeFailureCategory::Unknown,
    }
}

fn public_transcode_failure_message(category: Option<TranscodeFailureCategory>) -> String {
    match category {
        Some(TranscodeFailureCategory::InvalidRequest) => "playback request was invalid",
        Some(TranscodeFailureCategory::Probe) => "playback media probing failed",
        Some(TranscodeFailureCategory::Plan) => "playback transcode planning failed",
        Some(TranscodeFailureCategory::Staging) => "playback staging operation failed",
        Some(TranscodeFailureCategory::Budget) => "playback resource budget was exhausted",
        Some(TranscodeFailureCategory::HardwareFallback) => {
            "playback hardware acceleration was unavailable"
        }
        Some(TranscodeFailureCategory::Runner) => "playback transcode runner failed",
        Some(TranscodeFailureCategory::Timeout) => "playback transcode operation timed out",
        Some(TranscodeFailureCategory::Storage) => "playback storage operation failed",
        Some(TranscodeFailureCategory::Stale) => "playback session was stale at startup",
        Some(TranscodeFailureCategory::Cancelled) => "playback session was cancelled",
        Some(TranscodeFailureCategory::Unknown) | None => "playback transcode operation failed",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use nako_core::{
        CanonicalMetadata, LibraryId, MediaItem, MediaItemId, MediaSource, MediaSourceId,
        TranscodeSessionId, TranscodeSessionKind, TranscodeSessionState, UserPrincipalId,
    };

    #[test]
    fn page_info_adapter_keeps_server_pagination_out_of_protocol_types() {
        let page = PageRequest {
            limit: 25,
            offset: 50,
        };

        let info = page_info_from_request(page, usize::MAX);

        assert_eq!(info.limit, 25);
        assert_eq!(info.offset, 50);
        assert_eq!(info.returned, u32::MAX);
    }

    #[test]
    fn media_item_dto_serializes_field_level_payload() {
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "DTO Demo".to_owned(),
                tags: vec!["favorite".to_owned()],
                ..CanonicalMetadata::default()
            },
        };

        let value = serde_json::to_value(media_item_to_dto(item)).unwrap();

        assert_eq!(value["kind"], "movie");
        assert_eq!(value["metadata"]["title"], "DTO Demo");
        assert_eq!(value["metadata"]["tags"][0], "favorite");
        assert!(value.get("input_json").is_none());
    }

    #[test]
    fn media_source_dto_hides_raw_locator() {
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "local:///Movies/Private/Demo.mkv".to_owned(),
            file_name: "Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256:demo".to_owned()),
        };

        let value = serde_json::to_value(media_source_to_dto(source)).unwrap();

        assert_eq!(value["file_name"], "Demo.mkv");
        assert!(value.get("locator").is_none());
    }

    #[test]
    fn transcode_session_response_hides_server_output_path() {
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id: MediaSourceId::new(),
            kind: TranscodeSessionKind::Remux,
            request_key: "transcode-profile:v1;kind=remux;container=mp4".to_owned(),
            output_path: PathBuf::from("cache/remux/output.mp4"),
            state: TranscodeSessionState::Finished,
            failure_category: None,
            failure_message: None,
            created_at: "2026-05-16T00:00:00Z".to_owned(),
            updated_at: "2026-05-16T00:01:00Z".to_owned(),
            started_at: Some("2026-05-16T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-16T00:01:00Z".to_owned()),
        };

        let response = transcode_session_response_from_record(session);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["session"]["kind"], "remux");
        assert_eq!(value["session"]["state"], "finished");
        assert!(value["session"].get("output_path").is_none());
    }

    #[test]
    fn transcode_session_response_redacts_raw_failure_message() {
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id: MediaSourceId::new(),
            kind: TranscodeSessionKind::Remux,
            request_key: "transcode-profile:v1;kind=remux;container=mp4".to_owned(),
            output_path: PathBuf::from("cache/remux/private/output.mp4"),
            state: TranscodeSessionState::Failed,
            failure_category: Some(TranscodeFailureCategory::Runner),
            failure_message: Some(
                "ffmpeg failed at C:\\secret\\movie.mkv with webdav:///Movies/secret.mkv"
                    .to_owned(),
            ),
            created_at: "2026-05-16T00:00:00Z".to_owned(),
            updated_at: "2026-05-16T00:01:00Z".to_owned(),
            started_at: Some("2026-05-16T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-16T00:01:00Z".to_owned()),
        };

        let response = transcode_session_response_from_record(session);
        let serialized = serde_json::to_string(&response).unwrap();

        assert_eq!(
            response.session.failure_category,
            Some(ClientTranscodeFailureCategory::Runner)
        );
        assert_eq!(
            response.session.failure_message.as_deref(),
            Some("playback transcode runner failed")
        );
        assert!(!serialized.contains("C:\\secret"));
        assert!(!serialized.contains("webdav:///"));
        assert!(!serialized.contains("cache/remux/private"));
    }

    #[test]
    fn playback_decision_dto_hides_internal_selection_plan() {
        let source_id = MediaSourceId::new();
        let library_id = LibraryId::new();
        let transcode_plan = TranscodePlan {
            input_locator: "local:///Movies/Demo.mkv".to_owned(),
            output_container: OutputContainer::Hls,
            video_codec: Some("h264".to_owned()),
            audio_codec: Some("aac".to_owned()),
        };
        let decision = PlaybackDecision {
            mode: PlaybackMode::Transcode,
            reason: nako_playback::PlaybackDecisionReason::ClientDisabledDirectPlay,
            selected_source: nako_playback::PlaybackSelectedSource {
                source_id,
                library_id,
                locator: "local:///Movies/Demo.mp4".to_owned(),
                file_name: "Demo.mp4".to_owned(),
            },
            execution: nako_playback::PlaybackExecutionPlan::Transcode(transcode_plan.clone()),
            direct_play: None,
            transcode_plan: Some(transcode_plan),
            denial: None,
        };

        let value = serde_json::to_value(playback_decision_to_dto(decision)).unwrap();

        assert_eq!(value["mode"], "transcode");
        assert_eq!(value["reason"], "client_disabled_direct_play");
        assert_eq!(value["transcode_plan"]["output_container"], "hls");
        assert!(value["transcode_plan"].get("input_locator").is_none());
        assert!(
            value["transcode_plan"]
                .get("hardware_acceleration")
                .is_none()
        );
        assert!(value.get("selected_source").is_none());
        assert!(value.get("execution").is_none());
    }

    #[test]
    fn user_playback_state_response_hides_principal_and_formats_timestamps() {
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let state = UserPlaybackState {
            principal_id: UserPrincipalId::local_admin(),
            item_id,
            source_id: Some(source_id),
            resume_position_ms: Some(120_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(10_000),
            updated_at_ms: 10_000,
            version: 2,
        };

        let response = user_playback_state_response_from_state(state);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["state"]["item_id"], item_id.to_string());
        assert_eq!(value["state"]["source_id"], source_id.to_string());
        let progress = value["state"]["progress_percent"].as_f64().unwrap();
        assert!((progress - 0.2).abs() < 0.000_001);
        assert_eq!(value["state"]["last_played_at"], "1970-01-01T00:00:10Z");
        assert_eq!(value["state"]["updated_at"], "1970-01-01T00:00:10Z");
        assert!(value["state"].get("principal_id").is_none());
        assert!(value["state"].get("user_id").is_none());
    }
}
