use std::sync::Arc;

use nako_core::*;
use nako_search::{SearchDocument, SearchHit, SearchIndex, SearchQuery};

use crate::{
    backend::{DatabaseBackendKind, DatabaseConnectOptions},
    postgres::PostgresStore,
    sqlite::SqliteStore,
};

trait DatabaseBackendAdapter:
    AcquisitionIntakeRepository
    + AdminSettingsRepository
    + AddonEventDeliveryRepository
    + AddonRepository
    + AddonTaskRunRepository
    + AutomationRepository
    + CatalogRepository
    + CatalogGovernanceRepository
    + IngestionFailureRepository
    + IdentityAccessRepository
    + JobRepository
    + JobLeaseRepository
    + EventOutboxRepository
    + LibraryRepository
    + LibraryItemRepository
    + MediaRepository
    + MediaProbeRepository
    + ArtworkTaskRepository
    + ArtworkCandidateRepository
    + ManagedArtworkRepository
    + ManagedImportRepository
    + NfoSidecarApplyRepository
    + MetadataRepository
    + ProviderMappingRepository
    + MetadataCandidateReviewRepository
    + SourceDuplicateRepository
    + LocalInferenceRepository
    + ScanRepository
    + DatabaseLifecycle
    + PlaybackPolicyRepository
    + PlaybackSessionRepository
    + RendererSessionRepository
    + TranscodeSessionRepository
    + UserPlaybackStateRepository
    + UserPlaylistRepository
    + VfsCacheRepository
    + StorageBackendHealthRepository
    + StagingManifestRepository
    + WebhookRepository
    + SearchIndex
    + std::fmt::Debug
    + Send
    + Sync
{
}

impl<T> DatabaseBackendAdapter for T where
    T: AcquisitionIntakeRepository
        + AdminSettingsRepository
        + AddonEventDeliveryRepository
        + AddonRepository
        + AddonTaskRunRepository
        + AutomationRepository
        + CatalogRepository
        + CatalogGovernanceRepository
        + IngestionFailureRepository
        + IdentityAccessRepository
        + JobRepository
        + JobLeaseRepository
        + EventOutboxRepository
        + LibraryRepository
        + LibraryItemRepository
        + MediaRepository
        + MediaProbeRepository
        + ArtworkTaskRepository
        + ArtworkCandidateRepository
        + ManagedArtworkRepository
        + ManagedImportRepository
        + NfoSidecarApplyRepository
        + MetadataRepository
        + ProviderMappingRepository
        + MetadataCandidateReviewRepository
        + SourceDuplicateRepository
        + LocalInferenceRepository
        + ScanRepository
        + DatabaseLifecycle
        + PlaybackPolicyRepository
        + PlaybackSessionRepository
        + RendererSessionRepository
        + TranscodeSessionRepository
        + UserPlaybackStateRepository
        + UserPlaylistRepository
        + VfsCacheRepository
        + StorageBackendHealthRepository
        + StagingManifestRepository
        + WebhookRepository
        + SearchIndex
        + std::fmt::Debug
        + Send
        + Sync
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatabaseBackendCapabilities {
    pub acquisition_intake: bool,
    pub lifecycle: bool,
    pub libraries: bool,
    pub jobs: bool,
    pub job_leases: bool,
    pub media: bool,
    pub scan_commits: bool,
    pub metadata: bool,
    pub catalog: bool,
    pub playback_sessions: bool,
    pub playback_state: bool,
    pub transcode_sessions: bool,
    pub event_outbox: bool,
    pub addons: bool,
    pub automation: bool,
    pub managed_artwork: bool,
    pub managed_import: bool,
    pub nfo_sidecar_apply: bool,
    pub vfs_cache: bool,
    pub storage_backend_health: bool,
    pub webhooks: bool,
    pub search_index: bool,
}

impl DatabaseBackendCapabilities {
    #[must_use]
    pub const fn production_ready() -> Self {
        Self {
            lifecycle: true,
            acquisition_intake: true,
            libraries: true,
            jobs: true,
            job_leases: true,
            media: true,
            scan_commits: true,
            metadata: true,
            catalog: true,
            playback_sessions: true,
            playback_state: true,
            transcode_sessions: true,
            event_outbox: true,
            addons: true,
            automation: true,
            managed_artwork: true,
            managed_import: true,
            nfo_sidecar_apply: true,
            vfs_cache: true,
            storage_backend_health: true,
            webhooks: true,
            search_index: true,
        }
    }

