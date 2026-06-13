use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA, ADDON_EXTERNAL_ACQUISITION_ACTION_TASK_ID,
    ADDON_PROTOCOL_VERSION, ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA,
    ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
    ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA, ADDON_SUBTITLE_REQUEST_SCHEMA,
    ADDON_SUBTITLE_RESPONSE_SCHEMA, AddonAuth, AddonConfigurationSchema,
    AddonEntryPointDeclaration, AddonEntryPointKind, AddonEventSubscriptionDeclaration,
    AddonHostedPageDeclaration, AddonInstallDescriptor, AddonManifest, AddonResource,
    AddonResourceDeclaration, AddonRuntimeKind, AddonRuntimeRequirement, AddonScope,
    AddonSecretReferenceFieldDeclaration, AddonTaskDeclaration,
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

pub mod resource_search {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.resource-search";
    pub const ADDON_NAME: &str = "Nako Resource Search";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9130";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-resource-search:9130";
    pub const RUNTIME_BINARY: &str = "nako-resource-search";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-resource-search:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako resource search sidecar for external resource discovery, link classification, and result fusion.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.resource-search.config.v1";
    pub const RESOURCE_SEARCH_RESOURCE_PATH: &str = "/resource-search";
    pub const RESOURCE_LINK_CHECK_RESOURCE_PATH: &str = "/resource-link-check";
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "resource-search-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "Resource Search Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_LIMIT: usize = 20;
    pub const DEFAULT_MAX_LIMIT: usize = 100;
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 1;
    pub const PROVIDER_FIXTURE: &str = "fixture";
    pub const PROVIDER_PANSOU_COMPATIBLE: &str = "pansou_compatible";
    pub const PANSOU_DEFAULT_SOURCE_TYPE: &str = "all";

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
    pub const fn default_provider_toggles() -> [ProviderToggle; 2] {
        [
            ProviderToggle::new(PROVIDER_FIXTURE, true),
            ProviderToggle::new(PROVIDER_PANSOU_COMPATIBLE, false),
        ]
    }

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_BASE_URL,
            default_provider_toggles(),
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
            DEFAULT_TIMEOUT_MS,
        )
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_CONTAINER_BASE_URL,
            default_provider_toggles(),
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
            DEFAULT_TIMEOUT_MS,
        )
    }

    #[must_use]
    pub fn manifest(base_url: impl Into<String>) -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            base_url,
            default_provider_toggles(),
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
            DEFAULT_TIMEOUT_MS,
        )
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
        providers: impl IntoIterator<Item = ProviderToggle>,
        default_limit: usize,
        max_limit: usize,
        search_timeout_ms: u64,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![
                AddonResourceDeclaration {
                    kind: AddonResource::ResourceSearch,
                    path: RESOURCE_SEARCH_RESOURCE_PATH.to_owned(),
                    input_schema: Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned()),
                    output_schema: Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned()),
                    required_scopes: vec![AddonScope::AcquisitionSearchRead],
                    timeout_ms: Some(search_timeout_ms),
                    max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
                },
                AddonResourceDeclaration {
                    kind: AddonResource::ResourceLinkCheck,
                    path: RESOURCE_LINK_CHECK_RESOURCE_PATH.to_owned(),
                    input_schema: Some(ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA.to_owned()),
                    output_schema: Some(ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA.to_owned()),
                    required_scopes: vec![AddonScope::AcquisitionLinkCheckRead],
                    timeout_ms: Some(search_timeout_ms),
                    max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
                },
            ],
            entry_points: vec![AddonEntryPointDeclaration::hosted_page(
                DIAGNOSTICS_ENTRY_POINT_ID,
                AddonEntryPointKind::Diagnostics,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                DIAGNOSTICS_HOSTED_PAGE_ID,
                vec![AddonScope::AcquisitionSearchRead],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration::new(
                DIAGNOSTICS_HOSTED_PAGE_ID,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                vec![AddonScope::AcquisitionSearchRead],
            )],
            configuration_schema: Some(configuration_schema(
                providers,
                default_limit,
                max_limit,
                search_timeout_ms,
            )),
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::None,
            default_timeout_ms: Some(search_timeout_ms),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![
                AddonScope::AcquisitionSearchRead,
                AddonScope::AcquisitionLinkCheckRead,
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
                format!(
                    "Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."
                ),
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
                format!(
                    "Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/resource-search` Dockerfile."
                ),
                "Run the sidecar outside Nako and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn configuration_schema(
        providers: impl IntoIterator<Item = ProviderToggle>,
        default_limit: usize,
        max_limit: usize,
        search_timeout_ms: u64,
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

        AddonConfigurationSchema::new(
            CONFIG_SCHEMA_ID,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "providers": {
                        "type": "object",
                        "properties": provider_properties,
                        "additionalProperties": false
                    },
                    "pansou": {
                        "type": "object",
                        "properties": {
                            "base_url": {
                                "type": "string",
                                "default": ""
                            },
                            "source_type": {
                                "type": "string",
                                "default": PANSOU_DEFAULT_SOURCE_TYPE
                            },
                            "plugins": {
                                "type": "array",
                                "items": { "type": "string" },
                                "default": []
                            },
                            "cloud_types": {
                                "type": "array",
                                "items": { "type": "string" },
                                "default": []
                            },
                            "concurrency": {
                                "type": ["integer", "null"],
                                "default": null,
                                "minimum": 1
                            },
                            "timeout_ms": {
                                "type": "integer",
                                "default": DEFAULT_TIMEOUT_MS,
                                "minimum": 250,
                                "maximum": 60000
                            }
                        },
                        "additionalProperties": false
                    },
                    "default_limit": {
                        "type": "integer",
                        "default": default_limit,
                        "minimum": 1,
                        "maximum": max_limit
                    },
                    "max_limit": {
                        "type": "integer",
                        "default": max_limit,
                        "minimum": 1,
                        "maximum": 500
                    },
                    "search_timeout_ms": {
                        "type": "integer",
                        "default": search_timeout_ms,
                        "minimum": 250,
                        "maximum": 60000
                    }
                },
                "additionalProperties": false
            }),
        )
    }
}

