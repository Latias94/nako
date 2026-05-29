use std::{collections::BTreeMap, str::FromStr};

use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA,
    AddonExternalAcquisitionActionRequest, AddonExternalAcquisitionActionResponse,
    AddonExternalAcquisitionActionStatus, AddonExternalAcquisitionMaterializationPurpose,
    AddonExternalAcquisitionMaterializationRequest,
    AddonExternalAcquisitionMaterializationResponse, AddonExternalAcquisitionMaterializedLink,
    AddonExternalAcquisitionOperation, AddonExternalAcquisitionTargetRef, AddonResourceLinkType,
    AddonTaskDeclaration,
};
use nako_api::extension::{AddonTaskRunDispatchMode, CreateAddonTaskRunRequest};
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateRecord, AcquisitionIntakeRepository,
    AcquisitionIntakeSourceKind, AddonTaskRunRecord, AddonTaskRunRepository, JobId, JobStatus,
    NakoError, Result,
};
use time::{Duration as TimeDuration, OffsetDateTime, format_description::well_known::Rfc3339};

use super::{
    AddonAppService, fingerprint_key, redact_uri,
    task_runtime::{addon_task_run_input, ensure_run_belongs_to_addon},
};

const EXTERNAL_ACQUISITION_MATERIALIZATION_TTL_SECONDS: i64 = 60;

#[derive(Debug)]
pub(super) struct ExternalAcquisitionActionTaskOutput {
    pub output: serde_json::Value,
    pub safe_failure_code: Option<String>,
    pub progress_metrics: serde_json::Value,
}

pub(super) fn normalize_external_acquisition_action_task_request(
    task: &AddonTaskDeclaration,
    request: CreateAddonTaskRunRequest,
) -> Result<CreateAddonTaskRunRequest> {
    if task.input_schema.as_deref() != Some(ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA) {
        return Ok(request);
    }
    if task.output_schema.as_deref() != Some(ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA) {
        return Err(invalid_external_acquisition_action_request());
    }
    if request.dispatch != AddonTaskRunDispatchMode::Direct {
        return Err(invalid_external_acquisition_action_request());
    }

    let mut action =
        serde_json::from_value::<AddonExternalAcquisitionActionRequest>(request.payload.clone())
            .map_err(|_err| invalid_external_acquisition_action_request())?;
    validate_external_acquisition_action_request(&request, &action)?;
    action.idempotency_key.clone_from(&request.idempotency_key);
    let payload = serde_json::to_value(action).map_err(|_err| NakoError::InvalidInput {
        message: "invalid external acquisition action payload".to_owned(),
    })?;

    Ok(CreateAddonTaskRunRequest { payload, ..request })
}

pub(super) fn normalize_external_acquisition_action_task_output(
    task: &AddonTaskDeclaration,
    output: serde_json::Value,
) -> std::result::Result<ExternalAcquisitionActionTaskOutput, ()> {
    if task.output_schema.as_deref() != Some(ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA) {
        return Ok(ExternalAcquisitionActionTaskOutput {
            output,
            safe_failure_code: None,
            progress_metrics: serde_json::json!({}),
        });
    }

    let response = serde_json::from_value::<AddonExternalAcquisitionActionResponse>(output)
        .map_err(|_err| ())?;
    if response.schema != ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA {
        return Err(());
    }

    let safe_failure_code = match response.status {
        AddonExternalAcquisitionActionStatus::Accepted
        | AddonExternalAcquisitionActionStatus::AlreadyExists => None,
        AddonExternalAcquisitionActionStatus::Rejected
        | AddonExternalAcquisitionActionStatus::NotFound
        | AddonExternalAcquisitionActionStatus::Failed => Some(
            response
                .safe_message
                .as_deref()
                .and_then(normalized_safe_failure_code)
                .unwrap_or_else(|| fallback_action_failure_code(response.status)),
        ),
    };
    let progress_metrics = serde_json::json!({
        "external_acquisition_status": response.status.as_str(),
        "external_acquisition_state": response.state.as_str(),
        "external_acquisition_retryable": response.retryable,
        "external_acquisition_percent_milli": response
            .progress
            .as_ref()
            .and_then(|progress| progress.percent_milli),
    });
    let output = serde_json::to_value(response).map_err(|_err| ())?;

    Ok(ExternalAcquisitionActionTaskOutput {
        output,
        safe_failure_code,
        progress_metrics,
    })
}