    #[must_use]
    pub const fn postgres_supported_scope() -> Self {
        Self {
            lifecycle: true,
            acquisition_intake: true,
            libraries: true,
            jobs: true,
            job_leases: true,
            media: true,
            scan_commits: true,
            metadata: true,
            catalog: true,
            playback_sessions: true,
            playback_state: true,
            transcode_sessions: true,
            event_outbox: true,
            addons: true,
            automation: true,
            managed_artwork: true,
            managed_import: true,
            nfo_sidecar_apply: true,
            vfs_cache: true,
            storage_backend_health: true,
            webhooks: true,
            search_index: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NakoDatabase {
    backend: Arc<dyn DatabaseBackendAdapter>,
    backend_kind: DatabaseBackendKind,
    capabilities: DatabaseBackendCapabilities,
}

impl NakoDatabase {
    pub async fn connect_with_options(options: DatabaseConnectOptions) -> Result<Self> {
        match options.backend {
            DatabaseBackendKind::Sqlite => {
                let sqlite = match options.sqlite_runtime {
                    Some(runtime) => {
                        SqliteStore::connect_with_runtime(&options.url, runtime).await?
                    }
                    None => SqliteStore::connect(&options.url).await?,
                };
                Ok(Self::from_sqlite(sqlite))
            }
            DatabaseBackendKind::Postgres => Ok(Self::from_postgres(
                PostgresStore::connect(&options.url).await?,
            )),
        }
    }

    pub async fn connect_in_memory() -> Result<Self> {
        Ok(Self::from_sqlite(SqliteStore::connect_in_memory().await?))
    }

    fn from_sqlite(sqlite: SqliteStore) -> Self {
        Self {
            backend: Arc::new(sqlite),
            backend_kind: DatabaseBackendKind::Sqlite,
            capabilities: DatabaseBackendCapabilities::production_ready(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_sqlite_for_tests(sqlite: SqliteStore) -> Self {
        Self::from_sqlite(sqlite)
    }

    fn from_postgres(postgres: PostgresStore) -> Self {
        Self {
            backend: Arc::new(postgres),
            backend_kind: DatabaseBackendKind::Postgres,
            capabilities: DatabaseBackendCapabilities::postgres_supported_scope(),
        }
    }

    #[must_use]
    pub const fn backend_kind(&self) -> DatabaseBackendKind {
        self.backend_kind
    }

    #[must_use]
    pub const fn capabilities(&self) -> DatabaseBackendCapabilities {
        self.capabilities
    }

    #[must_use]
    fn backend(&self) -> &dyn DatabaseBackendAdapter {
        self.backend.as_ref()
    }
}

#[async_trait::async_trait]
impl AcquisitionIntakeRepository for NakoDatabase {
    async fn upsert_acquisition_intake_candidate(
        &self,
        candidate: NewAcquisitionIntakeCandidate,
    ) -> Result<AcquisitionIntakeCandidateRecord> {
        self.backend()
            .upsert_acquisition_intake_candidate(candidate)
            .await
    }

    async fn get_acquisition_intake_candidate(
        &self,
        id: AcquisitionIntakeCandidateId,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        self.backend().get_acquisition_intake_candidate(id).await
    }

    async fn find_acquisition_intake_candidate_by_source_key(
        &self,
        target_library_id: LibraryId,
        source_kind: &AcquisitionIntakeSourceKind,
        source_key: &str,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        self.backend()
            .find_acquisition_intake_candidate_by_source_key(
                target_library_id,
                source_kind,
                source_key,
            )
            .await
    }

    async fn list_acquisition_intake_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<Vec<AcquisitionIntakeCandidateRecord>> {
        self.backend()
            .list_acquisition_intake_candidates(filter, page)
            .await
    }

    async fn set_acquisition_intake_candidate_state(
        &self,
        id: AcquisitionIntakeCandidateId,
        state: AcquisitionIntakeCandidateState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        self.backend()
            .set_acquisition_intake_candidate_state(id, state, updated_at_ms, diagnostics_json)
            .await
    }

    async fn link_acquisition_intake_candidate_managed_import_artifact(
        &self,
        id: AcquisitionIntakeCandidateId,
        managed_import_artifact_id: ManagedImportArtifactId,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        self.backend()
            .link_acquisition_intake_candidate_managed_import_artifact(
                id,
                managed_import_artifact_id,
                updated_at_ms,
                diagnostics_json,
            )
            .await
    }
}

#[async_trait::async_trait]
impl AdminSettingsRepository for NakoDatabase {
    async fn upsert_admin_metadata_raw_cache_settings(
        &self,
        record: AdminMetadataRawCacheSettingsRecord,
    ) -> Result<AdminMetadataRawCacheSettingsRecord> {
        self.backend()
            .upsert_admin_metadata_raw_cache_settings(record)
            .await
    }

    async fn get_admin_metadata_raw_cache_settings(
        &self,
    ) -> Result<Option<AdminMetadataRawCacheSettingsRecord>> {
        self.backend().get_admin_metadata_raw_cache_settings().await
    }

    async fn upsert_admin_settings_document(
        &self,
        record: AdminSettingsDocumentRecord,
    ) -> Result<AdminSettingsDocumentRecord> {
        self.backend().upsert_admin_settings_document(record).await
    }

    async fn get_admin_settings_document(
        &self,
        key: AdminSettingsDocumentKey,
    ) -> Result<Option<AdminSettingsDocumentRecord>> {
        self.backend().get_admin_settings_document(key).await
    }
}

#[async_trait::async_trait]
impl IdentityAccessRepository for NakoDatabase {
    async fn create_user_invitation(&self, invitation: &UserInvitationRecord) -> Result<()> {
        self.backend().create_user_invitation(invitation).await
    }

    async fn get_user_invitation(
        &self,
        invitation_id: UserInvitationId,
    ) -> Result<Option<UserInvitationRecord>> {
        self.backend().get_user_invitation(invitation_id).await
    }

    async fn get_user_invitation_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserInvitationRecord>> {
        self.backend()
            .get_user_invitation_by_token_hash(token_hash)
            .await
    }

    async fn list_user_invitations(&self, page: PageRequest) -> Result<Vec<UserInvitationRecord>> {
        self.backend().list_user_invitations(page).await
    }

    async fn mark_user_invitation_redeemed(
        &self,
        invitation_id: UserInvitationId,
        redeemed_by_user_id: UserId,
        redeemed_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        self.backend()
            .mark_user_invitation_redeemed(invitation_id, redeemed_by_user_id, redeemed_at_ms)
            .await
    }

    async fn redeem_user_invitation(
        &self,
        invitation_id: UserInvitationId,
        user: &User,
        credential: &LocalCredentialRecord,
        assignments: &[RoleAssignment],
        redeemed_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        self.backend()
            .redeem_user_invitation(invitation_id, user, credential, assignments, redeemed_at_ms)
            .await
    }

    async fn revoke_user_invitation(
        &self,
        invitation_id: UserInvitationId,
        revoked_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        self.backend()
            .revoke_user_invitation(invitation_id, revoked_at_ms)
            .await
    }

    async fn upsert_user(&self, user: &User) -> Result<()> {
        self.backend().upsert_user(user).await
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        self.backend().get_user(id).await
    }

    async fn get_user_by_principal(&self, principal_id: &UserPrincipalId) -> Result<Option<User>> {
        self.backend().get_user_by_principal(principal_id).await
    }

    async fn list_users(&self, page: PageRequest) -> Result<Vec<User>> {
        self.backend().list_users(page).await
    }

    async fn upsert_local_credential(&self, credential: &LocalCredentialRecord) -> Result<()> {
        self.backend().upsert_local_credential(credential).await
    }

    async fn get_local_credential_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<LocalCredentialRecord>> {
        self.backend().get_local_credential_by_user(user_id).await
    }

    async fn get_local_credential_by_username(
        &self,
        username: &str,
    ) -> Result<Option<LocalCredentialRecord>> {
        self.backend()
            .get_local_credential_by_username(username)
            .await
    }

    async fn delete_local_credential(&self, user_id: UserId) -> Result<()> {
        self.backend().delete_local_credential(user_id).await
    }

    async fn create_user_session(&self, session: &UserSessionRecord) -> Result<()> {
        self.backend().create_user_session(session).await
    }

    async fn get_user_session(&self, id: UserSessionId) -> Result<Option<UserSessionRecord>> {
        self.backend().get_user_session(id).await
    }

    async fn get_user_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserSessionRecord>> {
        self.backend()
            .get_user_session_by_token_hash(token_hash)
            .await
    }

    async fn touch_user_session(
        &self,
        id: UserSessionId,
        last_seen_at_ms: i64,
    ) -> Result<Option<UserSessionRecord>> {
        self.backend().touch_user_session(id, last_seen_at_ms).await
    }

    async fn revoke_user_session(
        &self,
        id: UserSessionId,
        revoked_at_ms: i64,
    ) -> Result<Option<UserSessionRecord>> {
        self.backend().revoke_user_session(id, revoked_at_ms).await
    }

    async fn replace_role_assignments(
        &self,
        user_id: UserId,
        assignments: &[RoleAssignment],
    ) -> Result<()> {
        self.backend()
            .replace_role_assignments(user_id, assignments)
            .await
    }

    async fn list_role_assignments(&self, user_id: UserId) -> Result<Vec<RoleAssignment>> {
        self.backend().list_role_assignments(user_id).await
    }

    async fn upsert_library_access_policy(&self, policy: &LibraryAccessPolicy) -> Result<()> {
        self.backend().upsert_library_access_policy(policy).await
    }

    async fn delete_library_access_policy(
        &self,
        scope: LibraryAccessPolicyScope,
        library_id: LibraryId,
    ) -> Result<()> {
        self.backend()
            .delete_library_access_policy(scope, library_id)
            .await
    }

    async fn list_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<LibraryAccessPolicy>> {
        self.backend()
            .list_library_access_policies(filter, page)
            .await
    }

    async fn resolve_effective_library_access(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectiveLibraryAccess> {
        self.backend()
            .resolve_effective_library_access(user_id, library_id)
            .await
    }
}

#[async_trait::async_trait]
impl PlaybackPolicyRepository for NakoDatabase {
    async fn upsert_playback_policy(&self, policy: &PlaybackPolicy) -> Result<()> {
        self.backend().upsert_playback_policy(policy).await
    }

    async fn delete_playback_policy(
        &self,
        scope: PlaybackPolicyScope,
        library_id: LibraryId,
    ) -> Result<()> {
        self.backend()
            .delete_playback_policy(scope, library_id)
            .await
    }

    async fn list_playback_policies(
        &self,
        filter: PlaybackPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackPolicy>> {
        self.backend().list_playback_policies(filter, page).await
    }

    async fn resolve_effective_playback_policy(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectivePlaybackPolicy> {
        self.backend()
            .resolve_effective_playback_policy(user_id, library_id)
            .await
    }
}

#[async_trait::async_trait]
impl AddonRepository for NakoDatabase {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord> {
        self.backend().upsert_addon_registration(addon).await
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        self.backend().get_addon_registration(id).await
    }

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        self.backend()
            .find_addon_registration_by_manifest_id(manifest_id)
            .await
    }

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>> {
        self.backend().list_addon_registrations(status).await
    }

    async fn update_addon_registration_status(
        &self,
        id: AddonId,
        status: AddonStatus,
    ) -> Result<Option<AddonRegistrationRecord>> {
        self.backend()
            .update_addon_registration_status(id, status)
            .await
    }

    async fn unregister_addon_registration(
        &self,
        id: AddonId,
    ) -> Result<Option<AddonRegistrationRecord>> {
        self.backend().unregister_addon_registration(id).await
    }

    async fn create_addon_token(&self, token: NewAddonToken) -> Result<AddonTokenRecord> {
        self.backend().create_addon_token(token).await
    }

    async fn get_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        self.backend().get_addon_token(id).await
    }

    async fn find_addon_token_by_hash(&self, token_hash: &str) -> Result<Option<AddonTokenRecord>> {
        self.backend().find_addon_token_by_hash(token_hash).await
    }

    async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<Vec<AddonTokenRecord>> {
        self.backend().list_addon_tokens(addon_id).await
    }

    async fn mark_addon_token_used(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        self.backend().mark_addon_token_used(id).await
    }

    async fn rotate_addon_token(
        &self,
        rotated_token_id: AddonTokenId,
        new_token: NewAddonToken,
    ) -> Result<(AddonTokenRecord, AddonTokenRecord)> {
        self.backend()
            .rotate_addon_token(rotated_token_id, new_token)
            .await
    }

    async fn revoke_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        self.backend().revoke_addon_token(id).await
    }

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>> {
        self.backend().replace_addon_grants(addon_id, grants).await
    }

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>> {
        self.backend().list_addon_grants(addon_id).await
    }

    async fn replace_addon_routing_plans(
        &self,
        addon_id: AddonId,
        plans: Vec<NewAddonRoutingPlan>,
    ) -> Result<Vec<AddonRoutingPlanRecord>> {
        self.backend()
            .replace_addon_routing_plans(addon_id, plans)
            .await
    }

    async fn list_addon_routing_plans(
        &self,
        addon_id: AddonId,
    ) -> Result<Vec<AddonRoutingPlanRecord>> {
        self.backend().list_addon_routing_plans(addon_id).await
    }

    async fn create_addon_side_effect(
        &self,
        side_effect: NewAddonSideEffect,
    ) -> Result<AddonSideEffectRecord> {
        self.backend().create_addon_side_effect(side_effect).await
    }

    async fn find_addon_side_effect_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonSideEffectRecord>> {
        self.backend()
            .find_addon_side_effect_by_idempotency_key(addon_id, idempotency_key)
            .await
    }

    async fn set_addon_side_effect_apply_outcome(
        &self,
        id: AddonSideEffectId,
        outcome: AddonSideEffectApplyOutcome,
    ) -> Result<AddonSideEffectRecord> {
        self.backend()
            .set_addon_side_effect_apply_outcome(id, outcome)
            .await
    }
}

#[async_trait::async_trait]
impl AddonTaskRunRepository for NakoDatabase {
    async fn create_addon_task_run(
        &self,
        job: NewJob,
        run: NewAddonTaskRun,
    ) -> Result<CreatedAddonTaskRun> {
        self.backend().create_addon_task_run(job, run).await
    }

    async fn get_addon_task_run(&self, job_id: JobId) -> Result<Option<AddonTaskRunRecord>> {
        self.backend().get_addon_task_run(job_id).await
    }

    async fn list_addon_task_runs(
        &self,
        filter: AddonTaskRunListFilter,
        page: PageRequest,
    ) -> Result<Vec<AddonTaskRunRecord>> {
        self.backend().list_addon_task_runs(filter, page).await
    }

    async fn claim_next_addon_task_run(
        &self,
        request: AddonTaskRunClaimRequest,
    ) -> Result<Option<LeasedAddonTaskRun>> {
        self.backend().claim_next_addon_task_run(request).await
    }

    async fn report_addon_task_run_progress(
        &self,
        progress: ReportAddonTaskRunProgress,
    ) -> Result<LeasedAddonTaskRun> {
        self.backend()
            .report_addon_task_run_progress(progress)
            .await
    }

    async fn complete_addon_task_run(
        &self,
        completion: CompleteAddonTaskRun,
    ) -> Result<AddonTaskRunRecord> {
        self.backend().complete_addon_task_run(completion).await
    }

    async fn fail_addon_task_run(&self, failure: FailAddonTaskRun) -> Result<AddonTaskRunRecord> {
        self.backend().fail_addon_task_run(failure).await
    }

    async fn cancel_addon_task_run(
        &self,
        cancellation: CancelAddonTaskRun,
    ) -> Result<AddonTaskRunRecord> {
        self.backend().cancel_addon_task_run(cancellation).await
    }

    async fn find_addon_task_run_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonTaskRunRecord>> {
        self.backend()
            .find_addon_task_run_by_idempotency_key(addon_id, idempotency_key)
            .await
    }
}

#[async_trait::async_trait]
impl AutomationRepository for NakoDatabase {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord> {
        self.backend().upsert_automation_provider(provider).await
    }

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>> {
        self.backend().get_automation_provider(id).await
    }

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>> {
        self.backend().list_enabled_automation_providers().await
    }

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord> {
        self.backend().create_automation_artifact(artifact).await
    }

    async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>> {
        self.backend().get_automation_artifact(id).await
    }

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord> {
        self.backend()
            .set_automation_artifact_status(id, status)
            .await
    }

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        self.backend()
            .list_automation_artifacts_for_job(job_id)
            .await
    }

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        self.backend()
            .list_automation_artifacts_for_item(item_id, page)
            .await
    }

    async fn list_generated_artifact_proposals(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactProposal>> {
        self.backend().list_generated_artifact_proposals(page).await
    }

    async fn find_generated_artifact_metadata_apply_outcome(
        &self,
        artifact_id: AutomationArtifactId,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataApplyOutcomeRecord>> {
        self.backend()
            .find_generated_artifact_metadata_apply_outcome(artifact_id, idempotency_key)
            .await
    }

    async fn get_generated_artifact_metadata_apply_outcome(
        &self,
        outcome_id: GeneratedArtifactMetadataApplyOutcomeId,
    ) -> Result<Option<GeneratedArtifactMetadataApplyOutcomeRecord>> {
        self.backend()
            .get_generated_artifact_metadata_apply_outcome(outcome_id)
            .await
    }

    async fn list_generated_artifact_metadata_apply_outcomes(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactMetadataApplyOutcomeRecord>> {
        self.backend()
            .list_generated_artifact_metadata_apply_outcomes(page)
            .await
    }

    async fn list_generated_artifact_metadata_apply_recovery_entries(
        &self,
        filter: GeneratedArtifactMetadataApplyRecoveryFilter,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactMetadataApplyRecoveryEntryRecord>> {
        self.backend()
            .list_generated_artifact_metadata_apply_recovery_entries(filter, page)
            .await
    }

    async fn commit_generated_artifact_metadata_apply_outcome(
        &self,
        commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord> {
        self.backend()
            .commit_generated_artifact_metadata_apply_outcome(commit)
            .await
    }

    async fn get_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        self.backend()
            .get_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await
    }

    async fn find_generated_artifact_metadata_bulk_apply_batch(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        self.backend()
            .find_generated_artifact_metadata_bulk_apply_batch(idempotency_key)
            .await
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        self.backend()
            .commit_generated_artifact_metadata_bulk_apply_batch(commit)
            .await
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        self.backend()
            .commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(commit)
            .await
    }

    async fn update_generated_artifact_metadata_bulk_apply_batch_status(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
        expected: GeneratedArtifactMetadataBulkApplyBatchStatus,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        self.backend()
            .update_generated_artifact_metadata_bulk_apply_batch_status(batch_id, expected, status)
            .await
    }
}

#[async_trait::async_trait]
impl CatalogRepository for NakoDatabase {
    async fn replace_item_catalog_graph(
        &self,
        item_id: MediaItemId,
        replacement: &CatalogItemGraphReplacement,
    ) -> Result<()> {
        self.backend()
            .replace_item_catalog_graph(item_id, replacement)
            .await
    }

    async fn commit_item_projection(&self, commit: &CatalogItemProjectionCommit) -> Result<()> {
        self.backend().commit_item_projection(commit).await
    }

    async fn upsert_search_projection(&self, projection: &CatalogSearchProjection) -> Result<()> {
        self.backend().upsert_search_projection(projection).await
    }

    async fn upsert_person(&self, person: &Person) -> Result<()> {
        self.backend().upsert_person(person).await
    }

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>> {
        self.backend().get_person(id).await
    }

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>> {
        self.backend().find_person_by_external_id(external_id).await
    }

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>> {
        self.backend().find_person_by_name(name).await
    }

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>> {
        self.backend().list_people(page).await
    }

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()> {
        self.backend().upsert_item_credit(credit).await
    }

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().clear_item_credits(item_id).await
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        self.backend().list_item_credits(item_id).await
    }

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>> {
        self.backend().list_person_credits(person_id).await
    }

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        self.backend().list_person_items(person_id, page).await
    }

    async fn upsert_genre(&self, genre: &Genre) -> Result<()> {
        self.backend().upsert_genre(genre).await
    }

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>> {
        self.backend().get_genre(id).await
    }

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Genre>> {
        self.backend().find_genre_by_name_source(name, source).await
    }

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>> {
        self.backend().list_genres(page).await
    }

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()> {
        self.backend().upsert_item_genre(item_genre).await
    }

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().clear_item_genres(item_id).await
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        self.backend().list_item_genres(item_id).await
    }

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        self.backend().list_genre_items(genre_id, page).await
    }

    async fn upsert_tag(&self, tag: &Tag) -> Result<()> {
        self.backend().upsert_tag(tag).await
    }

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        self.backend().get_tag(id).await
    }

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Tag>> {
        self.backend().find_tag_by_name_source(name, source).await
    }

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>> {
        self.backend().list_tags(page).await
    }

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()> {
        self.backend().upsert_item_tag(item_tag).await
    }

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().clear_item_tags(item_id).await
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        self.backend().list_item_tags(item_id).await
    }

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>> {
        self.backend().list_tag_items(tag_id, page).await
    }

    async fn upsert_collection(&self, collection: &Collection) -> Result<()> {
        self.backend().upsert_collection(collection).await
    }

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        self.backend().get_collection(id).await
    }

    async fn find_collection_by_external_id(
        &self,
        external_id: &ExternalId,
    ) -> Result<Option<Collection>> {
        self.backend()
            .find_collection_by_external_id(external_id)
            .await
    }

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Collection>> {
        self.backend()
            .find_collection_by_name_source(name, source)
            .await
    }

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>> {
        self.backend().list_collections(page).await
    }

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()> {
        self.backend().upsert_collection_item(item).await
    }

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().clear_item_collections(item_id).await
    }

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>> {
        self.backend().list_item_collections(item_id).await
    }

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>> {
        self.backend().list_collection_items(collection_id).await
    }

