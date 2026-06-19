use std::time::Instant;

use nako_addon_client::{
    AddonClientError, ReqwestAddonTransport, call_addon_resource_with_outcome,
};
use nako_api::extension::{
    AdminAddonResourceCallDiagnosticRequest, AdminAddonResourceCallDiagnosticResponse,
    AdminAddonResourceCallDiagnosticStatus,
};
use nako_core::{AddonId, AddonStatus, NakoError, Result};

use super::{AddonAppService, helpers::stored_granted_scopes};
impl AddonAppService {
    pub async fn diagnose_addon_resource_call(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceCallDiagnosticRequest,
    ) -> Result<AdminAddonResourceCallDiagnosticResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let started = Instant::now();
        let response = call_addon_resource_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            request.resource,
            &granted_scopes,
            format!("addon-diagnostic-{addon_id}"),
            request.payload,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => Ok(AdminAddonResourceCallDiagnosticResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                resource: request.resource,
                status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                latency_ms,
                attempts: outcome.attempts,
                http_status: Some(outcome.http_status),
                safe_error_code: None,
            }),
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceCallDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    resource: request.resource,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }
}

pub(super) fn resource_diagnostic_status_for_client_error(
    err: &AddonClientError,
) -> AdminAddonResourceCallDiagnosticStatus {
    match err {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => AdminAddonResourceCallDiagnosticStatus::AuthorizationGap,
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::ResourceNotDeclared { .. },
        ) => AdminAddonResourceCallDiagnosticStatus::MissingResource,
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScope { .. },
        ) => AdminAddonResourceCallDiagnosticStatus::MissingGrant,
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => AdminAddonResourceCallDiagnosticStatus::UnsafeResponse,
        AddonClientError::Protocol(_) => AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch,
        AddonClientError::InvalidRequest { .. } => {
            AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch
        }
        AddonClientError::InvalidResponse { .. } | AddonClientError::UnsafeRequestBody => {
            AdminAddonResourceCallDiagnosticStatus::UnsafeResponse
        }
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => AdminAddonResourceCallDiagnosticStatus::RetryableHttpFailure,
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => AdminAddonResourceCallDiagnosticStatus::HttpFailure,
        AddonClientError::Http { .. } => AdminAddonResourceCallDiagnosticStatus::Unreachable,
    }
}

pub(super) fn safe_resource_diagnostic_error_code(err: &AddonClientError) -> &'static str {
    match err {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => "authorization_gap",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::ResourceNotDeclared { .. },
        ) => "missing_resource",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScope { .. },
        ) => "missing_grant",
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => "unsafe_response",
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => "retryable_http_failure",
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => "http_failure",
        AddonClientError::Http { .. } => "transport_failure",
    }
}
