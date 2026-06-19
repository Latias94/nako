use std::{collections::HashMap, time::Instant};

use nako_addon_client::{
    ReqwestAddonTransport, call_addon_resource_link_check_with_outcome,
    call_addon_resource_search_with_outcome,
};
use nako_addon_protocol::{
    ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
    AddonResourceLink, AddonResourceLinkCheckRequest, AddonResourceSearchRequest,
    AddonResourceSearchResult,
};
use nako_api::extension::{
    AddonAcquisitionCandidateSummary, AdminAddonResourceCallDiagnosticStatus,
    AdminAddonResourceLinkCheckRequest, AdminAddonResourceLinkCheckResponse,
    AdminAddonResourceSearchDiagnosticRequest, AdminAddonResourceSearchDiagnosticResponse,
    AdminAddonResourceSearchLinkSummary, AdminAddonResourceSearchProviderDiagnostic,
    AdminAddonResourceSearchRequest, AdminAddonResourceSearchResponse,
    AdminAddonResourceSearchResultSummary, AdminAddonResourceSearchSelectionRequest,
    AdminAddonResourceSearchSelectionResponse,
};
use nako_core::{AddonId, AddonStatus, NakoError, Result};

use crate::app::acquisition_intake::{
    AcquisitionIntakeCandidateDiagnostic, RecordResourceSearchSelectionRequest,
};

use super::{
    AddonAppService,
    diagnostics::{
        resource_diagnostic_status_for_client_error, safe_resource_diagnostic_error_code,
    },
    fingerprint_key,
    helpers::stored_granted_scopes,
    optional_non_empty, redact_uri,
    resource_flow::{SelectionSession, SelectionSessionLookup, SelectionSessionStore},
    sha256_hex,
};
const RESOURCE_SEARCH_DIAGNOSTIC_DEFAULT_LIMIT: usize = 20;
const RESOURCE_SEARCH_DIAGNOSTIC_MAX_LIMIT: usize = 50;

#[derive(Clone, Debug)]
struct ResourceSearchSessionContext {
    query: String,
}

#[derive(Clone, Debug)]
struct ResourceSearchSelection {
    result: AddonResourceSearchResult,
    selected_link: AddonResourceLink,
}

#[derive(Clone, Debug)]
struct ResourceSearchSelectionHandoff {
    manifest_id: String,
    query: String,
    selection: ResourceSearchSelection,
}

#[derive(Debug, Default)]
pub(super) struct ResourceSearchSessionStore {
    sessions: SelectionSessionStore<ResourceSearchSelection, ResourceSearchSessionContext>,
}

impl ResourceSearchSessionStore {
    fn insert(
        &mut self,
        search_id: String,
        addon_id: AddonId,
        manifest_id: String,
        query: String,
        created_at_ms: i64,
        selections: HashMap<String, ResourceSearchSelection>,
    ) {
        self.sessions.insert(SelectionSession::new(
            search_id,
            addon_id,
            manifest_id,
            ResourceSearchSessionContext { query },
            created_at_ms,
            selections,
        ));
    }

    fn get_selection(
        &mut self,
        addon_id: AddonId,
        manifest_id: &str,
        search_id: &str,
        selection_id: &str,
        now_ms: i64,
    ) -> Result<ResourceSearchSelectionHandoff> {
        match self
            .sessions
            .get_selection(addon_id, manifest_id, search_id, selection_id, now_ms)
        {
            SelectionSessionLookup::Found(handoff) => Ok(ResourceSearchSelectionHandoff {
                manifest_id: handoff.manifest_id,
                query: handoff.context.query,
                selection: handoff.selection,
            }),
            SelectionSessionLookup::Missing => Err(NakoError::NotFound {
                entity: "resource_search_selection",
                id: selection_id.to_owned(),
            }),
            SelectionSessionLookup::ManifestMismatch => Err(NakoError::Conflict {
                message: "resource search session belongs to a different addon manifest".to_owned(),
            }),
        }
    }
}