    async fn upsert_studio(&self, studio: &Studio) -> Result<()> {
        self.backend().upsert_studio(studio).await
    }

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>> {
        self.backend().get_studio(id).await
    }

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>> {
        self.backend().find_studio_by_external_id(external_id).await
    }

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Studio>> {
        self.backend()
            .find_studio_by_name_source(name, source)
            .await
    }

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>> {
        self.backend().list_studios(page).await
    }

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()> {
        self.backend().upsert_item_studio(item_studio).await
    }

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().clear_item_studios(item_id).await
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        self.backend().list_item_studios(item_id).await
    }

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()> {
        self.backend().upsert_image_asset(image).await
    }

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>> {
        self.backend().get_image_asset(id).await
    }

    async fn find_image_asset_by_source(
        &self,
        owner: &ImageOwner,
        kind: &ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>> {
        self.backend()
            .find_image_asset_by_source(owner, kind, source_uri)
            .await
    }

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>> {
        self.backend().list_item_images(item_id).await
    }
}

#[async_trait::async_trait]
impl CatalogGovernanceRepository for NakoDatabase {
    async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>> {
        self.backend()
            .list_catalog_governance_items(filter, page)
            .await
    }
}

#[async_trait::async_trait]
impl IngestionFailureRepository for NakoDatabase {
    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord> {
        self.backend().record_ingestion_failure(failure).await
    }

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.backend()
            .resolve_ingestion_failure(library_id, phase, target_uri, resolved_at_ms)
            .await
    }

    async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        ignored_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.backend()
            .ignore_ingestion_failure(library_id, phase, target_uri, ignored_at_ms)
            .await
    }

    async fn list_ingestion_failures(
        &self,
        filter: IngestionFailureFilter,
        page: PageRequest,
    ) -> Result<Vec<IngestionFailureRecord>> {
        self.backend().list_ingestion_failures(filter, page).await
    }

    async fn count_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: IngestionFailureStatus,
    ) -> Result<u64> {
        self.backend()
            .count_ingestion_failures(library_id, phase, status)
            .await
    }
}

