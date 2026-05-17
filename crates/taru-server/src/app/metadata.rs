use std::{collections::HashSet, sync::Arc, time::Duration};

use serde::Serialize;
use taru_api::{
    EnqueueMetadataMaintenanceRequest, MetadataMaintenancePlanError, MetadataMaintenancePlanItem,
    MetadataMaintenancePlanResponse, MetadataProviderAttemptDiagnostic,
    MetadataProviderAttemptsResponse, MetadataProviderDiagnosticsResponse,
    MetadataRawCleanupResponse, MetadataRawResponsesResponse, page_info_from_request,
};
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, ExternalProvider, Job,
    JobId, JobKind, JobRepository, Library, LibraryId, MediaItem, MediaItemId, MediaRepository,
    MetadataAttemptFilter, MetadataProfile, MetadataProviderAttemptRecord,
    MetadataProviderAttemptStatus, MetadataRefreshMode, MetadataRepository, NewJob, NewOutboxEvent,
    PageRequest, ProviderRawResponseFilter, Result, TaruError,
};
use taru_db::SqliteStore;
use taru_metadata::{
    MetadataProviderRegistry, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshSummary, MetadataStrategyExecutor,
};
use time::OffsetDateTime;
use tokio::sync::Semaphore;
use tracing::{Instrument, info, info_span, warn};

use super::metadata_runtime::provider_resource_name;
use super::runtime::RuntimeSupervisor;
use crate::config::{MetadataMaintenancePolicyConfig, TaruServerConfig, libraries_from_config};

#[derive(Clone, Debug, Serialize)]
pub struct MetadataRefreshCommandOutput {
    pub job: Job,
    pub refresh: MetadataRefreshSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataMaintenanceCommandOutput {
    pub job: Job,
    pub summary: MetadataMaintenanceSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataMaintenanceSummary {
    pub job_id: JobId,
    pub library_id: Option<LibraryId>,
    pub requested_items: u32,
    pub attempted_items: u32,
    pub succeeded_items: u32,
    pub failed_items: u32,
    pub no_match_items: u32,
    pub rate_limited_items: u32,
    pub skipped_items: u32,
    pub provider_attempts: Vec<MetadataMaintenanceProviderAttemptCount>,
    pub errors: Vec<MetadataMaintenanceItemError>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataMaintenanceProviderAttemptCount {
    pub provider: ExternalProvider,
    pub status: MetadataProviderAttemptStatus,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataMaintenanceItemError {
    pub item_id: MediaItemId,
    pub message: String,
}

#[derive(Clone, Debug)]
pub(crate) struct MetadataAppService {
    config: TaruServerConfig,
    store: SqliteStore,
    permits: Arc<Semaphore>,
    providers: MetadataProviderRegistry,
    runtime: RuntimeSupervisor,
}

impl MetadataAppService {
    pub(super) fn new(
        config: TaruServerConfig,
        store: SqliteStore,
        permits: Arc<Semaphore>,
        providers: MetadataProviderRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            config,
            store,
            permits,
            providers,
            runtime,
        }
    }

    pub(crate) async fn enqueue_metadata_refresh(&self, item_id: MediaItemId) -> Result<Job> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        let job_id = job.id;
        let resource_class = job.resource_class.clone();
        let service = self.clone();

        self.runtime.spawn_job(
            "metadata_refresh_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_metadata_refresh_job(job_id, item_id).await }
                    .instrument(info_span!(
                        "metadata_refresh_background_job",
                        job_id = %job_id,
                        item_id = %item_id,
                        resource_class = %resource_class
                    ))
            },
        );

        Ok(job)
    }

    pub async fn refresh_item_metadata(
        &self,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        self.execute_metadata_refresh_job(job.id, item_id).await
    }

    pub async fn enqueue_metadata_maintenance(
        &self,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<Job> {
        let job = self.create_metadata_maintenance_job(&request).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "metadata_maintenance_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move {
                    service
                        .finish_metadata_maintenance_job(job_id, request)
                        .await
                }
                .instrument(info_span!(
                    "metadata_maintenance_background_job",
                    job_id = %job_id,
                    resource_class = "metadata.maintenance"
                ))
            },
        );