impl AddonAppService {
    pub async fn materialize_external_acquisition(
        &self,
        raw_token: &str,
        request: AddonExternalAcquisitionMaterializationRequest,
    ) -> Result<AddonExternalAcquisitionMaterializationResponse> {
        validate_materialization_request_shape(&request)?;
        let principal = self.resolve_addon_principal(raw_token).await?;
        let job_id = parse_job_id(&request.job_id)?;
        let run = self
            .store
            .get_addon_task_run(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_task_run",
                id: job_id.to_string(),
            })?;
        ensure_run_belongs_to_addon(&run, principal.addon.id)?;
        validate_materialization_run_context(&run, &request)?;

        let action = action_request_from_run(&run)?;
        validate_materialization_action_context(&run, &action, &request)?;

        let candidate = self
            .materialization_candidate_for_target(&request.target_ref, run.job.library_id)
            .await?;
        let material = materialized_link_from_candidate(&candidate)?;
        let mut safe_facts = BTreeMap::new();
        let (source_kind, _) = candidate.source_kind.as_parts();
        safe_facts.insert(
            "target_kind".to_owned(),
            target_ref_kind(&request.target_ref).to_owned(),
        );
        safe_facts.insert("source_kind".to_owned(), source_kind.to_owned());
        safe_facts.insert(
            "link_type".to_owned(),
            material.link_type.as_str().to_owned(),
        );
        safe_facts.insert(
            "source_ref_redacted".to_owned(),
            redact_uri(&candidate.source_uri),
        );

        Ok(AddonExternalAcquisitionMaterializationResponse {
            schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA.to_owned(),
            materialization_ref: materialization_ref(&run, &candidate),
            target_ref: request.target_ref,
            expires_at: materialization_expires_at()?,
            material,
            safe_facts,
        })
    }

    async fn materialization_candidate_for_target(
        &self,
        target_ref: &AddonExternalAcquisitionTargetRef,
        run_library_id: Option<nako_core::LibraryId>,
    ) -> Result<AcquisitionIntakeCandidateRecord> {
        let Some(library_id) = run_library_id else {
            return Err(invalid_external_acquisition_materialization_request());
        };
        let (candidate_id, require_resource_search_selection) = match target_ref {
            AddonExternalAcquisitionTargetRef::SelectedLink { selected_link_ref } => {
                (parse_candidate_id(selected_link_ref)?, true)
            }
            AddonExternalAcquisitionTargetRef::IntakeCandidate {
                intake_candidate_ref,
            } => (parse_candidate_id(intake_candidate_ref)?, false),
            AddonExternalAcquisitionTargetRef::RunnerJob { .. } => {
                return Err(invalid_external_acquisition_materialization_request());
            }
        };
        let candidate = self
            .store
            .get_acquisition_intake_candidate(candidate_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "acquisition_intake_candidate",
                id: candidate_id.to_string(),
            })?;
        if candidate.target_library_id != library_id {
            return Err(invalid_external_acquisition_materialization_request());
        }
        if require_resource_search_selection
            && candidate.source_kind != AcquisitionIntakeSourceKind::ResourceSearchSelection
        {
            return Err(invalid_external_acquisition_materialization_request());
        }
        if !matches!(
            candidate.state,
            nako_core::AcquisitionIntakeCandidateState::Ready
                | nako_core::AcquisitionIntakeCandidateState::Accepted
        ) {
            return Err(NakoError::Conflict {
                message: "external acquisition materialization target is not ready".to_owned(),
            });
        }
        if candidate.source_uri.trim().is_empty() {
            return Err(invalid_external_acquisition_materialization_request());
        }

        Ok(candidate)
    }
}

fn validate_materialization_request_shape(
    request: &AddonExternalAcquisitionMaterializationRequest,
) -> Result<()> {
    if request.schema != ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA {
        return Err(invalid_external_acquisition_materialization_request());
    }
    validate_non_empty_materialization_reference(&request.job_id)?;
    validate_non_empty_materialization_reference(&request.declaration_id)?;
    validate_non_empty_materialization_reference(&request.runner_profile_id)?;
    validate_non_empty_materialization_reference(&request.idempotency_key)?;
    validate_non_empty_materialization_reference(&request.audit_ref)?;
    if request.purpose != AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue
    {
        return Err(invalid_external_acquisition_materialization_request());
    }
    if !request.operation.can_materialize_target() {
        return Err(invalid_external_acquisition_materialization_request());
    }

    Ok(())
}

