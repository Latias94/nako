use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use nako_core::{
    AdminSettingsRepository, EffectiveLibraryAccess, IdentityAccessRepository,
    JobQueuePressureSummary, JobRepository, LibraryAccessPolicy, LibraryAccessPolicyFilter,
    LibraryAccessPolicyScope, LibraryId, ManagedArtworkRepository, MediaItemId, MediaRepository,
    MediaSource, MediaSourceId, NakoError, PageRequest, Result, RoleAssignment, SelectedArtworkId,
    SelectedArtworkRecord, User, UserId, UserInvitationId, UserInvitationRecord,
    UserInvitationStatus, UserPrincipalId, UserRole, UserSessionId, UserSessionRecord, UserStatus,
};
use nako_db::{
    DatabaseBackendCapabilities, DatabaseBackendKind, DatabaseConnectOptions, NakoDatabase,
};
use sha2::{Digest, Sha256};

use crate::{
    api_mapping::{
        admin_hardware_acceleration, admin_hardware_fallback, transcode_hardware_acceleration,
        transcode_hardware_fallback,
    },
    config::{NakoServerConfig, resolve_database_url},
};

pub(crate) mod acquisition_intake;
mod addons;
mod artwork;
mod automation;
pub(crate) mod casting;
mod catalog;
mod composition;
mod job_runtime;
mod jobs;
mod library;
mod library_reconciliation;
mod managed_import;
mod management_context;
mod metadata;
mod metadata_application;
mod metadata_runtime;
mod metadata_scan;
mod nfo;
pub(crate) mod playback;
mod playback_ticket;
pub(crate) mod renderer;
pub(crate) mod renderer_adapter;
mod renderer_transport_ticket;
mod runtime;
mod staging;
mod startup;
mod storage;
mod subtitle_sidecar;
pub(crate) mod user_playback;
pub(crate) mod user_playlist;
mod watch_folder_runtime;
mod watch_folder_suppression;
mod webhooks;

use acquisition_intake::AcquisitionIntakeAppService;
use addons::AddonAppService;
#[cfg(test)]
pub(crate) use addons::set_test_outbound_task_dispatch_secret;
use artwork::ManagedArtworkAppService;
pub(crate) use artwork::{ImageVariantRequest, ManagedArtworkImageBytes};
use automation::AutomationAppService;
use casting::CastingAppService;
use catalog::CatalogAppService;
use composition::{NakoAppComposition, NakoAppServices};
use jobs::{JobAppService, LibraryScanAppService};
use library::LibraryAppService;
use management_context::ManagementContextAppService;
pub(crate) use management_context::ManagementContextRequest;
use metadata::MetadataAppService;
use nfo::NfoAppService;
#[cfg(test)]
pub(crate) use playback::DirectPlayStreamBody;
use playback::PlaybackAppService;
pub(crate) use playback::{
    BrowserPlaybackTicketValidationRequest, DirectPlaySourceBody, DirectPlaybackPreflightRequest,
    DirectPlaybackSessionStreamRequest, DirectPlaybackStreamRequest, HlsPlaylistPlaybackRequest,
    HlsPlaylistSessionRequest, HlsSourceRequest, PlaybackSessionHeartbeatRequest,
    RemuxPlaybackPreflightRequest, RemuxPlaybackSessionStreamRequest, RemuxPlaybackStreamRequest,
    RemuxSourceRequest, RendererPlaybackTransportPlan, StartPlaybackSessionRequest,
    SubtitlePlaybackRequest,
};
pub(crate) use playback_ticket::{
    BrowserPlaybackTicketMode, BrowserPlaybackTicketService, IssuedBrowserPlaybackTicket,
};
pub(crate) use renderer::RegisterRendererAdapterSessionRequest;
use renderer::RendererAppService;
use renderer_adapter::RendererAdapterBridgeService;
pub(crate) use renderer_adapter::{
    BuildRendererAdapterCommandEnvelopeRequest, PublishRendererAdapterTargetRequest,
    RendererAdapterTargetRecord,
};
pub(crate) use renderer_transport_ticket::{
    IssueRendererTransportTicketRequest, RendererTransportTicketScope,
    RendererTransportTicketService, ValidateRendererTransportTicketRequest,
};
pub(crate) use runtime::{RuntimeResourceClassDiagnostics, RuntimeSupervisorDiagnostics};
#[cfg(test)]
use staging::cleanup_expired_staging_inputs;
use startup::ServerStartupReport;
use storage::StorageDiagnosticsAppService;
pub(crate) use storage::{
    StagingBudgetPolicySlice, StorageStagingPressureStatus, storage_staging_pressure_status,
};
use user_playback::UserPlaybackAppService;
use user_playlist::UserPlaylistAppService;
use watch_folder_runtime::WatchFolderRuntimeAppService;
pub(crate) use watch_folder_suppression::{
    BeginPlannedWatchFolderWriteSuppressionRequest, PlannedWatchFolderWriteCompletion,
    PlannedWatchFolderWriteSuppressionDiagnostic, WatchFolderSuppressionAppService,
};
use webhooks::WebhookAppService;