#[async_trait::async_trait]
impl JobRepository for NakoDatabase {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job> {
        self.backend().enqueue_job(job).await
    }

    async fn enqueue_job_retry(&self, retry: EnqueueJobRetry) -> Result<Job> {
        self.backend().enqueue_job_retry(retry).await
    }

    async fn start_job(&self, id: JobId) -> Result<Job> {
        self.backend().start_job(id).await
    }

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job> {
        self.backend().succeed_job(id, summary_json).await
    }

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job> {
        self.backend().fail_job(id, error).await
    }

    async fn fail_unfinished_jobs(&self, error: String) -> Result<u64> {
        self.backend().fail_unfinished_jobs(error).await
    }

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
        self.backend().get_job(id).await
    }

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>> {
        self.backend().list_jobs(filter, page).await
    }

    async fn summarize_job_queue_pressure(&self) -> Result<Vec<JobQueuePressureSummary>> {
        self.backend().summarize_job_queue_pressure().await
    }
}

#[async_trait::async_trait]
impl JobLeaseRepository for NakoDatabase {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        self.backend().claim_next_job_lease(request).await
    }

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob> {
        self.backend().heartbeat_job_lease(heartbeat).await
    }

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job> {
        self.backend().succeed_leased_job(completion).await
    }

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job> {
        self.backend().fail_leased_job(failure).await
    }

    async fn request_job_cancellation(
        &self,
        request: RequestJobCancellation,
    ) -> Result<JobCancellationRequestRecord> {
        self.backend().request_job_cancellation(request).await
    }

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job> {
        self.backend().cancel_leased_job(cancellation).await
    }

    async fn recover_expired_job_leases(&self, recovery: RecoverExpiredJobLeases) -> Result<u64> {
        self.backend().recover_expired_job_leases(recovery).await
    }
}

#[async_trait::async_trait]
impl EventOutboxRepository for NakoDatabase {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        self.backend().enqueue_outbox_event(event).await
    }

    async fn get_outbox_event(&self, id: EventId) -> Result<Option<OutboxEventRecord>> {
        self.backend().get_outbox_event(id).await
    }

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>> {
        self.backend()
            .find_outbox_event_by_idempotency_key(kind, idempotency_key)
            .await
    }

    async fn list_outbox_events(
        &self,
        filter: OutboxEventListFilter,
        page: PageRequest,
    ) -> Result<Vec<OutboxEventRecord>> {
        self.backend().list_outbox_events(filter, page).await
    }
}