fn validate_materialization_run_context(
    run: &AddonTaskRunRecord,
    request: &AddonExternalAcquisitionMaterializationRequest,
) -> Result<()> {
    if run.declaration_id != request.declaration_id {
        return Err(invalid_external_acquisition_materialization_request());
    }
    if run.idempotency_key != request.idempotency_key {
        return Err(invalid_external_acquisition_materialization_request());
    }
    if run.job.status != JobStatus::Running {
        return Err(NakoError::Conflict {
            message: "external acquisition materialization requires a running action task"
                .to_owned(),
        });
    }

    Ok(())
}

fn action_request_from_run(
    run: &AddonTaskRunRecord,
) -> Result<AddonExternalAcquisitionActionRequest> {
    let input = addon_task_run_input(&run.input_json)?;
    let payload = input
        .get("payload")
        .cloned()
        .ok_or_else(invalid_external_acquisition_materialization_request)?;
    let action = serde_json::from_value::<AddonExternalAcquisitionActionRequest>(payload)
        .map_err(|_err| invalid_external_acquisition_materialization_request())?;
    if action.schema != ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA {
        return Err(invalid_external_acquisition_materialization_request());
    }

    Ok(action)
}

fn validate_materialization_action_context(
    run: &AddonTaskRunRecord,
    action: &AddonExternalAcquisitionActionRequest,
    request: &AddonExternalAcquisitionMaterializationRequest,
) -> Result<()> {
    if action.target_ref != request.target_ref
        || action.runner_profile_id != request.runner_profile_id
        || action.idempotency_key != request.idempotency_key
        || action.idempotency_key != run.idempotency_key
        || action.operation != request.operation
        || action.audit_ref.as_deref() != Some(request.audit_ref.as_str())
    {
        return Err(invalid_external_acquisition_materialization_request());
    }

    Ok(())
}

fn parse_job_id(value: &str) -> Result<JobId> {
    JobId::from_str(value).map_err(|_err| invalid_external_acquisition_materialization_request())
}

fn parse_candidate_id(value: &str) -> Result<AcquisitionIntakeCandidateId> {
    AcquisitionIntakeCandidateId::from_str(value)
        .map_err(|_err| invalid_external_acquisition_materialization_request())
}

fn materialized_link_from_candidate(
    candidate: &AcquisitionIntakeCandidateRecord,
) -> Result<AddonExternalAcquisitionMaterializedLink> {
    let link_type = candidate_link_type(candidate);
    ensure_external_runner_link_type(link_type)?;

    Ok(AddonExternalAcquisitionMaterializedLink {
        link_type,
        uri: candidate.source_uri.clone(),
        password: None,
    })
}

fn ensure_external_runner_link_type(link_type: AddonResourceLinkType) -> Result<()> {
    if matches!(
        link_type,
        AddonResourceLinkType::Magnet | AddonResourceLinkType::Ed2k | AddonResourceLinkType::Web
    ) {
        return Ok(());
    }

    Err(NakoError::InvalidInput {
        message: "external acquisition materialization target link type is unsupported".to_owned(),
    })
}

fn candidate_link_type(candidate: &AcquisitionIntakeCandidateRecord) -> AddonResourceLinkType {
    candidate
        .diagnostics_json
        .as_deref()
        .and_then(diagnostics_link_type)
        .unwrap_or_else(|| link_type_from_uri(&candidate.source_uri))
}

fn diagnostics_link_type(value: &str) -> Option<AddonResourceLinkType> {
    let value = serde_json::from_str::<serde_json::Value>(value).ok()?;
    let link_type = value
        .get("link")
        .and_then(|link| link.get("type"))
        .and_then(serde_json::Value::as_str)?;
    serde_json::from_value::<AddonResourceLinkType>(serde_json::Value::String(link_type.to_owned()))
        .ok()
}

fn link_type_from_uri(value: &str) -> AddonResourceLinkType {
    let value = value.trim().to_ascii_lowercase();
    if value.starts_with("magnet:") {
        return AddonResourceLinkType::Magnet;
    }
    if value.starts_with("ed2k:") {
        return AddonResourceLinkType::Ed2k;
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        return AddonResourceLinkType::Web;
    }

    AddonResourceLinkType::Other
}

fn target_ref_kind(target_ref: &AddonExternalAcquisitionTargetRef) -> &'static str {
    match target_ref {
        AddonExternalAcquisitionTargetRef::SelectedLink { .. } => "selected_link",
        AddonExternalAcquisitionTargetRef::IntakeCandidate { .. } => "intake_candidate",
        AddonExternalAcquisitionTargetRef::RunnerJob { .. } => "runner_job",
    }
}

