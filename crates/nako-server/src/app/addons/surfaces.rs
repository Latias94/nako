use std::time::Instant;

use nako_addon_client::{AddonClientError, ReqwestAddonTransport, check_addon_health};
use nako_addon_protocol::{ensure_scope_grant, validate_manifest};
use nako_api::extension::{
    AdminAddonConfigurationSchemaSurface, AdminAddonEntryPointSurface,
    AdminAddonEventSubscriptionSurface, AdminAddonHealthCheckResponse, AdminAddonHealthCheckStatus,
    AdminAddonHostedPageSurface, AdminAddonRuntimeReadinessCheck,
    AdminAddonRuntimeReadinessCheckName, AdminAddonRuntimeReadinessDiagnostics,
    AdminAddonRuntimeReadinessReason, AdminAddonRuntimeReadinessResponse,
    AdminAddonSecretReferenceFieldSurface, AdminAddonSurfacesResponse, AdminAddonTaskSurface,
};
use nako_core::{AddonId, AddonStatus, NakoError, Result};

use super::{AddonAppService, addon_surface_url, helpers::stored_granted_scopes};
impl AddonAppService {
    pub async fn check_addon_health(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonHealthCheckResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let manifest = self.stored_manifest(&addon)?;
        let started = Instant::now();
        let response = check_addon_health(
            &ReqwestAddonTransport::default(),
            &manifest,
            format!("addon-health-{addon_id}"),
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(response) => Ok(AdminAddonHealthCheckResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                status: response.status.into(),
                latency_ms,
                protocol_version: Some(response.protocol_version),
                addon_version: Some(response.manifest.addon_version),
                resource_count: Some(response.manifest.resource_count),
                protocol_checked_at: Some(response.checked_at),
                safe_error_code: None,
            }),
            Err(err) => Ok(AdminAddonHealthCheckResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                status: health_status_for_client_error(&err),
                latency_ms,
                protocol_version: None,
                addon_version: None,
                resource_count: None,
                protocol_checked_at: None,
                safe_error_code: Some(safe_health_error_code(&err).to_owned()),
            }),
        }
    }

    pub async fn check_addon_runtime_readiness(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRuntimeReadinessResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        let mut checks = Vec::new();
        let mut should_call_sidecar = true;

        if let Err(err) = validate_manifest(&manifest) {
            return Ok(AdminAddonRuntimeReadinessResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                readiness: AdminAddonRuntimeReadinessDiagnostics::from_checks(vec![
                    AdminAddonRuntimeReadinessCheck::unavailable(
                        AdminAddonRuntimeReadinessCheckName::Manifest,
                        manifest_runtime_readiness_reason(&err),
                        "invalid_manifest",
                    ),
                ]),
            });
        }

        let granted_scopes = stored_granted_scopes(&addon)?;
        let missing_grant = manifest
            .resources
            .iter()
            .any(|resource| ensure_scope_grant(&manifest, resource.kind, &granted_scopes).is_err());
        if missing_grant {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::Grants,
                AdminAddonRuntimeReadinessReason::MissingGrant,
                "missing_grant",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::Grants,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if manifest
            .secret_reference_fields
            .iter()
            .any(|field| field.required)
        {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::SecretReferences,
                AdminAddonRuntimeReadinessReason::MissingSecretReference,
                "missing_secret_reference",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::SecretReferences,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if addon_runtime_network_policy_blocked(&manifest.base_url) {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::unavailable(
                AdminAddonRuntimeReadinessCheckName::Network,
                AdminAddonRuntimeReadinessReason::NetworkPolicyBlocked,
                "network_policy_blocked",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::Network,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if should_call_sidecar {
            let health = check_addon_health(
                &ReqwestAddonTransport::default(),
                &manifest,
                format!("addon-readiness-{addon_id}"),
            )
            .await;
            match health {
                Ok(response) => {
                    checks.push(runtime_readiness_check_for_health_status(response.status));
                    checks.extend([
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Protocol,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Manifest,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Safety,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                    ]);
                }
                Err(err) => checks.push(runtime_readiness_check_for_client_error(&err)),
            }
        }

        Ok(AdminAddonRuntimeReadinessResponse {
            addon_id,
            manifest_id: addon.manifest_id,
            readiness: AdminAddonRuntimeReadinessDiagnostics::from_checks(checks),
        })
    }

    pub async fn get_addon_surfaces(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonSurfacesResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        Ok(AdminAddonSurfacesResponse {
            addon_id,
            manifest_id: addon.manifest_id,
            entry_points: manifest
                .entry_points
                .into_iter()
                .map(|entry_point| AdminAddonEntryPointSurface {
                    id: entry_point.id,
                    kind: entry_point.kind,
                    label: entry_point.label,
                    path: entry_point.path,
                    hosted_page_id: entry_point.hosted_page_id,
                    required_scopes: entry_point.required_scopes,
                })
                .collect(),
            hosted_pages: manifest
                .hosted_pages
                .into_iter()
                .map(|hosted_page| {
                    let url = addon_surface_url(&addon.base_url, &hosted_page.path);
                    AdminAddonHostedPageSurface {
                        id: hosted_page.id,
                        title: hosted_page.title,
                        path: hosted_page.path,
                        url,
                        required_scopes: hosted_page.required_scopes,
                    }
                })
                .collect(),
            configuration_schema: manifest
                .configuration_schema
                .map(AdminAddonConfigurationSchemaSurface::from),
            secret_reference_fields: manifest
                .secret_reference_fields
                .into_iter()
                .map(|field| AdminAddonSecretReferenceFieldSurface {
                    id: field.id,
                    label: field.label,
                    description: field.description,
                    required: field.required,
                })
                .collect(),
            tasks: manifest
                .tasks
                .into_iter()
                .map(AdminAddonTaskSurface::from)
                .collect(),
            event_subscriptions: manifest
                .event_subscriptions
                .into_iter()
                .map(|subscription| AdminAddonEventSubscriptionSurface {
                    id: subscription.id,
                    event_kind: subscription.event_kind,
                    path: subscription.path,
                    required_scopes: subscription.required_scopes,
                    filters: subscription.filters,
                })
                .collect(),
        })
    }
}

fn health_status_for_client_error(err: &AddonClientError) -> AdminAddonHealthCheckStatus {
    match err {
        AddonClientError::Protocol(_)
        | AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody => AdminAddonHealthCheckStatus::ProtocolMismatch,
        AddonClientError::HttpStatus { .. } | AddonClientError::Http { .. } => {
            AdminAddonHealthCheckStatus::Unreachable
        }
    }
}

fn runtime_readiness_check_for_client_error(
    err: &AddonClientError,
) -> AdminAddonRuntimeReadinessCheck {
    let name = runtime_readiness_check_name_for_client_error(err);
    let (reason, code) = runtime_readiness_reason_and_code(err);
    AdminAddonRuntimeReadinessCheck::unavailable(name, reason, code)
}

fn runtime_readiness_check_for_health_status(
    status: nako_addon_protocol::AddonHealthStatus,
) -> AdminAddonRuntimeReadinessCheck {
    match status {
        nako_addon_protocol::AddonHealthStatus::Ok => AdminAddonRuntimeReadinessCheck::ready(
            AdminAddonRuntimeReadinessCheckName::Reachability,
            AdminAddonRuntimeReadinessReason::Ready,
        ),
        nako_addon_protocol::AddonHealthStatus::Degraded => {
            AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::Reachability,
                AdminAddonRuntimeReadinessReason::SidecarDegraded,
                "sidecar_degraded",
            )
        }
        nako_addon_protocol::AddonHealthStatus::Unhealthy => {
            AdminAddonRuntimeReadinessCheck::unavailable(
                AdminAddonRuntimeReadinessCheckName::Reachability,
                AdminAddonRuntimeReadinessReason::SidecarUnhealthy,
                "sidecar_unhealthy",
            )
        }
    }
}

fn runtime_readiness_check_name_for_client_error(
    err: &AddonClientError,
) -> AdminAddonRuntimeReadinessCheckName {
    match err {
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. },
        ) => AdminAddonRuntimeReadinessCheckName::Protocol,
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            message,
        }) if health_envelope_manifest_mismatch(message) => {
            AdminAddonRuntimeReadinessCheckName::Manifest
        }
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => AdminAddonRuntimeReadinessCheckName::Safety,
        AddonClientError::Protocol(_) => AdminAddonRuntimeReadinessCheckName::Safety,
        AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody => AdminAddonRuntimeReadinessCheckName::Safety,
        AddonClientError::HttpStatus { .. } | AddonClientError::Http { .. } => {
            AdminAddonRuntimeReadinessCheckName::Reachability
        }
    }
}