#[cfg(test)]
use playback::plan_direct_play_with_backend;

#[derive(Clone, Debug)]
pub struct NakoApp {
    inner: Arc<NakoAppComposition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DatabaseDiagnostics {
    pub(crate) backend_kind: DatabaseBackendKind,
    pub(crate) capabilities: DatabaseBackendCapabilities,
}

#[derive(Clone, Debug)]
pub(crate) struct IssuedUserSession {
    pub(crate) token: String,
    pub(crate) session: UserSessionRecord,
}

#[derive(Clone, Debug)]
pub(crate) struct IssuedUserInvitation {
    pub(crate) token: String,
    pub(crate) invitation: UserInvitationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedSessionPrincipal {
    pub(crate) principal: nako_core::AuthenticatedPrincipal,
    pub(crate) session_id: UserSessionId,
}

const USER_SESSION_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1_000;
const USER_SESSION_TOKEN_PREFIX: &str = "nako_sess_";
const USER_INVITATION_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const USER_INVITATION_TOKEN_PREFIX: &str = "nako_inv_";
const MIN_LOCAL_PASSWORD_LEN: usize = 8;

impl NakoApp {
    pub async fn new(config: NakoServerConfig) -> Result<Self> {
        let store = NakoDatabase::connect_with_options(DatabaseConnectOptions {
            backend: config.database_backend,
            url: resolve_database_url(&config)?,
            sqlite_runtime: None,
        })
        .await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: NakoServerConfig, store: NakoDatabase) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(NakoAppComposition::build(config, store).await?),
        })
    }

    #[must_use]
    pub fn config(&self) -> &NakoServerConfig {
        &self.inner.config
    }

    #[must_use]
    pub(crate) fn database_diagnostics(&self) -> DatabaseDiagnostics {
        DatabaseDiagnostics {
            backend_kind: self.inner.store.backend_kind(),
            capabilities: self.inner.store.capabilities(),
        }
    }

    fn services(&self) -> &NakoAppServices {
        &self.inner.services
    }

    #[must_use]
    pub(crate) fn addons(&self) -> AddonAppService {
        self.services().addons.clone()
    }

    #[must_use]
    pub(crate) fn acquisition_intake(&self) -> AcquisitionIntakeAppService {
        self.services().acquisition_intake.clone()
    }

    #[must_use]
    pub(crate) fn artwork(&self) -> ManagedArtworkAppService {
        self.services().artwork.clone()
    }

    #[must_use]
    pub(crate) fn automation(&self) -> AutomationAppService {
        self.services().automation.clone()
    }

    #[must_use]
    pub(crate) fn webhooks(&self) -> WebhookAppService {
        self.services().webhooks.clone()
    }

    #[must_use]
    pub(crate) fn catalog(&self) -> CatalogAppService {
        self.services().catalog.clone()
    }

    #[must_use]
    pub(crate) fn casting(&self) -> CastingAppService {
        self.services().casting.clone()
    }

    #[must_use]
    pub(crate) fn library(&self) -> LibraryAppService {
        self.services().library.clone()
    }

    #[must_use]
    pub(crate) fn management_context(&self) -> ManagementContextAppService {
        self.services().management_context.clone()
    }

    #[must_use]
    pub(crate) fn storage(&self) -> StorageDiagnosticsAppService {
        self.services().storage.clone()
    }

    #[must_use]
    pub(crate) fn jobs(&self) -> JobAppService {
        self.services().jobs.clone()
    }

    #[must_use]
    pub(crate) fn library_scan(&self) -> LibraryScanAppService {
        self.services().library_scan.clone()
    }

    #[must_use]
    pub(crate) fn watch_folder_runtime(&self) -> WatchFolderRuntimeAppService {
        self.services().watch_folder_runtime.clone()
    }

    #[must_use]
    pub(crate) fn watch_folder_suppression(&self) -> WatchFolderSuppressionAppService {
        self.services().watch_folder_suppression.clone()
    }

    #[must_use]
    pub(crate) fn nfo(&self) -> NfoAppService {
        self.services().nfo.clone()
    }

    #[must_use]
    pub(crate) fn metadata(&self) -> MetadataAppService {
        self.services().metadata.clone()
    }

    #[must_use]
    pub(crate) fn managed_import(&self) -> managed_import::ManagedImportAppService {
        self.services().managed_import.clone()
    }

    #[must_use]
    pub(crate) fn playback(&self) -> PlaybackAppService {
        self.services().playback.clone()
    }

    #[must_use]
    pub(crate) fn playback_tickets(&self) -> BrowserPlaybackTicketService {
        self.services().playback_tickets.clone()
    }

    #[must_use]
    pub(crate) fn renderer_transport_tickets(&self) -> RendererTransportTicketService {
        self.services().renderer_transport_tickets.clone()
    }

    #[must_use]
    pub(crate) fn renderer_adapters(&self) -> RendererAdapterBridgeService {
        self.services().renderer_adapters.clone()
    }

    #[must_use]
    pub(crate) fn renderer(&self) -> RendererAppService {
        self.services().renderer.clone()
    }

    #[must_use]
    pub(crate) fn user_playback(&self) -> UserPlaybackAppService {
        self.services().user_playback.clone()
    }

    #[must_use]
    pub(crate) fn user_playlist(&self) -> UserPlaylistAppService {
        self.services().user_playlist.clone()
    }

    pub(crate) fn runtime_diagnostics(&self) -> RuntimeSupervisorDiagnostics {
        self.inner.runtime.diagnostics()
    }

    pub(crate) fn runtime_resource_class_diagnostics(
        &self,
    ) -> Vec<RuntimeResourceClassDiagnostics> {
        self.inner.runtime_resource_classes.diagnostics()
    }

    pub(crate) async fn job_queue_pressure_diagnostics(
        &self,
    ) -> Result<Vec<JobQueuePressureSummary>> {
        self.inner.store.summarize_job_queue_pressure().await
    }

    pub(crate) fn startup_report(&self) -> &ServerStartupReport {
        &self.inner.startup_report
    }

    pub(crate) async fn get_user_by_principal(
        &self,
        principal: &UserPrincipalId,
    ) -> Result<Option<User>> {
        self.inner.store.get_user_by_principal(principal).await
    }

    pub(crate) async fn upsert_user(&self, user: &User) -> Result<()> {
        self.inner.store.upsert_user(user).await
    }

    pub(crate) async fn get_user(&self, user_id: UserId) -> Result<Option<User>> {
        self.inner.store.get_user(user_id).await
    }

    pub(crate) async fn list_users(&self, page: PageRequest) -> Result<Vec<User>> {
        self.inner.store.list_users(page).await
    }

    pub(crate) async fn create_user_invitation(
        &self,
        created_by_user_id: UserId,
        email_or_username: Option<String>,
        roles: Vec<UserRole>,
        expires_in_ms: Option<i64>,
    ) -> Result<IssuedUserInvitation> {
        validate_user_roles(&roles)?;
        let now_ms = current_time_ms()?;
        let ttl_ms = expires_in_ms.unwrap_or(USER_INVITATION_TTL_MS);
        if ttl_ms <= 0 {
            return Err(NakoError::InvalidInput {
                message: "invitation expires_in_ms must be greater than zero".to_owned(),
            });
        }
        let expires_at_ms = now_ms
            .checked_add(ttl_ms)
            .ok_or_else(|| NakoError::InvalidInput {
                message: "invitation expiry overflowed".to_owned(),
            })?;
        let email_or_username = email_or_username
            .map(|value| validate_user_text("email_or_username", value))
            .transpose()?;

        for _ in 0..8 {
            let token = generate_user_invitation_token();
            let invitation = UserInvitationRecord {
                id: UserInvitationId::new(),
                created_by_user_id,
                email_or_username: email_or_username.clone(),
                token_hash: hash_invitation_token(&token),
                roles: roles.clone(),
                status: UserInvitationStatus::Pending,
                expires_at_ms,
                redeemed_at_ms: None,
                redeemed_by_user_id: None,
                revoked_at_ms: None,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            };
            match self.inner.store.create_user_invitation(&invitation).await {
                Ok(()) => return Ok(IssuedUserInvitation { token, invitation }),
                Err(NakoError::Database { message }) if message.contains("UNIQUE") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(NakoError::Conflict {
            message: "could not allocate a unique user invitation token".to_owned(),
        })
    }

    pub(crate) async fn list_user_invitations(
        &self,
        page: PageRequest,
    ) -> Result<Vec<UserInvitationRecord>> {
        self.inner.store.list_user_invitations(page).await
    }

    pub(crate) async fn revoke_user_invitation(
        &self,
        invitation_id: UserInvitationId,
    ) -> Result<UserInvitationRecord> {
        self.inner
            .store
            .revoke_user_invitation(invitation_id, current_time_ms()?)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "user_invitation",
                id: invitation_id.to_string(),
            })
    }

    pub(crate) async fn redeem_user_invitation(
        &self,
        token: &str,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(IssuedUserSession, User, Vec<UserRole>)> {
        validate_local_password(password)?;
        let username = validate_user_text("username", username.to_owned())?;
        let display_name = validate_user_text("display_name", display_name.to_owned())?;
        let now_ms = current_time_ms()?;
        let invitation = self
            .inner
            .store
            .get_user_invitation_by_token_hash(&hash_invitation_token(token))
            .await?
            .ok_or_else(invalid_invitation)?;
        if !invitation.is_redeemable_at(now_ms) {
            return Err(invalid_invitation());
        }

        let user_id = UserId::new();
        let user = User {
            id: user_id,
            principal_id: UserPrincipalId::new(format!("local-user:{user_id}"))?,
            username,
            display_name,
            status: UserStatus::Active,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };
        let roles = invitation.roles.clone();
        let assignments = role_assignments_for_user(user.id, &roles, now_ms);

        let redeemed = self
            .inner
            .store
            .redeem_user_invitation(
                invitation.id,
                &user,
                &nako_core::LocalCredentialRecord {
                    user_id: user.id,
                    password_hash: hash_local_password(password)?,
                    updated_at_ms: now_ms,
                },
                &assignments,
                now_ms,
            )
            .await?
            .ok_or_else(invalid_invitation)?;
        if redeemed.redeemed_by_user_id != Some(user.id) {
            return Err(invalid_invitation());
        }
        let session = self.issue_user_session(user.id).await?;

        Ok((session, user, roles))
    }

    pub(crate) async fn set_local_password(&self, user_id: UserId, password: &str) -> Result<()> {
        validate_local_password(password)?;
        let user = self
            .get_user(user_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "user",
                id: user_id.to_string(),
            })?;
        if !user.status.can_authenticate() {
            return Err(NakoError::InvalidInput {
                message: "cannot set local password for disabled user".to_owned(),
            });
        }
        let now_ms = current_time_ms()?;
        self.inner
            .store
            .upsert_local_credential(&nako_core::LocalCredentialRecord {
                user_id,
                password_hash: hash_local_password(password)?,
                updated_at_ms: now_ms,
            })
            .await
    }

    pub(crate) async fn get_local_credential_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<nako_core::LocalCredentialRecord>> {
        self.inner.store.get_local_credential_by_user(user_id).await
    }

    pub(crate) async fn delete_local_password(&self, user_id: UserId) -> Result<()> {
        self.inner.store.delete_local_credential(user_id).await
    }

    pub(crate) async fn login_with_local_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(IssuedUserSession, User, Vec<UserRole>)> {
        let credential = self
            .inner
            .store
            .get_local_credential_by_username(username)
            .await?
            .ok_or_else(invalid_login)?;
        verify_local_password(password, &credential.password_hash)?;

        let user = self
            .get_user(credential.user_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "user",
                id: credential.user_id.to_string(),
            })?;
        if !user.status.can_authenticate() {
            return Err(invalid_login());
        }
        let roles = self
            .list_role_assignments(user.id)
            .await?
            .into_iter()
            .map(|assignment| assignment.role)
            .collect::<Vec<_>>();
        let session = self.issue_user_session(user.id).await?;

        Ok((session, user, roles))
    }

    pub(crate) async fn resolve_user_session_token(
        &self,
        token: &str,
    ) -> Result<Option<ResolvedSessionPrincipal>> {
        if token.trim().is_empty() {
            return Ok(None);
        }

        let now_ms = current_time_ms()?;
        let token_hash = hash_session_token(token);
        let Some(session) = self
            .inner
            .store
            .get_user_session_by_token_hash(&token_hash)
            .await?
        else {
            return Ok(None);
        };
        if !session.is_active_at(now_ms) {
            return Ok(None);
        }

        let Some(user) = self.get_user(session.user_id).await? else {
            return Ok(None);
        };
        if !user.status.can_authenticate() {
            return Ok(None);
        }

        let roles = self
            .list_role_assignments(user.id)
            .await?
            .into_iter()
            .map(|assignment| assignment.role)
            .collect::<Vec<_>>();
        let _ = self
            .inner
            .store
            .touch_user_session(session.id, now_ms)
            .await?;

        Ok(Some(ResolvedSessionPrincipal {
            principal: nako_core::AuthenticatedPrincipal {
                user_id: user.id,
                principal_id: user.principal_id,
                roles,
                bootstrap: false,
            },
            session_id: session.id,
        }))
    }

    pub(crate) async fn revoke_user_session(&self, id: UserSessionId) -> Result<bool> {
        let revoked = self
            .inner
            .store
            .revoke_user_session(id, current_time_ms()?)
            .await?;
        Ok(revoked.is_some())
    }

    pub(crate) async fn replace_role_assignments(
        &self,
        user_id: UserId,
        assignments: &[RoleAssignment],
    ) -> Result<()> {
        self.inner
            .store
            .replace_role_assignments(user_id, assignments)
            .await
    }

    pub(crate) async fn list_role_assignments(
        &self,
        user_id: UserId,
    ) -> Result<Vec<RoleAssignment>> {
        self.inner.store.list_role_assignments(user_id).await
    }

    pub(crate) async fn upsert_library_access_policy(
        &self,
        policy: &LibraryAccessPolicy,
    ) -> Result<()> {
        self.inner.store.upsert_library_access_policy(policy).await
    }

    pub(crate) async fn delete_library_access_policy(
        &self,
        scope: LibraryAccessPolicyScope,
        library_id: LibraryId,
    ) -> Result<()> {
        self.inner
            .store
            .delete_library_access_policy(scope, library_id)
            .await
    }

    pub(crate) async fn list_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<LibraryAccessPolicy>> {
        self.inner
            .store
            .list_library_access_policies(filter, page)
            .await
    }

    pub(crate) async fn resolve_effective_library_access(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectiveLibraryAccess> {
        self.inner
            .store
            .resolve_effective_library_access(user_id, library_id)
            .await
    }

    async fn issue_user_session(&self, user_id: UserId) -> Result<IssuedUserSession> {
        let now_ms = current_time_ms()?;
        let expires_at_ms =
            now_ms
                .checked_add(USER_SESSION_TTL_MS)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "user session expiry overflowed".to_owned(),
                })?;

        for _ in 0..8 {
            let token = generate_user_session_token();
            let session = UserSessionRecord {
                id: UserSessionId::new(),
                user_id,
                token_hash: hash_session_token(&token),
                created_at_ms: now_ms,
                last_seen_at_ms: now_ms,
                expires_at_ms,
                revoked_at_ms: None,
            };
            match self.inner.store.create_user_session(&session).await {
                Ok(()) => return Ok(IssuedUserSession { token, session }),
                Err(NakoError::Database { message }) if message.contains("UNIQUE") => continue,
                Err(error) => return Err(error),
            }
        }

        Err(NakoError::Conflict {
            message: "could not allocate a unique user session token".to_owned(),
        })
    }

    pub(crate) async fn get_media_source_record(
        &self,
        source_id: MediaSourceId,
    ) -> Result<Option<MediaSource>> {
        MediaRepository::get_media_source(&self.inner.store, source_id).await
    }

    pub(crate) async fn list_item_media_sources(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<MediaSource>> {
        MediaRepository::list_item_sources(&self.inner.store, item_id, PageRequest::first_page())
            .await
    }

    pub(crate) async fn get_selected_artwork_record(
        &self,
        selected_id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        ManagedArtworkRepository::get_selected_artwork(&self.inner.store, selected_id).await
    }

    pub(crate) fn shutdown_runtime(&self) {
        self.inner.shutdown_runtime();
    }

    pub(crate) async fn get_admin_metadata_raw_cache_settings(
        &self,
    ) -> Result<nako_api::admin::AdminMetadataRawCacheSettingsResponse> {
        let record = self
            .inner
            .store
            .get_admin_metadata_raw_cache_settings()
            .await?;

        Ok(admin_metadata_raw_cache_settings_response(
            configured_metadata_raw_cache_settings(&self.inner.config),
            record,
        ))
    }

    pub(crate) async fn update_admin_metadata_raw_cache_settings(
        &self,
        request: nako_api::admin::AdminUpdateMetadataRawCacheSettingsRequest,
    ) -> Result<nako_api::admin::AdminMetadataRawCacheSettingsResponse> {
        validate_metadata_raw_cache_settings_request(&request)?;
        let record = nako_core::AdminMetadataRawCacheSettingsRecord {
            settings: nako_core::AdminMetadataRawCacheSettings {
                retention_ms: request.retention_ms,
                cleanup_on_startup: request.cleanup_on_startup,
            },
            source: nako_core::AdminSettingsSource::Admin,
            effect: nako_core::AdminSettingsEffect::RequiresRestart,
            updated_at_ms: current_time_ms()?,
        };
        let record = self
            .inner
            .store
            .upsert_admin_metadata_raw_cache_settings(record)
            .await?;

        Ok(admin_metadata_raw_cache_settings_response(
            configured_metadata_raw_cache_settings(&self.inner.config),
            Some(record),
        ))
    }

    pub(crate) async fn get_admin_playback_runtime_settings(
        &self,
    ) -> Result<nako_api::admin::AdminPlaybackRuntimeSettingsResponse> {
        let record = self
            .inner
            .store
            .get_admin_settings_document(nako_core::AdminSettingsDocumentKey::PlaybackRuntime)
            .await?;

        admin_playback_runtime_settings_response(
            configured_playback_runtime_settings(&self.inner.config),
            record,
        )
    }

    pub(crate) async fn update_admin_playback_runtime_settings(
        &self,
        request: nako_api::admin::AdminUpdatePlaybackRuntimeSettingsRequest,
    ) -> Result<nako_api::admin::AdminPlaybackRuntimeSettingsResponse> {
        validate_playback_runtime_settings(&request.settings)?;
        let payload_json =
            serde_json::to_string(&request.settings).map_err(|err| NakoError::InvalidInput {
                message: format!("failed to serialize playback runtime settings: {err}"),
            })?;
        let record = nako_core::AdminSettingsDocumentRecord {
            key: nako_core::AdminSettingsDocumentKey::PlaybackRuntime,
            payload_json,
            source: nako_core::AdminSettingsSource::Admin,
            effect: nako_core::AdminSettingsEffect::RequiresRestart,
            updated_at_ms: current_time_ms()?,
        };
        let record = self
            .inner
            .store
            .upsert_admin_settings_document(record)
            .await?;

        admin_playback_runtime_settings_response(
            configured_playback_runtime_settings(&self.inner.config),
            Some(record),
        )
    }
}

