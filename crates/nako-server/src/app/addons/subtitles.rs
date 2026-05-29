use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use nako_addon_client::{ReqwestAddonTransport, call_addon_subtitle_search_with_outcome};
use nako_addon_protocol::{
    ADDON_SUBTITLE_REQUEST_SCHEMA, AddonSubtitleCandidate, AddonSubtitleDelivery,
    AddonSubtitleSearchRequest,
};
use nako_api::extension::{
    AdminAddonResourceCallDiagnosticStatus, AdminAddonSubtitleCandidateSummary,
    AdminAddonSubtitleDeliveryKind, AdminAddonSubtitleImportApplyRequest,
    AdminAddonSubtitleImportApplyResponse, AdminAddonSubtitleImportPlanRequest,
    AdminAddonSubtitleImportPlanResponse, AdminAddonSubtitleProviderDiagnostic,
    AdminAddonSubtitleSearchRequest, AdminAddonSubtitleSearchResponse,
    AdminAddonSubtitleSelectedReference, AdminAddonSubtitleSelectionRequest,
    AdminAddonSubtitleSelectionResponse, AdminSubtitleImportApplyReport,
    AdminSubtitleImportApplyStatus, AdminSubtitleImportBackupPolicy,
    AdminSubtitleImportConflictPolicy, AdminSubtitleImportFactSummary, AdminSubtitleImportPlan,
    AdminSubtitleImportPlanReason, AdminSubtitleImportPlanStatus, AdminSubtitleImportTargetSummary,
    AdminSubtitleSidecarPlan, AdminSubtitleSidecarRole,
};
use nako_core::{
    AddonId, AddonStatus, MediaProbeRepository, MediaProbeResult, MediaRepository, MediaSource,
    MediaStreamDisposition, MediaStreamInfo, MediaStreamKind, MediaStreamOrigin,
    MediaStreamTechnicalFacts, NakoError, Result,
};

use crate::app::subtitle_sidecar::{
    SubtitleSidecarRole, safe_media_file_name, subtitle_sidecar_file_name,
};

use super::{
    AddonAppService,
    diagnostics::{
        resource_diagnostic_status_for_client_error, safe_resource_diagnostic_error_code,
    },
    fingerprint_key, library_file_write, optional_non_empty, sha256_hex, stored_granted_scopes,
};
const SUBTITLE_SEARCH_DEFAULT_LIMIT: usize = 10;
const SUBTITLE_SEARCH_MAX_LIMIT: usize = 50;
const SUBTITLE_SEARCH_SESSION_TTL_MS: i64 = 15 * 60 * 1_000;
const SUBTITLE_SEARCH_SESSION_MAX_COUNT: usize = 64;
const SUBTITLE_IMPORT_MAX_BYTES: usize = 2 * 1024 * 1024;
const SUBTITLE_IMPORT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
struct SubtitleSearchSession {
    search_id: String,
    addon_id: AddonId,
    manifest_id: String,
    created_at_ms: i64,
    expires_at_ms: i64,
    selections: HashMap<String, SubtitleSearchSelection>,
}

#[derive(Clone, Debug)]
struct SubtitleSearchSelection {
    candidate: AddonSubtitleCandidate,
}

#[derive(Clone, Debug)]
struct SubtitleSearchSelectionHandoff {
    manifest_id: String,
    selection: SubtitleSearchSelection,
}

#[derive(Clone, Debug)]
struct SubtitleImportPlanContext {
    selected_ref: AdminAddonSubtitleSelectedReference,
    candidate_summary: AdminAddonSubtitleCandidateSummary,
    plan: AdminSubtitleImportPlan,
    candidate: AddonSubtitleCandidate,
    source: MediaSource,
}

#[derive(Debug, Default)]
pub(super) struct SubtitleSearchSessionStore {
    sessions: HashMap<String, SubtitleSearchSession>,
}

impl SubtitleSearchSessionStore {
    fn insert(&mut self, session: SubtitleSearchSession) {
        self.prune(session.created_at_ms);
        self.sessions.insert(session.search_id.clone(), session);
        self.enforce_max_count();
    }