pub mod external_acquisition_runner {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.external-acquisition-runner";
    pub const ADDON_NAME: &str = "Nako External Acquisition Runner";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9160";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-external-acquisition-runner:9160";
    pub const RUNTIME_BINARY: &str = "nako-external-acquisition-runner";
    pub const RUNTIME_IMAGE: &str =
        "ghcr.io/latias94/nako-external-acquisition-runner:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako external acquisition action sidecar for dispatching host-approved selected-link or intake-candidate references to configured runner profiles.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.external-acquisition-runner.config.v1";
    pub const ACTION_TASK_ID: &str = ADDON_EXTERNAL_ACQUISITION_ACTION_TASK_ID;
    pub const ACTION_TASK_NAME: &str = "External acquisition action";
    pub const ACTION_TASK_PATH: &str = "/tasks/external-acquisition-action";
    pub const ACTION_TASK_DESCRIPTION: &str =
        "Dispatches a host-approved external acquisition action to a configured runner profile";
    pub const ACTION_REQUEST_SCHEMA: &str = ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA;
    pub const ACTION_RESPONSE_SCHEMA: &str = ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA;
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "external-acquisition-runner-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "External Acquisition Runner Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_RUNNER_PROFILE_ID: &str = "fixture";
    pub const TRANSMISSION_RUNNER_PROFILE_ID: &str = "transmission";
    pub const TRANSMISSION_PASSWORD_SECRET_FIELD_ID: &str = "transmission_password";
    pub const TRANSMISSION_DEFAULT_RPC_URL: &str = "http://127.0.0.1:9091/transmission/rpc";
    pub const TRANSMISSION_DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const TASK_TIMEOUT_MS: u64 = 30_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 1;

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_BASE_URL, DEFAULT_RUNNER_PROFILE_ID)
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_CONTAINER_BASE_URL,
            DEFAULT_RUNNER_PROFILE_ID,
        )
    }

    #[must_use]
    pub fn manifest(base_url: impl Into<String>) -> AddonManifest {
        manifest_with_version(ADDON_VERSION, base_url, DEFAULT_RUNNER_PROFILE_ID)
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
        default_runner_profile_id: impl Into<String>,
    ) -> AddonManifest {
        let default_runner_profile_id = default_runner_profile_id.into();
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: Vec::new(),
            entry_points: vec![AddonEntryPointDeclaration::hosted_page(
                DIAGNOSTICS_ENTRY_POINT_ID,
                AddonEntryPointKind::Diagnostics,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                DIAGNOSTICS_HOSTED_PAGE_ID,
                vec![AddonScope::AcquisitionActionRun],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration::new(
                DIAGNOSTICS_HOSTED_PAGE_ID,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                vec![AddonScope::AcquisitionActionRun],
            )],
            configuration_schema: Some(configuration_schema(&default_runner_profile_id)),
            secret_reference_fields: secret_reference_fields(),
            event_subscriptions: Vec::new(),
            tasks: vec![
                AddonTaskDeclaration::new(
                    ACTION_TASK_ID,
                    ACTION_TASK_NAME,
                    ACTION_TASK_PATH,
                    vec![AddonScope::AcquisitionActionRun],
                )
                .with_schemas(ACTION_REQUEST_SCHEMA, ACTION_RESPONSE_SCHEMA)
                .with_description(ACTION_TASK_DESCRIPTION)
                .with_execution_bounds(Some(TASK_TIMEOUT_MS), Some(DEFAULT_MAX_ATTEMPTS)),
            ],
            auth: AddonAuth::None,
            default_timeout_ms: Some(TASK_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![AddonScope::AcquisitionActionRun],
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
                format!(
                    "Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."
                ),
                "Run the sidecar outside Nako; Nako dispatches only host-owned opaque acquisition references after policy approval.".to_owned(),
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
                format!(
                    "Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/external-acquisition-runner` Dockerfile."
                ),
                "Configure runner profiles in the sidecar environment; browser clients never submit raw links or credentials to this action task.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn secret_reference_fields() -> Vec<AddonSecretReferenceFieldDeclaration> {
        vec![AddonSecretReferenceFieldDeclaration::new(
            TRANSMISSION_PASSWORD_SECRET_FIELD_ID,
            "Transmission password",
            Some(
                "Optional Transmission RPC password resolved by the runner runtime; never place the raw password in task payloads."
                    .to_owned(),
            ),
            false,
        )]
    }

    #[must_use]
    fn configuration_schema(default_runner_profile_id: &str) -> AddonConfigurationSchema {
        AddonConfigurationSchema::new(
            CONFIG_SCHEMA_ID,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "default_runner_profile_id": {
                        "type": "string",
                        "default": default_runner_profile_id
                    },
                    "profiles": {
                        "type": "object",
                        "properties": {
                            "fixture": {
                                "type": "object",
                                "properties": {
                                    "enabled": {
                                        "type": "boolean",
                                        "default": true
                                    },
                                    "mode": {
                                        "type": "string",
                                        "default": "noop"
                                    }
                                },
                                "additionalProperties": false
                            },
                            "transmission": {
                                "type": "object",
                                "properties": {
                                    "enabled": {
                                        "type": "boolean",
                                        "default": false
                                    },
                                    "mode": {
                                        "type": "string",
                                        "default": "rpc"
                                    },
                                    "rpc_url": {
                                        "type": "string",
                                        "default": TRANSMISSION_DEFAULT_RPC_URL
                                    },
                                    "username": {
                                        "type": "string"
                                    },
                                    "password_secret_ref": {
                                        "type": "string",
                                        "description": "Secret reference for the optional Transmission RPC password."
                                    },
                                    "timeout_ms": {
                                        "type": "integer",
                                        "default": TRANSMISSION_DEFAULT_TIMEOUT_MS,
                                        "minimum": 250,
                                        "maximum": 60000
                                    },
                                    "allow_invalid_tls_certificates": {
                                        "type": "boolean",
                                        "default": false
                                    }
                                },
                                "additionalProperties": false
                            }
                        },
                        "additionalProperties": true
                    }
                },
                "additionalProperties": false
            }),
        )
    }
}