fn validate_local_password(password: &str) -> Result<()> {
    if password.len() < MIN_LOCAL_PASSWORD_LEN {
        return Err(NakoError::InvalidInput {
            message: format!("local password must be at least {MIN_LOCAL_PASSWORD_LEN} characters"),
        });
    }
    if password.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: "local password cannot contain control characters".to_owned(),
        });
    }
    Ok(())
}

fn hash_local_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| NakoError::InvalidInput {
            message: format!("could not hash local password: {err}"),
        })
}

fn verify_local_password(password: &str, password_hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(password_hash).map_err(|_| invalid_login())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| invalid_login())
}

fn invalid_login() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid username or password".to_owned(),
    }
}

fn invalid_invitation() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid or expired invitation".to_owned(),
    }
}

fn generate_user_session_token() -> String {
    format!(
        "{USER_SESSION_TOKEN_PREFIX}{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn hash_session_token(token: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

fn generate_user_invitation_token() -> String {
    format!(
        "{USER_INVITATION_TOKEN_PREFIX}{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn hash_invitation_token(token: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

fn validate_user_text(field: &str, value: String) -> Result<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("{field} cannot be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: format!("{field} cannot contain control characters"),
        });
    }

    Ok(value)
}

fn validate_user_roles(roles: &[UserRole]) -> Result<()> {
    let mut unique = std::collections::HashSet::new();
    for role in roles {
        if !unique.insert(*role) {
            return Err(NakoError::InvalidInput {
                message: format!("duplicate Role in request: {}", role.as_str()),
            });
        }
    }

    Ok(())
}

fn role_assignments_for_user(
    user_id: UserId,
    roles: &[UserRole],
    granted_at_ms: i64,
) -> Vec<RoleAssignment> {
    roles
        .iter()
        .copied()
        .map(|role| RoleAssignment {
            user_id,
            role,
            granted_at_ms,
        })
        .collect()
}

fn validate_metadata_raw_cache_settings_request(
    request: &nako_api::admin::AdminUpdateMetadataRawCacheSettingsRequest,
) -> Result<()> {
    if request.retention_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "metadata raw cache retention_ms must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn admin_metadata_raw_cache_settings_response(
    configured: nako_core::AdminMetadataRawCacheSettings,
    record: Option<nako_core::AdminMetadataRawCacheSettingsRecord>,
) -> nako_api::admin::AdminMetadataRawCacheSettingsResponse {
    let (settings, source, effect, updated_at_ms) = match record {
        Some(record) => {
            let effect = if record.settings == configured {
                nako_core::AdminSettingsEffect::Active
            } else {
                nako_core::AdminSettingsEffect::RequiresRestart
            };

            (
                record.settings,
                nako_core::AdminSettingsSource::Admin,
                effect,
                Some(record.updated_at_ms),
            )
        }
        None => (
            configured,
            nako_core::AdminSettingsSource::Configured,
            nako_core::AdminSettingsEffect::Active,
            None,
        ),
    };

    nako_api::admin::AdminMetadataRawCacheSettingsResponse {
        admin_api_version: nako_api::admin::ADMIN_API_VERSION.to_owned(),
        retention_ms: settings.retention_ms,
        cleanup_on_startup: settings.cleanup_on_startup,
        source,
        effect,
        updated_at_ms,
    }
}

fn configured_metadata_raw_cache_settings(
    config: &NakoServerConfig,
) -> nako_core::AdminMetadataRawCacheSettings {
    nako_core::AdminMetadataRawCacheSettings {
        retention_ms: config.metadata.raw_cache_retention_ms,
        cleanup_on_startup: config.metadata.maintenance.raw_cache_cleanup_on_startup,
    }
}

fn validate_playback_runtime_settings(
    settings: &nako_api::admin::AdminPlaybackRuntimeSettingsPayload,
) -> Result<()> {
    if settings.cpu_concurrency == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime cpu_concurrency must be greater than zero".to_owned(),
        });
    }
    if settings.gpu_concurrency == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime gpu_concurrency must be greater than zero".to_owned(),
        });
    }
    if settings.remux_concurrency == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime remux_concurrency must be greater than zero".to_owned(),
        });
    }
    if settings.remote_stream_concurrency == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime remote_stream_concurrency must be greater than zero"
                .to_owned(),
        });
    }
    if settings.remote_stage_concurrency == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime remote_stage_concurrency must be greater than zero"
                .to_owned(),
        });
    }
    if settings.remux_timeout_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime remux_timeout_ms must be greater than zero".to_owned(),
        });
    }
    if settings.staging_retention_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime staging_retention_ms must be greater than zero".to_owned(),
        });
    }
    if settings.hls_segment_cleanup_enabled && settings.hls_segment_keep_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime hls_segment_keep_ms must be greater than zero when cleanup is enabled".to_owned(),
        });
    }
    if settings.transcode_throttle_enabled && settings.transcode_throttle_delay_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "playback runtime transcode_throttle_delay_ms must be greater than zero when throttling is enabled".to_owned(),
        });
    }

    Ok(())
}