    fn get_selection(
        &mut self,
        addon_id: AddonId,
        search_id: &str,
        selection_id: &str,
        now_ms: i64,
    ) -> Option<SubtitleSearchSelectionHandoff> {
        self.prune(now_ms);
        let session = self.sessions.get(search_id)?;
        if session.addon_id != addon_id {
            return None;
        }
        let selection = session.selections.get(selection_id)?.clone();

        Some(SubtitleSearchSelectionHandoff {
            manifest_id: session.manifest_id.clone(),
            selection,
        })
    }

    fn prune(&mut self, now_ms: i64) {
        self.sessions
            .retain(|_, session| session.expires_at_ms > now_ms);
    }

    fn enforce_max_count(&mut self) {
        while self.sessions.len() > SUBTITLE_SEARCH_SESSION_MAX_COUNT {
            let Some(oldest_search_id) = self
                .sessions
                .iter()
                .min_by_key(|(_, session)| session.created_at_ms)
                .map(|(search_id, _)| search_id.clone())
            else {
                break;
            };
            self.sessions.remove(&oldest_search_id);
        }
    }
}

impl AddonAppService {
    pub async fn search_addon_subtitles(
        &self,
        addon_id: AddonId,
        request: AdminAddonSubtitleSearchRequest,
    ) -> Result<AdminAddonSubtitleSearchResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let query = request.query.trim().to_owned();
        if query.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "subtitle search query cannot be empty".to_owned(),
            });
        }
        let limit = normalize_subtitle_search_limit(request.limit)?;
        let languages = normalize_subtitle_languages(request.languages);
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let search_id = new_subtitle_search_id();
        let search_request = AddonSubtitleSearchRequest {
            schema: ADDON_SUBTITLE_REQUEST_SCHEMA.to_owned(),
            query: query.clone(),
            languages,
            limit: Some(limit),
            context: serde_json::Value::Null,
        };
        let started = Instant::now();
        let response = call_addon_subtitle_search_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-subtitle-search-{addon_id}"),
            search_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                let total = response.total;
                let provider_executions = response
                    .provider_executions
                    .into_iter()
                    .map(AdminAddonSubtitleProviderDiagnostic::from)
                    .collect();
                let (subtitles, selections) =
                    safe_subtitle_search_candidates(&search_id, response.subtitles);
                let result_count = subtitles.len();
                let now_ms = crate::app::current_time_ms()?;
                self.subtitle_search_sessions
                    .lock()
                    .await
                    .insert(SubtitleSearchSession {
                        search_id: search_id.clone(),
                        addon_id,
                        manifest_id: addon.manifest_id.clone(),
                        created_at_ms: now_ms,
                        expires_at_ms: now_ms.saturating_add(SUBTITLE_SEARCH_SESSION_TTL_MS),
                        selections,
                    });

                Ok(AdminAddonSubtitleSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    limit,
                    total,
                    result_count,
                    subtitles,
                    provider_executions,
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonSubtitleSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    limit,
                    total: 0,
                    result_count: 0,
                    subtitles: Vec::new(),
                    provider_executions: Vec::new(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }

    pub async fn select_addon_subtitle_search_candidate(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        _request: AdminAddonSubtitleSelectionRequest,
    ) -> Result<AdminAddonSubtitleSelectionResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self
            .subtitle_search_sessions
            .lock()
            .await
            .get_selection(
                addon_id,
                &search_id,
                &selection_id,
                crate::app::current_time_ms()?,
            )
            .ok_or_else(|| NakoError::NotFound {
                entity: "subtitle_search_selection",
                id: selection_id.clone(),
            })?;
        if handoff.manifest_id != addon.manifest_id {
            return Err(NakoError::Conflict {
                message: "subtitle search session belongs to a different addon manifest".to_owned(),
            });
        }

        let candidate =
            admin_subtitle_candidate_summary(&selection_id, &handoff.selection.candidate);
        let selected_ref = AdminAddonSubtitleSelectedReference {
            addon_id,
            manifest_id: handoff.manifest_id,
            search_id,
            selection_id,
            candidate_ref_fingerprint: candidate.candidate_ref_fingerprint.clone(),
            delivery_kind: candidate.delivery_kind,
        };

        Ok(AdminAddonSubtitleSelectionResponse {
            selected_ref,
            candidate,
        })
    }

    pub async fn plan_addon_subtitle_import(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonSubtitleImportPlanRequest,
    ) -> Result<AdminAddonSubtitleImportPlanResponse> {
        let context = self
            .addon_subtitle_import_plan_context(addon_id, search_id, selection_id, request)
            .await?;

        Ok(AdminAddonSubtitleImportPlanResponse {
            selected_ref: context.selected_ref,
            candidate: context.candidate_summary,
            plan: context.plan,
        })
    }

    pub async fn apply_addon_subtitle_import(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonSubtitleImportApplyRequest,
    ) -> Result<AdminAddonSubtitleImportApplyResponse> {
        let expected_key = optional_non_empty(Some(request.plan_idempotency_key.clone()))
            .ok_or_else(|| NakoError::InvalidInput {
                message: "subtitle import plan idempotency key cannot be empty".to_owned(),
            })?;
        let plan_request = AdminAddonSubtitleImportPlanRequest {
            media_item_id: request.media_item_id,
            media_source_id: request.media_source_id,
            language: request.language,
            format: request.format,
            sidecar_role: request.sidecar_role,
            conflict_policy: request.conflict_policy,
            backup_policy: request.backup_policy,
        };
        let context = self
            .addon_subtitle_import_plan_context(addon_id, search_id, selection_id, plan_request)
            .await?;
        if context.plan.idempotency_key != expected_key {
            return Err(NakoError::InvalidInput {
                message: "subtitle import plan idempotency key does not match request".to_owned(),
            });
        }
        if context.plan.status != AdminSubtitleImportPlanStatus::Ready {
            return Err(NakoError::InvalidInput {
                message: "subtitle import plan is not ready to apply".to_owned(),
            });
        }

        let content = resolve_subtitle_import_content(&context.candidate).await?;
        validate_subtitle_import_content(&content, context.plan.sidecar.format)?;
        let writer = library_file_write::LibraryFileWriteRuntime::new(
            self.store.clone(),
            self.permits.clone(),
            self.storage_backends.clone(),
        );
        let write_report = writer
            .write_subtitle_sidecar(library_file_write::SubtitleSidecarWriteRequest {
                library_id: context.source.library_id,
                source: context.source.clone(),
                file_name: context.plan.sidecar.file_name.clone(),
                content: content.clone(),
                conflict_policy: subtitle_sidecar_conflict_policy(context.plan.conflict_policy),
                backup_policy: subtitle_sidecar_backup_policy(context.plan.backup_policy),
            })
            .await?;
        let refreshed_fact = self
            .refresh_subtitle_import_fact(&context.source, &context.plan.sidecar)
            .await?;
        let apply_status = match write_report.status {
            library_file_write::SubtitleSidecarWriteStatus::Applied => {
                AdminSubtitleImportApplyStatus::Applied
            }
            library_file_write::SubtitleSidecarWriteStatus::AlreadyApplied => {
                AdminSubtitleImportApplyStatus::AlreadyApplied
            }
        };

        Ok(AdminAddonSubtitleImportApplyResponse {
            selected_ref: context.selected_ref,
            candidate: context.candidate_summary,
            apply: AdminSubtitleImportApplyReport {
                idempotency_key: context.plan.idempotency_key.clone(),
                status: apply_status,
                target: context.plan.target.clone(),
                sidecar: context.plan.sidecar.clone(),
                refreshed_fact,
                conflict_policy: context.plan.conflict_policy,
                backup_policy: context.plan.backup_policy,
                write_mode: write_report.write_mode.to_owned(),
                content_ref_fingerprint: fingerprint_key(&content),
                byte_len: write_report.byte_len,
                target_existed: write_report.target_existed,
                backup_created: write_report.backup_created,
                preview_only: false,
                writes_library: matches!(
                    write_report.status,
                    library_file_write::SubtitleSidecarWriteStatus::Applied
                ),
            },
            plan: context.plan,
        })
    }

    async fn addon_subtitle_import_plan_context(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonSubtitleImportPlanRequest,
    ) -> Result<SubtitleImportPlanContext> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self
            .subtitle_search_sessions
            .lock()
            .await
            .get_selection(
                addon_id,
                &search_id,
                &selection_id,
                crate::app::current_time_ms()?,
            )
            .ok_or_else(|| NakoError::NotFound {
                entity: "subtitle_search_selection",
                id: selection_id.clone(),
            })?;
        if handoff.manifest_id != addon.manifest_id {
            return Err(NakoError::Conflict {
                message: "subtitle search session belongs to a different addon manifest".to_owned(),
            });
        }

        let item = self
            .store
            .get_media_item(request.media_item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: request.media_item_id.to_string(),
            })?;
        let source = self
            .store
            .get_media_source(request.media_source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: request.media_source_id.to_string(),
            })?;
        if source.item_id != item.id {
            return Err(NakoError::InvalidInput {
                message: "media source does not belong to subtitle import media item".to_owned(),
            });
        }

        let language = normalize_subtitle_plan_language(&request.language)?;
        let candidate = handoff.selection.candidate;
        let candidate_summary = admin_subtitle_candidate_summary(&selection_id, &candidate);
        let selected_ref = AdminAddonSubtitleSelectedReference {
            addon_id,
            manifest_id: handoff.manifest_id,
            search_id: search_id.clone(),
            selection_id: selection_id.clone(),
            candidate_ref_fingerprint: candidate_summary.candidate_ref_fingerprint.clone(),
            delivery_kind: candidate_summary.delivery_kind,
        };
        let sidecar_file_name = subtitle_sidecar_file_name(
            &source.file_name,
            &language,
            subtitle_sidecar_role(request.sidecar_role),
            request.format.as_str(),
        )?;
        let media_file_name = safe_media_file_name(&source.file_name);
        let mut reasons = vec![AdminSubtitleImportPlanReason::MediaSourceMatchesItem];
        let mut status = AdminSubtitleImportPlanStatus::Ready;
        if candidate.language.trim().to_ascii_lowercase() != language {
            status = AdminSubtitleImportPlanStatus::Blocked;
            reasons.push(AdminSubtitleImportPlanReason::CandidateLanguageMismatch);
        }
        if candidate.format != request.format {
            status = AdminSubtitleImportPlanStatus::Blocked;
            reasons.push(AdminSubtitleImportPlanReason::CandidateFormatMismatch);
        }
        if status == AdminSubtitleImportPlanStatus::Ready {
            reasons.push(AdminSubtitleImportPlanReason::Ready);
        }

        let plan = AdminSubtitleImportPlan {
            idempotency_key: subtitle_import_plan_idempotency_key(
                addon_id,
                &addon.manifest_id,
                &search_id,
                &selection_id,
                &candidate,
                &request,
                &language,
            ),
            status,
            reasons,
            target: AdminSubtitleImportTargetSummary {
                library_id: source.library_id,
                media_item_id: item.id,
                media_source_id: source.id,
                item_title: item.metadata.title,
                media_file_name,
                source_ref_fingerprint: fingerprint_key(&source.locator),
            },
            sidecar: AdminSubtitleSidecarPlan {
                file_name: sidecar_file_name,
                language,
                format: request.format,
                role: request.sidecar_role,
            },
            conflict_policy: request.conflict_policy,
            backup_policy: request.backup_policy,
            preview_only: true,
            writes_library: false,
        };

        Ok(SubtitleImportPlanContext {
            selected_ref,
            candidate_summary,
            plan,
            candidate,
            source,
        })
    }

    async fn refresh_subtitle_import_fact(
        &self,
        source: &MediaSource,
        sidecar: &AdminSubtitleSidecarPlan,
    ) -> Result<AdminSubtitleImportFactSummary> {
        let mut probe = self
            .store
            .get_media_probe(source.id)
            .await?
            .unwrap_or_else(|| MediaProbeResult {
                duration_ms: None,
                container: None,
                bit_rate: None,
                streams: Vec::new(),
            });
        let stream_index = probe
            .streams
            .iter()
            .find(|stream| subtitle_sidecar_stream_matches(stream, sidecar))
            .map(|stream| stream.index)
            .unwrap_or_else(|| {
                probe
                    .streams
                    .iter()
                    .map(|stream| stream.index)
                    .max()
                    .and_then(|index| index.checked_add(1))
                    .unwrap_or(0)
            });
        let fact = subtitle_sidecar_stream_info(stream_index, sidecar);

        if let Some(existing) = probe
            .streams
            .iter_mut()
            .find(|stream| subtitle_sidecar_stream_matches(stream, sidecar))
        {
            *existing = fact;
        } else {
            probe.streams.push(fact);
        }
        probe.streams.sort_by_key(|stream| stream.index);
        self.store.upsert_media_probe(source.id, &probe).await?;

        Ok(AdminSubtitleImportFactSummary {
            media_source_id: source.id,
            stream_index,
            origin: "sidecar".to_owned(),
            language: sidecar.language.clone(),
            format: sidecar.format,
            role: sidecar.role,
        })
    }
}