#[async_trait::async_trait]
impl AddonEventDeliveryRepository for NakoDatabase {
    async fn create_addon_event_delivery_attempt(
        &self,
        attempt: NewAddonEventDeliveryAttempt,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        self.backend()
            .create_addon_event_delivery_attempt(attempt)
            .await
    }

    async fn claim_addon_event_delivery_attempt(
        &self,
        claim: ClaimAddonEventDeliveryAttempt,
    ) -> Result<Option<AddonEventDeliveryAttemptRecord>> {
        self.backend()
            .claim_addon_event_delivery_attempt(claim)
            .await
    }

    async fn set_addon_event_delivery_attempt_result(
        &self,
        id: AddonEventDeliveryAttemptId,
        status: AddonEventDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        self.backend()
            .set_addon_event_delivery_attempt_result(id, status, http_status, error, next_retry_at)
            .await
    }

    async fn list_addon_event_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>> {
        self.backend()
            .list_addon_event_delivery_attempts(event_id)
            .await
    }

    async fn list_addon_event_delivery_attempts_for_addon(
        &self,
        addon_id: AddonId,
        event_id: EventId,
        declaration_id: &str,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>> {
        self.backend()
            .list_addon_event_delivery_attempts_for_addon(addon_id, event_id, declaration_id)
            .await
    }

    async fn list_addon_event_scheduler_work(
        &self,
        event_id: EventId,
    ) -> Result<Vec<AddonEventSchedulerWorkRecord>> {
        self.backend()
            .list_addon_event_scheduler_work(event_id)
            .await
    }
}

#[async_trait::async_trait]
impl LibraryRepository for NakoDatabase {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        self.backend().upsert_library(library).await
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        self.backend().get_library(id).await
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        self.backend().list_libraries(page).await
    }
}

#[async_trait::async_trait]
impl LibraryItemRepository for NakoDatabase {
    async fn upsert_library_item_state(&self, state: &LibraryItemState) -> Result<()> {
        self.backend().upsert_library_item_state(state).await
    }

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>> {
        self.backend()
            .get_library_item_state(library_id, item_id)
            .await
    }

    async fn list_library_item_states_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<LibraryItemState>> {
        self.backend()
            .list_library_item_states_for_item(item_id)
            .await
    }

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>> {
        self.backend()
            .find_library_item_by_kind_parent_title(library_id, kind, parent_id, title)
            .await
    }
}

#[async_trait::async_trait]
impl MediaRepository for NakoDatabase {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()> {
        self.backend().upsert_media_item(item).await
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        self.backend().get_media_item(id).await
    }

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>> {
        self.backend().list_media_items(page).await
    }

    async fn list_media_items_for_library(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        self.backend()
            .list_media_items_for_library(library_id, page)
            .await
    }

    async fn list_library_item_added_at(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<LibraryItemAddedAt>> {
        self.backend().list_library_item_added_at(library_id).await
    }

    async fn upsert_media_source(&self, source: &MediaSource) -> Result<()> {
        self.backend().upsert_media_source(source).await
    }

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>> {
        self.backend().get_media_source(id).await
    }

    async fn get_media_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>> {
        self.backend()
            .get_media_source_by_locator(library_id, locator)
            .await
    }

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        self.backend().list_item_sources(item_id, page).await
    }

    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        self.backend().list_media_sources(library_id, page).await
    }
}

#[async_trait::async_trait]
impl MediaProbeRepository for NakoDatabase {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        self.backend().upsert_media_probe(source_id, result).await
    }

    async fn get_media_probe(&self, source_id: MediaSourceId) -> Result<Option<MediaProbeResult>> {
        self.backend().get_media_probe(source_id).await
    }
}

#[async_trait::async_trait]
impl ArtworkTaskRepository for NakoDatabase {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()> {
        self.backend().enqueue_artwork_task(task).await
    }

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>> {
        self.backend().get_artwork_task(id).await
    }

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>> {
        self.backend().list_artwork_tasks(page).await
    }
}

#[async_trait::async_trait]
impl ArtworkCandidateRepository for NakoDatabase {
    async fn create_artwork_candidate(
        &self,
        candidate: NewArtworkCandidate,
    ) -> Result<ArtworkCandidateRecord> {
        self.backend().create_artwork_candidate(candidate).await
    }

    async fn get_artwork_candidate(
        &self,
        id: ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        self.backend().get_artwork_candidate(id).await
    }

    async fn set_artwork_candidate_status(
        &self,
        id: ArtworkCandidateId,
        status: ArtworkCandidateStatus,
    ) -> Result<ArtworkCandidateRecord> {
        self.backend()
            .set_artwork_candidate_status(id, status)
            .await
    }

    async fn find_artwork_candidate_by_source(
        &self,
        addon_id: AddonId,
        library_id: LibraryId,
        item_id: MediaItemId,
        kind: &ImageKind,
        source_kind: ArtworkCandidateSourceKind,
        source_uri: &str,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        self.backend()
            .find_artwork_candidate_by_source(
                addon_id,
                library_id,
                item_id,
                kind,
                source_kind,
                source_uri,
            )
            .await
    }

    async fn list_artwork_candidates_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ArtworkCandidateRecord>> {
        self.backend()
            .list_artwork_candidates_for_item(item_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl ManagedArtworkRepository for NakoDatabase {
    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord> {
        self.backend()
            .accept_managed_artwork_candidate_ingest(candidate_id, ingest, job)
            .await
    }

    async fn get_managed_artwork_ingest(
        &self,
        id: ManagedArtworkIngestId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        self.backend().get_managed_artwork_ingest(id).await
    }

    async fn find_managed_artwork_ingest_by_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        self.backend()
            .find_managed_artwork_ingest_by_candidate(candidate_id)
            .await
    }

    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>> {
        self.backend()
            .claim_next_queued_managed_artwork_ingest()
            .await
    }

    async fn commit_managed_artwork_artifact(
        &self,
        ingest_id: ManagedArtworkIngestId,
        artifact: NewManagedArtworkArtifact,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        self.backend()
            .commit_managed_artwork_artifact(ingest_id, artifact, job_summary_json)
            .await
    }

    async fn fail_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        self.backend()
            .fail_managed_artwork_ingest(ingest_id, failure_code, job_error, job_summary_json)
            .await
    }

    async fn fail_unfinished_managed_artwork_ingests(
        &self,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<u64> {
        self.backend()
            .fail_unfinished_managed_artwork_ingests(failure_code, job_error, job_summary_json)
            .await
    }

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord> {
        self.backend()
            .requeue_managed_artwork_ingest(ingest_id)
            .await
    }

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>> {
        self.backend().get_managed_artwork_artifact(id).await
    }

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        self.backend().publish_selected_artwork(artifact_id).await
    }

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        self.backend()
            .publish_selected_artwork_for_item_kind(item_id, kind, artifact_id)
            .await
    }

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord> {
        self.backend()
            .unpublish_selected_artwork_for_item_kind(item_id, kind)
            .await
    }

    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        self.backend().get_selected_artwork(id).await
    }

    async fn list_selected_artwork_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<SelectedArtworkRecord>> {
        self.backend().list_selected_artwork_for_item(item_id).await
    }

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot> {
        self.backend()
            .get_managed_artwork_gallery_for_item(item_id, page)
            .await
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        self.backend()
            .list_managed_artwork_artifact_lifecycle(filter, page)
            .await
    }

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport> {
        self.backend()
            .cleanup_unselected_managed_artwork_artifacts(page)
            .await
    }
}