        Ok(job)
    }

    pub async fn run_metadata_maintenance(
        &self,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<MetadataMaintenanceCommandOutput> {
        let job = self.create_metadata_maintenance_job(&request).await?;
        self.execute_metadata_maintenance_job(job.id, request).await
    }

    pub async fn plan_metadata_maintenance(
        &self,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<MetadataMaintenancePlanResponse> {
        self.validate_metadata_maintenance_request(&request).await?;
        let items = self.metadata_maintenance_items(&request).await?;
        let mut planned = Vec::new();
        let mut errors = Vec::new();

        for item in items {
            match self
                .metadata_maintenance_profile_for_item(&item, &request)
                .await
            {
                Ok(profile) => {
                    let library_id = if let Some(library_id) = request.library_id {
                        Some(library_id)
                    } else {
                        self.library_for_item(item.id)
                            .await
                            .ok()
                            .map(|library| library.id)
                    };
                    planned.push(MetadataMaintenancePlanItem {
                        item_id: item.id,
                        library_id,
                        kind: item.kind,
                        title: item.metadata.title.clone(),
                        providers: profile.metadata_providers,
                        language: profile.language,
                        refresh_mode: profile.refresh_mode,
                    });
                }
                Err(err) => {
                    errors.push(MetadataMaintenancePlanError {
                        item_id: item.id,
                        message: err.to_string(),
                    });
                }
            }
        }

        Ok(MetadataMaintenancePlanResponse {
            request,
            planned_items: usize_to_u32(planned.len()),
            skipped_items: usize_to_u32(errors.len()),
            items: planned,
            errors,
        })
    }

    pub(super) async fn create_metadata_refresh_job(&self, item_id: MediaItemId) -> Result<Job> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        let provider = self.first_metadata_provider(&profile)?;
        let input = MetadataRefreshJobInput {
            item_id,
            provider: Some(provider.clone()),
            force: false,
            language: profile.language.clone(),
            refresh_mode: profile.refresh_mode,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize metadata refresh job input: {err}"),
        })?;

        self.store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: format!("metadata.{}", provider_resource_name(&provider)),
                library_id: Some(library.id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_metadata_maintenance_job(
        &self,
        request: &EnqueueMetadataMaintenanceRequest,
    ) -> Result<Job> {
        self.validate_metadata_maintenance_request(request).await?;
        let input_json = serde_json::to_string(request).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize metadata maintenance job input: {err}"),
        })?;

        self.store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataMaintenance,
                resource_class: metadata_maintenance_resource_class(request),
                library_id: request.library_id,
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn finish_metadata_refresh_job(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<Job> {
        let output = self.execute_metadata_refresh_job(job_id, item_id).await?;
        info!(
            job_id = %output.job.id,
            item_id = %item_id,
            provider_key = %output.refresh.provider_key,
            status = ?output.job.status,
            "metadata refresh job completed"
        );
        Ok(output.job)
    }

    async fn finish_metadata_maintenance_job(
        &self,
        job_id: JobId,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<Job> {
        let output = self
            .execute_metadata_maintenance_job(job_id, request)
            .await?;
        info!(
            job_id = %output.job.id,
            attempted_items = output.summary.attempted_items,
            succeeded_items = output.summary.succeeded_items,
            failed_items = output.summary.failed_items,
            status = ?output.job.status,
            "metadata maintenance job completed"
        );
        Ok(output.job)
    }

    async fn execute_metadata_refresh_job(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("metadata concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        self.store.start_job(job_id).await?;

        match self.run_metadata_refresh(job_id, item_id).await {
            Ok(refresh) => {
                let summary_json =
                    serde_json::to_string(&refresh).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize metadata refresh job summary: {err}"),
                    })?;
                let job = self.store.succeed_job(job_id, Some(summary_json)).await?;
                self.record_metadata_refreshed_event(job_id, item_id, &refresh)
                    .await;

                Ok(MetadataRefreshCommandOutput { job, refresh })
            }
            Err(err) => {
                if let Err(update_err) = self.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        item_id = %item_id,
                        error = %update_err,
                        "failed to persist failed metadata refresh job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn execute_metadata_maintenance_job(
        &self,
        job_id: JobId,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<MetadataMaintenanceCommandOutput> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("metadata concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        self.store.start_job(job_id).await?;

        match self.run_metadata_maintenance_job(job_id, request).await {
            Ok(summary) => {
                let summary_json =
                    serde_json::to_string(&summary).map_err(|err| TaruError::InvalidInput {
                        message: format!(
                            "failed to serialize metadata maintenance job summary: {err}"
                        ),
                    })?;
                let job = self.store.succeed_job(job_id, Some(summary_json)).await?;
                self.record_metadata_maintenance_completed_event(&summary)
                    .await;

                Ok(MetadataMaintenanceCommandOutput { job, summary })
            }
            Err(err) => {
                if let Err(update_err) = self.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        error = %update_err,
                        "failed to persist failed metadata maintenance job state"
                    );
                }

                Err(err)
            }
        }
    }

    pub(super) async fn record_metadata_refreshed_event(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
        refresh: &MetadataRefreshSummary,
    ) {
        let library_id = match self.library_for_item(item_id).await {
            Ok(library) => Some(library.id),
            Err(err) => {
                warn!(
                    job_id = %job_id,
                    item_id = %item_id,
                    error = %err,
                    "failed to resolve library while recording metadata event"
                );
                None
            }
        };
        let payload = serde_json::json!({
            "job_id": job_id,
            "item_id": item_id,
            "provider": &refresh.provider,
            "selected_provider": &refresh.selected_provider,
            "provider_key": &refresh.provider_key,
            "matched_by": refresh.matched_by,
            "refresh_mode": refresh.refresh_mode,
            "updated": refresh.updated,
            "attempted_providers": refresh.attempted_providers.len(),
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::ItemMetadataRefreshed,
            subject: DomainEventSubject::Item(item_id),
            library_id,
            source_id: None,
            idempotency_key: format!("item.metadata_refreshed:{job_id}:{item_id}"),
            payload_json: payload.to_string(),
        })
        .await;
    }

    async fn record_metadata_maintenance_completed_event(
        &self,
        summary: &MetadataMaintenanceSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": summary.job_id,
            "library_id": summary.library_id,
            "requested_items": summary.requested_items,
            "attempted_items": summary.attempted_items,
            "succeeded_items": summary.succeeded_items,
            "failed_items": summary.failed_items,
            "no_match_items": summary.no_match_items,
            "rate_limited_items": summary.rate_limited_items,
            "skipped_items": summary.skipped_items,
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::MetadataMaintenanceCompleted,
            subject: DomainEventSubject::Job(summary.job_id),
            library_id: summary.library_id,
            source_id: None,
            idempotency_key: format!("metadata.maintenance_completed:{}", summary.job_id),
            payload_json: payload.to_string(),
        })
        .await;
    }

    pub(super) async fn run_metadata_refresh(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshSummary> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        self.run_metadata_refresh_with_profile(job_id, item_id, profile, false)
            .await
    }

    async fn run_metadata_refresh_with_profile(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
        profile: MetadataProfile,
        force: bool,
    ) -> Result<MetadataRefreshSummary> {
        let registry = self.metadata_provider_registry();
        let executor = MetadataStrategyExecutor::new(registry, self.store.clone());

        executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id,
                profile,
                force,
            })
            .await
    }

    async fn run_metadata_maintenance_job(
        &self,
        job_id: JobId,
        request: EnqueueMetadataMaintenanceRequest,
    ) -> Result<MetadataMaintenanceSummary> {
        self.validate_metadata_maintenance_request(&request).await?;
        let items = self.metadata_maintenance_items(&request).await?;
        let mut summary = MetadataMaintenanceSummary {
            job_id,
            library_id: request.library_id,
            requested_items: usize_to_u32(items.len()),
            attempted_items: 0,
            succeeded_items: 0,
            failed_items: 0,
            no_match_items: 0,
            rate_limited_items: 0,
            skipped_items: 0,
            provider_attempts: Vec::new(),
            errors: Vec::new(),
        };

        for item in items {
            let profile = match self
                .metadata_maintenance_profile_for_item(&item, &request)
                .await
            {
                Ok(profile) => profile,
                Err(err) => {
                    summary.skipped_items += 1;
                    summary.errors.push(MetadataMaintenanceItemError {
                        item_id: item.id,
                        message: err.to_string(),
                    });
                    continue;
                }
            };

            summary.attempted_items += 1;
            match self
                .run_metadata_refresh_with_profile(job_id, item.id, profile, request.force)
                .await
            {
                Ok(refresh) => {
                    summary.succeeded_items += 1;
                    self.record_metadata_refreshed_event(job_id, item.id, &refresh)
                        .await;
                }
                Err(err) => {
                    let attempts = self.metadata_attempts_for_job_item(job_id, item.id).await?;
                    classify_metadata_maintenance_failure(&mut summary, item.id, &attempts, &err);
                }
            }
        }

        let attempts = self.store.list_metadata_provider_attempts(job_id).await?;
        summary.provider_attempts = summarize_metadata_attempt_counts(&attempts);

        Ok(summary)
    }

    pub async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        filter: MetadataAttemptFilter,
        page: PageRequest,
    ) -> Result<MetadataProviderAttemptsResponse> {
        self.ensure_metadata_item_exists(item_id).await?;
        let attempts = self
            .store
            .list_metadata_provider_attempts_for_item(item_id, filter, page)
            .await?;
        let returned = attempts.len();

        Ok(MetadataProviderAttemptsResponse {
            item_id,
            page: page_info_from_request(page, returned),
            attempts: attempts
                .into_iter()
                .map(MetadataProviderAttemptDiagnostic::from_record)
                .collect(),
        })
    }

    pub async fn list_provider_raw_responses_for_item(
        &self,
        item_id: MediaItemId,
        filter: ProviderRawResponseFilter,
        page: PageRequest,
    ) -> Result<MetadataRawResponsesResponse> {
        self.ensure_metadata_item_exists(item_id).await?;
        let responses = self
            .store
            .list_provider_raw_responses(item_id, filter, page)
            .await?;

        Ok(MetadataRawResponsesResponse {
            item_id,
            page: page_info_from_request(page, responses.len()),
            responses,
        })
    }

    pub async fn cleanup_provider_raw_responses(
        &self,
        filter: ProviderRawResponseFilter,
        fetched_before: Option<String>,
        retention_ms: Option<u64>,
    ) -> Result<MetadataRawCleanupResponse> {
        let fetched_before = match fetched_before {
            Some(value) if !value.trim().is_empty() => value,
            Some(_) => {
                return Err(TaruError::InvalidInput {
                    message: "fetched_before must not be empty".to_owned(),
                });
            }
            None => metadata_raw_retention_cutoff(
                retention_ms.unwrap_or(self.config.metadata.raw_cache_retention_ms),
            )?,
        };
        let cleanup = self
            .store
            .cleanup_provider_raw_responses(filter, &fetched_before)
            .await?;

        Ok(MetadataRawCleanupResponse { cleanup })
    }

    pub fn list_metadata_provider_diagnostics(&self) -> MetadataProviderDiagnosticsResponse {
        MetadataProviderDiagnosticsResponse {
            providers: super::metadata_runtime::metadata_provider_diagnostics(
                &self.config,
                &self.providers,
            ),
        }
    }

    pub(super) async fn cleanup_metadata_raw_cache_on_startup(&self) -> Result<u64> {
        if !self
            .config
            .metadata
            .maintenance
            .raw_cache_cleanup_on_startup
        {
            return Ok(0);
        }

        let cleanup = self
            .cleanup_provider_raw_responses(ProviderRawResponseFilter::default(), None, None)
            .await?;
        if cleanup.cleanup.deleted > 0 {
            info!(
                deleted = cleanup.cleanup.deleted,
                fetched_before = cleanup.cleanup.fetched_before,
                "cleaned metadata raw cache during startup"
            );
        }

        Ok(cleanup.cleanup.deleted)
    }

    pub(super) fn start_metadata_lifecycle_tasks(&self) -> usize {
        self.start_metadata_raw_cache_cleanup_task()
            + self.start_metadata_maintenance_policy_tasks()
    }

    fn start_metadata_raw_cache_cleanup_task(&self) -> usize {
        let interval_ms = self
            .config
            .metadata
            .maintenance
            .raw_cache_cleanup_interval_ms;
        if interval_ms == 0 {
            return 0;
        }

        let app = self.clone();
        let token = self.runtime.shutdown_token();
        self.runtime.spawn(
            "metadata_raw_cache_cleanup",
            "metadata.raw_cache.cleanup",
            async move {
                loop {
                    tokio::select! {
                        () = token.cancelled() => break,
                        () = tokio::time::sleep(Duration::from_millis(interval_ms)) => {}
                    }
                    match app
                        .cleanup_provider_raw_responses(
                            ProviderRawResponseFilter::default(),
                            None,
                            None,
                        )
                        .await
                    {
                        Ok(cleanup) => {
                            if cleanup.cleanup.deleted > 0 {
                                info!(
                                    deleted = cleanup.cleanup.deleted,
                                    fetched_before = cleanup.cleanup.fetched_before,
                                    "cleaned metadata raw cache in background"
                                );
                            }
                        }
                        Err(err) => {
                            warn!(error = %err, "metadata raw cache background cleanup failed");
                        }
                    }
                }
            },
        );
        1
    }

    fn start_metadata_maintenance_policy_tasks(&self) -> usize {
        let mut started = 0;
        for policy in self
            .config
            .metadata
            .maintenance
            .policies
            .iter()
            .filter(|policy| policy.enabled)
            .cloned()
        {
            let app = self.clone();
            let token = self.runtime.shutdown_token();
            self.runtime.spawn(
                "metadata_maintenance_policy",
                "metadata.maintenance.schedule",
                async move {
                    if policy.initial_delay_ms > 0 {
                        tokio::select! {
                            () = token.cancelled() => return,
                            () = tokio::time::sleep(Duration::from_millis(policy.initial_delay_ms)) => {}
                        }
                    }

                    loop {
                        let request = app.metadata_maintenance_request_from_policy(&policy);
                        match app.enqueue_metadata_maintenance(request).await {
                            Ok(job) => {
                                info!(
                                    policy_id = %policy.id,
                                    job_id = %job.id,
                                    "queued scheduled metadata maintenance job"
                                );
                            }
                            Err(err) => {
                                warn!(
                                    policy_id = %policy.id,
                                    error = %err,
                                    "scheduled metadata maintenance enqueue failed"
                                );
                            }
                        }

                        tokio::select! {
                            () = token.cancelled() => break,
                            () = tokio::time::sleep(Duration::from_millis(policy.interval_ms.max(1))) => {}
                        }
                    }
                },
            );
            started += 1;
        }
        started
    }

    pub(super) fn metadata_maintenance_request_from_policy(
        &self,
        policy: &MetadataMaintenancePolicyConfig,
    ) -> EnqueueMetadataMaintenanceRequest {
        EnqueueMetadataMaintenanceRequest {
            library_id: policy.library_id,
            item_ids: policy.item_ids.clone(),
            providers: policy.providers.clone(),
            item_kinds: policy.item_kinds.clone(),
            profile: policy.profile.clone(),
            language: policy.language.clone(),
            refresh_mode: policy.refresh_mode,
            force: policy.force,
        }
    }

    async fn validate_metadata_maintenance_request(
        &self,
        request: &EnqueueMetadataMaintenanceRequest,
    ) -> Result<()> {
        let has_library = request.library_id.is_some();
        let has_items = !request.item_ids.is_empty();
        if has_library == has_items {
            return Err(TaruError::InvalidInput {
                message: "metadata maintenance must target either library_id or item_ids"
                    .to_owned(),
            });
        }

        if let Some(library_id) = request.library_id {
            self.configured_library_for(library_id)?;
        }

        if let Some(providers) = request.providers.as_ref() {
            if providers.is_empty() {
                return Err(TaruError::InvalidInput {
                    message: "metadata maintenance providers override must not be empty".to_owned(),
                });
            }
        }

        Ok(())
    }

    async fn metadata_maintenance_items(
        &self,
        request: &EnqueueMetadataMaintenanceRequest,
    ) -> Result<Vec<MediaItem>> {
        let mut items = if let Some(library_id) = request.library_id {
            self.list_metadata_maintenance_library_items(library_id)
                .await?
        } else {
            self.load_metadata_maintenance_explicit_items(&request.item_ids)
                .await?
        };

        if !request.item_kinds.is_empty() {
            items.retain(|item| request.item_kinds.contains(&item.kind));
        }

        Ok(items)
    }

    async fn list_metadata_maintenance_library_items(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<MediaItem>> {
        let mut items = Vec::new();
        let mut offset = 0;

        loop {
            let page = PageRequest {
                limit: PageRequest::MAX_LIMIT,
                offset,
            };
            let mut page_items = self
                .store
                .list_media_items_for_library(library_id, page)
                .await?;
            let returned = page_items.len();
            items.append(&mut page_items);

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(items)
    }

    async fn load_metadata_maintenance_explicit_items(
        &self,
        item_ids: &[MediaItemId],
    ) -> Result<Vec<MediaItem>> {
        let mut seen = HashSet::new();
        let mut items = Vec::with_capacity(item_ids.len());

        for item_id in item_ids {
            if !seen.insert(*item_id) {
                continue;
            }

            let item =
                self.store
                    .get_media_item(*item_id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "media_item",
                        id: item_id.to_string(),
                    })?;
            items.push(item);
        }

        Ok(items)
    }

    async fn metadata_maintenance_profile_for_item(
        &self,
        item: &MediaItem,
        request: &EnqueueMetadataMaintenanceRequest,
    ) -> Result<MetadataProfile> {
        let mut profile = if let Some(profile) = request.profile.clone() {
            profile
        } else {
            let library = if let Some(library_id) = request.library_id {
                self.configured_library_for(library_id)?
            } else {
                self.library_for_item(item.id).await?
            };
            self.effective_metadata_profile(&library, item.kind)?
        };

        validate_profile_applies_to_item_kind(&profile, item.kind)?;

        if let Some(providers) = request.providers.as_ref() {
            profile.metadata_providers = providers.clone();
        }
        if let Some(language) = request
            .language
            .as_ref()
            .filter(|language| !language.trim().is_empty())
        {
            profile.language = Some(language.clone());
        }
        if let Some(refresh_mode) = request.refresh_mode {
            profile.refresh_mode = refresh_mode;
        } else if request.force {
            profile.refresh_mode = MetadataRefreshMode::FullRefresh;
        }

        Ok(profile)
    }

    async fn metadata_attempts_for_job_item(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        Ok(self
            .store
            .list_metadata_provider_attempts(job_id)
            .await?
            .into_iter()
            .filter(|attempt| attempt.item_id == item_id)
            .collect())
    }

    async fn ensure_metadata_item_exists(&self, item_id: MediaItemId) -> Result<()> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;

        Ok(())
    }

    fn effective_metadata_profile(
        &self,
        library: &taru_core::Library,
        item_kind: taru_core::MediaKind,
    ) -> Result<MetadataProfile> {
        let profile = library.options.metadata_profile.clone();

        validate_profile_applies_to_item_kind(&profile, item_kind)?;

        Ok(profile)
    }

    fn first_metadata_provider(&self, profile: &MetadataProfile) -> Result<ExternalProvider> {
        let Some(provider) = profile.metadata_providers.first().cloned() else {
            return Err(TaruError::InvalidInput {
                message: "library metadata profile does not enable any metadata provider"
                    .to_owned(),
            });
        };

        Ok(provider)
    }

    fn metadata_provider_registry(&self) -> MetadataProviderRegistry {
        self.providers.clone()
    }

    async fn record_outbox_event(&self, event: NewOutboxEvent) {
        let kind = event.kind.as_str();
        let idempotency_key = event.idempotency_key.clone();
        if let Err(err) = self.store.enqueue_outbox_event(event).await {
            warn!(
                kind,
                idempotency_key,
                error = %err,
                "failed to persist outbox event"
            );
        }
    }

    async fn library_for_item(&self, item_id: MediaItemId) -> Result<Library> {
        let source = self
            .store
            .list_item_sources(item_id, PageRequest::new(1, 0))
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| TaruError::InvalidInput {
                message: format!("media item {item_id} has no persisted media source"),
            })?;

        self.configured_library_for(source.library_id)
    }

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        libraries_from_config(&self.config)
            .into_iter()
            .find(|library| library.id == library_id)
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

