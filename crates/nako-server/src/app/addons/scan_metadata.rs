use nako_api::extension::{AddonTaskRunDispatchMode, CreateAddonTaskRunRequest};
use nako_core::{
    AddonId, AddonRegistrationRecord, AddonSideEffectTarget, AddonStatus, AddonTaskRunRecord,
    ExternalProvider, JobId, JobStatus, Library, LibraryId, MediaItem, MediaItemId, MediaKind,
    MediaRepository, MediaSource, MediaSourceId, NakoError, PageRequest, Result,
};
use nako_official_addon_catalog::metadata_scraper;
use serde::Serialize;

use super::{
    AddonAppService,
    task_runtime::{retry_dispatch_from_previous_input, retry_payload_from_previous_input},
};

const SCAN_ADDON_SCRAPE_BATCH_SIZE: usize = 12;
const SCAN_ADDON_SCRAPE_SOURCE_LIMIT: u32 = 500;

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct ScanAddonBulkMetadataScrapeSummary {
    pub task_runs: Vec<ScanAddonBulkMetadataScrapeTaskRunSummary>,
    pub skipped_addons: Vec<ScanAddonBulkMetadataScrapeSkippedAddon>,
    pub total_sources: usize,
    pub enqueued_items: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ScanAddonBulkMetadataScrapeTaskRunSummary {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub job_id: JobId,
    pub declaration_id: String,
    pub status: JobStatus,
    pub idempotent_replay: bool,
    pub item_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ScanAddonBulkMetadataScrapeSkippedAddon {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ScanAddonBulkMetadataScrapeRequest<'a> {
    pub scan_job_id: JobId,
    pub library: &'a Library,
    pub writeback: bool,
}

impl AddonAppService {
    pub(crate) async fn create_scan_bulk_metadata_scrape_task_runs(
        &self,
        request: ScanAddonBulkMetadataScrapeRequest<'_>,
    ) -> Result<ScanAddonBulkMetadataScrapeSummary> {
        let sources = self
            .scan_bulk_metadata_scrape_sources(request.library.id)
            .await?;
        let payload = self
            .scan_bulk_metadata_scrape_payload(
                request.scan_job_id,
                request.library,
                &sources,
                request.writeback,
            )
            .await?;
        let mut summary = ScanAddonBulkMetadataScrapeSummary {
            total_sources: sources.len(),
            enqueued_items: payload.items.len(),
            truncated: sources.len() >= SCAN_ADDON_SCRAPE_SOURCE_LIMIT as usize,
            ..ScanAddonBulkMetadataScrapeSummary::default()
        };

        if payload.items.is_empty() {
            return Ok(summary);
        }

        let addons = self
            .registration_store
            .list_addon_registrations(Some(AddonStatus::Enabled))
            .await?;
        for addon in addons {
            match self
                .create_scan_bulk_metadata_scrape_task_run_for_addon(
                    &addon,
                    request.scan_job_id,
                    request.library.id,
                    &payload,
                )
                .await
            {
                Ok(Some(run)) => summary.task_runs.push(run),
                Ok(None) => summary
                    .skipped_addons
                    .push(scan_addon_bulk_metadata_scrape_skip(
                        &addon,
                        "task_not_declared",
                    )),
                Err(err) => summary
                    .skipped_addons
                    .push(scan_addon_bulk_metadata_scrape_skip(
                        &addon,
                        scan_addon_bulk_metadata_scrape_safe_error_code(&err),
                    )),
            }
        }

        Ok(summary)
    }

    async fn create_scan_bulk_metadata_scrape_task_run_for_addon(
        &self,
        addon: &AddonRegistrationRecord,
        scan_job_id: JobId,
        library_id: LibraryId,
        payload: &ScanAddonBulkMetadataScrapePayload,
    ) -> Result<Option<ScanAddonBulkMetadataScrapeTaskRunSummary>> {
        let manifest = self.stored_manifest(addon)?;
        let Some(task) = manifest
            .tasks
            .iter()
            .find(|task| task.id == metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID)
        else {
            return Ok(None);
        };

        self.ensure_executable_task_routing_plan(addon.id, &task.id)
            .await?;
        let response = self
            .create_addon_task_run(
                addon.id,
                CreateAddonTaskRunRequest {
                    declaration_id: task.id.clone(),
                    idempotency_key: format!(
                        "library-scan:{scan_job_id}:addon-bulk-metadata-scrape:{}",
                        addon.id
                    ),
                    dispatch: AddonTaskRunDispatchMode::Direct,
                    library_id: Some(library_id),
                    source_id: None,
                    payload: serde_json::to_value(payload).map_err(|err| {
                        NakoError::InvalidInput {
                            message: format!(
                                "failed to serialize scan addon metadata scrape payload: {err}"
                            ),
                        }
                    })?,
                },
            )
            .await?;

        Ok(Some(ScanAddonBulkMetadataScrapeTaskRunSummary {
            addon_id: response.run.addon_id,
            manifest_id: response.run.manifest_id,
            job_id: response.run.job_id,
            declaration_id: response.run.declaration_id,
            status: response.run.status,
            idempotent_replay: response.idempotent_replay,
            item_count: payload.items.len(),
        }))
    }

    async fn scan_bulk_metadata_scrape_sources(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<MediaSource>> {
        self.store
            .list_media_sources(
                library_id,
                PageRequest::new(SCAN_ADDON_SCRAPE_SOURCE_LIMIT, 0),
            )
            .await
    }

    async fn scan_bulk_metadata_scrape_payload(
        &self,
        scan_job_id: JobId,
        library: &Library,
        sources: &[MediaSource],
        writeback: bool,
    ) -> Result<ScanAddonBulkMetadataScrapePayload> {
        let mut items = Vec::with_capacity(sources.len());
        for source in sources {
            let item = self.store.get_media_item(source.item_id).await?;
            items.push(scan_addon_bulk_metadata_scrape_item(
                scan_job_id,
                library,
                source,
                item.as_ref(),
                writeback,
            ));
        }

        Ok(ScanAddonBulkMetadataScrapePayload {
            cursor: 0,
            batch_size: SCAN_ADDON_SCRAPE_BATCH_SIZE,
            items,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
struct ScanAddonBulkMetadataScrapePayload {
    items: Vec<ScanAddonBulkMetadataScrapeItem>,
    cursor: usize,
    batch_size: usize,
}

pub(super) fn scan_addon_bulk_metadata_scrape_continuation_request(
    completed: &AddonTaskRunRecord,
    output: &serde_json::Value,
) -> Result<Option<CreateAddonTaskRunRequest>> {
    if completed.declaration_id != metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID {
        return Ok(None);
    }

    let Some(next_cursor) = output
        .get("next_cursor")
        .and_then(serde_json::Value::as_u64)
    else {
        return Ok(None);
    };
    let Some(payload) = scan_addon_bulk_metadata_scrape_continuation_payload(
        &completed.input_json,
        next_cursor,
        output,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(CreateAddonTaskRunRequest {
        declaration_id: completed.declaration_id.clone(),
        idempotency_key: scan_addon_bulk_metadata_scrape_continuation_idempotency_key(
            &completed.idempotency_key,
            next_cursor,
        ),
        dispatch: retry_dispatch_from_previous_input(&completed.input_json)?,
        library_id: completed.job.library_id,
        source_id: completed.job.source_id,
        payload,
    }))
}

fn scan_addon_bulk_metadata_scrape_continuation_payload(
    previous_input_json: &str,
    next_cursor: u64,
    output: &serde_json::Value,
) -> Result<Option<serde_json::Value>> {
    let mut payload = retry_payload_from_previous_input(previous_input_json)?;
    let Some(object) = payload.as_object_mut() else {
        return Err(NakoError::InvalidInput {
            message: "scan addon metadata scrape continuation payload must be an object".to_owned(),
        });
    };
    let current_cursor = object
        .get("cursor")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let total_items = object
        .get("items")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.len() as u64);
    if next_cursor <= current_cursor || total_items.is_some_and(|total| next_cursor >= total) {
        return Ok(None);
    }

    object.insert("cursor".to_owned(), serde_json::json!(next_cursor));
    if let Some(batch_size) = output.get("batch_size") {
        object.insert("batch_size".to_owned(), batch_size.clone());
    }
    if let Some(provider_policy) = output.get("provider_policy") {
        object.insert("provider_policy".to_owned(), provider_policy.clone());
    }
    if let Some(resume_state) = output.get("resume_state") {
        object.insert("resume_state".to_owned(), resume_state.clone());
    }

    Ok(Some(payload))
}

fn scan_addon_bulk_metadata_scrape_continuation_idempotency_key(
    previous: &str,
    next_cursor: u64,
) -> String {
    let root = previous
        .split_once(":cursor:")
        .map_or(previous, |(root, _)| root);
    format!("{root}:cursor:{next_cursor}")
}

#[derive(Clone, Debug, Serialize)]
struct ScanAddonBulkMetadataScrapeItem {
    library_id: LibraryId,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    kind: MediaKind,
    locator_scheme: String,
    file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
    external_ids: Vec<ScanAddonBulkMetadataScrapeExternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    writeback: Option<ScanAddonBulkMetadataScrapeWriteback>,
}

#[derive(Clone, Debug, Serialize)]
struct ScanAddonBulkMetadataScrapeExternalId {
    provider: String,
    value: String,
}

#[derive(Clone, Debug, Serialize)]
struct ScanAddonBulkMetadataScrapeWriteback {
    library_id: LibraryId,
    target: AddonSideEffectTarget,
    idempotency_key: String,
}

fn scan_addon_bulk_metadata_scrape_item(
    scan_job_id: JobId,
    library: &Library,
    source: &MediaSource,
    item: Option<&MediaItem>,
    writeback: bool,
) -> ScanAddonBulkMetadataScrapeItem {
    let metadata = item.map(|item| &item.metadata);
    let title = metadata
        .map(|metadata| metadata.title.trim())
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| source_file_stem(&source.file_name))
        .to_owned();
    let release_date = metadata.and_then(|metadata| metadata.release_date.clone());

    ScanAddonBulkMetadataScrapeItem {
        library_id: library.id,
        item_id: source.item_id,
        source_id: source.id,
        title,
        original_title: metadata.and_then(|metadata| metadata.original_title.clone()),
        sort_title: metadata.and_then(|metadata| metadata.sort_title.clone()),
        year: release_date.as_deref().and_then(year_from_release_date),
        release_date,
        language: library.options.metadata_profile.language.clone(),
        kind: item.map_or(MediaKind::Unknown, |item| item.kind),
        locator_scheme: source_locator_scheme(&source.locator).to_owned(),
        file_name: source.file_name.clone(),
        size_bytes: source.size_bytes,
        external_ids: metadata.map_or_else(Vec::new, |metadata| {
            metadata
                .external_ids
                .iter()
                .map(|external_id| ScanAddonBulkMetadataScrapeExternalId {
                    provider: external_provider_wire_value(&external_id.provider).to_owned(),
                    value: external_id.value.clone(),
                })
                .collect()
        }),
        writeback: writeback.then(|| ScanAddonBulkMetadataScrapeWriteback {
            library_id: library.id,
            target: AddonSideEffectTarget::media_source(source.id),
            idempotency_key: format!(
                "library-scan:{scan_job_id}:addon-bulk-metadata-writeback:{}",
                source.id
            ),
        }),
    }
}

fn scan_addon_bulk_metadata_scrape_skip(
    addon: &AddonRegistrationRecord,
    reason: impl Into<String>,
) -> ScanAddonBulkMetadataScrapeSkippedAddon {
    ScanAddonBulkMetadataScrapeSkippedAddon {
        addon_id: addon.id,
        manifest_id: addon.manifest_id.clone(),
        reason: reason.into(),
    }
}

fn scan_addon_bulk_metadata_scrape_safe_error_code(err: &NakoError) -> &'static str {
    match err {
        NakoError::Conflict { .. } => "routing_plan_not_executable",
        NakoError::Forbidden { .. } => "missing_grant",
        NakoError::InvalidInput { .. } => "invalid_addon_contract",
        NakoError::NotFound { .. } => "addon_task_not_found",
        _ => "task_run_create_failed",
    }
}

fn source_file_stem(file_name: &str) -> &str {
    file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _extension)| stem)
}

fn source_locator_scheme(locator: &str) -> &str {
    locator
        .split_once(':')
        .map_or("unknown", |(scheme, _)| scheme)
}

fn year_from_release_date(value: &str) -> Option<i32> {
    let year = value.get(0..4)?;
    if !year.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    year.parse().ok()
}

fn external_provider_wire_value(provider: &ExternalProvider) -> &str {
    match provider {
        ExternalProvider::Tmdb => "tmdb",
        ExternalProvider::Douban => "douban",
        ExternalProvider::Bangumi => "bangumi",
        ExternalProvider::Imdb => "imdb",
        ExternalProvider::Local => "local",
        ExternalProvider::Other(value) => value,
    }
}