fn admin_playback_runtime_settings_response(
    configured: nako_api::admin::AdminPlaybackRuntimeSettingsPayload,
    record: Option<nako_core::AdminSettingsDocumentRecord>,
) -> Result<nako_api::admin::AdminPlaybackRuntimeSettingsResponse> {
    let (settings, source, effect, updated_at_ms) = match record {
        Some(record) => {
            let settings: nako_api::admin::AdminPlaybackRuntimeSettingsPayload =
                serde_json::from_str(&record.payload_json).map_err(|err| NakoError::Database {
                    message: format!("invalid persisted playback runtime settings: {err}"),
                })?;
            let effect = if settings == configured {
                nako_core::AdminSettingsEffect::Active
            } else {
                nako_core::AdminSettingsEffect::RequiresRestart
            };

            (
                settings,
                nako_core::AdminSettingsSource::Admin,
                effect,
                Some(record.updated_at_ms),
            )
        }
        None => (
            configured,
            nako_core::AdminSettingsSource::Configured,
            nako_core::AdminSettingsEffect::Active,
            None,
        ),
    };

    Ok(nako_api::admin::AdminPlaybackRuntimeSettingsResponse {
        admin_api_version: nako_api::admin::ADMIN_API_VERSION.to_owned(),
        settings,
        source,
        effect,
        updated_at_ms,
    })
}