#[async_trait::async_trait]
impl ManagedImportRepository for NakoDatabase {
    async fn upsert_managed_import_artifact(
        &self,
        artifact: NewManagedImportArtifact,
    ) -> Result<ManagedImportArtifactRecord> {
        self.backend()
            .upsert_managed_import_artifact(artifact)
            .await
    }

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        self.backend().get_managed_import_artifact(id).await
    }

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        self.backend()
            .find_managed_import_artifact_by_source(target_library_id, source_kind, source_uri)
            .await
    }

    async fn list_managed_import_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportArtifactRecord>> {
        self.backend()
            .list_managed_import_artifacts(filter, page)
            .await
    }

    async fn set_managed_import_artifact_state(
        &self,
        id: ManagedImportArtifactId,
        state: ManagedImportArtifactState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        self.backend()
            .set_managed_import_artifact_state(id, state, updated_at_ms, diagnostics_json)
            .await
    }

    async fn upsert_managed_import_promotion_apply(
        &self,
        apply: NewManagedImportPromotionApply,
    ) -> Result<ManagedImportPromotionApplyRecord> {
        self.backend()
            .upsert_managed_import_promotion_apply(apply)
            .await
    }

    async fn get_managed_import_promotion_apply(
        &self,
        id: ManagedImportPromotionApplyId,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>> {
        self.backend().get_managed_import_promotion_apply(id).await
    }

    async fn find_managed_import_promotion_apply_by_idempotency_key(
        &self,
        target_library_id: LibraryId,
        idempotency_key: &str,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>> {
        self.backend()
            .find_managed_import_promotion_apply_by_idempotency_key(
                target_library_id,
                idempotency_key,
            )
            .await
    }

    async fn list_managed_import_promotion_applies_for_artifact(
        &self,
        artifact_id: ManagedImportArtifactId,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportPromotionApplyRecord>> {
        self.backend()
            .list_managed_import_promotion_applies_for_artifact(artifact_id, page)
            .await
    }

    async fn set_managed_import_promotion_apply_state(
        &self,
        id: ManagedImportPromotionApplyId,
        state: ManagedImportPromotionApplyState,
        updated_at_ms: i64,
        outcome_json: Option<String>,
        safe_error_code: Option<String>,
        safe_message: Option<String>,
    ) -> Result<Option<ManagedImportPromotionApplyRecord>> {
        self.backend()
            .set_managed_import_promotion_apply_state(
                id,
                state,
                updated_at_ms,
                outcome_json,
                safe_error_code,
                safe_message,
            )
            .await
    }
}

#[async_trait::async_trait]
impl NfoSidecarApplyRepository for NakoDatabase {
    async fn upsert_nfo_sidecar_apply(
        &self,
        apply: NewNfoSidecarApply,
    ) -> Result<NfoSidecarApplyRecord> {
        self.backend().upsert_nfo_sidecar_apply(apply).await
    }

    async fn get_nfo_sidecar_apply(
        &self,
        id: NfoSidecarApplyId,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        self.backend().get_nfo_sidecar_apply(id).await
    }

    async fn find_nfo_sidecar_apply_by_idempotency_key(
        &self,
        target_library_id: LibraryId,
        idempotency_key: &str,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        self.backend()
            .find_nfo_sidecar_apply_by_idempotency_key(target_library_id, idempotency_key)
            .await
    }

    async fn list_nfo_sidecar_applies_for_item(
        &self,
        media_item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<NfoSidecarApplyRecord>> {
        self.backend()
            .list_nfo_sidecar_applies_for_item(media_item_id, page)
            .await
    }

    async fn set_nfo_sidecar_apply_state(
        &self,
        id: NfoSidecarApplyId,
        state: NfoSidecarApplyState,
        updated_at_ms: i64,
        outcome_json: Option<String>,
        safe_error_code: Option<String>,
        safe_message: Option<String>,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        self.backend()
            .set_nfo_sidecar_apply_state(
                id,
                state,
                updated_at_ms,
                outcome_json,
                safe_error_code,
                safe_message,
            )
            .await
    }
}

#[async_trait::async_trait]
impl MetadataRepository for NakoDatabase {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        self.backend().upsert_field_lock(lock).await
    }

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>> {
        self.backend().list_field_locks(item_id).await
    }

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()> {
        self.backend().upsert_provider_raw_response(response).await
    }

    async fn commit_metadata_refresh(
        &self,
        commit: &MetadataRefreshPersistenceCommit,
    ) -> Result<MetadataRefreshPersistenceSummary> {
        self.backend().commit_metadata_refresh(commit).await
    }

    async fn commit_nfo_import(
        &self,
        commit: &NfoImportPersistenceCommit,
    ) -> Result<NfoImportPersistenceSummary> {
        self.backend().commit_nfo_import(commit).await
    }

    async fn commit_addon_metadata_write(
        &self,
        commit: &AddonMetadataWritePersistenceCommit,
    ) -> Result<AddonMetadataWritePersistenceSummary> {
        self.backend().commit_addon_metadata_write(commit).await
    }

    async fn commit_metadata_application(
        &self,
        commit: &MetadataApplicationPersistenceCommit,
    ) -> Result<MetadataApplicationPersistenceSummary> {
        self.backend().commit_metadata_application(commit).await
    }

    async fn commit_metadata_item(&self, item: &MediaItem) -> Result<()> {
        self.backend().commit_metadata_item(item).await
    }

    async fn get_provider_raw_response(
        &self,
        item_id: MediaItemId,
        provider: &ExternalProvider,
        provider_key: &str,
    ) -> Result<Option<ProviderRawResponse>> {
        self.backend()
            .get_provider_raw_response(item_id, provider, provider_key)
            .await
    }

    async fn list_provider_raw_responses(
        &self,
        item_id: MediaItemId,
        filter: ProviderRawResponseFilter,
        page: PageRequest,
    ) -> Result<Vec<ProviderRawResponse>> {
        self.backend()
            .list_provider_raw_responses(item_id, filter, page)
            .await
    }

    async fn cleanup_provider_raw_responses(
        &self,
        filter: ProviderRawResponseFilter,
        fetched_before: &str,
    ) -> Result<ProviderRawResponseCleanup> {
        self.backend()
            .cleanup_provider_raw_responses(filter, fetched_before)
            .await
    }

    async fn insert_metadata_provider_attempt(
        &self,
        attempt: NewMetadataProviderAttempt,
    ) -> Result<()> {
        self.backend()
            .insert_metadata_provider_attempt(attempt)
            .await
    }

    async fn list_metadata_provider_attempts(
        &self,
        job_id: JobId,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        self.backend().list_metadata_provider_attempts(job_id).await
    }

    async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        filter: MetadataAttemptFilter,
        page: PageRequest,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        self.backend()
            .list_metadata_provider_attempts_for_item(item_id, filter, page)
            .await
    }
}

#[async_trait::async_trait]
impl ProviderMappingRepository for NakoDatabase {
    async fn upsert_provider_subject(&self, subject: &ProviderSubject) -> Result<()> {
        self.backend().upsert_provider_subject(subject).await
    }

    async fn get_provider_subject(&self, id: ProviderSubjectId) -> Result<Option<ProviderSubject>> {
        self.backend().get_provider_subject(id).await
    }

    async fn find_provider_subject(
        &self,
        provider: &ExternalProvider,
        subject_kind: &ProviderSubjectKind,
        subject_key: &str,
    ) -> Result<Option<ProviderSubject>> {
        self.backend()
            .find_provider_subject(provider, subject_kind, subject_key)
            .await
    }

    async fn list_provider_subjects_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderSubject>> {
        self.backend()
            .list_provider_subjects_for_item(item_id, page)
            .await
    }

    async fn upsert_provider_mapping(&self, mapping: &ProviderMapping) -> Result<()> {
        self.backend().upsert_provider_mapping(mapping).await
    }

    async fn list_provider_mappings_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderMapping>> {
        self.backend()
            .list_provider_mappings_for_item(item_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl MetadataCandidateReviewRepository for NakoDatabase {
    async fn upsert_metadata_candidate_review(
        &self,
        review: NewMetadataCandidateReview,
    ) -> Result<MetadataCandidateReviewRecord> {
        self.backend()
            .upsert_metadata_candidate_review(review)
            .await
    }

    async fn get_metadata_candidate_review(
        &self,
        id: MetadataCandidateReviewId,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        self.backend().get_metadata_candidate_review(id).await
    }

    async fn find_metadata_candidate_review(
        &self,
        item_id: MediaItemId,
        source: &MetadataCandidateSource,
        source_key: &str,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        self.backend()
            .find_metadata_candidate_review(item_id, source, source_key)
            .await
    }

    async fn set_metadata_candidate_review_status(
        &self,
        id: MetadataCandidateReviewId,
        status: MetadataCandidateReviewStatus,
        updated_at_ms: i64,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        self.backend()
            .set_metadata_candidate_review_status(id, status, updated_at_ms)
            .await
    }

    async fn list_metadata_candidate_reviews_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MetadataCandidateReviewRecord>> {
        self.backend()
            .list_metadata_candidate_reviews_for_item(item_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl SourceDuplicateRepository for NakoDatabase {
    async fn upsert_source_duplicate_relationship(
        &self,
        relationship: &SourceDuplicateRelationship,
    ) -> Result<()> {
        self.backend()
            .upsert_source_duplicate_relationship(relationship)
            .await
    }

    async fn get_source_duplicate_relationship(
        &self,
        id: SourceDuplicateRelationshipId,
    ) -> Result<Option<SourceDuplicateRelationship>> {
        self.backend().get_source_duplicate_relationship(id).await
    }

    async fn list_source_duplicate_relationships(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<SourceDuplicateRelationship>> {
        self.backend()
            .list_source_duplicate_relationships(source_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl LocalInferenceRepository for NakoDatabase {
    async fn upsert_local_inference_evidence(
        &self,
        evidence: &LocalInferenceEvidence,
    ) -> Result<()> {
        self.backend()
            .upsert_local_inference_evidence(evidence)
            .await
    }

    async fn get_local_inference_evidence(
        &self,
        id: LocalInferenceEvidenceId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        self.backend().get_local_inference_evidence(id).await
    }

    async fn list_local_inference_evidence_for_source(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<LocalInferenceEvidence>> {
        self.backend()
            .list_local_inference_evidence_for_source(source_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl ScanRepository for NakoDatabase {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        self.backend()
            .begin_scan_snapshot(id, library_id, root)
            .await
    }

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        self.backend()
            .complete_scan_snapshot(id, status, error)
            .await
    }

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>> {
        self.backend().get_scan_snapshot(id).await
    }

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()> {
        self.backend().upsert_directory_snapshot(snapshot).await
    }

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>> {
        self.backend().list_directory_snapshots(scan_id).await
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        self.backend().upsert_source_state(state).await
    }

    async fn commit_library_scan_source(
        &self,
        commit: &LibraryScanSourcePersistenceCommit,
    ) -> Result<LibraryScanSourcePersistenceSummary> {
        self.backend().commit_library_scan_source(commit).await
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        self.backend().get_source_state(library_id, uri).await
    }

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>> {
        self.backend().list_source_states(library_id, page).await
    }
}

#[async_trait::async_trait]
impl DatabaseLifecycle for NakoDatabase {
    async fn migrate(&self) -> Result<()> {
        self.backend().migrate().await
    }
}

#[async_trait::async_trait]
impl PlaybackSessionRepository for NakoDatabase {
    async fn create_playback_session(
        &self,
        session: NewPlaybackSession,
    ) -> Result<PlaybackSessionRecord> {
        self.backend().create_playback_session(session).await
    }

    async fn get_playback_session(
        &self,
        id: PlaybackSessionId,
    ) -> Result<Option<PlaybackSessionRecord>> {
        self.backend().get_playback_session(id).await
    }

    async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>> {
        self.backend().list_playback_sessions(filter, page).await
    }

    async fn link_playback_session_transcode(
        &self,
        id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord> {
        self.backend()
            .link_playback_session_transcode(id, transcode_session_id)
            .await
    }

    async fn record_playback_session_heartbeat(
        &self,
        heartbeat: PlaybackSessionHeartbeat,
    ) -> Result<Option<PlaybackSessionRecord>> {
        self.backend()
            .record_playback_session_heartbeat(heartbeat)
            .await
    }

    async fn set_playback_session_state(
        &self,
        id: PlaybackSessionId,
        state: PlaybackSessionState,
        ended_at_ms: Option<i64>,
    ) -> Result<Option<PlaybackSessionRecord>> {
        self.backend()
            .set_playback_session_state(id, state, ended_at_ms)
            .await
    }
}

#[async_trait::async_trait]
impl TranscodeSessionRepository for NakoDatabase {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord> {
        self.backend().create_transcode_session(session).await
    }

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>> {
        self.backend().get_transcode_session(id).await
    }

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>> {
        self.backend().list_transcode_sessions(filter, page).await
    }

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        self.backend()
            .find_latest_transcode_session(source_id, kind, request_key)
            .await
    }

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        self.backend()
            .find_active_transcode_session(source_id, kind, request_key)
            .await
    }

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord> {
        self.backend()
            .set_transcode_session_state(id, state, failure_category, failure_message)
            .await
    }

    async fn update_transcode_session_runtime_metrics(
        &self,
        id: TranscodeSessionId,
        metrics: TranscodeSessionRuntimeMetrics,
    ) -> Result<Option<TranscodeSessionRecord>> {
        self.backend()
            .update_transcode_session_runtime_metrics(id, metrics)
            .await
    }

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>> {
        self.backend()
            .request_transcode_session_cancellation(id, failure_message)
            .await
    }

    async fn fail_stale_transcode_sessions(
        &self,
        failure_category: TranscodeFailureCategory,
        failure_message: String,
    ) -> Result<u64> {
        self.backend()
            .fail_stale_transcode_sessions(failure_category, failure_message)
            .await
    }
}

#[async_trait::async_trait]
impl RendererSessionRepository for NakoDatabase {
    async fn upsert_renderer_session(
        &self,
        session: NewRendererSession,
    ) -> Result<RendererSessionRecord> {
        self.backend().upsert_renderer_session(session).await
    }

    async fn get_renderer_session(
        &self,
        id: RendererSessionId,
    ) -> Result<Option<RendererSessionRecord>> {
        self.backend().get_renderer_session(id).await
    }

    async fn list_renderer_sessions(
        &self,
        filter: RendererSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererSessionRecord>> {
        self.backend().list_renderer_sessions(filter, page).await
    }

    async fn record_renderer_session_heartbeat(
        &self,
        heartbeat: RendererSessionHeartbeat,
    ) -> Result<Option<RendererSessionRecord>> {
        self.backend()
            .record_renderer_session_heartbeat(heartbeat)
            .await
    }

    async fn attach_renderer_playback_session(
        &self,
        id: RendererSessionId,
        playback_session_id: Option<PlaybackSessionId>,
        updated_at_ms: i64,
    ) -> Result<Option<RendererSessionRecord>> {
        self.backend()
            .attach_renderer_playback_session(id, playback_session_id, updated_at_ms)
            .await
    }

    async fn create_renderer_command(
        &self,
        command: NewRendererCommand,
    ) -> Result<RendererCommandRecord> {
        self.backend().create_renderer_command(command).await
    }

    async fn get_renderer_command(
        &self,
        id: RendererCommandId,
    ) -> Result<Option<RendererCommandRecord>> {
        self.backend().get_renderer_command(id).await
    }

    async fn list_renderer_commands(
        &self,
        filter: RendererCommandListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererCommandRecord>> {
        self.backend().list_renderer_commands(filter, page).await
    }

    async fn claim_next_renderer_command(
        &self,
        renderer_session_id: RendererSessionId,
        delivered_at_ms: i64,
    ) -> Result<Option<RendererCommandRecord>> {
        self.backend()
            .claim_next_renderer_command(renderer_session_id, delivered_at_ms)
            .await
    }

    async fn complete_renderer_command(
        &self,
        completion: RendererCommandCompletion,
    ) -> Result<Option<RendererCommandRecord>> {
        self.backend().complete_renderer_command(completion).await
    }
}

#[async_trait::async_trait]
impl UserPlaybackStateRepository for NakoDatabase {
    async fn upsert_user_playback_state(
        &self,
        state: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState> {
        self.backend().upsert_user_playback_state(state).await
    }

    async fn get_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>> {
        self.backend()
            .get_user_playback_state(principal_id, item_id)
            .await
    }

    async fn list_continue_watching_states(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        self.backend()
            .list_continue_watching_states(principal_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl UserPlaylistRepository for NakoDatabase {
    async fn create_user_playlist(&self, playlist: NewUserPlaylist) -> Result<UserPlaylistRecord> {
        self.backend().create_user_playlist(playlist).await
    }

    async fn get_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistRecord>> {
        self.backend()
            .get_user_playlist(principal_id, playlist_id)
            .await
    }

    async fn list_user_playlists(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>> {
        self.backend().list_user_playlists(principal_id, page).await
    }

    async fn update_user_playlist_name(
        &self,
        update: UserPlaylistNameUpdate,
    ) -> Result<Option<UserPlaylistRecord>> {
        self.backend().update_user_playlist_name(update).await
    }

    async fn delete_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<bool> {
        self.backend()
            .delete_user_playlist(principal_id, playlist_id)
            .await
    }

    async fn add_user_playlist_item(
        &self,
        write: UserPlaylistItemWrite,
    ) -> Result<Option<UserPlaylistRecord>> {
        self.backend().add_user_playlist_item(write).await
    }

    async fn remove_user_playlist_item(
        &self,
        removal: UserPlaylistItemRemoval,
    ) -> Result<Option<UserPlaylistRecord>> {
        self.backend().remove_user_playlist_item(removal).await
    }

    async fn replace_user_playlist_item_order(
        &self,
        reorder: UserPlaylistReorder,
    ) -> Result<Option<UserPlaylistRecord>> {
        self.backend()
            .replace_user_playlist_item_order(reorder)
            .await
    }

    async fn list_user_playlist_items(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>> {
        self.backend()
            .list_user_playlist_items(principal_id, playlist_id, page)
            .await
    }
}

#[async_trait::async_trait]
impl VfsCacheRepository for NakoDatabase {
    async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()> {
        self.backend().upsert_vfs_cache_object(object).await
    }

    async fn upsert_vfs_cache_listing(&self, listing: &VfsCachedListing) -> Result<()> {
        self.backend().upsert_vfs_cache_listing(listing).await
    }

    async fn get_vfs_cache_object(&self, uri: &str) -> Result<Option<VfsCachedObject>> {
        self.backend().get_vfs_cache_object(uri).await
    }

    async fn get_vfs_cache_listing(&self, uri: &str) -> Result<Option<VfsCachedListing>> {
        self.backend().get_vfs_cache_listing(uri).await
    }

    async fn record_vfs_cache_failure(
        &self,
        failure: NewVfsCacheFailure,
    ) -> Result<VfsCacheFailure> {
        self.backend().record_vfs_cache_failure(failure).await
    }

    async fn get_vfs_cache_failure(
        &self,
        uri: &str,
        operation: VfsCacheOperation,
    ) -> Result<Option<VfsCacheFailure>> {
        self.backend().get_vfs_cache_failure(uri, operation).await
    }

    async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        self.backend().summarize_vfs_cache(now_ms).await
    }
}

#[async_trait::async_trait]
impl StorageBackendHealthRepository for NakoDatabase {
    async fn upsert_storage_backend_health(
        &self,
        record: StorageBackendHealthRecord,
    ) -> Result<StorageBackendHealthRecord> {
        self.backend().upsert_storage_backend_health(record).await
    }

    async fn get_storage_backend_health(
        &self,
        backend_key: &str,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        self.backend().get_storage_backend_health(backend_key).await
    }

    async fn list_storage_backend_health(
        &self,
        filter: StorageBackendHealthListFilter,
        page: PageRequest,
    ) -> Result<Vec<StorageBackendHealthRecord>> {
        self.backend()
            .list_storage_backend_health(filter, page)
            .await
    }

    async fn clear_storage_backend_health(
        &self,
        backend_key: &str,
        cleared_at_ms: i64,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        self.backend()
            .clear_storage_backend_health(backend_key, cleared_at_ms)
            .await
    }
}

#[async_trait::async_trait]
impl StagingManifestRepository for NakoDatabase {
    async fn upsert_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        self.backend().upsert_staging_manifest_record(record).await
    }

    async fn reserve_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
        max_total_bytes: u64,
        now_ms: i64,
    ) -> Result<StagingManifestRecord> {
        self.backend()
            .reserve_staging_manifest_record(record, max_total_bytes, now_ms)
            .await
    }

    async fn start_staging_manifest_record(
        &self,
        id: StagingManifestId,
        started_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        self.backend()
            .start_staging_manifest_record(id, started_at_ms)
            .await
    }

    async fn complete_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        self.backend()
            .complete_staging_manifest_record(record)
            .await
    }

    async fn fail_staging_manifest_record(
        &self,
        id: StagingManifestId,
        failed_at_ms: i64,
        validation_error: String,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend()
            .fail_staging_manifest_record(id, failed_at_ms, validation_error)
            .await
    }

    async fn expire_staging_manifest_record(
        &self,
        id: StagingManifestId,
        expired_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend()
            .expire_staging_manifest_record(id, expired_at_ms)
            .await
    }

    async fn mark_deleted_staging_manifest_record(
        &self,
        id: StagingManifestId,
        deleted_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend()
            .mark_deleted_staging_manifest_record(id, deleted_at_ms)
            .await
    }

    async fn acquire_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        leased_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        self.backend()
            .acquire_staging_manifest_lease(id, leased_at_ms)
            .await
    }

    async fn release_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        released_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        self.backend()
            .release_staging_manifest_lease(id, released_at_ms)
            .await
    }

    async fn get_staging_manifest_record(
        &self,
        id: StagingManifestId,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend().get_staging_manifest_record(id).await
    }

    async fn find_staging_manifest_record_by_path(
        &self,
        local_path: &str,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend()
            .find_staging_manifest_record_by_path(local_path)
            .await
    }

    async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        self.backend()
            .list_staging_manifest_records(purpose, state, page)
            .await
    }

    async fn list_staging_cleanup_candidates(
        &self,
        now_ms: i64,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        self.backend()
            .list_staging_cleanup_candidates(now_ms, page)
            .await
    }

    async fn touch_staging_manifest_record(
        &self,
        id: StagingManifestId,
        accessed_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        self.backend()
            .touch_staging_manifest_record(id, accessed_at_ms)
            .await
    }

    async fn delete_staging_manifest_record(&self, id: StagingManifestId) -> Result<()> {
        self.backend().delete_staging_manifest_record(id).await
    }

    async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        self.backend().sum_staging_manifest_bytes().await
    }
}

#[async_trait::async_trait]
impl WebhookRepository for NakoDatabase {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord> {
        self.backend().upsert_webhook_endpoint(endpoint).await
    }

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>> {
        self.backend().get_webhook_endpoint(id).await
    }

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        self.backend().list_enabled_webhook_endpoints().await
    }

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        self.backend()
            .create_webhook_delivery_attempt(attempt)
            .await
    }

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        self.backend()
            .set_webhook_delivery_attempt_result(id, status, http_status, error, next_retry_at)
            .await
    }

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>> {
        self.backend()
            .list_webhook_delivery_attempts(event_id)
            .await
    }
}

#[async_trait::async_trait]
impl SearchIndex for NakoDatabase {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        self.backend().upsert(document).await
    }

    async fn delete(&self, item_id: MediaItemId) -> Result<()> {
        self.backend().delete(item_id).await
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        self.backend().search(query).await
    }
}
