use nako_core::*;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub(crate) struct GeneratedArtifactProposalFacts {
    pub(crate) artifact: AutomationArtifactRecord,
    pub(crate) provider_exists: bool,
    pub(crate) provider_name: Option<String>,
    pub(crate) job_exists: bool,
    pub(crate) job_input_json: Option<String>,
    pub(crate) job_summary_json: Option<String>,
    pub(crate) library_exists: bool,
    pub(crate) item_exists: bool,
    pub(crate) source_exists: bool,
    pub(crate) source_library_id: Option<LibraryId>,
    pub(crate) source_item_id: Option<MediaItemId>,
}

pub(crate) fn generated_artifact_proposal(
    facts: GeneratedArtifactProposalFacts,
) -> GeneratedArtifactProposal {
    let artifact = &facts.artifact;
    let target = GeneratedArtifactTarget::from_artifact(artifact);
    let payload = summarize_generated_artifact_payload(&artifact.artifact_json);
    let input = facts
        .job_input_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<AutomationJobInput>(value).ok());
    let summary = facts
        .job_summary_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<AutomationJobSummary>(value).ok());
    let readiness = generated_artifact_readiness(artifact, &payload, &facts, input.as_ref());
    let provenance = GeneratedArtifactProvenance {
        provider_id: artifact.provider_id,
        provider_name: facts.provider_name,
        job_id: artifact.job_id,
        capability: artifact.capability,
        idempotency_key_fingerprint: input
            .as_ref()
            .map(|input| stable_fingerprint(&input.idempotency_key)),
        prompt_fingerprint: input
            .as_ref()
            .map(|input| stable_fingerprint(&input.prompt_json)),
        attempt_count: summary.map(|summary| summary.attempt_count),
        artifact_created_at: artifact.created_at.clone(),
    };

    GeneratedArtifactProposal {
        id: artifact.id,
        kind: artifact.kind,
        capability: artifact.capability,
        status: artifact.status,
        target,
        provenance,
        payload,
        readiness,
        created_at: artifact.created_at.clone(),
        updated_at: artifact.updated_at.clone(),
        accepted_at: artifact.accepted_at.clone(),
    }
}

fn generated_artifact_readiness(
    artifact: &AutomationArtifactRecord,
    payload: &GeneratedArtifactPayloadSummary,
    facts: &GeneratedArtifactProposalFacts,
    input: Option<&AutomationJobInput>,
) -> GeneratedArtifactReadiness {
    let mut reasons = Vec::new();

    match artifact.status {
        AutomationArtifactStatus::Proposed => {}
        AutomationArtifactStatus::Accepted => {
            reasons.push(GeneratedArtifactReadinessReason::ArtifactAlreadyAccepted);
        }
        AutomationArtifactStatus::Rejected => {
            reasons.push(GeneratedArtifactReadinessReason::ArtifactAlreadyRejected);
        }
    }

    if !payload.valid_json {
        reasons.push(GeneratedArtifactReadinessReason::InvalidPayloadJson);
    }
    if !facts.job_exists {
        reasons.push(GeneratedArtifactReadinessReason::MissingJob);
    }
    if !facts.provider_exists {
        reasons.push(GeneratedArtifactReadinessReason::MissingProvider);
    }
    if artifact.library_id.is_none() && artifact.item_id.is_none() && artifact.source_id.is_none() {
        reasons.push(GeneratedArtifactReadinessReason::TargetRequired);
    }
    if artifact.library_id.is_some() && !facts.library_exists {
        reasons.push(GeneratedArtifactReadinessReason::MissingLibrary);
    }
    if artifact.item_id.is_some() && !facts.item_exists {
        reasons.push(GeneratedArtifactReadinessReason::MissingMediaItem);
    }
    if artifact.source_id.is_some() && !facts.source_exists {
        reasons.push(GeneratedArtifactReadinessReason::MissingMediaSource);
    }
    if artifact.source_id.is_some()
        && facts.source_exists
        && (artifact.library_id != facts.source_library_id
            || artifact.item_id != facts.source_item_id)
    {
        reasons.push(GeneratedArtifactReadinessReason::TargetMismatch);
    }
    if facts.job_exists && input.is_none() {
        reasons.push(GeneratedArtifactReadinessReason::JobInputMismatch);
    }
    if let Some(input) = input {
        if input.provider_id != artifact.provider_id
            || input.capability != artifact.capability
            || input.library_id != artifact.library_id
            || input.item_id != artifact.item_id
            || input.source_id != artifact.source_id
        {
            reasons.push(GeneratedArtifactReadinessReason::JobInputMismatch);
        }
    }

    GeneratedArtifactReadiness::from_reasons(reasons)
}

fn summarize_generated_artifact_payload(value: &str) -> GeneratedArtifactPayloadSummary {
    let payload_bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
    let payload_fingerprint = stable_fingerprint(value);

    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return GeneratedArtifactPayloadSummary {
            valid_json: false,
            shape: GeneratedArtifactPayloadShape::InvalidJson,
            payload_fingerprint,
            payload_bytes,
            object_field_count: None,
            array_item_count: None,
            has_textual_values: false,
            has_explanation: false,
            confidence_milli: None,
        };
    };

    let shape = match &json {
        serde_json::Value::Object(_) => GeneratedArtifactPayloadShape::Object,
        serde_json::Value::Array(_) => GeneratedArtifactPayloadShape::Array,
        serde_json::Value::String(_) => GeneratedArtifactPayloadShape::String,
        serde_json::Value::Number(_) => GeneratedArtifactPayloadShape::Number,
        serde_json::Value::Bool(_) => GeneratedArtifactPayloadShape::Boolean,
        serde_json::Value::Null => GeneratedArtifactPayloadShape::Null,
    };

    GeneratedArtifactPayloadSummary {
        valid_json: true,
        shape,
        payload_fingerprint,
        payload_bytes,
        object_field_count: json
            .as_object()
            .map(|object| u32::try_from(object.len()).unwrap_or(u32::MAX)),
        array_item_count: json
            .as_array()
            .map(|array| u32::try_from(array.len()).unwrap_or(u32::MAX)),
        has_textual_values: has_textual_value(&json),
        has_explanation: has_explanation(&json),
        confidence_milli: confidence_milli(&json),
    }
}

fn has_textual_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(value) => !value.trim().is_empty(),
        serde_json::Value::Array(values) => values.iter().any(has_textual_value),
        serde_json::Value::Object(values) => values.values().any(has_textual_value),
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
            false
        }
    }
}

fn has_explanation(value: &serde_json::Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key("explanation") || object.contains_key("reason"))
}

fn confidence_milli(value: &serde_json::Value) -> Option<u16> {
    let value = value.as_object()?.get("confidence_milli")?.as_u64()?;
    u16::try_from(value.min(1_000)).ok()
}

fn stable_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let prefix = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{prefix}")
}
