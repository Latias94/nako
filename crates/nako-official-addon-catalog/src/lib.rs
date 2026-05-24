use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonEventSubscriptionDeclaration, AddonHostedPageDeclaration,
    AddonInstallDescriptor, AddonManifest, AddonResource, AddonResourceDeclaration,
    AddonRuntimeKind, AddonRuntimeRequirement, AddonScope, AddonSecretReferenceFieldDeclaration,
    AddonTaskDeclaration,
};

pub mod metadata_scraper {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.metadata-scraper";
    pub const ADDON_NAME: &str = "Nako Metadata Scraper";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9100";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-metadata-scraper:9100";
    pub const RUNTIME_BINARY: &str = "nako-metadata-scraper";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-metadata-scraper:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako metadata scraper sidecar. It returns metadata suggestions and can submit explicit Nako-owned metadata/artwork side effects when configured.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.metadata-scraper.config.v1";
    pub const METADATA_RESOURCE_PATH: &str = "/metadata";
    pub const METADATA_REQUEST_SCHEMA: &str = "nako.metadata.request.v1";
    pub const METADATA_RESPONSE_SCHEMA: &str = "nako.metadata.response.v1";
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "metadata-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "Metadata Scraper Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const BULK_METADATA_SCRAPE_TASK_ID: &str = "bulk-metadata-scrape";
    pub const BULK_METADATA_SCRAPE_TASK_NAME: &str = "Bulk metadata scrape";
    pub const BULK_METADATA_SCRAPE_TASK_PATH: &str = "/tasks/bulk-metadata-scrape";
    pub const BULK_METADATA_SCRAPE_TASK_DESCRIPTION: &str =
        "Runs metadata suggestions for a bounded batch of items";
    pub const LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID: &str = "library-scanned";
    pub const LIBRARY_SCANNED_EVENT_KIND: &str = "library.scanned";
    pub const LIBRARY_SCANNED_EVENT_PATH: &str = "/events/library-scanned";
    pub const DEFAULT_LANGUAGE: &str = "en-US";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 2;
    pub const TASK_TIMEOUT_MS: u64 = 30_000;
    pub const PROVIDER_FIXTURE: &str = "fixture";
    pub const PROVIDER_TMDB: &str = "tmdb";
    pub const PROVIDER_BANGUMI: &str = "bangumi";
    pub const PROVIDER_BROWSER_WORKER: &str = "browser_worker";
    pub const PROVIDER_DOUBAN: &str = "douban";

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct ProviderToggle {
        pub id: &'static str,
        pub enabled: bool,
    }

    impl ProviderToggle {
        #[must_use]
        pub const fn new(id: &'static str, enabled: bool) -> Self {
            Self { id, enabled }
        }
    }