fn runtime_readiness_reason_and_code(
    err: &AddonClientError,
) -> (AdminAddonRuntimeReadinessReason, &'static str) {
    match err {
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. },
        ) => (
            AdminAddonRuntimeReadinessReason::ProtocolMismatch,
            "protocol_mismatch",
        ),
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            message,
        }) if health_envelope_manifest_mismatch(message) => (
            AdminAddonRuntimeReadinessReason::ManifestMismatch,
            "manifest_mismatch",
        ),
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_response",
        ),
        AddonClientError::Protocol(_) => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_response",
        ),
        AddonClientError::InvalidRequest { .. } => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "invalid_request",
        ),
        AddonClientError::InvalidResponse { .. } => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "invalid_response",
        ),
        AddonClientError::UnsafeRequestBody => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_request_body",
        ),
        AddonClientError::HttpStatus { .. } => (
            AdminAddonRuntimeReadinessReason::Unavailable,
            safe_health_error_code(err),
        ),
        AddonClientError::Http { .. } => (
            AdminAddonRuntimeReadinessReason::Unavailable,
            "transport_failure",
        ),
    }
}

fn manifest_runtime_readiness_reason(
    err: &nako_addon_protocol::AddonManifestError,
) -> AdminAddonRuntimeReadinessReason {
    match err {
        nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. } => {
            AdminAddonRuntimeReadinessReason::ProtocolMismatch
        }
        _ => AdminAddonRuntimeReadinessReason::UnsafeResponse,
    }
}