pub(super) fn configured_playback_runtime_settings(
    config: &NakoServerConfig,
) -> nako_api::admin::AdminPlaybackRuntimeSettingsPayload {
    nako_api::admin::AdminPlaybackRuntimeSettingsPayload {
        hardware_acceleration: admin_hardware_acceleration(config.transcode.hardware_acceleration),
        hardware_fallback: admin_hardware_fallback(config.transcode.hardware_fallback),
        cpu_concurrency: usize_to_u32(config.transcode.cpu_concurrency),
        gpu_concurrency: usize_to_u32(config.transcode.gpu_concurrency),
        remux_concurrency: usize_to_u32(config.remux_concurrency),
        remux_timeout_ms: config.remux_timeout_ms,
        remote_stream_concurrency: usize_to_u32(config.playback.remote_stream_concurrency),
        remote_stage_concurrency: usize_to_u32(config.playback.remote_stage_concurrency),
        staging_max_bytes: config.staging.max_bytes,
        staging_retention_ms: config.staging.retention_ms,
        staging_cleanup_on_startup: config.staging.cleanup_on_startup,
        transcode_artifact_retention_ms: config.playback.transcode_artifact_retention_ms,
        transcode_artifact_cleanup_on_startup: config
            .playback
            .transcode_artifact_cleanup_on_startup,
        hls_segment_cleanup_enabled: config.playback.hls_segment_cleanup_enabled,
        hls_segment_keep_ms: config.playback.hls_segment_keep_ms,
        transcode_throttle_enabled: config.playback.transcode_throttle_enabled,
        transcode_throttle_delay_ms: config.playback.transcode_throttle_delay_ms,
    }
}