fn normalize_subtitle_search_limit(limit: Option<usize>) -> Result<usize> {
    let limit = limit.unwrap_or(SUBTITLE_SEARCH_DEFAULT_LIMIT);
    if limit == 0 {
        return Err(NakoError::InvalidInput {
            message: "subtitle search limit must be greater than zero".to_owned(),
        });
    }

    Ok(limit.min(SUBTITLE_SEARCH_MAX_LIMIT))
}

fn normalize_subtitle_languages(languages: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    languages
        .into_iter()
        .filter_map(|language| optional_non_empty(Some(language)))
        .map(|language| language.to_ascii_lowercase())
        .filter(|language| seen.insert(language.clone()))
        .collect()
}

fn normalize_subtitle_plan_language(language: &str) -> Result<String> {
    let Some(language) = optional_non_empty(Some(language.to_owned())) else {
        return Err(NakoError::InvalidInput {
            message: "subtitle import language cannot be empty".to_owned(),
        });
    };
    let language = language.to_ascii_lowercase();
    let valid = language.len() <= 35
        && !language.starts_with('-')
        && !language.ends_with('-')
        && language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-');
    if !valid {
        return Err(NakoError::InvalidInput {
            message: "subtitle import language must be a safe BCP-47-like tag".to_owned(),
        });
    }

    Ok(language)
}

