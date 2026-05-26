use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use nako_core::{
    AdminSettingsRepository, EffectiveLibraryAccess, IdentityAccessRepository, LibraryAccessPolicy,
    LibraryAccessPolicyFilter, LibraryAccessPolicyScope, LibraryId, ManagedArtworkRepository,
    MediaItemId, MediaRepository, MediaSource, MediaSourceId, NakoError, PageRequest, Result,
    RoleAssignment, SelectedArtworkId, SelectedArtworkRecord, User, UserId, UserPrincipalId,
    UserRole, UserSessionId, UserSessionRecord,
};
use nako_db::{
    DatabaseBackendCapabilities, DatabaseBackendKind, DatabaseConnectOptions, NakoDatabase,
};
use sha2::{Digest, Sha256};

use crate::config::{NakoServerConfig, resolve_database_url};

pub(crate) mod acquisition_intake;
mod addons;
mod artwork;
mod automation;
mod catalog;
mod composition;
mod job_runtime;
mod jobs;
mod library;
mod library_reconciliation;
mod managed_import;
mod metadata;
mod metadata_application;
mod metadata_runtime;
mod metadata_scan;
mod nfo;
pub(crate) mod playback;
mod playback_ticket;
mod runtime;
mod staging;
mod startup;
mod storage;
pub(crate) mod user_playback;
mod webhooks;

use acquisition_intake::AcquisitionIntakeAppService;
use addons::AddonAppService;
#[cfg(test)]
pub(crate) use addons::set_test_outbound_task_dispatch_secret;
use artwork::ManagedArtworkAppService;
pub(crate) use artwork::{ImageVariantRequest, ManagedArtworkImageBytes};
use automation::AutomationAppService;
use catalog::CatalogAppService;
use composition::{NakoAppComposition, NakoAppServices};
use jobs::{JobAppService, LibraryScanAppService};
use library::LibraryAppService;
use metadata::MetadataAppService;
use nfo::NfoAppService;
#[cfg(test)]
pub(crate) use playback::DirectPlayStreamBody;
use playback::PlaybackAppService;
pub(crate) use playback::{
    DirectPlaySourceBody, HlsSourceRequest, RemuxSourceDisposition, RemuxSourceRequest,
};
pub(crate) use playback_ticket::{
    BrowserPlaybackTicketMode, BrowserPlaybackTicketService, IssuedBrowserPlaybackTicket,
};
pub(crate) use runtime::RuntimeSupervisorDiagnostics;
#[cfg(test)]
use staging::cleanup_expired_staging_inputs;
use startup::ServerStartupReport;
use storage::StorageDiagnosticsAppService;
use user_playback::UserPlaybackAppService;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedSessionPrincipal {
    pub(crate) principal: nako_core::AuthenticatedPrincipal,
    pub(crate) session_id: UserSessionId,
}

const USER_SESSION_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1_000;
const USER_SESSION_TOKEN_PREFIX: &str = "nako_sess_";
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
    pub(crate) fn library(&self) -> LibraryAppService {
        self.services().library.clone()
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
    pub(crate) fn user_playback(&self) -> UserPlaybackAppService {
        self.services().user_playback.clone()
    }

    pub(crate) fn runtime_diagnostics(&self) -> RuntimeSupervisorDiagnostics {
        self.inner.runtime.diagnostics()
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
