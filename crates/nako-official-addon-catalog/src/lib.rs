use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA,
    ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
    ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA, AddonAuth, AddonConfigurationSchema,
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

#[cfg(test)]
mod tests {
    use nako_addon_protocol::{AddonRuntimeReferenceKind, addon_install_guide, validate_manifest};

    use super::chromecast_renderer;
    use super::metadata_scraper::*;
    use super::notification_bridge;
    use super::resource_search;

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
}