    #[must_use]
    pub const fn default_provider_toggles() -> [ProviderToggle; 5] {
        [
            ProviderToggle::new(PROVIDER_FIXTURE, true),
            ProviderToggle::new(PROVIDER_TMDB, false),
            ProviderToggle::new(PROVIDER_BANGUMI, false),
            ProviderToggle::new(PROVIDER_BROWSER_WORKER, false),
            ProviderToggle::new(PROVIDER_DOUBAN, false),
        ]
    }

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_BASE_URL,
            DEFAULT_LANGUAGE,
            default_provider_toggles(),
            Vec::new(),
        )
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_CONTAINER_BASE_URL,
            DEFAULT_LANGUAGE,
            default_provider_toggles(),
            Vec::new(),
        )
    }

    #[must_use]
    pub fn manifest(
        base_url: impl Into<String>,
        preferred_language: impl Into<String>,
        providers: impl IntoIterator<Item = ProviderToggle>,
        secret_reference_fields: Vec<AddonSecretReferenceFieldDeclaration>,
    ) -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            base_url,
            preferred_language,
            providers,
            secret_reference_fields,
        )
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
        preferred_language: impl Into<String>,
        providers: impl IntoIterator<Item = ProviderToggle>,
        secret_reference_fields: Vec<AddonSecretReferenceFieldDeclaration>,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::Metadata,
                path: METADATA_RESOURCE_PATH.to_owned(),
                input_schema: Some(METADATA_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(METADATA_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![
                    AddonScope::ItemMetadataRead,
                    AddonScope::ItemMetadataSuggest,
                ],
                timeout_ms: Some(DEFAULT_TIMEOUT_MS),
                max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            }],
            entry_points: vec![AddonEntryPointDeclaration::hosted_page(
                DIAGNOSTICS_ENTRY_POINT_ID,
                AddonEntryPointKind::Diagnostics,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                DIAGNOSTICS_HOSTED_PAGE_ID,
                vec![AddonScope::ItemMetadataRead],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration {
                id: DIAGNOSTICS_HOSTED_PAGE_ID.to_owned(),
                title: DIAGNOSTICS_LABEL.to_owned(),
                path: DIAGNOSTICS_PATH.to_owned(),
                required_scopes: vec![AddonScope::ItemMetadataRead],
            }],
            configuration_schema: Some(configuration_schema(preferred_language, providers)),
            secret_reference_fields,
            event_subscriptions: vec![AddonEventSubscriptionDeclaration::new(
                LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID,
                LIBRARY_SCANNED_EVENT_KIND,
                LIBRARY_SCANNED_EVENT_PATH,
                vec![AddonScope::WebhookEventRead],
                serde_json::Value::Null,
            )],
            tasks: vec![
                AddonTaskDeclaration::new(
                    BULK_METADATA_SCRAPE_TASK_ID,
                    BULK_METADATA_SCRAPE_TASK_NAME,
                    BULK_METADATA_SCRAPE_TASK_PATH,
                    vec![AddonScope::AutomationRun],
                )
                .with_description(BULK_METADATA_SCRAPE_TASK_DESCRIPTION)
                .with_execution_bounds(Some(TASK_TIMEOUT_MS), Some(DEFAULT_MAX_ATTEMPTS)),
            ],
            auth: AddonAuth::None,
            default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
                AddonScope::WebhookEventRead,
            ],
        }
    }

    #[must_use]
    pub fn binary_install_descriptor() -> AddonInstallDescriptor {
        AddonInstallDescriptor {
            manifest: default_manifest(),
            runtime: AddonRuntimeRequirement {
                kind: AddonRuntimeKind::HttpSidecar,
                image: None,
                binary: Some(RUNTIME_BINARY.to_owned()),
                command: None,
            },
            secret_reference_bindings: Vec::new(),
            install_notes: vec![
                format!("Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."),
                "Run the sidecar outside Nako and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
            ],
        }
    }

    #[must_use]
    pub fn container_install_descriptor() -> AddonInstallDescriptor {
        AddonInstallDescriptor {
            manifest: container_manifest(),
            runtime: AddonRuntimeRequirement {
                kind: AddonRuntimeKind::HttpSidecar,
                image: Some(RUNTIME_IMAGE.to_owned()),
                binary: None,
                command: None,
            },
            secret_reference_bindings: Vec::new(),
            install_notes: vec![
                format!("Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/metadata-scraper` Dockerfile."),
                "Run the sidecar outside Nako and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn configuration_schema(
        preferred_language: impl Into<String>,
        providers: impl IntoIterator<Item = ProviderToggle>,
    ) -> AddonConfigurationSchema {
        let mut provider_properties = serde_json::Map::new();
        for provider in providers {
            provider_properties.insert(
                provider.id.to_owned(),
                serde_json::json!({
                    "type": "boolean",
                    "default": provider.enabled,
                }),
            );
        }

        AddonConfigurationSchema {
            schema_id: CONFIG_SCHEMA_ID.to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "preferred_language": {
                        "type": "string",
                        "default": preferred_language.into(),
                    },
                    "providers": {
                        "type": "object",
                        "properties": provider_properties,
                        "additionalProperties": false,
                    },
                },
                "additionalProperties": false,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonRuntimeReferenceKind, addon_install_guide, validate_manifest};

    use super::metadata_scraper::*;

    #[test]
    fn metadata_scraper_default_manifest_matches_official_catalog_facts() {
        let manifest = default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, ADDON_ID);
        assert_eq!(manifest.version, ADDON_VERSION);
        assert_eq!(manifest.base_url, DEFAULT_BASE_URL);
        assert_eq!(manifest.entry_points[0].id, DIAGNOSTICS_ENTRY_POINT_ID);
        assert_eq!(manifest.hosted_pages[0].id, DIAGNOSTICS_HOSTED_PAGE_ID);
        assert_eq!(manifest.tasks[0].id, BULK_METADATA_SCRAPE_TASK_ID);
        assert_eq!(manifest.secret_reference_fields, Vec::new());
        assert_eq!(manifest.event_subscriptions.len(), 1);
        assert_eq!(
            manifest.event_subscriptions[0].id,
            LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(
            manifest.event_subscriptions[0].event_kind,
            LIBRARY_SCANNED_EVENT_KIND
        );
        assert_eq!(
            manifest.event_subscriptions[0].path,
            LIBRARY_SCANNED_EVENT_PATH
        );
        assert_eq!(
            manifest.event_subscriptions[0].required_scopes,
            vec![nako_addon_protocol::AddonScope::WebhookEventRead]
        );
    }

    #[test]
    fn metadata_scraper_binary_descriptor_is_catalog_safe() {
        let descriptor = binary_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Binary
        );
        assert_eq!(guide.runtime_reference.value, RUNTIME_BINARY);
        assert_eq!(guide.task_count, 1);
        assert_eq!(guide.event_subscription_count, 1);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn metadata_scraper_container_descriptor_matches_checked_in_manifest_shape() {
        let descriptor = container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(descriptor.manifest.base_url, DEFAULT_CONTAINER_BASE_URL);
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(guide.runtime_reference.value, RUNTIME_IMAGE);
        assert_eq!(guide.task_count, 1);
        assert_eq!(guide.event_subscription_count, 1);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }
}