pub mod subtitle_provider {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.subtitle-provider";
    pub const ADDON_NAME: &str = "Nako Subtitle Provider";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9140";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-subtitle-provider:9140";
    pub const RUNTIME_BINARY: &str = "nako-subtitle-provider";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-subtitle-provider:0.1.0-alpha.2";
    pub const DESCRIPTION: &str =
        "Official Nako subtitle provider sidecar for read-only subtitle candidate discovery.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.subtitle-provider.config.v1";
    pub const SUBTITLE_RESOURCE_PATH: &str = "/subtitle";
    pub const SUBTITLE_REQUEST_SCHEMA: &str = ADDON_SUBTITLE_REQUEST_SCHEMA;
    pub const SUBTITLE_RESPONSE_SCHEMA: &str = ADDON_SUBTITLE_RESPONSE_SCHEMA;
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "subtitle-provider-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "Subtitle Provider Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_LANGUAGE: &str = "en";
    pub const DEFAULT_LIMIT: usize = 10;
    pub const DEFAULT_MAX_LIMIT: usize = 50;
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 1;

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_BASE_URL,
            true,
            DEFAULT_LANGUAGE,
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
        )
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_CONTAINER_BASE_URL,
            true,
            DEFAULT_LANGUAGE,
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
        )
    }

    #[must_use]
    pub fn manifest(base_url: impl Into<String>) -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            base_url,
            true,
            DEFAULT_LANGUAGE,
            DEFAULT_LIMIT,
            DEFAULT_MAX_LIMIT,
        )
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
        fixture_provider_enabled: bool,
        default_language: impl Into<String>,
        default_limit: usize,
        max_limit: usize,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::Subtitle,
                path: SUBTITLE_RESOURCE_PATH.to_owned(),
                input_schema: Some(SUBTITLE_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(SUBTITLE_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![AddonScope::SubtitleRead],
                timeout_ms: Some(DEFAULT_TIMEOUT_MS),
                max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            }],
            entry_points: vec![AddonEntryPointDeclaration::hosted_page(
                DIAGNOSTICS_ENTRY_POINT_ID,
                AddonEntryPointKind::Diagnostics,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                DIAGNOSTICS_HOSTED_PAGE_ID,
                vec![AddonScope::SubtitleRead],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration::new(
                DIAGNOSTICS_HOSTED_PAGE_ID,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                vec![AddonScope::SubtitleRead],
            )],
            configuration_schema: Some(configuration_schema(
                fixture_provider_enabled,
                default_language,
                default_limit,
                max_limit,
            )),
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::None,
            default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![AddonScope::SubtitleRead],
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
                format!(
                    "Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."
                ),
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
                format!(
                    "Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/subtitle-provider` Dockerfile."
                ),
                "Run the sidecar outside Nako and register the resolved manifest through the existing Admin Addon APIs; subtitle import/write policy remains Nako-owned.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn configuration_schema(
        fixture_provider_enabled: bool,
        default_language: impl Into<String>,
        default_limit: usize,
        max_limit: usize,
    ) -> AddonConfigurationSchema {
        AddonConfigurationSchema::new(
            CONFIG_SCHEMA_ID,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "providers": {
                        "type": "object",
                        "properties": {
                            "fixture": {
                                "type": "boolean",
                                "default": fixture_provider_enabled
                            }
                        },
                        "additionalProperties": false
                    },
                    "default_language": {
                        "type": "string",
                        "default": default_language.into()
                    },
                    "default_limit": {
                        "type": "integer",
                        "default": default_limit,
                        "minimum": 1,
                        "maximum": max_limit
                    },
                    "max_limit": {
                        "type": "integer",
                        "default": max_limit,
                        "minimum": 1,
                        "maximum": 200
                    }
                },
                "additionalProperties": false
            }),
        )
    }
}

pub mod dlna_renderer {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.dlna-renderer";
    pub const ADDON_NAME: &str = "Nako DLNA Renderer";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9150";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-dlna-renderer:9150";
    pub const RUNTIME_BINARY: &str = "nako-dlna-renderer";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-dlna-renderer:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako DLNA renderer adapter sidecar. The foundation release validates host-owned renderer command envelopes and returns plan-only results.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.dlna-renderer.config.v1";
    pub const RENDERER_ADAPTER_RESOURCE_PATH: &str = "/renderer-adapter";
    pub const RENDERER_ADAPTER_REQUEST_SCHEMA: &str = "nako.renderer-adapter.request.v1";
    pub const RENDERER_ADAPTER_RESPONSE_SCHEMA: &str = "nako.renderer-adapter.response.v1";
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "dlna-renderer-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "DLNA Renderer Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 1;

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_BASE_URL)
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL)
    }

    #[must_use]
    pub fn manifest(base_url: impl Into<String>) -> AddonManifest {
        manifest_with_version(ADDON_VERSION, base_url)
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::RendererAdapter,
                path: RENDERER_ADAPTER_RESOURCE_PATH.to_owned(),
                input_schema: Some(RENDERER_ADAPTER_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(RENDERER_ADAPTER_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![
                    AddonScope::RendererAdapterRead,
                    AddonScope::RendererAdapterControl,
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
                vec![AddonScope::RendererAdapterRead],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration::new(
                DIAGNOSTICS_HOSTED_PAGE_ID,
                DIAGNOSTICS_LABEL,
                DIAGNOSTICS_PATH,
                vec![AddonScope::RendererAdapterRead],
            )],
            configuration_schema: Some(configuration_schema()),
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::None,
            default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![
                AddonScope::RendererAdapterRead,
                AddonScope::RendererAdapterControl,
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
                format!(
                    "Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."
                ),
                "Run the plan-only sidecar on the same trusted LAN as DLNA renderers and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
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
                format!(
                    "Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/dlna-renderer` Dockerfile."
                ),
                "Run the plan-only sidecar on the same trusted LAN as DLNA renderers and register the resolved manifest through the existing Admin Addon APIs; live SSDP discovery and UPnP control are not implemented in this foundation release.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn configuration_schema() -> AddonConfigurationSchema {
        AddonConfigurationSchema::new(
            CONFIG_SCHEMA_ID,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "manual_devices": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["stable_device_id", "display_name", "host"],
                            "properties": {
                                "stable_device_id": { "type": "string" },
                                "display_name": { "type": "string" },
                                "host": { "type": "string" },
                                "port": {
                                    "type": "integer",
                                    "default": 8200,
                                    "minimum": 1,
                                    "maximum": 65535
                                },
                                "model": { "type": "string" }
                            },
                            "additionalProperties": false
                        },
                        "default": []
                    },
                    "plan_only": {
                        "type": "boolean",
                        "default": true,
                        "description": "Foundation release validates commands but does not perform live DLNA control."
                    }
                },
                "additionalProperties": false
            }),
        )
    }
}