fn health_envelope_manifest_mismatch(message: &str) -> bool {
    message.contains("manifest_id")
        || message.contains("addon_version")
        || message.contains("resource_count")
}

fn addon_runtime_network_policy_blocked(base_url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(base_url) else {
        return true;
    };
    if url.username() != "" || url.password().is_some() {
        return true;
    }
    if url.scheme() == "https" {
        return false;
    }
    if url.scheme() != "http" {
        return true;
    }

    let Some(host) = url.host_str() else {
        return true;
    };
    !matches!(host, "127.0.0.1" | "::1" | "localhost")
}

fn safe_health_error_code(err: &AddonClientError) -> &'static str {
    match err {
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
        AddonClientError::HttpStatus { status, .. } if *status == 401 || *status == 403 => {
            "unauthorized"
        }
        AddonClientError::HttpStatus { status, .. } if *status == 404 => "health_endpoint_missing",
        AddonClientError::HttpStatus { status, .. } if *status == 408 => "timeout",
        AddonClientError::HttpStatus { status, .. } if *status == 429 => "rate_limited",
        AddonClientError::HttpStatus { status, .. } if (500..600).contains(status) => {
            "sidecar_unhealthy"
        }
        AddonClientError::HttpStatus { .. } => "http_failure",
        AddonClientError::Http { .. } => "transport_failure",
    }
}
