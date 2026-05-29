use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA, AddonExternalAcquisitionActionRequest,
    AddonExternalAcquisitionActionResponse, AddonExternalAcquisitionActionStatus,
    AddonExternalAcquisitionOperation, AddonExternalAcquisitionTargetRef, AddonTaskDeclaration,
};
use nako_api::extension::{AddonTaskRunDispatchMode, CreateAddonTaskRunRequest};
use nako_core::{NakoError, Result};

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

fn invalid_external_acquisition_action_request() -> NakoError {
    NakoError::InvalidInput {
        message: "invalid external acquisition action request".to_owned(),
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