async fn resolve_subtitle_import_content(candidate: &AddonSubtitleCandidate) -> Result<String> {
    match &candidate.delivery {
        AddonSubtitleDelivery::Inline { text } => Ok(text.clone()),
        AddonSubtitleDelivery::DownloadUrl { url, .. } => {
            download_subtitle_import_content(url).await
        }
        AddonSubtitleDelivery::ArtifactRef { .. } => Err(NakoError::Unsupported(
            "subtitle artifact-ref import apply requires a host artifact resolver",
        )),
    }
}

async fn download_subtitle_import_content(url: &str) -> Result<String> {
    let url = reqwest::Url::parse(url).map_err(|_| NakoError::InvalidInput {
        message: "subtitle download URL is invalid".to_owned(),
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(NakoError::InvalidInput {
            message: "subtitle download URL must use http or https".to_owned(),
        });
    }
    let client = reqwest::Client::builder()
        .timeout(SUBTITLE_IMPORT_DOWNLOAD_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|_| NakoError::Provider {
            provider: "subtitle_download".to_owned(),
            message: "subtitle download client could not be built".to_owned(),
        })?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| NakoError::Provider {
            provider: "subtitle_download".to_owned(),
            message: "subtitle download failed".to_owned(),
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(NakoError::Provider {
            provider: "subtitle_download".to_owned(),
            message: format!("subtitle download returned HTTP status {}", status.as_u16()),
        });
    }
    if response
        .content_length()
        .is_some_and(|len| len > SUBTITLE_IMPORT_MAX_BYTES as u64)
    {
        return Err(NakoError::InvalidInput {
            message: "subtitle content exceeds import size limit".to_owned(),
        });
    }
    let bytes = response.bytes().await.map_err(|_| NakoError::Provider {
        provider: "subtitle_download".to_owned(),
        message: "subtitle download body could not be read".to_owned(),
    })?;
    if bytes.len() > SUBTITLE_IMPORT_MAX_BYTES {
        return Err(NakoError::InvalidInput {
            message: "subtitle content exceeds import size limit".to_owned(),
        });
    }

    String::from_utf8(bytes.to_vec()).map_err(|_| NakoError::InvalidInput {
        message: "subtitle content must be UTF-8 text".to_owned(),
    })
}

fn validate_subtitle_import_content(
    content: &str,
    format: nako_addon_protocol::AddonSubtitleFormat,
) -> Result<()> {
    if content.is_empty() || content.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "subtitle content cannot be empty".to_owned(),
        });
    }
    if content.len() > SUBTITLE_IMPORT_MAX_BYTES {
        return Err(NakoError::InvalidInput {
            message: "subtitle content exceeds import size limit".to_owned(),
        });
    }
    if content.contains('\0') {
        return Err(NakoError::InvalidInput {
            message: "subtitle content contains invalid NUL bytes".to_owned(),
        });
    }

    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    match format {
        nako_addon_protocol::AddonSubtitleFormat::Vtt => {
            if !trimmed.starts_with("WEBVTT") {
                return Err(NakoError::InvalidInput {
                    message: "VTT subtitle content must start with WEBVTT".to_owned(),
                });
            }
        }
        nako_addon_protocol::AddonSubtitleFormat::Srt => {
            if trimmed.starts_with("WEBVTT") || !trimmed.contains("-->") {
                return Err(NakoError::InvalidInput {
                    message: "SRT subtitle content must contain cue timings".to_owned(),
                });
            }
        }
    }

    Ok(())
}