pub mod chromecast_renderer {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.chromecast-renderer";
    pub const ADDON_NAME: &str = "Nako Chromecast Renderer";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9120";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-chromecast-renderer:9120";
    pub const RUNTIME_BINARY: &str = "nako-chromecast-renderer";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-chromecast-renderer:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako Chromecast renderer adapter sidecar. It discovers local Cast receivers and translates host-owned renderer command envelopes into Chromecast protocol actions.";
    pub const CONFIG_SCHEMA_ID: &str = "nako.official.chromecast-renderer.config.v1";
    pub const RENDERER_ADAPTER_RESOURCE_PATH: &str = "/renderer-adapter";
    pub const RENDERER_ADAPTER_REQUEST_SCHEMA: &str = "nako.renderer-adapter.request.v1";
    pub const RENDERER_ADAPTER_RESPONSE_SCHEMA: &str = "nako.renderer-adapter.response.v1";
    pub const DIAGNOSTICS_ENTRY_POINT_ID: &str = "chromecast-renderer-diagnostics";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "Chromecast Renderer Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_RECEIVER_APP_ID: &str = "CC1AD845";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 2;

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_BASE_URL, DEFAULT_RECEIVER_APP_ID)
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(
            ADDON_VERSION,
            DEFAULT_CONTAINER_BASE_URL,
            DEFAULT_RECEIVER_APP_ID,
        )
    }

    #[must_use]
    pub fn manifest(
        base_url: impl Into<String>,
        receiver_app_id: impl Into<String>,
    ) -> AddonManifest {
        manifest_with_version(ADDON_VERSION, base_url, receiver_app_id)
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
        receiver_app_id: impl Into<String>,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::RendererAdapter,
                path: RENDERER_ADAPTER_RESOURCE_PATH.to_owned(),
                input_schema: Some(RENDERER_ADAPTER_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(RENDERER_ADAPTER_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![
                    AddonScope::RendererAdapterRead,
                    AddonScope::RendererAdapterControl,
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
                vec![AddonScope::RendererAdapterRead],
            )],
            hosted_pages: vec![AddonHostedPageDeclaration {
                id: DIAGNOSTICS_HOSTED_PAGE_ID.to_owned(),
                title: DIAGNOSTICS_LABEL.to_owned(),
                path: DIAGNOSTICS_PATH.to_owned(),
                required_scopes: vec![AddonScope::RendererAdapterRead],
            }],
            configuration_schema: Some(configuration_schema(receiver_app_id)),
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::None,
            default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![
                AddonScope::RendererAdapterRead,
                AddonScope::RendererAdapterControl,
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
                "Run the sidecar on the same trusted LAN as Chromecast receivers and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
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
                format!("Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/chromecast-renderer` Dockerfile."),
                "Run the sidecar on the same trusted LAN as Chromecast receivers and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
            ],
        }
    }

    #[must_use]
    fn configuration_schema(receiver_app_id: impl Into<String>) -> AddonConfigurationSchema {
        AddonConfigurationSchema {
            schema_id: CONFIG_SCHEMA_ID.to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "receiver_app_id": {
                        "type": "string",
                        "default": receiver_app_id.into(),
                        "description": "Google Cast receiver application id. Defaults to the Cast Default Media Receiver."
                    },
                    "discovery_timeout_ms": {
                        "type": "integer",
                        "default": 3000,
                        "minimum": 250,
                        "maximum": 30000
                    },
                    "manual_devices": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["stable_device_id", "display_name", "host"],
                            "properties": {
                                "stable_device_id": { "type": "string" },
                                "display_name": { "type": "string" },
                                "host": { "type": "string" },
                                "port": {
                                    "type": "integer",
                                    "default": 8009,
                                    "minimum": 1,
                                    "maximum": 65535
                                }
                            },
                            "additionalProperties": false
                        },
                        "default": []
                    }
                },
                "additionalProperties": false,
            }),
        }
    }
}

pub mod notification_bridge {
    use super::*;

    pub const ADDON_ID: &str = "nako.official.notification-bridge";
    pub const ADDON_NAME: &str = "Nako Notification Bridge";
    pub const ADDON_VERSION: &str = "0.1.0-alpha.2";
    pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:9110";
    pub const DEFAULT_CONTAINER_BASE_URL: &str = "http://nako-notification-bridge:9110";
    pub const RUNTIME_BINARY: &str = "nako-notification-bridge";
    pub const RUNTIME_IMAGE: &str = "ghcr.io/latias94/nako-notification-bridge:0.1.0-alpha.2";
    pub const DESCRIPTION: &str = "Official Nako notification bridge sidecar. The first proof acknowledges scheduled Addon Events without provider fan-out.";
    pub const LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID: &str = "library-scanned-notification";
    pub const LIBRARY_SCANNED_EVENT_KIND: &str = "library.scanned";
    pub const LIBRARY_SCANNED_EVENT_PATH: &str = "/events/library-scanned";
    pub const WEBHOOK_RESOURCE_PATH: &str = "/events/library-scanned";
    pub const WEBHOOK_REQUEST_SCHEMA: &str = "nako.addon.event.library-scanned.request.v1";
    pub const WEBHOOK_RESPONSE_SCHEMA: &str =
        "nako.official.notification-bridge.library-scanned.event.v1";
    pub const DIAGNOSTICS_HOSTED_PAGE_ID: &str = "diagnostics";
    pub const DIAGNOSTICS_LABEL: &str = "Notification Bridge Diagnostics";
    pub const DIAGNOSTICS_PATH: &str = "/ui/diagnostics";
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 2;

    #[must_use]
    pub fn default_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_BASE_URL)
    }

    #[must_use]
    pub fn container_manifest() -> AddonManifest {
        manifest_with_version(ADDON_VERSION, DEFAULT_CONTAINER_BASE_URL)
    }

    #[must_use]
    pub fn manifest(base_url: impl Into<String>) -> AddonManifest {
        manifest_with_version(ADDON_VERSION, base_url)
    }

    #[must_use]
    pub fn manifest_with_version(
        version: impl Into<String>,
        base_url: impl Into<String>,
    ) -> AddonManifest {
        AddonManifest {
            id: ADDON_ID.to_owned(),
            name: ADDON_NAME.to_owned(),
            version: version.into(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: base_url.into(),
            description: Some(DESCRIPTION.to_owned()),
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::Webhook,
                path: WEBHOOK_RESOURCE_PATH.to_owned(),
                input_schema: Some(WEBHOOK_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(WEBHOOK_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![AddonScope::WebhookEventRead],
                timeout_ms: Some(DEFAULT_TIMEOUT_MS),
                max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            }],
            entry_points: Vec::new(),
            hosted_pages: vec![AddonHostedPageDeclaration {
                id: DIAGNOSTICS_HOSTED_PAGE_ID.to_owned(),
                title: DIAGNOSTICS_LABEL.to_owned(),
                path: DIAGNOSTICS_PATH.to_owned(),
                required_scopes: vec![AddonScope::WebhookEventRead],
            }],
            configuration_schema: None,
            secret_reference_fields: Vec::new(),
            event_subscriptions: vec![AddonEventSubscriptionDeclaration::new(
                LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID,
                LIBRARY_SCANNED_EVENT_KIND,
                LIBRARY_SCANNED_EVENT_PATH,
                vec![AddonScope::WebhookEventRead],
                serde_json::Value::Null,
            )],
            tasks: Vec::new(),
            auth: AddonAuth::None,
            default_timeout_ms: Some(DEFAULT_TIMEOUT_MS),
            default_max_attempts: Some(DEFAULT_MAX_ATTEMPTS),
            scopes: vec![AddonScope::WebhookEventRead],
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
                format!(
                    "Install from crates.io with `cargo install {RUNTIME_BINARY} --version {ADDON_VERSION} --locked`."
                ),
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
                format!(
                    "Run the official container image `{RUNTIME_IMAGE}` or build from the `addons/notification-bridge` Dockerfile."
                ),
                "Run the sidecar outside Nako and register the resolved manifest through the existing Admin Addon APIs.".to_owned(),
            ],
        }
    }
}