pub(super) fn apply_playback_runtime_settings(
    config: &mut NakoServerConfig,
    settings: &nako_api::admin::AdminPlaybackRuntimeSettingsPayload,
) {
    config.transcode.hardware_acceleration =
        transcode_hardware_acceleration(settings.hardware_acceleration);
    config.transcode.hardware_fallback = transcode_hardware_fallback(settings.hardware_fallback);
    config.transcode.cpu_concurrency = settings.cpu_concurrency as usize;
    config.transcode.gpu_concurrency = settings.gpu_concurrency as usize;
    config.remux_concurrency = settings.remux_concurrency as usize;
    config.remux_timeout_ms = settings.remux_timeout_ms;
    config.playback.remote_stream_concurrency = settings.remote_stream_concurrency as usize;
    config.playback.remote_stage_concurrency = settings.remote_stage_concurrency as usize;
    config.staging.max_bytes = settings.staging_max_bytes;
    config.staging.retention_ms = settings.staging_retention_ms;
    config.staging.cleanup_on_startup = settings.staging_cleanup_on_startup;
    config.playback.transcode_artifact_retention_ms = settings.transcode_artifact_retention_ms;
    config.playback.transcode_artifact_cleanup_on_startup =
        settings.transcode_artifact_cleanup_on_startup;
    config.playback.hls_segment_cleanup_enabled = settings.hls_segment_cleanup_enabled;
    config.playback.hls_segment_keep_ms = settings.hls_segment_keep_ms;
    config.playback.transcode_throttle_enabled = settings.transcode_throttle_enabled;
    config.playback.transcode_throttle_delay_ms = settings.transcode_throttle_delay_ms;
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(crate) fn current_time_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("system time is before UNIX epoch: {err}"),
        })?;

    i64::try_from(duration.as_millis()).map_err(|err| NakoError::InvalidInput {
        message: format!("current timestamp does not fit i64 milliseconds: {err}"),
    })
}

#[cfg(test)]
mod tests;