fn subtitle_sidecar_conflict_policy(
    policy: AdminSubtitleImportConflictPolicy,
) -> library_file_write::SubtitleSidecarConflictPolicy {
    match policy {
        AdminSubtitleImportConflictPolicy::CreateMissing => {
            library_file_write::SubtitleSidecarConflictPolicy::CreateMissing
        }
        AdminSubtitleImportConflictPolicy::ReplaceExisting => {
            library_file_write::SubtitleSidecarConflictPolicy::ReplaceExisting
        }
    }
}

fn subtitle_sidecar_backup_policy(
    policy: AdminSubtitleImportBackupPolicy,
) -> library_file_write::SubtitleSidecarBackupPolicy {
    match policy {
        AdminSubtitleImportBackupPolicy::None => {
            library_file_write::SubtitleSidecarBackupPolicy::None
        }
        AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest => {
            library_file_write::SubtitleSidecarBackupPolicy::ExistingFileKeepLatest
        }
    }
}

fn subtitle_sidecar_stream_info(index: u32, sidecar: &AdminSubtitleSidecarPlan) -> MediaStreamInfo {
    MediaStreamInfo {
        index,
        kind: MediaStreamKind::Subtitle,
        codec: Some(sidecar.format.as_str().to_owned()),
        language: Some(sidecar.language.clone()),
        duration_ms: None,
        bit_rate: None,
        width: None,
        height: None,
        channels: None,
        sample_rate: None,
        technical: MediaStreamTechnicalFacts {
            origin: Some(MediaStreamOrigin::Sidecar),
            disposition: subtitle_sidecar_disposition(sidecar.role),
            ..MediaStreamTechnicalFacts::default()
        },
    }
}