pub const OFFICIAL_ADDON_CATALOG_ARTIFACT_PATH: &str = "docs/addons/OFFICIAL_ADDON_CATALOG.md";
pub const COMPATIBLE_NAKO_VERSION_RANGE: &str = ">=0.1.0-alpha.2 <0.2.0";
pub const ADDON_HEALTH_CHECK_PATH: &str = "/health";
pub const OFFICIAL_ADDON_CATALOG_EXCLUDED_HELPERS: &[&str] = &["browser-worker"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfficialAddonCatalogEntry {
    pub manifest: AddonManifest,
    pub binary_install_descriptor: AddonInstallDescriptor,
    pub container_install_descriptor: AddonInstallDescriptor,
    pub compatible_nako_version_range: &'static str,
    pub health_check_path: &'static str,
    pub trust_tier: &'static str,
    pub smoke_status: &'static str,
    pub install_docs: &'static str,
    pub compose_reference: Option<&'static str>,
    pub local_smoke_reference: &'static str,
    pub live_smoke_reference: Option<&'static str>,
}

impl OfficialAddonCatalogEntry {
    #[must_use]
    pub fn addon_id(&self) -> &str {
        &self.manifest.id
    }
}

#[must_use]
pub fn official_addon_catalog() -> Vec<OfficialAddonCatalogEntry> {
    vec![
        catalog_entry(
            metadata_scraper::binary_install_descriptor(),
            metadata_scraper::container_install_descriptor(),
            "official side-effect",
            "local smoke; Nako-mediated alpha smoke",
            "../nako-official-addons/addons/metadata-scraper/README.md",
            Some("../nako-official-addons/addons/metadata-scraper/compose.example.yml"),
            "../nako-official-addons/addons/metadata-scraper/smoke.local.ps1",
            None,
        ),
        catalog_entry(
            resource_search::binary_install_descriptor(),
            resource_search::container_install_descriptor(),
            "official read-only",
            "local smoke",
            "../nako-official-addons/addons/resource-search/README.md",
            Some("../nako-official-addons/addons/resource-search/compose.example.yml"),
            "../nako-official-addons/addons/resource-search/smoke.local.ps1",
            None,
        ),
        catalog_entry(
            subtitle_provider::binary_install_descriptor(),
            subtitle_provider::container_install_descriptor(),
            "official read-only",
            "local smoke",
            "../nako-official-addons/addons/subtitle-provider/README.md",
            None,
            "../nako-official-addons/addons/subtitle-provider/smoke.local.ps1",
            None,
        ),
        catalog_entry(
            chromecast_renderer::binary_install_descriptor(),
            chromecast_renderer::container_install_descriptor(),
            "renderer adapter",
            "local smoke",
            "../nako-official-addons/addons/chromecast-renderer/README.md",
            Some("../nako-official-addons/addons/chromecast-renderer/compose.example.yml"),
            "../nako-official-addons/addons/chromecast-renderer/smoke.local.ps1",
            None,
        ),
        catalog_entry(
            dlna_renderer::binary_install_descriptor(),
            dlna_renderer::container_install_descriptor(),
            "renderer adapter",
            "local smoke",
            "../nako-official-addons/addons/dlna-renderer/README.md",
            Some("../nako-official-addons/addons/dlna-renderer/compose.example.yml"),
            "../nako-official-addons/addons/dlna-renderer/smoke.local.ps1",
            None,
        ),
        catalog_entry(
            notification_bridge::binary_install_descriptor(),
            notification_bridge::container_install_descriptor(),
            "notification fan-out",
            "local smoke; optional live smoke",
            "../nako-official-addons/addons/notification-bridge/README.md",
            Some("../nako-official-addons/addons/notification-bridge/compose.example.yml"),
            "../nako-official-addons/addons/notification-bridge/smoke.local.ps1",
            Some("../nako-official-addons/addons/notification-bridge/smoke.live.ps1"),
        ),
        catalog_entry(
            external_acquisition_runner::binary_install_descriptor(),
            external_acquisition_runner::container_install_descriptor(),
            "official side-effect",
            "local smoke",
            "../nako-official-addons/addons/external-acquisition-runner/README.md",
            Some("../nako-official-addons/addons/external-acquisition-runner/compose.example.yml"),
            "../nako-official-addons/addons/external-acquisition-runner/smoke.local.ps1",
            None,
        ),
    ]
}

#[must_use]
pub fn render_official_addon_catalog_markdown() -> String {
    let mut markdown = String::from(
        "# Official Addon Catalog\n\n\
         This generated catalog is the operator-visible inventory for official Nako Addons. \
         It is derived from `crates/nako-official-addon-catalog` manifest and install descriptor \
         builders, then verified by crate tests.\n\n\
         Catalog scope is discovery, compatibility, install references, and smoke status. \
         It is not an Addon Manager: Nako still does not install, update, start, stop, remove, \
         log, or supervise Addon Sidecar processes.\n\n",
    );
    markdown.push_str(&format!(
        "- Catalog artifact: `{OFFICIAL_ADDON_CATALOG_ARTIFACT_PATH}`\n"
    ));
    markdown.push_str(&format!(
        "- Compatible Nako version range: `{COMPATIBLE_NAKO_VERSION_RANGE}`\n"
    ));
    markdown.push_str(&format!(
        "- Excluded helper surfaces: `{}` is a browser-render helper, not an Addon catalog entry.\n\n",
        OFFICIAL_ADDON_CATALOG_EXCLUDED_HELPERS.join("`, `")
    ));
    markdown.push_str(
        "| Addon | Addon Version | Addon Protocol Version | Compatible Nako | Runtime | Default Base URLs | Health Check | Capabilities | Required Scopes | Trust Tier | Smoke Status | Install References |\n",
    );
    markdown
        .push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for entry in official_addon_catalog() {
        let manifest = &entry.manifest;
        let binary = entry
            .binary_install_descriptor
            .runtime
            .binary
            .as_deref()
            .unwrap_or("none");
        let image = entry
            .container_install_descriptor
            .runtime
            .image
            .as_deref()
            .unwrap_or("none");
        let container_base_url = &entry.container_install_descriptor.manifest.base_url;

        markdown.push_str(&format!(
            "| `{}`<br>{} | `{}` | `{}` | `{}` | binary `{}`<br>image `{}` | local `{}`<br>container `{}` | `POST {}` | resources {}<br>tasks {}<br>events {}<br>hosted pages {} | {} | {} | {} | docs `{}`<br>compose {}<br>smoke `{}`{} |\n",
            manifest.id,
            manifest.name,
            manifest.version,
            manifest.protocol_version,
            entry.compatible_nako_version_range,
            binary,
            image,
            manifest.base_url,
            container_base_url,
            entry.health_check_path,
            resource_summary(manifest),
            task_summary(manifest),
            event_summary(manifest),
            hosted_page_summary(manifest),
            scope_summary(manifest),
            entry.trust_tier,
            entry.smoke_status,
            entry.install_docs,
            optional_reference(entry.compose_reference),
            entry.local_smoke_reference,
            optional_live_smoke(entry.live_smoke_reference),
        ));
    }

    markdown.push_str(
        "\n## Validation\n\n\
         Run `cargo nextest run -p nako-official-addon-catalog --no-fail-fast` to verify that every \
         catalog entry validates its manifest and binary/container install descriptors, and that this \
         artifact matches the crate renderer.\n",
    );
    markdown
}

#[must_use]
fn catalog_entry(
    binary_install_descriptor: AddonInstallDescriptor,
    container_install_descriptor: AddonInstallDescriptor,
    trust_tier: &'static str,
    smoke_status: &'static str,
    install_docs: &'static str,
    compose_reference: Option<&'static str>,
    local_smoke_reference: &'static str,
    live_smoke_reference: Option<&'static str>,
) -> OfficialAddonCatalogEntry {
    OfficialAddonCatalogEntry {
        manifest: binary_install_descriptor.manifest.clone(),
        binary_install_descriptor,
        container_install_descriptor,
        compatible_nako_version_range: COMPATIBLE_NAKO_VERSION_RANGE,
        health_check_path: ADDON_HEALTH_CHECK_PATH,
        trust_tier,
        smoke_status,
        install_docs,
        compose_reference,
        local_smoke_reference,
        live_smoke_reference,
    }
}

#[must_use]
fn resource_summary(manifest: &AddonManifest) -> String {
    if manifest.resources.is_empty() {
        return "none".to_owned();
    }

    manifest
        .resources
        .iter()
        .map(|resource| resource.kind.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
fn task_summary(manifest: &AddonManifest) -> String {
    if manifest.tasks.is_empty() {
        return "none".to_owned();
    }

    manifest
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
fn event_summary(manifest: &AddonManifest) -> String {
    if manifest.event_subscriptions.is_empty() {
        return "none".to_owned();
    }

    manifest
        .event_subscriptions
        .iter()
        .map(|event| event.event_kind.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
fn hosted_page_summary(manifest: &AddonManifest) -> String {
    if manifest.hosted_pages.is_empty() {
        return "none".to_owned();
    }

    manifest
        .hosted_pages
        .iter()
        .map(|page| page.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
fn scope_summary(manifest: &AddonManifest) -> String {
    manifest
        .scopes
        .iter()
        .map(|scope| format!("`{}`", scope.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
fn optional_reference(reference: Option<&str>) -> String {
    reference
        .map(|reference| format!("`{reference}`"))
        .unwrap_or_else(|| "none".to_owned())
}

#[must_use]
fn optional_live_smoke(reference: Option<&str>) -> String {
    reference
        .map(|reference| format!("<br>live smoke `{reference}`"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{
        AddonRuntimeReferenceKind, addon_install_guide, validate_install_descriptor,
        validate_manifest,
    };

    use super::chromecast_renderer;
    use super::dlna_renderer;
    use super::external_acquisition_runner;
    use super::metadata_scraper::*;
    use super::notification_bridge;
    use super::official_addon_catalog;
    use super::render_official_addon_catalog_markdown;
    use super::resource_search;
    use super::subtitle_provider;
    use super::{COMPATIBLE_NAKO_VERSION_RANGE, OFFICIAL_ADDON_CATALOG_EXCLUDED_HELPERS};

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

    #[test]
    fn notification_bridge_default_manifest_matches_official_catalog_facts() {
        let manifest = notification_bridge::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, notification_bridge::ADDON_ID);
        assert_eq!(manifest.version, notification_bridge::ADDON_VERSION);
        assert_eq!(manifest.base_url, notification_bridge::DEFAULT_BASE_URL);
        assert_eq!(
            manifest.scopes,
            vec![nako_addon_protocol::AddonScope::WebhookEventRead]
        );
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::Webhook
        );
        assert_eq!(
            manifest.resources[0].path,
            notification_bridge::WEBHOOK_RESOURCE_PATH
        );
        assert!(manifest.tasks.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
        assert_eq!(manifest.event_subscriptions.len(), 1);
        assert_eq!(
            manifest.event_subscriptions[0].id,
            notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
        );
        assert_eq!(
            manifest.event_subscriptions[0].event_kind,
            notification_bridge::LIBRARY_SCANNED_EVENT_KIND
        );
        assert_eq!(
            manifest.event_subscriptions[0].path,
            notification_bridge::LIBRARY_SCANNED_EVENT_PATH
        );
    }

    #[test]
    fn resource_search_default_manifest_matches_official_catalog_facts() {
        let manifest = resource_search::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, resource_search::ADDON_ID);
        assert_eq!(manifest.version, resource_search::ADDON_VERSION);
        assert_eq!(manifest.base_url, resource_search::DEFAULT_BASE_URL);
        assert_eq!(
            manifest.scopes,
            vec![
                nako_addon_protocol::AddonScope::AcquisitionSearchRead,
                nako_addon_protocol::AddonScope::AcquisitionLinkCheckRead,
            ]
        );
        assert_eq!(manifest.resources.len(), 2);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::ResourceSearch
        );
        assert_eq!(
            manifest.resources[0].path,
            resource_search::RESOURCE_SEARCH_RESOURCE_PATH
        );
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(nako_addon_protocol::ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(nako_addon_protocol::ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.resources[1].kind,
            nako_addon_protocol::AddonResource::ResourceLinkCheck
        );
        assert_eq!(
            manifest.resources[1].path,
            resource_search::RESOURCE_LINK_CHECK_RESOURCE_PATH
        );
        assert_eq!(
            manifest.resources[1].input_schema.as_deref(),
            Some(nako_addon_protocol::ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[1].output_schema.as_deref(),
            Some(nako_addon_protocol::ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.entry_points[0].id,
            resource_search::DIAGNOSTICS_ENTRY_POINT_ID
        );
        assert_eq!(
            manifest.hosted_pages[0].id,
            resource_search::DIAGNOSTICS_HOSTED_PAGE_ID
        );
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fixture"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["providers"]["properties"]["pansou_compatible"]["default"],
            false
        );
        assert_eq!(schema["properties"]["pansou"]["type"], "object");
        assert_eq!(
            schema["properties"]["default_limit"]["default"],
            resource_search::DEFAULT_LIMIT
        );
        assert_eq!(
            schema["properties"]["max_limit"]["default"],
            resource_search::DEFAULT_MAX_LIMIT
        );
        assert_eq!(
            schema["properties"]["search_timeout_ms"]["default"],
            resource_search::DEFAULT_TIMEOUT_MS
        );
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn external_acquisition_runner_default_manifest_matches_official_catalog_facts() {
        let manifest = external_acquisition_runner::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, external_acquisition_runner::ADDON_ID);
        assert_eq!(manifest.version, external_acquisition_runner::ADDON_VERSION);
        assert_eq!(
            manifest.base_url,
            external_acquisition_runner::DEFAULT_BASE_URL
        );
        assert_eq!(
            manifest.scopes,
            vec![nako_addon_protocol::AddonScope::AcquisitionActionRun]
        );
        assert!(manifest.resources.is_empty());
        assert_eq!(manifest.tasks.len(), 1);
        assert_eq!(
            manifest.tasks[0].id,
            external_acquisition_runner::ACTION_TASK_ID
        );
        assert_eq!(
            manifest.tasks[0].path,
            external_acquisition_runner::ACTION_TASK_PATH
        );
        assert_eq!(
            manifest.tasks[0].input_schema.as_deref(),
            Some(external_acquisition_runner::ACTION_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.tasks[0].output_schema.as_deref(),
            Some(external_acquisition_runner::ACTION_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.tasks[0].required_scopes,
            vec![nako_addon_protocol::AddonScope::AcquisitionActionRun]
        );
        assert_eq!(
            manifest.entry_points[0].id,
            external_acquisition_runner::DIAGNOSTICS_ENTRY_POINT_ID
        );
        assert_eq!(
            manifest.hosted_pages[0].id,
            external_acquisition_runner::DIAGNOSTICS_HOSTED_PAGE_ID
        );
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(
            schema["properties"]["default_runner_profile_id"]["default"],
            external_acquisition_runner::DEFAULT_RUNNER_PROFILE_ID
        );
        assert_eq!(
            schema["properties"]["profiles"]["properties"]["fixture"]["properties"]["mode"]["default"],
            "noop"
        );
        assert_eq!(
            schema["properties"]["profiles"]["properties"]["transmission"]["properties"]["enabled"]
                ["default"],
            false
        );
        assert_eq!(
            schema["properties"]["profiles"]["properties"]["transmission"]["properties"]["rpc_url"]
                ["default"],
            external_acquisition_runner::TRANSMISSION_DEFAULT_RPC_URL
        );
        assert_eq!(
            schema["properties"]["profiles"]["properties"]["transmission"]["properties"]["timeout_ms"]
                ["default"],
            external_acquisition_runner::TRANSMISSION_DEFAULT_TIMEOUT_MS
        );
        assert!(manifest.event_subscriptions.is_empty());
        assert_eq!(manifest.secret_reference_fields.len(), 1);
        assert_eq!(
            manifest.secret_reference_fields[0].id,
            external_acquisition_runner::TRANSMISSION_PASSWORD_SECRET_FIELD_ID
        );
        assert!(!manifest.secret_reference_fields[0].required);
    }

    #[test]
    fn chromecast_renderer_default_manifest_matches_official_catalog_facts() {
        let manifest = chromecast_renderer::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, chromecast_renderer::ADDON_ID);
        assert_eq!(manifest.version, chromecast_renderer::ADDON_VERSION);
        assert_eq!(manifest.base_url, chromecast_renderer::DEFAULT_BASE_URL);
        assert_eq!(
            manifest.scopes,
            vec![
                nako_addon_protocol::AddonScope::RendererAdapterRead,
                nako_addon_protocol::AddonScope::RendererAdapterControl,
            ]
        );
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::RendererAdapter
        );
        assert_eq!(
            manifest.resources[0].path,
            chromecast_renderer::RENDERER_ADAPTER_RESOURCE_PATH
        );
        assert_eq!(
            manifest.entry_points[0].id,
            chromecast_renderer::DIAGNOSTICS_ENTRY_POINT_ID
        );
        assert_eq!(
            manifest.hosted_pages[0].id,
            chromecast_renderer::DIAGNOSTICS_HOSTED_PAGE_ID
        );
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn subtitle_provider_default_manifest_matches_official_catalog_facts() {
        let manifest = subtitle_provider::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, subtitle_provider::ADDON_ID);
        assert_eq!(manifest.version, subtitle_provider::ADDON_VERSION);
        assert_eq!(manifest.base_url, subtitle_provider::DEFAULT_BASE_URL);
        assert_eq!(
            manifest.scopes,
            vec![nako_addon_protocol::AddonScope::SubtitleRead]
        );
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::Subtitle
        );
        assert_eq!(
            manifest.resources[0].path,
            subtitle_provider::SUBTITLE_RESOURCE_PATH
        );
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(subtitle_provider::SUBTITLE_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(subtitle_provider::SUBTITLE_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.entry_points[0].id,
            subtitle_provider::DIAGNOSTICS_ENTRY_POINT_ID
        );
        assert_eq!(
            manifest.hosted_pages[0].id,
            subtitle_provider::DIAGNOSTICS_HOSTED_PAGE_ID
        );
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(
            schema["properties"]["providers"]["properties"]["fixture"]["default"],
            true
        );
        assert_eq!(
            schema["properties"]["default_language"]["default"],
            subtitle_provider::DEFAULT_LANGUAGE
        );
        assert_eq!(
            schema["properties"]["default_limit"]["default"],
            subtitle_provider::DEFAULT_LIMIT
        );
        assert_eq!(
            schema["properties"]["max_limit"]["default"],
            subtitle_provider::DEFAULT_MAX_LIMIT
        );
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn dlna_renderer_default_manifest_matches_official_catalog_facts() {
        let manifest = dlna_renderer::default_manifest();

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.id, dlna_renderer::ADDON_ID);
        assert_eq!(manifest.version, dlna_renderer::ADDON_VERSION);
        assert_eq!(manifest.base_url, dlna_renderer::DEFAULT_BASE_URL);
        assert_eq!(
            manifest.scopes,
            vec![
                nako_addon_protocol::AddonScope::RendererAdapterRead,
                nako_addon_protocol::AddonScope::RendererAdapterControl,
            ]
        );
        assert_eq!(manifest.resources.len(), 1);
        assert_eq!(
            manifest.resources[0].kind,
            nako_addon_protocol::AddonResource::RendererAdapter
        );
        assert_eq!(
            manifest.resources[0].path,
            dlna_renderer::RENDERER_ADAPTER_RESOURCE_PATH
        );
        assert_eq!(
            manifest.resources[0].input_schema.as_deref(),
            Some(dlna_renderer::RENDERER_ADAPTER_REQUEST_SCHEMA)
        );
        assert_eq!(
            manifest.resources[0].output_schema.as_deref(),
            Some(dlna_renderer::RENDERER_ADAPTER_RESPONSE_SCHEMA)
        );
        assert_eq!(
            manifest.entry_points[0].id,
            dlna_renderer::DIAGNOSTICS_ENTRY_POINT_ID
        );
        assert_eq!(
            manifest.hosted_pages[0].id,
            dlna_renderer::DIAGNOSTICS_HOSTED_PAGE_ID
        );
        let schema = &manifest.configuration_schema.as_ref().unwrap().schema;
        assert_eq!(schema["properties"]["manual_devices"]["type"], "array");
        assert_eq!(schema["properties"]["plan_only"]["default"], true);
        assert!(manifest.tasks.is_empty());
        assert!(manifest.event_subscriptions.is_empty());
        assert!(manifest.secret_reference_fields.is_empty());
    }

    #[test]
    fn chromecast_renderer_container_descriptor_matches_renderer_adapter_shape() {
        let descriptor = chromecast_renderer::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            chromecast_renderer::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(
            guide.runtime_reference.value,
            chromecast_renderer::RUNTIME_IMAGE
        );
        assert_eq!(
            guide.declared_resources,
            vec![nako_addon_protocol::AddonResource::RendererAdapter]
        );
        assert_eq!(guide.task_count, 0);
        assert_eq!(guide.event_subscription_count, 0);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn subtitle_provider_container_descriptor_matches_read_only_shape() {
        let descriptor = subtitle_provider::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            subtitle_provider::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(
            guide.runtime_reference.value,
            subtitle_provider::RUNTIME_IMAGE
        );
        assert_eq!(
            guide.declared_resources,
            vec![nako_addon_protocol::AddonResource::Subtitle]
        );
        assert_eq!(guide.task_count, 0);
        assert_eq!(guide.event_subscription_count, 0);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn dlna_renderer_container_descriptor_matches_plan_only_shape() {
        let descriptor = dlna_renderer::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            dlna_renderer::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(guide.runtime_reference.value, dlna_renderer::RUNTIME_IMAGE);
        assert_eq!(
            guide.declared_resources,
            vec![nako_addon_protocol::AddonResource::RendererAdapter]
        );
        assert_eq!(guide.task_count, 0);
        assert_eq!(guide.event_subscription_count, 0);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn notification_bridge_container_descriptor_matches_ack_only_shape() {
        let descriptor = notification_bridge::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            notification_bridge::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(
            guide.runtime_reference.value,
            notification_bridge::RUNTIME_IMAGE
        );
        assert_eq!(guide.task_count, 0);
        assert_eq!(guide.event_subscription_count, 1);
        assert_eq!(guide.entry_point_count, 0);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn resource_search_container_descriptor_matches_search_and_link_check_shape() {
        let descriptor = resource_search::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            resource_search::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(
            guide.runtime_reference.value,
            resource_search::RUNTIME_IMAGE
        );
        assert_eq!(
            guide.declared_resources,
            vec![
                nako_addon_protocol::AddonResource::ResourceSearch,
                nako_addon_protocol::AddonResource::ResourceLinkCheck,
            ]
        );
        assert_eq!(guide.task_count, 0);
        assert_eq!(guide.event_subscription_count, 0);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn external_acquisition_runner_container_descriptor_matches_action_task_shape() {
        let descriptor = external_acquisition_runner::container_install_descriptor();
        nako_addon_protocol::validate_install_descriptor(&descriptor).unwrap();

        let guide = addon_install_guide(&descriptor);
        assert_eq!(
            descriptor.manifest.base_url,
            external_acquisition_runner::DEFAULT_CONTAINER_BASE_URL
        );
        assert_eq!(
            guide.runtime_reference.kind,
            AddonRuntimeReferenceKind::Image
        );
        assert_eq!(
            guide.runtime_reference.value,
            external_acquisition_runner::RUNTIME_IMAGE
        );
        assert!(guide.declared_resources.is_empty());
        assert_eq!(guide.task_count, 1);
        assert_eq!(
            descriptor.manifest.tasks[0].id,
            external_acquisition_runner::ACTION_TASK_ID
        );
        assert_eq!(
            descriptor.manifest.tasks[0].input_schema.as_deref(),
            Some(external_acquisition_runner::ACTION_REQUEST_SCHEMA)
        );
        assert_eq!(
            descriptor.manifest.tasks[0].output_schema.as_deref(),
            Some(external_acquisition_runner::ACTION_RESPONSE_SCHEMA)
        );
        assert_eq!(guide.event_subscription_count, 0);
        assert_eq!(guide.entry_point_count, 1);
        assert_eq!(guide.hosted_page_count, 1);
    }

    #[test]
    fn official_catalog_contains_every_addon_and_excludes_helpers() {
        let catalog = official_addon_catalog();
        let addon_ids = catalog
            .iter()
            .map(|entry| entry.addon_id())
            .collect::<Vec<_>>();

        assert_eq!(
            addon_ids,
            vec![
                ADDON_ID,
                resource_search::ADDON_ID,
                subtitle_provider::ADDON_ID,
                chromecast_renderer::ADDON_ID,
                dlna_renderer::ADDON_ID,
                notification_bridge::ADDON_ID,
                external_acquisition_runner::ADDON_ID,
            ]
        );
        assert!(!addon_ids.contains(&"browser-worker"));
        assert_eq!(OFFICIAL_ADDON_CATALOG_EXCLUDED_HELPERS, &["browser-worker"]);
    }

    #[test]
    fn official_catalog_entries_validate_manifest_and_install_descriptors() {
        for entry in official_addon_catalog() {
            validate_manifest(&entry.manifest).unwrap();
            validate_install_descriptor(&entry.binary_install_descriptor).unwrap();
            validate_install_descriptor(&entry.container_install_descriptor).unwrap();

            assert_eq!(
                entry.compatible_nako_version_range,
                COMPATIBLE_NAKO_VERSION_RANGE
            );
            assert_eq!(entry.health_check_path, super::ADDON_HEALTH_CHECK_PATH);
            assert_eq!(
                entry.manifest.protocol_version,
                nako_addon_protocol::ADDON_PROTOCOL_VERSION
            );
            assert_eq!(
                entry.manifest.id,
                entry.binary_install_descriptor.manifest.id
            );
            assert_eq!(
                entry.manifest.id,
                entry.container_install_descriptor.manifest.id
            );
            assert!(!entry.trust_tier.is_empty());
            assert!(!entry.smoke_status.is_empty());
            assert!(!entry.install_docs.is_empty());
            assert!(!entry.local_smoke_reference.is_empty());
        }
    }

    #[test]
    fn official_catalog_artifact_matches_renderer() {
        let artifact = include_str!("../../../docs/addons/OFFICIAL_ADDON_CATALOG.md");

        assert_eq!(artifact, render_official_addon_catalog_markdown());
    }
}