fn metadata_maintenance_resource_class(request: &EnqueueMetadataMaintenanceRequest) -> String {
    request
        .providers
        .as_ref()
        .and_then(|providers| providers.first())
        .map(|provider| format!("metadata.{}", provider_resource_name(provider)))
        .unwrap_or_else(|| "metadata.maintenance".to_owned())
}

fn validate_profile_applies_to_item_kind(
    profile: &MetadataProfile,
    item_kind: taru_core::MediaKind,
) -> Result<()> {
    if !profile.item_kinds.is_empty()
        && !profile.item_kinds.contains(&item_kind)
        && !profile.item_kinds.contains(&taru_core::MediaKind::Unknown)
    {
        return Err(TaruError::Unsupported(
            "metadata profile does not apply to this item kind",
        ));
    }

    Ok(())
}

fn classify_metadata_maintenance_failure(
    summary: &mut MetadataMaintenanceSummary,
    item_id: MediaItemId,
    attempts: &[MetadataProviderAttemptRecord],
    error: &TaruError,
) {
    if attempts
        .iter()
        .any(|attempt| attempt.status == MetadataProviderAttemptStatus::RateLimited)
    {
        summary.rate_limited_items += 1;
    } else if attempts
        .iter()
        .any(|attempt| attempt.status == MetadataProviderAttemptStatus::NoMatch)
    {
        summary.no_match_items += 1;
    } else {
        summary.failed_items += 1;
    }

    summary.errors.push(MetadataMaintenanceItemError {
        item_id,
        message: error.to_string(),
    });
}