fn subtitle_sidecar_stream_matches(
    stream: &MediaStreamInfo,
    sidecar: &AdminSubtitleSidecarPlan,
) -> bool {
    stream.kind == MediaStreamKind::Subtitle
        && stream.technical.origin.as_ref() == Some(&MediaStreamOrigin::Sidecar)
        && stream.codec.as_deref() == Some(sidecar.format.as_str())
        && stream.language.as_deref() == Some(sidecar.language.as_str())
        && stream.technical.disposition == subtitle_sidecar_disposition(sidecar.role)
}

fn subtitle_sidecar_disposition(role: AdminSubtitleSidecarRole) -> MediaStreamDisposition {
    match role {
        AdminSubtitleSidecarRole::Default => MediaStreamDisposition {
            default: true,
            ..MediaStreamDisposition::default()
        },
        AdminSubtitleSidecarRole::Forced => MediaStreamDisposition {
            forced: true,
            ..MediaStreamDisposition::default()
        },
        AdminSubtitleSidecarRole::Sdh => MediaStreamDisposition {
            hearing_impaired: true,
            ..MediaStreamDisposition::default()
        },
        AdminSubtitleSidecarRole::Commentary => MediaStreamDisposition {
            commentary: true,
            ..MediaStreamDisposition::default()
        },
    }
}
fn safe_subtitle_search_candidates(
    search_id: &str,
    candidates: Vec<AddonSubtitleCandidate>,
) -> (
    Vec<AdminAddonSubtitleCandidateSummary>,
    HashMap<String, SubtitleSearchSelection>,
) {
    let mut summaries = Vec::with_capacity(candidates.len());
    let mut selections = HashMap::new();

    for (candidate_index, candidate) in candidates.into_iter().enumerate() {
        let selection_id = subtitle_search_selection_id(search_id, candidate_index, &candidate.id);
        summaries.push(admin_subtitle_candidate_summary(&selection_id, &candidate));
        selections.insert(selection_id, SubtitleSearchSelection { candidate });
    }

    (summaries, selections)
}