impl AddonAppService {
    pub async fn diagnose_addon_resource_search(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceSearchDiagnosticRequest,
    ) -> Result<AdminAddonResourceSearchDiagnosticResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let query = request.query.trim().to_owned();
        if query.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "resource search query cannot be empty".to_owned(),
            });
        }
        let limit = normalize_resource_search_diagnostic_limit(request.limit)?;
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let search_request = AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: request.intent,
            query,
            limit: Some(limit),
            sources: request.sources,
            link_types: request.link_types,
            refresh: request.refresh,
            context: request.context,
        };
        let started = Instant::now();
        let response = call_addon_resource_search_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-search-{addon_id}"),
            search_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                let link_count = response
                    .results
                    .iter()
                    .map(|result| result.links.len())
                    .sum();
                let merged_link_count = response.merged_by_type.values().map(Vec::len).sum();
                Ok(AdminAddonResourceSearchDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    limit,
                    total: response.total,
                    result_count: response.results.len(),
                    link_count,
                    merged_link_count,
                    provider_executions: response
                        .provider_executions
                        .into_iter()
                        .map(AdminAddonResourceSearchProviderDiagnostic::from)
                        .collect(),
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceSearchDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    limit,
                    total: 0,
                    result_count: 0,
                    link_count: 0,
                    merged_link_count: 0,
                    provider_executions: Vec::new(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }

    pub async fn search_addon_resources(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceSearchRequest,
    ) -> Result<AdminAddonResourceSearchResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let query = request.query.trim().to_owned();
        if query.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "resource search query cannot be empty".to_owned(),
            });
        }
        let limit = normalize_resource_search_diagnostic_limit(request.limit)?;
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let search_id = new_resource_search_id();
        let search_request = AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: request.intent,
            query: query.clone(),
            limit: Some(limit),
            sources: request.sources,
            link_types: request.link_types,
            refresh: request.refresh,
            context: request.context,
        };
        let started = Instant::now();
        let response = call_addon_resource_search_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-search-{addon_id}"),
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
                    .map(AdminAddonResourceSearchProviderDiagnostic::from)
                    .collect();
                let (results, selections) =
                    safe_resource_search_results(&search_id, response.results);
                let result_count = results.len();
                let now_ms = crate::app::current_time_ms()?;
                self.resource_search_sessions.lock().await.insert(
                    search_id.clone(),
                    addon_id,
                    addon.manifest_id.clone(),
                    query,
                    now_ms,
                    selections,
                );

                Ok(AdminAddonResourceSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    limit,
                    total,
                    result_count,
                    results,
                    provider_executions,
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    limit,
                    total: 0,
                    result_count: 0,
                    results: Vec::new(),
                    provider_executions: Vec::new(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }

    pub async fn select_addon_resource_search_result(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonResourceSearchSelectionRequest,
    ) -> Result<AdminAddonResourceSearchSelectionResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self.resource_search_sessions.lock().await.get_selection(
            addon_id,
            &addon.manifest_id,
            &search_id,
            &selection_id,
            crate::app::current_time_ms()?,
        )?;

        let diagnostic =
            crate::app::acquisition_intake::AcquisitionIntakeAppService::new_with_storage(
                self.store.clone(),
                self.storage_backends.clone(),
            )
            .record_resource_search_selection(RecordResourceSearchSelectionRequest {
                target_library_id: request.target_library_id,
                addon_id,
                manifest_id: handoff.manifest_id.clone(),
                query: handoff.query,
                result: handoff.selection.result,
                selected_link: handoff.selection.selected_link,
            })
            .await?;

        Ok(AdminAddonResourceSearchSelectionResponse {
            addon_id,
            manifest_id: handoff.manifest_id,
            search_id,
            selection_id,
            candidate: addon_acquisition_candidate_summary(diagnostic.candidate),
            idempotent_replay: diagnostic.idempotent_replay,
        })
    }

    pub async fn check_addon_resource_search_selection_link(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonResourceLinkCheckRequest,
    ) -> Result<AdminAddonResourceLinkCheckResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self.resource_search_sessions.lock().await.get_selection(
            addon_id,
            &addon.manifest_id,
            &search_id,
            &selection_id,
            crate::app::current_time_ms()?,
        )?;

        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let selected_link = handoff.selection.selected_link.clone();
        let link_type = selected_link.link_type;
        let link_check_request = AddonResourceLinkCheckRequest {
            schema: ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA.to_owned(),
            link: selected_link,
            refresh: request.refresh,
            context: resource_link_check_selection_context(&search_id, &selection_id, &handoff),
        };
        let started = Instant::now();
        let response = call_addon_resource_link_check_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-link-check-{addon_id}"),
            link_check_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                if response.link_type != link_type {
                    return Ok(AdminAddonResourceLinkCheckResponse {
                        addon_id,
                        manifest_id: handoff.manifest_id,
                        search_id,
                        selection_id,
                        status: AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch,
                        latency_ms,
                        attempts: outcome.attempts,
                        link_type,
                        check_status: None,
                        checked_at_ms: None,
                        requires_password: None,
                        retryable: None,
                        retry_after_ms: None,
                        has_safe_message: false,
                        safe_facts: Default::default(),
                        http_status: Some(outcome.http_status),
                        safe_error_code: Some("link_type_mismatch".to_owned()),
                    });
                }

                Ok(AdminAddonResourceLinkCheckResponse {
                    addon_id,
                    manifest_id: handoff.manifest_id,
                    search_id,
                    selection_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    link_type,
                    check_status: Some(response.status),
                    checked_at_ms: Some(response.checked_at_ms),
                    requires_password: Some(response.requires_password),
                    retryable: Some(response.retryable),
                    retry_after_ms: response.retry_after_ms,
                    has_safe_message: response.safe_message.is_some(),
                    safe_facts: response.safe_facts,
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceLinkCheckResponse {
                    addon_id,
                    manifest_id: handoff.manifest_id,
                    search_id,
                    selection_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    link_type,
                    check_status: None,
                    checked_at_ms: None,
                    requires_password: None,
                    retryable: None,
                    retry_after_ms: None,
                    has_safe_message: false,
                    safe_facts: Default::default(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }
}

fn normalize_resource_search_diagnostic_limit(limit: Option<usize>) -> Result<usize> {
    let limit = limit.unwrap_or(RESOURCE_SEARCH_DIAGNOSTIC_DEFAULT_LIMIT);
    if limit == 0 {
        return Err(NakoError::InvalidInput {
            message: "resource search limit must be greater than zero".to_owned(),
        });
    }

    Ok(limit.min(RESOURCE_SEARCH_DIAGNOSTIC_MAX_LIMIT))
}
fn safe_resource_search_results(
    search_id: &str,
    results: Vec<AddonResourceSearchResult>,
) -> (
    Vec<AdminAddonResourceSearchResultSummary>,
    HashMap<String, ResourceSearchSelection>,
) {
    let mut summaries = Vec::with_capacity(results.len());
    let mut selections = HashMap::new();

    for (result_index, result) in results.into_iter().enumerate() {
        let mut links = Vec::new();
        for (link_index, link) in result.links.iter().enumerate() {
            let Some(source_uri) = resource_search_link_uri(link) else {
                continue;
            };
            let selection_id =
                resource_search_selection_id(search_id, result_index, link_index, &source_uri);
            links.push(AdminAddonResourceSearchLinkSummary {
                selection_id: selection_id.clone(),
                link_type: link.link_type,
                source: link.source.clone(),
                source_ref_redacted: redact_uri(&source_uri),
                has_password: link.password.is_some(),
                has_note: link
                    .note
                    .as_ref()
                    .is_some_and(|note| !note.trim().is_empty()),
            });
            selections.insert(
                selection_id,
                ResourceSearchSelection {
                    result: resource_search_selection_result_snapshot(&result),
                    selected_link: link.clone(),
                },
            );
        }

        summaries.push(AdminAddonResourceSearchResultSummary {
            result_ref_fingerprint: fingerprint_key(&result.id),
            title: result.title,
            content: optional_non_empty(result.content),
            source: result.source,
            tags: result
                .tags
                .into_iter()
                .filter_map(|tag| optional_non_empty(Some(tag)))
                .collect(),
            score: result.score,
            links,
        });
    }

    (summaries, selections)
}
fn resource_search_link_uri(link: &AddonResourceLink) -> Option<String> {
    optional_non_empty(Some(link.normalized_url.clone()))
        .or_else(|| optional_non_empty(Some(link.url.clone())))
}

fn resource_search_selection_result_snapshot(
    result: &AddonResourceSearchResult,
) -> AddonResourceSearchResult {
    AddonResourceSearchResult {
        id: result.id.clone(),
        title: result.title.clone(),
        source: result.source.clone(),
        content: result.content.clone(),
        links: result
            .links
            .iter()
            .map(resource_search_link_count_placeholder)
            .collect(),
        tags: result.tags.clone(),
        images: result.images.iter().map(|_| String::new()).collect(),
        score: result.score,
    }
}

fn resource_search_link_count_placeholder(link: &AddonResourceLink) -> AddonResourceLink {
    AddonResourceLink {
        url: String::new(),
        normalized_url: String::new(),
        link_type: link.link_type,
        source: link.source.clone(),
        password: None,
        note: None,
    }
}

fn resource_search_selection_id(
    search_id: &str,
    result_index: usize,
    link_index: usize,
    source_uri: &str,
) -> String {
    let material = format!(
        "nako.resource-search-selection-id.v1\0{search_id}\0{result_index}\0{link_index}\0{source_uri}"
    );
    format!("sel_{}", &sha256_hex(&material)[..32])
}
fn resource_link_check_selection_context(
    search_id: &str,
    selection_id: &str,
    handoff: &ResourceSearchSelectionHandoff,
) -> serde_json::Value {
    let source_ref_redacted = resource_search_link_uri(&handoff.selection.selected_link)
        .map(|source_uri| redact_uri(&source_uri));

    serde_json::json!({
        "schema": "nako.resource_link_check.selection_context.v1",
        "search_id": search_id,
        "selection_id": selection_id,
        "query_fingerprint": fingerprint_key(&handoff.query),
        "result_ref_fingerprint": fingerprint_key(&handoff.selection.result.id),
        "link_type": handoff.selection.selected_link.link_type.as_str(),
        "source_ref_redacted": source_ref_redacted,
    })
}

fn new_resource_search_id() -> String {
    format!("rs_{}", uuid::Uuid::new_v4().simple())
}
fn addon_acquisition_candidate_summary(
    diagnostic: AcquisitionIntakeCandidateDiagnostic,
) -> AddonAcquisitionCandidateSummary {
    AddonAcquisitionCandidateSummary {
        id: diagnostic.id,
        target_library_id: diagnostic.target_library_id,
        state: diagnostic.state,
        source_kind: diagnostic.source_kind,
        source_scheme: diagnostic.source_scheme,
        source_ref_redacted: diagnostic.source_uri_redacted,
        source_key_fingerprint: diagnostic.source_key_fingerprint,
        has_display_name: diagnostic.has_display_name,
        has_intended_locator: diagnostic.has_intended_locator,
        size_bytes: diagnostic.size_bytes,
        has_fingerprint: diagnostic.has_fingerprint,
        has_diagnostics: diagnostic.has_diagnostics,
        managed_import_artifact_id: diagnostic.managed_import_artifact_id,
        writes_library: false,
        creates_media_source: false,
        creates_managed_import: false,
        promotion_apply: false,
    }
}