fn summarize_metadata_attempt_counts(
    attempts: &[MetadataProviderAttemptRecord],
) -> Vec<MetadataMaintenanceProviderAttemptCount> {
    let mut counts: Vec<MetadataMaintenanceProviderAttemptCount> = Vec::new();

    for attempt in attempts {
        if let Some(count) = counts
            .iter_mut()
            .find(|count| count.provider == attempt.provider && count.status == attempt.status)
        {
            count.count += 1;
            continue;
        }

        counts.push(MetadataMaintenanceProviderAttemptCount {
            provider: attempt.provider.clone(),
            status: attempt.status,
            count: 1,
        });
    }

    counts.sort_by(|left, right| {
        provider_resource_name(&left.provider)
            .cmp(provider_resource_name(&right.provider))
            .then_with(|| left.status.as_str().cmp(right.status.as_str()))
    });
    counts
}

fn metadata_raw_retention_cutoff(retention_ms: u64) -> Result<String> {
    let millis = i64::try_from(retention_ms).map_err(|_| TaruError::InvalidInput {
        message: "metadata raw cache retention_ms is too large".to_owned(),
    })?;
    let cutoff = OffsetDateTime::now_utc() - time::Duration::milliseconds(millis);

    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        cutoff.year(),
        u8::from(cutoff.month()),
        cutoff.day(),
        cutoff.hour(),
        cutoff.minute(),
        cutoff.second(),
        cutoff.millisecond()
    ))
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