fn admin_subtitle_candidate_summary(
    selection_id: &str,
    candidate: &AddonSubtitleCandidate,
) -> AdminAddonSubtitleCandidateSummary {
    AdminAddonSubtitleCandidateSummary {
        selection_id: selection_id.to_owned(),
        candidate_ref_fingerprint: fingerprint_key(&candidate.id),
        title: candidate.title.clone(),
        language: candidate.language.clone(),
        format: candidate.format,
        source: candidate.source.clone(),
        release: candidate.release.clone(),
        score: candidate.score,
        delivery_kind: subtitle_delivery_kind(&candidate.delivery),
    }
}

fn subtitle_delivery_kind(delivery: &AddonSubtitleDelivery) -> AdminAddonSubtitleDeliveryKind {
    match delivery {
        AddonSubtitleDelivery::Inline { .. } => AdminAddonSubtitleDeliveryKind::Inline,
        AddonSubtitleDelivery::DownloadUrl { .. } => AdminAddonSubtitleDeliveryKind::DownloadUrl,
        AddonSubtitleDelivery::ArtifactRef { .. } => AdminAddonSubtitleDeliveryKind::ArtifactRef,
    }
}
fn subtitle_search_selection_id(
    search_id: &str,
    candidate_index: usize,
    candidate_id: &str,
) -> String {
    let material = format!(
        "nako.subtitle-search-selection-id.v1\0{search_id}\0{candidate_index}\0{candidate_id}"
    );
    format!("sel_{}", &sha256_hex(&material)[..32])
}
fn new_subtitle_search_id() -> String {
    format!("sub_{}", uuid::Uuid::new_v4().simple())
}

fn subtitle_sidecar_role(role: AdminSubtitleSidecarRole) -> SubtitleSidecarRole {
    match role {
        AdminSubtitleSidecarRole::Default => SubtitleSidecarRole::Default,
        AdminSubtitleSidecarRole::Forced => SubtitleSidecarRole::Forced,
        AdminSubtitleSidecarRole::Sdh => SubtitleSidecarRole::Sdh,
        AdminSubtitleSidecarRole::Commentary => SubtitleSidecarRole::Commentary,
    }
}
fn subtitle_import_plan_idempotency_key(
    addon_id: AddonId,
    manifest_id: &str,
    search_id: &str,
    selection_id: &str,
    candidate: &AddonSubtitleCandidate,
    request: &AdminAddonSubtitleImportPlanRequest,
    normalized_language: &str,
) -> String {
    let material = format!(
        "nako.subtitle-import-plan.v1\0{addon_id}\0{manifest_id}\0{search_id}\0{selection_id}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        candidate.id,
        request.media_item_id,
        request.media_source_id,
        normalized_language,
        request.format.as_str(),
        subtitle_import_conflict_policy_key(request.conflict_policy),
        subtitle_import_backup_policy_key(request.backup_policy),
    );
    format!("sip_{}", &sha256_hex(&material)[..32])
}

fn subtitle_import_conflict_policy_key(policy: AdminSubtitleImportConflictPolicy) -> &'static str {
    match policy {
        AdminSubtitleImportConflictPolicy::CreateMissing => "create_missing",
        AdminSubtitleImportConflictPolicy::ReplaceExisting => "replace_existing",
    }
}

fn subtitle_import_backup_policy_key(policy: AdminSubtitleImportBackupPolicy) -> &'static str {
    match policy {
        AdminSubtitleImportBackupPolicy::None => "none",
        AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest => "existing_file_keep_latest",
    }
}