fn materialization_ref(
    run: &AddonTaskRunRecord,
    candidate: &AcquisitionIntakeCandidateRecord,
) -> String {
    fingerprint_key(&format!(
        "nako.external-acquisition-materialization.v1\0{}\0{}\0{}",
        run.job.id, run.idempotency_key, candidate.id
    ))
}

fn materialization_expires_at() -> Result<String> {
    let expires_at = OffsetDateTime::now_utc()
        + TimeDuration::seconds(EXTERNAL_ACQUISITION_MATERIALIZATION_TTL_SECONDS);
    expires_at
        .format(&Rfc3339)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to format external acquisition materialization expiry: {err}"),
        })
}

fn validate_external_acquisition_action_request(
    request: &CreateAddonTaskRunRequest,
    action: &AddonExternalAcquisitionActionRequest,
) -> Result<()> {
    if action.schema != ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA {
        return Err(invalid_external_acquisition_action_request());
    }
    if action.idempotency_key != request.idempotency_key {
        return Err(invalid_external_acquisition_action_request());
    }
    validate_non_empty_reference(&action.runner_profile_id)?;
    validate_non_empty_reference(&action.idempotency_key)?;
    validate_target_for_operation(action)?;
    validate_audit_ref_for_operation(action)?;

    Ok(())
}

fn validate_target_for_operation(action: &AddonExternalAcquisitionActionRequest) -> Result<()> {
    match (&action.operation, &action.target_ref) {
        (
            AddonExternalAcquisitionOperation::Enqueue,
            AddonExternalAcquisitionTargetRef::SelectedLink { selected_link_ref },
        ) => validate_non_empty_reference(selected_link_ref),
        (
            AddonExternalAcquisitionOperation::Enqueue,
            AddonExternalAcquisitionTargetRef::IntakeCandidate {
                intake_candidate_ref,
            },
        ) => validate_non_empty_reference(intake_candidate_ref),
        (
            AddonExternalAcquisitionOperation::Cancel
            | AddonExternalAcquisitionOperation::Pause
            | AddonExternalAcquisitionOperation::Resume
            | AddonExternalAcquisitionOperation::QueryStatus,
            AddonExternalAcquisitionTargetRef::RunnerJob { runner_job_ref },
        ) => validate_non_empty_reference(runner_job_ref),
        _ => Err(invalid_external_acquisition_action_request()),
    }
}

fn validate_audit_ref_for_operation(action: &AddonExternalAcquisitionActionRequest) -> Result<()> {
    if let Some(audit_ref) = action.audit_ref.as_deref() {
        return validate_non_empty_reference(audit_ref);
    }

    match action.operation {
        AddonExternalAcquisitionOperation::Enqueue
        | AddonExternalAcquisitionOperation::Cancel
        | AddonExternalAcquisitionOperation::Pause
        | AddonExternalAcquisitionOperation::Resume => {
            Err(invalid_external_acquisition_action_request())
        }
        AddonExternalAcquisitionOperation::QueryStatus => Ok(()),
    }
}

fn validate_non_empty_reference(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(invalid_external_acquisition_action_request());
    }

    Ok(())
}

fn validate_non_empty_materialization_reference(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(invalid_external_acquisition_materialization_request());
    }

    Ok(())
}

fn invalid_external_acquisition_action_request() -> NakoError {
    NakoError::InvalidInput {
        message: "invalid external acquisition action request".to_owned(),
    }
}

fn invalid_external_acquisition_materialization_request() -> NakoError {
    NakoError::InvalidInput {
        message: "invalid external acquisition materialization request".to_owned(),
    }
}

fn normalized_safe_failure_code(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '_' || character == '-')
    {
        return None;
    }

    Some(value.to_owned())
}

fn fallback_action_failure_code(status: AddonExternalAcquisitionActionStatus) -> String {
    format!("external_acquisition_{}", status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_acquisition_materialization_link_type_policy_blocks_cloud_drive_links() {
        for allowed in [
            AddonResourceLinkType::Magnet,
            AddonResourceLinkType::Ed2k,
            AddonResourceLinkType::Web,
        ] {
            ensure_external_runner_link_type(allowed).unwrap();
        }

        for rejected in [
            AddonResourceLinkType::Quark,
            AddonResourceLinkType::Baidu,
            AddonResourceLinkType::Pikpak,
            AddonResourceLinkType::Other,
        ] {
            assert!(ensure_external_runner_link_type(rejected).is_err());
        }
    }
}
