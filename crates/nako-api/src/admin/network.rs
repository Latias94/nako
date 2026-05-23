use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkAccessDiagnostics {
    pub exposure_mode: AdminNetworkExposureMode,
    pub readiness: AdminNetworkReadinessDiagnostics,
    pub external_endpoint: AdminNetworkExternalEndpointDiagnostics,
    pub trusted_proxy: AdminTrustedProxyDiagnostics,
    pub origins: AdminOriginPolicyDiagnostics,
    pub tunnel_providers: Vec<AdminTunnelProviderDiagnostics>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkExposureMode {
    LocalOnly,
    PrivateNetwork,
    ReverseProxy,
    TunnelProvider,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkReadinessDiagnostics {
    pub status: AdminNetworkReadinessStatus,
    pub reason: AdminNetworkReadinessReason,
    pub checks: Vec<AdminNetworkReadinessCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessReason {
    Ready,
    LocalOnly,
    AuthDisabled,
    MissingExternalBaseUrl,
    MissingTrustedProxySources,
    MissingTunnelProvider,
    MissingTunnelToken,
    BrowserOriginsNotConfigured,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkReadinessCheck {
    pub name: AdminNetworkReadinessCheckName,
    pub status: AdminNetworkReadinessStatus,
    pub reason: AdminNetworkReadinessReason,
}

impl AdminNetworkReadinessCheck {
    #[must_use]
    pub const fn ready(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Ready,
            reason,
        }
    }

    #[must_use]
    pub const fn degraded(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Degraded,
            reason,
        }
    }

    #[must_use]
    pub const fn unavailable(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Unavailable,
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessCheckName {
    ExposureMode,
    Auth,
    ExternalEndpoint,
    TrustedProxy,
    OriginPolicy,
    TunnelProvider,
}

impl AdminNetworkReadinessDiagnostics {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminNetworkReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminNetworkReadinessStatus::Unavailable)
        {
            AdminNetworkReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminNetworkReadinessStatus::Degraded)
        {
            AdminNetworkReadinessStatus::Degraded
        } else {
            AdminNetworkReadinessStatus::Ready
        };
        let reason = checks
            .iter()
            .find(|check| check.status == status)
            .map_or(AdminNetworkReadinessReason::Ready, |check| check.reason);

        Self {
            status,
            reason,
            checks,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkExternalEndpointDiagnostics {
    pub configured: bool,
    pub scheme: Option<String>,
    pub host_fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTrustedProxyDiagnostics {
    pub headers_enabled: bool,
    pub source_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOriginPolicyDiagnostics {
    pub allowed_origin_count: u32,
    pub configured: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTunnelProviderDiagnostics {
    pub id: String,
    pub kind: AdminTunnelProviderKind,
    pub endpoint_configured: bool,
    pub endpoint_scheme: Option<String>,
    pub endpoint_host_fingerprint: Option<String>,
    pub token_env: Option<String>,
    pub token_present: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminTunnelProviderKind {
    External,
    CloudflareTunnel,
    TailscaleFunnel,
    Ngrok,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_readiness_diagnostics_prioritize_unavailable_over_degraded() {
        let readiness = AdminNetworkReadinessDiagnostics::from_checks(vec![
            AdminNetworkReadinessCheck::ready(
                AdminNetworkReadinessCheckName::Auth,
                AdminNetworkReadinessReason::Ready,
            ),
            AdminNetworkReadinessCheck::degraded(
                AdminNetworkReadinessCheckName::OriginPolicy,
                AdminNetworkReadinessReason::BrowserOriginsNotConfigured,
            ),
            AdminNetworkReadinessCheck::unavailable(
                AdminNetworkReadinessCheckName::TunnelProvider,
                AdminNetworkReadinessReason::MissingTunnelToken,
            ),
        ]);

        assert_eq!(readiness.status, AdminNetworkReadinessStatus::Unavailable);
        assert_eq!(
            readiness.reason,
            AdminNetworkReadinessReason::MissingTunnelToken
        );
        assert_eq!(readiness.checks.len(), 3);
    }

    #[test]
    fn network_access_diagnostics_serializes_without_secret_urls() {
        let response = AdminNetworkAccessDiagnostics {
            exposure_mode: AdminNetworkExposureMode::ReverseProxy,
            readiness: AdminNetworkReadinessDiagnostics::from_checks(vec![
                AdminNetworkReadinessCheck::ready(
                    AdminNetworkReadinessCheckName::Auth,
                    AdminNetworkReadinessReason::Ready,
                ),
                AdminNetworkReadinessCheck::degraded(
                    AdminNetworkReadinessCheckName::OriginPolicy,
                    AdminNetworkReadinessReason::BrowserOriginsNotConfigured,
                ),
            ]),
            external_endpoint: AdminNetworkExternalEndpointDiagnostics {
                configured: true,
                scheme: Some("https".to_owned()),
                host_fingerprint: Some("sha256:0123456789abcdef".to_owned()),
            },
            trusted_proxy: AdminTrustedProxyDiagnostics {
                headers_enabled: true,
                source_count: 2,
            },
            origins: AdminOriginPolicyDiagnostics {
                allowed_origin_count: 0,
                configured: false,
            },
            tunnel_providers: vec![AdminTunnelProviderDiagnostics {
                id: "cloudflared".to_owned(),
                kind: AdminTunnelProviderKind::CloudflareTunnel,
                endpoint_configured: true,
                endpoint_scheme: Some("https".to_owned()),
                endpoint_host_fingerprint: Some("sha256:fedcba9876543210".to_owned()),
                token_env: Some("NAKO_TUNNEL_TOKEN".to_owned()),
                token_present: true,
            }],
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["exposure_mode"], "reverse_proxy");
        assert_eq!(value["readiness"]["status"], "degraded");
        assert_eq!(
            value["readiness"]["reason"],
            "browser_origins_not_configured"
        );
        assert_eq!(value["external_endpoint"]["scheme"], "https");
        assert_eq!(value["trusted_proxy"]["source_count"], 2);
        assert_eq!(value["origins"]["allowed_origin_count"], 0);
        assert_eq!(value["tunnel_providers"][0]["kind"], "cloudflare_tunnel");
        assert_eq!(
            value["tunnel_providers"][0]["token_env"],
            "NAKO_TUNNEL_TOKEN"
        );
        assert_eq!(value["tunnel_providers"][0]["token_present"], true);
        assert!(!body.contains("external_base_url"));
        assert!(!body.contains("trusted_proxy_sources"));
        assert!(!body.contains("allowed_origins"));
        assert!(!body.contains("public_url"));
        assert!(!body.contains("nako.example"));
        assert!(!body.contains("cloudflare-token-secret"));
        assert!(!body.contains("Authorization"));
        assert!(!body.contains("x-forwarded"));
    }
}
