use std::{
    collections::{BTreeMap, HashSet},
    fmt,
};

use serde::{Deserialize, Serialize};

pub const ADDON_PROTOCOL_VERSION: &str = "0.1.0-alpha.1";
pub const SUPPORTED_ADDON_PROTOCOL_VERSIONS: &[&str] = &[ADDON_PROTOCOL_VERSION];
pub const ADDON_RUNTIME_ACCESS_CHECK_PATH: &str = "/addon/v1/access-check";
pub const ADDON_RUNTIME_SIDE_EFFECTS_PATH: &str = "/addon/v1/side-effects";
pub const ADDON_RUNTIME_GENERATED_ARTIFACTS_PATH: &str = "/addon/v1/generated-artifacts";
pub const ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH: &str =
    "/addon/v1/acquisition/intake/candidates";
pub const ADDON_RUNTIME_TASK_RUN_CLAIM_PATH: &str = "/addon/v1/task-runs/claim";
pub const ADDON_RUNTIME_TASK_RUN_PROGRESS_PATH: &str = "/addon/v1/task-runs/progress";
pub const ADDON_RUNTIME_TASK_RUN_COMPLETE_PATH: &str = "/addon/v1/task-runs/complete";
pub const ADDON_RUNTIME_TASK_RUN_FAIL_PATH: &str = "/addon/v1/task-runs/fail";
pub const ADDON_RUNTIME_TASK_RUN_CANCEL_PATH: &str = "/addon/v1/task-runs/cancel";
pub const ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA: &str = "nako.addon.resource_search.request.v1";
pub const ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA: &str = "nako.addon.resource_search.response.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddonRuntimeRoute {
    pub path: &'static str,
    pub method: AddonRuntimeHttpMethod,
    pub kind: AddonRuntimeRouteKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddonRuntimeHttpMethod {
    Post,
}

impl AddonRuntimeHttpMethod {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Post => "POST",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddonRuntimeRouteKind {
    AccessCheck,
    SideEffect,
    GeneratedArtifact,
    AcquisitionIntakeCandidate,
    TaskRunClaim,
    TaskRunProgress,
    TaskRunComplete,
    TaskRunFail,
    TaskRunCancel,
}

pub const ADDON_RUNTIME_ROUTES: &[AddonRuntimeRoute] = &[
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_ACCESS_CHECK_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::AccessCheck,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_SIDE_EFFECTS_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::SideEffect,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_GENERATED_ARTIFACTS_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::GeneratedArtifact,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::AcquisitionIntakeCandidate,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_TASK_RUN_CLAIM_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::TaskRunClaim,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_TASK_RUN_PROGRESS_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::TaskRunProgress,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_TASK_RUN_COMPLETE_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::TaskRunComplete,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_TASK_RUN_FAIL_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::TaskRunFail,
    },
    AddonRuntimeRoute {
        path: ADDON_RUNTIME_TASK_RUN_CANCEL_PATH,
        method: AddonRuntimeHttpMethod::Post,
        kind: AddonRuntimeRouteKind::TaskRunCancel,
    },
];

#[must_use]
pub fn addon_runtime_paths() -> impl ExactSizeIterator<Item = &'static str> {
    ADDON_RUNTIME_ROUTES.iter().map(|route| route.path)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub base_url: String,
    pub description: Option<String>,
    #[serde(default)]
    pub resources: Vec<AddonResourceDeclaration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry_points: Vec<AddonEntryPointDeclaration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosted_pages: Vec<AddonHostedPageDeclaration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_schema: Option<AddonConfigurationSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_reference_fields: Vec<AddonSecretReferenceFieldDeclaration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_subscriptions: Vec<AddonEventSubscriptionDeclaration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<AddonTaskDeclaration>,
    pub auth: AddonAuth,
    #[serde(default)]
    pub default_timeout_ms: Option<u64>,
    #[serde(default)]
    pub default_max_attempts: Option<u32>,
    #[serde(default)]
    pub scopes: Vec<AddonScope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonInstallDescriptor {
    pub manifest: AddonManifest,
    pub runtime: AddonRuntimeRequirement,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_reference_bindings: Vec<AddonSecretReferenceBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub install_notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRuntimeRequirement {
    pub kind: AddonRuntimeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRuntimeKind {
    HttpSidecar,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSecretReferenceBinding {
    pub field_id: String,
    pub secret_ref: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonInstallGuide {
    pub manifest_id: String,
    pub addon_name: String,
    pub protocol_version: String,
    pub runtime_kind: AddonRuntimeKind,
    pub runtime_reference: AddonRuntimeReference,
    pub base_url_scheme: String,
    pub base_url_configured: bool,
    pub declared_resources: Vec<AddonResource>,
    pub declared_scopes: Vec<AddonScope>,
    pub required_secret_fields: Vec<AddonInstallSecretField>,
    pub provided_secret_refs: Vec<String>,
    pub missing_required_secret_fields: Vec<String>,
    pub has_configuration_schema: bool,
    pub entry_point_count: u32,
    pub hosted_page_count: u32,
    pub task_count: u32,
    pub event_subscription_count: u32,
    pub install_steps: Vec<AddonInstallStep>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRuntimeReference {
    pub kind: AddonRuntimeReferenceKind,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRuntimeReferenceKind {
    Image,
    Binary,
    Command,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonInstallSecretField {
    pub id: String,
    pub label: String,
    pub required: bool,
    pub provided: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonInstallStep {
    pub kind: AddonInstallStepKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonInstallStepKind {
    RunSidecar,
    ConfigureSecretReference,
    RegisterManifest,
    GrantScopes,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceDeclaration {
    pub kind: AddonResource,
    pub path: String,
    #[serde(default)]
    pub input_schema: Option<String>,
    #[serde(default)]
    pub output_schema: Option<String>,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_attempts: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResource {
    Catalog,
    Metadata,
    Image,
    Stream,
    Subtitle,
    Recommendation,
    Automation,
    Webhook,
    RendererAdapter,
    ResourceSearch,
}

impl AddonResource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Metadata => "metadata",
            Self::Image => "image",
            Self::Stream => "stream",
            Self::Subtitle => "subtitle",
            Self::Recommendation => "recommendation",
            Self::Automation => "automation",
            Self::Webhook => "webhook",
            Self::RendererAdapter => "renderer_adapter",
            Self::ResourceSearch => "resource_search",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEntryPointDeclaration {
    pub id: String,
    pub kind: AddonEntryPointKind,
    pub label: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosted_page_id: Option<String>,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
}

impl AddonEntryPointDeclaration {
    #[must_use]
    pub fn hosted_page(
        id: impl Into<String>,
        kind: AddonEntryPointKind,
        label: impl Into<String>,
        path: impl Into<String>,
        hosted_page_id: impl Into<String>,
        required_scopes: Vec<AddonScope>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            path: path.into(),
            hosted_page_id: Some(hosted_page_id.into()),
            required_scopes,
        }
    }

    #[must_use]
    pub fn action(
        id: impl Into<String>,
        kind: AddonEntryPointKind,
        label: impl Into<String>,
        path: impl Into<String>,
        required_scopes: Vec<AddonScope>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            path: path.into(),
            hosted_page_id: None,
            required_scopes,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonEntryPointKind {
    ItemAction,
    LibraryAction,
    AdminAction,
    Settings,
    Diagnostics,
    TaskLauncher,
}

impl AddonEntryPointKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ItemAction => "item_action",
            Self::LibraryAction => "library_action",
            Self::AdminAction => "admin_action",
            Self::Settings => "settings",
            Self::Diagnostics => "diagnostics",
            Self::TaskLauncher => "task_launcher",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonHostedPageDeclaration {
    pub id: String,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
}

impl AddonHostedPageDeclaration {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        path: impl Into<String>,
        required_scopes: Vec<AddonScope>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            path: path.into(),
            required_scopes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonConfigurationSchema {
    pub schema_id: String,
    pub schema: serde_json::Value,
}

impl AddonConfigurationSchema {
    #[must_use]
    pub fn new(schema_id: impl Into<String>, schema: serde_json::Value) -> Self {
        Self {
            schema_id: schema_id.into(),
            schema,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSecretReferenceFieldDeclaration {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

impl AddonSecretReferenceFieldDeclaration {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: Option<String>,
        required: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description,
            required,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventSubscriptionDeclaration {
    pub id: String,
    pub event_kind: String,
    pub path: String,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub filters: serde_json::Value,
}

impl AddonEventSubscriptionDeclaration {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        event_kind: impl Into<String>,
        path: impl Into<String>,
        required_scopes: Vec<AddonScope>,
        filters: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            event_kind: event_kind.into(),
            path: path.into(),
            required_scopes,
            filters,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskDeclaration {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_attempts: Option<u32>,
}

impl AddonTaskDeclaration {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
        required_scopes: Vec<AddonScope>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            description: None,
            required_scopes,
            timeout_ms: None,
            max_attempts: None,
        }
    }

    #[must_use]
    pub const fn with_execution_bounds(
        mut self,
        timeout_ms: Option<u64>,
        max_attempts: Option<u32>,
    ) -> Self {
        self.timeout_ms = timeout_ms;
        self.max_attempts = max_attempts;
        self
    }

    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonScope {
    CatalogRead,
    ItemMetadataRead,
    ItemMetadataSuggest,
    ImageRead,
    SubtitleRead,
    StreamUrlRead,
    RecommendationWrite,
    AutomationRun,
    WebhookEventRead,
    RendererAdapterRead,
    RendererAdapterControl,
    AcquisitionSearchRead,
}

impl AddonScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CatalogRead => "catalog_read",
            Self::ItemMetadataRead => "item_metadata_read",
            Self::ItemMetadataSuggest => "item_metadata_suggest",
            Self::ImageRead => "image_read",
            Self::SubtitleRead => "subtitle_read",
            Self::StreamUrlRead => "stream_url_read",
            Self::RecommendationWrite => "recommendation_write",
            Self::AutomationRun => "automation_run",
            Self::WebhookEventRead => "webhook_event_read",
            Self::RendererAdapterRead => "renderer_adapter_read",
            Self::RendererAdapterControl => "renderer_adapter_control",
            Self::AcquisitionSearchRead => "acquisition_search_read",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonAuth {
    #[default]
    None,
    Bearer,
    SharedSecret,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceRequest {
    pub protocol_version: String,
    pub addon_id: String,
    pub resource: AddonResource,
    pub request_id: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceResponse {
    pub protocol_version: String,
    pub addon_id: String,
    pub resource: AddonResource,
    pub request_id: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub artifacts: Vec<AddonArtifact>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceSearchRequest {
    pub schema: String,
    pub intent: AddonResourceSearchIntent,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_types: Vec<AddonResourceLinkType>,
    #[serde(default)]
    pub refresh: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub context: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AddonResourceSearchIntent {
    FreeText {
        text: String,
    },
    MediaTitle {
        title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        year: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_kind: Option<String>,
    },
    ExternalId {
        id_kind: String,
        value: String,
    },
    ExactLink {
        url: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceSearchResponse {
    pub schema: String,
    pub query: String,
    pub intent: AddonResourceSearchIntent,
    pub total: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<AddonResourceSearchResult>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub merged_by_type: BTreeMap<AddonResourceLinkType, Vec<AddonMergedResourceLink>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_executions: Vec<AddonResourceSearchProviderExecution>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceSearchResult {
    pub id: String,
    pub title: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<AddonResourceLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
    pub score: u16,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceLink {
    pub url: String,
    pub normalized_url: String,
    pub link_type: AddonResourceLinkType,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl fmt::Debug for AddonResourceLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddonResourceLink")
            .field("url", &"<redacted>")
            .field("normalized_url", &"<redacted>")
            .field("link_type", &self.link_type)
            .field("source", &self.source)
            .field("has_password", &self.password.is_some())
            .field("note", &self.note)
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonMergedResourceLink {
    pub url: String,
    pub normalized_url: String,
    pub link_type: AddonResourceLinkType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
}

impl fmt::Debug for AddonMergedResourceLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddonMergedResourceLink")
            .field("url", &"<redacted>")
            .field("normalized_url", &"<redacted>")
            .field("link_type", &self.link_type)
            .field("has_password", &self.password.is_some())
            .field("note", &self.note)
            .field("sources", &self.sources)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResourceLinkType {
    Aliyun,
    Baidu,
    Quark,
    Tianyi,
    Uc,
    Mobile,
    #[serde(rename = "115")]
    OneOneFive,
    Pikpak,
    Xunlei,
    #[serde(rename = "123")]
    OneTwoThree,
    Magnet,
    Ed2k,
    Web,
    Other,
}

impl AddonResourceLinkType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aliyun => "aliyun",
            Self::Baidu => "baidu",
            Self::Quark => "quark",
            Self::Tianyi => "tianyi",
            Self::Uc => "uc",
            Self::Mobile => "mobile",
            Self::OneOneFive => "115",
            Self::Pikpak => "pikpak",
            Self::Xunlei => "xunlei",
            Self::OneTwoThree => "123",
            Self::Magnet => "magnet",
            Self::Ed2k => "ed2k",
            Self::Web => "web",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceSearchProviderExecution {
    pub provider_id: String,
    pub status: AddonResourceSearchProviderStatus,
    pub result_count: usize,
    pub finality: AddonResourceSearchProviderFinality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResourceSearchProviderStatus {
    Ok,
    Error,
    Skipped,
}

impl AddonResourceSearchProviderStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResourceSearchProviderFinality {
    Complete,
    Partial,
    Unknown,
}

impl AddonResourceSearchProviderFinality {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRequest {
    pub protocol_version: String,
    pub addon_id: String,
    pub task_id: String,
    pub job_id: String,
    pub request_id: String,
    pub attempt: u32,
    #[serde(default)]
    pub retry_of_job_id: Option<String>,
    #[serde(default)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskResponse {
    pub protocol_version: String,
    pub addon_id: String,
    pub task_id: String,
    pub job_id: String,
    pub request_id: String,
    #[serde(default)]
    pub output: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterProtocol {
    Chromecast,
    DlnaRenderer,
    Airplay,
}

impl AddonRendererAdapterProtocol {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chromecast => "chromecast",
            Self::DlnaRenderer => "dlna_renderer",
            Self::Airplay => "airplay",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AddonRendererAdapterRequest {
    InspectReadiness {
        protocol: AddonRendererAdapterProtocol,
    },
    DiscoverTargets {
        protocol: AddonRendererAdapterProtocol,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
    },
    DispatchCommand {
        protocol: AddonRendererAdapterProtocol,
        envelope: AddonRendererAdapterCommandEnvelope,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AddonRendererAdapterResponse {
    Readiness {
        readiness: AddonRendererAdapterReadiness,
    },
    Targets {
        targets: Vec<AddonRendererAdapterTarget>,
    },
    CommandResult {
        result: AddonRendererAdapterCommandResult,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterReadiness {
    pub protocol: AddonRendererAdapterProtocol,
    pub status: AddonRendererAdapterReadinessStatus,
    pub reason_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterReadinessStatus {
    Ready,
    Degraded,
    ConfigurationRequired,
    Unavailable,
}

impl AddonRendererAdapterReadinessStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::ConfigurationRequired => "configuration_required",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterTarget {
    pub stable_device_id: String,
    pub target_kind: AddonRendererAdapterProtocol,
    pub display_name: String,
    pub network_scope: AddonRendererAdapterNetworkScope,
    pub media_capabilities: AddonRendererAdapterMediaCapabilities,
    pub control_capabilities: AddonRendererAdapterControlCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovered_at_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterNetworkScope {
    Local,
    Remote,
    Unknown,
}

impl AddonRendererAdapterNetworkScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterMediaCapabilities {
    pub direct_play: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub video_codecs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audio_codecs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterControlCapabilities {
    pub play: bool,
    pub pause: bool,
    pub resume: bool,
    pub seek: bool,
    pub stop: bool,
    pub set_volume: bool,
}

impl AddonRendererAdapterControlCapabilities {
    #[must_use]
    pub const fn basic_playback() -> Self {
        Self {
            play: true,
            pause: true,
            resume: true,
            seek: true,
            stop: true,
            set_volume: false,
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterCommandEnvelope {
    pub adapter_id: String,
    pub stable_device_id: String,
    pub target_kind: AddonRendererAdapterProtocol,
    pub renderer_session_id: String,
    pub playback_session_id: String,
    pub source_id: String,
    pub command: AddonRendererAdapterCommand,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_percent: Option<u8>,
    pub transport: AddonRendererAdapterTransport,
}

impl fmt::Debug for AddonRendererAdapterCommandEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddonRendererAdapterCommandEnvelope")
            .field("adapter_id", &self.adapter_id)
            .field("stable_device_id", &self.stable_device_id)
            .field("target_kind", &self.target_kind)
            .field("renderer_session_id", &self.renderer_session_id)
            .field("playback_session_id", &self.playback_session_id)
            .field("source_id", &self.source_id)
            .field("command", &self.command)
            .field("position_ms", &self.position_ms)
            .field("volume_percent", &self.volume_percent)
            .field("transport", &self.transport)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterCommand {
    Play,
    Pause,
    Resume,
    Seek,
    Stop,
    SetVolume,
}

impl AddonRendererAdapterCommand {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Seek => "seek",
            Self::Stop => "stop",
            Self::SetVolume => "set_volume",
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterTransport {
    pub mode: AddonRendererAdapterTransportMode,
    pub expires_at: String,
    pub urls: Vec<AddonRendererAdapterTransportUrl>,
}

impl fmt::Debug for AddonRendererAdapterTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddonRendererAdapterTransport")
            .field("mode", &self.mode)
            .field("expires_at", &self.expires_at)
            .field("url_count", &self.urls.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterTransportMode {
    Direct,
    Remux,
    Hls,
}

impl AddonRendererAdapterTransportMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Remux => "remux",
            Self::Hls => "hls",
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterTransportUrl {
    pub kind: AddonRendererAdapterTransportUrlKind,
    pub url: String,
    pub content_type: String,
    pub supports_range_requests: bool,
}

impl fmt::Debug for AddonRendererAdapterTransportUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddonRendererAdapterTransportUrl")
            .field("kind", &self.kind)
            .field("url", &"<redacted>")
            .field("content_type", &self.content_type)
            .field("supports_range_requests", &self.supports_range_requests)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterTransportUrlKind {
    Stream,
    Playlist,
    SegmentBase,
}

impl AddonRendererAdapterTransportUrlKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Playlist => "playlist",
            Self::SegmentBase => "segment_base",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRendererAdapterCommandResult {
    pub stable_device_id: String,
    pub command: AddonRendererAdapterCommand,
    pub state: AddonRendererAdapterCommandState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_reason_code: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonRendererAdapterCommandState {
    Accepted,
    Rejected,
    Failed,
}

impl AddonRendererAdapterCommandState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventRequest {
    pub protocol_version: String,
    pub addon_id: String,
    pub subscription_id: String,
    pub event_id: String,
    pub event_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub occurred_at: String,
    pub attempt: u32,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventResponse {
    pub protocol_version: String,
    pub addon_id: String,
    pub subscription_id: String,
    pub event_id: String,
    #[serde(default)]
    pub output: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonHealthCheckRequest {
    pub protocol_version: String,
    pub manifest_id: String,
    pub request_id: String,
    pub expected_addon_version: String,
    pub expected_resource_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonHealthCheckResponse {
    pub protocol_version: String,
    pub manifest_id: String,
    pub status: AddonHealthStatus,
    pub checked_at: String,
    pub manifest: AddonHealthManifestFacts,
    #[serde(default)]
    pub diagnostics: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonHealthStatus {
    Ok,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonHealthManifestFacts {
    pub addon_version: String,
    pub resource_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonArtifact {
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataPatch {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub sort_title: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub runtime_minutes: Option<u32>,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub genres: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub ratings: Option<Vec<AddonMetadataContentRating>>,
    #[serde(default)]
    pub images: Option<Vec<AddonMetadataImage>>,
    #[serde(default)]
    pub credits: Option<Vec<AddonMetadataCredit>>,
    #[serde(default)]
    pub collections: Option<Vec<AddonMetadataCollection>>,
    #[serde(default)]
    pub studios: Option<Vec<AddonMetadataStudio>>,
    #[serde(default)]
    pub external_ids: Option<Vec<AddonMetadataExternalId>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataContentRating {
    pub source: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataImage {
    pub kind: String,
    pub uri: String,
    pub provider: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataCredit {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub order: Option<u32>,
    #[serde(default)]
    pub external_ids: Vec<AddonMetadataExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataCollection {
    pub name: String,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub sort_order: Option<u32>,
    #[serde(default)]
    pub external_ids: Vec<AddonMetadataExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataStudio {
    pub name: String,
    #[serde(default)]
    pub external_ids: Vec<AddonMetadataExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonMetadataExternalId {
    pub provider: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonArtworkIntent {
    ProposeArtwork,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonArtworkKind {
    Poster,
    Backdrop,
    Logo,
    Banner,
    Thumbnail,
}

impl AddonArtworkKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Poster => "poster",
            Self::Backdrop => "backdrop",
            Self::Logo => "logo",
            Self::Banner => "banner",
            Self::Thumbnail => "thumbnail",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonArtworkSourceKind {
    RemoteUrl,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonArtworkSourcePayload {
    pub kind: AddonArtworkSourceKind,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonArtworkWritePayload {
    pub intent: AddonArtworkIntent,
    pub kind: AddonArtworkKind,
    pub source: AddonArtworkSourcePayload,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonPermission {
    MetadataWrite,
    ArtworkWrite,
    SubtitleWrite,
    LibraryFileWrite,
}

impl AddonPermission {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataWrite => "metadata_write",
            Self::ArtworkWrite => "artwork_write",
            Self::SubtitleWrite => "subtitle_write",
            Self::LibraryFileWrite => "library_file_write",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonSideEffectTargetKind {
    MediaItem,
    MediaSource,
}

impl AddonSideEffectTargetKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MediaItem => "media_item",
            Self::MediaSource => "media_source",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectTarget {
    pub kind: AddonSideEffectTargetKind,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAccessCheckRequest {
    pub permission: AddonPermission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAccessCheckResponse {
    pub addon_id: String,
    pub token_id: String,
    pub permission: AddonPermission,
    #[serde(default)]
    pub library_id: Option<String>,
    pub allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitAddonSideEffectRequest {
    pub permission: AddonPermission,
    pub library_id: String,
    pub target: AddonSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct SubmitAddonMetadataWriteRequest {
    pub library_id: String,
    pub target: AddonSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub patch: AddonMetadataPatch,
}

#[derive(Clone, Debug)]
pub struct SubmitAddonArtworkWriteRequest {
    pub library_id: String,
    pub target: AddonSideEffectTarget,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub artwork: AddonArtworkWritePayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectResponse {
    pub side_effect: AddonSideEffectSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectSummary {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addon_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
    pub permission: AddonPermission,
    pub library_id: String,
    pub target: AddonSideEffectTarget,
    pub idempotency_key: String,
    pub validation_status: String,
    #[serde(default)]
    pub safe_error_code: Option<String>,
    pub apply_status: String,
    #[serde(default)]
    pub apply_error_code: Option<String>,
    #[serde(default)]
    pub applied_item_id: Option<String>,
    #[serde(default)]
    pub applied_source: Option<String>,
    #[serde(default)]
    pub apply_report: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonLibraryFileRole {
    Nfo,
}

impl AddonLibraryFileRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nfo => "nfo",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonLibraryFileWritePolicy {
    CreateMissing,
    ReplaceExistingPreserving,
}

impl AddonLibraryFileWritePolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateMissing => "create_missing",
            Self::ReplaceExistingPreserving => "replace_existing_preserving",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonLibraryFileWritePayload {
    pub file_role: AddonLibraryFileRole,
    pub policy: AddonLibraryFileWritePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddonManifestError {
    UnsupportedProtocolVersion {
        actual: String,
    },
    EmptyField {
        field: &'static str,
    },
    InvalidBaseUrl,
    InvalidResourcePath {
        path: String,
    },
    InvalidDeclarationPath {
        declaration: &'static str,
        path: String,
    },
    DuplicateResource {
        resource: AddonResource,
    },
    DuplicateDeclaration {
        declaration: &'static str,
        id: String,
    },
    UnknownHostedPageReference {
        entry_point_id: String,
        hosted_page_id: String,
    },
    EmptyResources,
    MissingDeclaredScope {
        resource: AddonResource,
        scope: AddonScope,
    },
    MissingDeclaredScopeForDeclaration {
        declaration: &'static str,
        scope: AddonScope,
    },
    InvalidConfigurationSchema {
        message: String,
    },
    InvalidTimeout {
        value: u64,
    },
    InvalidMaxAttempts {
        value: u32,
    },
    MissingAuthToken {
        auth: AddonAuth,
    },
    MissingRuntimeReference,
    InvalidRuntimeReference,
    UnknownSecretReferenceField {
        field_id: String,
    },
    DuplicateSecretReferenceBinding {
        field_id: String,
    },
    SecretReferenceContainsValue {
        field_id: String,
    },
    ResourceNotDeclared {
        resource: AddonResource,
    },
    EventSubscriptionNotDeclared {
        subscription_id: String,
    },
    TaskNotDeclared {
        task_id: String,
    },
    InvalidEnvelope {
        message: String,
    },
}

impl fmt::Display for AddonManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion { actual } => {
                write!(formatter, "unsupported addon protocol version: {actual}")
            }
            Self::EmptyField { field } => {
                write!(formatter, "addon manifest field is empty: {field}")
            }
            Self::InvalidBaseUrl => write!(formatter, "addon base_url must use http or https"),
            Self::InvalidResourcePath { path } => {
                write!(formatter, "addon resource path must be absolute: {path}")
            }
            Self::InvalidDeclarationPath { declaration, path } => {
                write!(
                    formatter,
                    "addon {declaration} path must be absolute: {path}"
                )
            }
            Self::DuplicateResource { resource } => {
                write!(formatter, "duplicate addon resource: {}", resource.as_str())
            }
            Self::DuplicateDeclaration { declaration, id } => {
                write!(formatter, "duplicate addon {declaration} declaration: {id}")
            }
            Self::UnknownHostedPageReference {
                entry_point_id,
                hosted_page_id,
            } => write!(
                formatter,
                "addon entry_point {entry_point_id} references unknown hosted_page {hosted_page_id}"
            ),
            Self::EmptyResources => write!(formatter, "addon manifest must declare resources"),
            Self::MissingDeclaredScope { resource, scope } => write!(
                formatter,
                "addon resource {} requires undeclared scope {}",
                resource.as_str(),
                scope.as_str()
            ),
            Self::MissingDeclaredScopeForDeclaration { declaration, scope } => write!(
                formatter,
                "addon {declaration} declaration requires undeclared scope {}",
                scope.as_str()
            ),
            Self::InvalidConfigurationSchema { message } => {
                write!(formatter, "invalid addon configuration schema: {message}")
            }
            Self::InvalidTimeout { value } => {
                write!(
                    formatter,
                    "addon timeout_ms is outside allowed range: {value}"
                )
            }
            Self::InvalidMaxAttempts { value } => {
                write!(
                    formatter,
                    "addon max_attempts is outside allowed range: {value}"
                )
            }
            Self::MissingAuthToken { auth } => {
                write!(
                    formatter,
                    "addon auth token is required for {auth:?} authentication"
                )
            }
            Self::MissingRuntimeReference => {
                write!(
                    formatter,
                    "addon install descriptor must declare an image, binary, or command"
                )
            }
            Self::InvalidRuntimeReference => {
                write!(
                    formatter,
                    "addon runtime reference must not contain credentials, URLs, or local paths"
                )
            }
            Self::UnknownSecretReferenceField { field_id } => {
                write!(
                    formatter,
                    "addon install descriptor references unknown secret field: {field_id}"
                )
            }
            Self::DuplicateSecretReferenceBinding { field_id } => {
                write!(
                    formatter,
                    "duplicate addon secret reference binding: {field_id}"
                )
            }
            Self::SecretReferenceContainsValue { field_id } => {
                write!(
                    formatter,
                    "addon secret reference binding {field_id} must contain a reference, not a secret value"
                )
            }
            Self::ResourceNotDeclared { resource } => {
                write!(
                    formatter,
                    "addon resource is not declared: {}",
                    resource.as_str()
                )
            }
            Self::EventSubscriptionNotDeclared { subscription_id } => {
                write!(
                    formatter,
                    "addon event subscription is not declared: {subscription_id}"
                )
            }
            Self::TaskNotDeclared { task_id } => {
                write!(formatter, "addon task is not declared: {task_id}")
            }
            Self::InvalidEnvelope { message } => {
                write!(formatter, "invalid addon envelope: {message}")
            }
        }
    }
}

impl std::error::Error for AddonManifestError {}

pub type AddonProtocolResult<T> = std::result::Result<T, AddonManifestError>;

#[must_use]
pub fn is_supported_addon_protocol_version(protocol_version: &str) -> bool {
    SUPPORTED_ADDON_PROTOCOL_VERSIONS.contains(&protocol_version)
}

pub fn validate_manifest(manifest: &AddonManifest) -> AddonProtocolResult<()> {
    validate_non_empty(&manifest.id, "id")?;
    validate_non_empty(&manifest.name, "name")?;
    validate_non_empty(&manifest.version, "version")?;
    if !is_supported_addon_protocol_version(&manifest.protocol_version) {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: manifest.protocol_version.clone(),
        });
    }
    if !has_http_base_url(&manifest.base_url) {
        return Err(AddonManifestError::InvalidBaseUrl);
    }
    if manifest.resources.is_empty() {
        return Err(AddonManifestError::EmptyResources);
    }
    if let Some(timeout) = manifest.default_timeout_ms {
        validate_timeout(timeout)?;
    }
    if let Some(max_attempts) = manifest.default_max_attempts {
        validate_max_attempts(max_attempts)?;
    }

    let declared_scopes = manifest.scopes.iter().copied().collect::<HashSet<_>>();
    let mut declared_resources = HashSet::new();
    for resource in &manifest.resources {
        if !declared_resources.insert(resource.kind) {
            return Err(AddonManifestError::DuplicateResource {
                resource: resource.kind,
            });
        }
        if !resource.path.starts_with('/') {
            return Err(AddonManifestError::InvalidResourcePath {
                path: resource.path.clone(),
            });
        }
        if let Some(timeout) = resource.timeout_ms {
            validate_timeout(timeout)?;
        }
        if let Some(max_attempts) = resource.max_attempts {
            validate_max_attempts(max_attempts)?;
        }
        for scope in &resource.required_scopes {
            if !declared_scopes.contains(scope) {
                return Err(AddonManifestError::MissingDeclaredScope {
                    resource: resource.kind,
                    scope: *scope,
                });
            }
        }
    }
    validate_manifest_declarations(manifest, &declared_scopes)?;

    Ok(())
}

pub fn ensure_scope_grant(
    manifest: &AddonManifest,
    resource: AddonResource,
    granted_scopes: &[AddonScope],
) -> AddonProtocolResult<()> {
    validate_manifest(manifest)?;
    let granted = granted_scopes.iter().copied().collect::<HashSet<_>>();
    let declaration = manifest
        .resources
        .iter()
        .find(|candidate| candidate.kind == resource)
        .ok_or(AddonManifestError::ResourceNotDeclared { resource })?;

    for scope in &declaration.required_scopes {
        if !granted.contains(scope) {
            return Err(AddonManifestError::MissingDeclaredScope {
                resource,
                scope: *scope,
            });
        }
    }

    Ok(())
}

pub fn ensure_task_scope_grant(
    manifest: &AddonManifest,
    task_id: &str,
    granted_scopes: &[AddonScope],
) -> AddonProtocolResult<()> {
    validate_manifest(manifest)?;
    let granted = granted_scopes.iter().copied().collect::<HashSet<_>>();
    let declaration = manifest
        .tasks
        .iter()
        .find(|candidate| candidate.id == task_id)
        .ok_or_else(|| AddonManifestError::TaskNotDeclared {
            task_id: task_id.to_owned(),
        })?;

    for scope in &declaration.required_scopes {
        if !granted.contains(scope) {
            return Err(AddonManifestError::MissingDeclaredScopeForDeclaration {
                declaration: "task",
                scope: *scope,
            });
        }
    }

    Ok(())
}

pub fn ensure_event_subscription_scope_grant(
    manifest: &AddonManifest,
    subscription_id: &str,
    granted_scopes: &[AddonScope],
) -> AddonProtocolResult<()> {
    validate_manifest(manifest)?;
    let granted = granted_scopes.iter().copied().collect::<HashSet<_>>();
    let declaration = manifest
        .event_subscriptions
        .iter()
        .find(|candidate| candidate.id == subscription_id)
        .ok_or_else(|| AddonManifestError::EventSubscriptionNotDeclared {
            subscription_id: subscription_id.to_owned(),
        })?;

    for scope in &declaration.required_scopes {
        if !granted.contains(scope) {
            return Err(AddonManifestError::MissingDeclaredScopeForDeclaration {
                declaration: "event_subscription",
                scope: *scope,
            });
        }
    }

    Ok(())
}

pub fn validate_install_descriptor(descriptor: &AddonInstallDescriptor) -> AddonProtocolResult<()> {
    validate_manifest(&descriptor.manifest)?;
    validate_runtime_requirement(&descriptor.runtime)?;

    let declared_secret_fields = descriptor
        .manifest
        .secret_reference_fields
        .iter()
        .map(|field| field.id.as_str())
        .collect::<HashSet<_>>();
    let mut bound_secret_fields = HashSet::new();
    for binding in &descriptor.secret_reference_bindings {
        validate_non_empty(&binding.field_id, "secret_reference_bindings.field_id")?;
        validate_non_empty(&binding.secret_ref, "secret_reference_bindings.secret_ref")?;
        if !declared_secret_fields.contains(binding.field_id.as_str()) {
            return Err(AddonManifestError::UnknownSecretReferenceField {
                field_id: binding.field_id.clone(),
            });
        }
        if !bound_secret_fields.insert(binding.field_id.clone()) {
            return Err(AddonManifestError::DuplicateSecretReferenceBinding {
                field_id: binding.field_id.clone(),
            });
        }
        if secret_reference_looks_like_value(&binding.secret_ref) {
            return Err(AddonManifestError::SecretReferenceContainsValue {
                field_id: binding.field_id.clone(),
            });
        }
    }

    Ok(())
}

#[must_use]
pub fn addon_install_guide(descriptor: &AddonInstallDescriptor) -> AddonInstallGuide {
    let provided_secret_refs = descriptor
        .secret_reference_bindings
        .iter()
        .map(|binding| binding.secret_ref.clone())
        .collect::<Vec<_>>();
    let provided_secret_fields = descriptor
        .secret_reference_bindings
        .iter()
        .map(|binding| binding.field_id.as_str())
        .collect::<HashSet<_>>();
    let required_secret_fields = descriptor
        .manifest
        .secret_reference_fields
        .iter()
        .map(|field| AddonInstallSecretField {
            id: field.id.clone(),
            label: field.label.clone(),
            required: field.required,
            provided: provided_secret_fields.contains(field.id.as_str()),
        })
        .collect::<Vec<_>>();
    let missing_required_secret_fields = required_secret_fields
        .iter()
        .filter(|field| field.required && !field.provided)
        .map(|field| field.id.clone())
        .collect::<Vec<_>>();

    AddonInstallGuide {
        manifest_id: descriptor.manifest.id.clone(),
        addon_name: descriptor.manifest.name.clone(),
        protocol_version: descriptor.manifest.protocol_version.clone(),
        runtime_kind: descriptor.runtime.kind,
        runtime_reference: runtime_reference(&descriptor.runtime),
        base_url_scheme: base_url_scheme(&descriptor.manifest.base_url).unwrap_or_default(),
        base_url_configured: has_http_base_url(&descriptor.manifest.base_url),
        declared_resources: descriptor
            .manifest
            .resources
            .iter()
            .map(|resource| resource.kind)
            .collect(),
        declared_scopes: descriptor.manifest.scopes.clone(),
        required_secret_fields,
        provided_secret_refs,
        missing_required_secret_fields,
        has_configuration_schema: descriptor.manifest.configuration_schema.is_some(),
        entry_point_count: usize_to_u32(descriptor.manifest.entry_points.len()),
        hosted_page_count: usize_to_u32(descriptor.manifest.hosted_pages.len()),
        task_count: usize_to_u32(descriptor.manifest.tasks.len()),
        event_subscription_count: usize_to_u32(descriptor.manifest.event_subscriptions.len()),
        install_steps: install_steps(descriptor),
    }
}

pub fn validate_resource_response(
    response: &AddonResourceResponse,
    manifest: &AddonManifest,
    resource: AddonResource,
    request_id: &str,
) -> AddonProtocolResult<()> {
    if !is_supported_addon_protocol_version(&response.protocol_version) {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: response.protocol_version.clone(),
        });
    }
    if response.protocol_version != manifest.protocol_version {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response protocol_version {} did not match manifest protocol_version {}",
                response.protocol_version, manifest.protocol_version
            ),
        });
    }
    if response.addon_id != manifest.id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response addon_id {} did not match {}",
                response.addon_id, manifest.id
            ),
        });
    }
    if response.resource != resource {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response resource {} did not match {}",
                response.resource.as_str(),
                resource.as_str()
            ),
        });
    }
    if response.request_id != request_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response request_id {} did not match {request_id}",
                response.request_id
            ),
        });
    }

    Ok(())
}

pub fn validate_task_response(
    response: &AddonTaskResponse,
    manifest: &AddonManifest,
    task_id: &str,
    job_id: &str,
    request_id: &str,
) -> AddonProtocolResult<()> {
    if !is_supported_addon_protocol_version(&response.protocol_version) {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: response.protocol_version.clone(),
        });
    }
    if response.protocol_version != manifest.protocol_version {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "task response protocol_version {} did not match manifest protocol_version {}",
                response.protocol_version, manifest.protocol_version
            ),
        });
    }
    if response.addon_id != manifest.id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "task response addon_id {} did not match {}",
                response.addon_id, manifest.id
            ),
        });
    }
    if response.task_id != task_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "task response task_id {} did not match {task_id}",
                response.task_id
            ),
        });
    }
    if response.job_id != job_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "task response job_id {} did not match {job_id}",
                response.job_id
            ),
        });
    }
    if response.request_id != request_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "task response request_id {} did not match {request_id}",
                response.request_id
            ),
        });
    }

    Ok(())
}

pub fn validate_event_response(
    response: &AddonEventResponse,
    manifest: &AddonManifest,
    subscription_id: &str,
    event_id: &str,
) -> AddonProtocolResult<()> {
    if !is_supported_addon_protocol_version(&response.protocol_version) {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: response.protocol_version.clone(),
        });
    }
    if response.protocol_version != manifest.protocol_version {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "event response protocol_version {} did not match manifest protocol_version {}",
                response.protocol_version, manifest.protocol_version
            ),
        });
    }
    if response.addon_id != manifest.id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "event response addon_id {} did not match {}",
                response.addon_id, manifest.id
            ),
        });
    }
    if response.subscription_id != subscription_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "event response subscription_id {} did not match {subscription_id}",
                response.subscription_id
            ),
        });
    }
    if response.event_id != event_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "event response event_id {} did not match {event_id}",
                response.event_id
            ),
        });
    }

    Ok(())
}

pub fn validate_health_check_response(
    response: &AddonHealthCheckResponse,
    manifest: &AddonManifest,
) -> AddonProtocolResult<()> {
    if !is_supported_addon_protocol_version(&response.protocol_version) {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: response.protocol_version.clone(),
        });
    }
    if response.protocol_version != manifest.protocol_version {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "health protocol_version {} did not match manifest protocol_version {}",
                response.protocol_version, manifest.protocol_version
            ),
        });
    }
    if response.manifest_id != manifest.id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "health manifest_id {} did not match {}",
                response.manifest_id, manifest.id
            ),
        });
    }
    if response.checked_at.trim().is_empty() {
        return Err(AddonManifestError::InvalidEnvelope {
            message: "health checked_at must not be empty".to_owned(),
        });
    }
    if response.manifest.addon_version != manifest.version {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "health addon_version {} did not match {}",
                response.manifest.addon_version, manifest.version
            ),
        });
    }
    if response.manifest.resource_count != manifest.resources.len() {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "health resource_count {} did not match {}",
                response.manifest.resource_count,
                manifest.resources.len()
            ),
        });
    }

    Ok(())
}

fn validate_non_empty(value: &str, field: &'static str) -> AddonProtocolResult<()> {
    if value.trim().is_empty() {
        Err(AddonManifestError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_manifest_declarations(
    manifest: &AddonManifest,
    declared_scopes: &HashSet<AddonScope>,
) -> AddonProtocolResult<()> {
    let mut entry_point_ids = HashSet::new();
    for entry_point in &manifest.entry_points {
        validate_non_empty(&entry_point.id, "entry_points.id")?;
        validate_non_empty(&entry_point.label, "entry_points.label")?;
        validate_absolute_declaration_path("entry_point", &entry_point.path)?;
        validate_unique_declaration_id("entry_point", &entry_point.id, &mut entry_point_ids)?;
        validate_declared_scopes("entry_point", &entry_point.required_scopes, declared_scopes)?;
    }

    let mut hosted_page_ids = HashSet::new();
    for hosted_page in &manifest.hosted_pages {
        validate_non_empty(&hosted_page.id, "hosted_pages.id")?;
        validate_non_empty(&hosted_page.title, "hosted_pages.title")?;
        validate_absolute_declaration_path("hosted_page", &hosted_page.path)?;
        validate_unique_declaration_id("hosted_page", &hosted_page.id, &mut hosted_page_ids)?;
        validate_declared_scopes("hosted_page", &hosted_page.required_scopes, declared_scopes)?;
    }

    for entry_point in &manifest.entry_points {
        if let Some(hosted_page_id) = &entry_point.hosted_page_id
            && !hosted_page_ids.contains(hosted_page_id)
        {
            return Err(AddonManifestError::UnknownHostedPageReference {
                entry_point_id: entry_point.id.clone(),
                hosted_page_id: hosted_page_id.clone(),
            });
        }
    }

    if let Some(configuration_schema) = &manifest.configuration_schema {
        validate_non_empty(
            &configuration_schema.schema_id,
            "configuration_schema.schema_id",
        )?;
        if !configuration_schema.schema.is_object() {
            return Err(AddonManifestError::InvalidConfigurationSchema {
                message: "schema must be a JSON object".to_owned(),
            });
        }
    }

    let mut secret_reference_ids = HashSet::new();
    for secret_reference in &manifest.secret_reference_fields {
        validate_non_empty(&secret_reference.id, "secret_reference_fields.id")?;
        validate_non_empty(&secret_reference.label, "secret_reference_fields.label")?;
        validate_unique_declaration_id(
            "secret_reference_field",
            &secret_reference.id,
            &mut secret_reference_ids,
        )?;
    }

    let mut event_subscription_ids = HashSet::new();
    for event_subscription in &manifest.event_subscriptions {
        validate_non_empty(&event_subscription.id, "event_subscriptions.id")?;
        validate_non_empty(
            &event_subscription.event_kind,
            "event_subscriptions.event_kind",
        )?;
        validate_absolute_declaration_path("event_subscription", &event_subscription.path)?;
        validate_unique_declaration_id(
            "event_subscription",
            &event_subscription.id,
            &mut event_subscription_ids,
        )?;
        validate_declared_scopes(
            "event_subscription",
            &event_subscription.required_scopes,
            declared_scopes,
        )?;
    }

    let mut task_ids = HashSet::new();
    for task in &manifest.tasks {
        validate_non_empty(&task.id, "tasks.id")?;
        validate_non_empty(&task.name, "tasks.name")?;
        validate_absolute_declaration_path("task", &task.path)?;
        validate_unique_declaration_id("task", &task.id, &mut task_ids)?;
        validate_declared_scopes("task", &task.required_scopes, declared_scopes)?;
        if let Some(timeout) = task.timeout_ms {
            validate_timeout(timeout)?;
        }
        if let Some(max_attempts) = task.max_attempts {
            validate_max_attempts(max_attempts)?;
        }
    }

    Ok(())
}

fn validate_absolute_declaration_path(
    declaration: &'static str,
    path: &str,
) -> AddonProtocolResult<()> {
    if path.starts_with('/') {
        Ok(())
    } else {
        Err(AddonManifestError::InvalidDeclarationPath {
            declaration,
            path: path.to_owned(),
        })
    }
}

fn validate_unique_declaration_id(
    declaration: &'static str,
    id: &str,
    seen: &mut HashSet<String>,
) -> AddonProtocolResult<()> {
    if seen.insert(id.to_owned()) {
        Ok(())
    } else {
        Err(AddonManifestError::DuplicateDeclaration {
            declaration,
            id: id.to_owned(),
        })
    }
}

fn validate_declared_scopes(
    declaration: &'static str,
    required_scopes: &[AddonScope],
    declared_scopes: &HashSet<AddonScope>,
) -> AddonProtocolResult<()> {
    for scope in required_scopes {
        if !declared_scopes.contains(scope) {
            return Err(AddonManifestError::MissingDeclaredScopeForDeclaration {
                declaration,
                scope: *scope,
            });
        }
    }

    Ok(())
}

fn validate_timeout(value: u64) -> AddonProtocolResult<()> {
    if (100..=120_000).contains(&value) {
        Ok(())
    } else {
        Err(AddonManifestError::InvalidTimeout { value })
    }
}

fn validate_max_attempts(value: u32) -> AddonProtocolResult<()> {
    if (1..=10).contains(&value) {
        Ok(())
    } else {
        Err(AddonManifestError::InvalidMaxAttempts { value })
    }
}

fn validate_runtime_requirement(runtime: &AddonRuntimeRequirement) -> AddonProtocolResult<()> {
    let runtime_reference_count = [&runtime.image, &runtime.binary, &runtime.command]
        .into_iter()
        .flatten()
        .count();
    if runtime_reference_count == 0 {
        return Err(AddonManifestError::MissingRuntimeReference);
    }
    if runtime_reference_count > 1 {
        return Err(AddonManifestError::InvalidRuntimeReference);
    }

    for value in [&runtime.image, &runtime.binary, &runtime.command]
        .into_iter()
        .flatten()
    {
        validate_non_empty(value, "runtime")?;
        if runtime_reference_is_sensitive(value) {
            return Err(AddonManifestError::InvalidRuntimeReference);
        }
    }

    Ok(())
}

fn runtime_reference_is_sensitive(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.contains('\\')
        || looks_like_local_path(value)
        || lower.starts_with("file:")
        || lower.contains("://")
        || lower.contains("token=")
        || lower.contains("secret=")
        || lower.contains("password=")
        || lower.contains("authorization:")
        || lower.contains("bearer ")
        || lower.contains("--env ")
        || lower.contains("-e ")
}

fn looks_like_local_path(value: &str) -> bool {
    let value = value.trim();
    value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with("~/")
        || value.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn secret_reference_looks_like_value(value: &str) -> bool {
    let value = value.trim();
    if let Some(name) = value.strip_prefix("env:") {
        return !valid_environment_reference_name(name);
    }
    if let Some(name) = value.strip_prefix("secret:") {
        return !valid_named_secret_reference(name);
    }
    if let Some(name) = value.strip_prefix("nako:") {
        return !valid_named_secret_reference(name);
    }

    let lower = value.to_ascii_lowercase();
    lower.contains("://")
        || lower.contains('=')
        || lower.starts_with("bearer ")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}

fn valid_environment_reference_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn valid_named_secret_reference(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character == '_'
                || character == '-'
                || character == '.'
                || character == '/'
                || character.is_ascii_alphanumeric()
        })
}

fn base_url_scheme(value: &str) -> Option<String> {
    value.split_once("://").map(|(scheme, _)| scheme.to_owned())
}

fn runtime_reference(runtime: &AddonRuntimeRequirement) -> AddonRuntimeReference {
    if let Some(value) = runtime.image.as_ref() {
        return AddonRuntimeReference {
            kind: AddonRuntimeReferenceKind::Image,
            value: value.clone(),
        };
    }
    if let Some(value) = runtime.binary.as_ref() {
        return AddonRuntimeReference {
            kind: AddonRuntimeReferenceKind::Binary,
            value: value.clone(),
        };
    }
    let value = runtime.command.clone().unwrap_or_default();
    AddonRuntimeReference {
        kind: AddonRuntimeReferenceKind::Command,
        value,
    }
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn install_steps(descriptor: &AddonInstallDescriptor) -> Vec<AddonInstallStep> {
    let mut steps = vec![AddonInstallStep {
        kind: AddonInstallStepKind::RunSidecar,
        summary: "Run the Addon Sidecar outside Nako using the declared runtime reference."
            .to_owned(),
    }];
    if descriptor
        .manifest
        .secret_reference_fields
        .iter()
        .any(|field| field.required)
    {
        steps.push(AddonInstallStep {
            kind: AddonInstallStepKind::ConfigureSecretReference,
            summary: "Configure required Secret References in Nako; do not paste secret values into the install guide."
                .to_owned(),
        });
    }
    steps.push(AddonInstallStep {
        kind: AddonInstallStepKind::RegisterManifest,
        summary:
            "Register the manifest through the Admin Addon API after the sidecar is reachable."
                .to_owned(),
    });
    if !descriptor.manifest.scopes.is_empty() {
        steps.push(AddonInstallStep {
            kind: AddonInstallStepKind::GrantScopes,
            summary:
                "Grant only the requested Addon Scopes and library access needed by this sidecar."
                    .to_owned(),
        });
    }

    steps
}

fn has_http_base_url(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https") {
        return false;
    }
    let Some(authority) = rest.split(['/', '?', '#']).next() else {
        return false;
    };
    !authority.trim().is_empty() && !authority.contains(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_manifest_resource_and_scope_contract() {
        let manifest = valid_manifest();

        validate_manifest(&manifest).unwrap();
        ensure_scope_grant(
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
        )
        .unwrap();
    }

    #[test]
    fn exposes_explicit_supported_protocol_versions() {
        assert_eq!(SUPPORTED_ADDON_PROTOCOL_VERSIONS, &[ADDON_PROTOCOL_VERSION]);
        assert!(is_supported_addon_protocol_version(ADDON_PROTOCOL_VERSION));
        assert!(!is_supported_addon_protocol_version("0.1.0-alpha.0"));
    }

    #[test]
    fn exposes_explicit_addon_runtime_route_inventory() {
        let paths = addon_runtime_paths().collect::<Vec<_>>();

        assert_eq!(
            paths,
            vec![
                ADDON_RUNTIME_ACCESS_CHECK_PATH,
                ADDON_RUNTIME_SIDE_EFFECTS_PATH,
                ADDON_RUNTIME_GENERATED_ARTIFACTS_PATH,
                ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH,
                ADDON_RUNTIME_TASK_RUN_CLAIM_PATH,
                ADDON_RUNTIME_TASK_RUN_PROGRESS_PATH,
                ADDON_RUNTIME_TASK_RUN_COMPLETE_PATH,
                ADDON_RUNTIME_TASK_RUN_FAIL_PATH,
                ADDON_RUNTIME_TASK_RUN_CANCEL_PATH,
            ]
        );
        assert!(paths.iter().all(|path| path.starts_with("/addon/v1/")));
        assert_eq!(
            ADDON_RUNTIME_ROUTES.len(),
            paths.iter().collect::<HashSet<_>>().len(),
            "Addon runtime route paths must stay unique"
        );
        assert!(
            ADDON_RUNTIME_ROUTES
                .iter()
                .all(|route| route.method == AddonRuntimeHttpMethod::Post)
        );
    }

    #[test]
    fn rejects_invalid_manifest_shape() {
        let mut manifest = valid_manifest();
        manifest.protocol_version = "2020-01-01".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::UnsupportedProtocolVersion { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.resources[0].path = "metadata".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidResourcePath { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.resources.push(manifest.resources[0].clone());
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::DuplicateResource { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.base_url = "file:///tmp/addon".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidBaseUrl)
        ));

        let mut manifest = valid_manifest();
        manifest.base_url = "https:///missing-authority".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidBaseUrl)
        ));
    }

    #[test]
    fn validates_manifest_declaration_contracts() {
        let mut manifest = valid_manifest();
        manifest.hosted_pages = vec![AddonHostedPageDeclaration {
            id: "diagnostics".to_owned(),
            title: "Addon Diagnostics".to_owned(),
            path: "/ui/diagnostics".to_owned(),
            required_scopes: vec![AddonScope::ItemMetadataRead],
        }];
        manifest.entry_points = vec![AddonEntryPointDeclaration::hosted_page(
            "metadata-action",
            AddonEntryPointKind::ItemAction,
            "Suggest Metadata",
            "/ui/metadata-action",
            "diagnostics",
            vec![AddonScope::ItemMetadataSuggest],
        )];
        manifest.configuration_schema = Some(AddonConfigurationSchema {
            schema_id: "nako.reference.metadata.config.v1".to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": { "type": "string" },
                    "api_key": { "type": "string", "x-nako-secret-reference": true }
                },
                "additionalProperties": false
            }),
        });
        manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration {
            id: "api_key".to_owned(),
            label: "API Key".to_owned(),
            description: Some("Resolved by Nako when the addon is called".to_owned()),
            required: true,
        }];
        manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration {
            id: "library-scan-finished".to_owned(),
            event_kind: "library_scan.succeeded".to_owned(),
            path: "/events/library-scan-finished".to_owned(),
            required_scopes: vec![AddonScope::WebhookEventRead],
            filters: serde_json::json!({ "library_preset": "movies" }),
        }];
        manifest.tasks = vec![AddonTaskDeclaration {
            id: "bulk-metadata-scrape".to_owned(),
            name: "Bulk metadata scrape".to_owned(),
            path: "/tasks/bulk-metadata-scrape".to_owned(),
            description: Some("Runs metadata suggestions for selected items".to_owned()),
            required_scopes: vec![AddonScope::AutomationRun],
            timeout_ms: Some(30_000),
            max_attempts: Some(2),
        }];
        manifest
            .scopes
            .extend([AddonScope::WebhookEventRead, AddonScope::AutomationRun]);

        validate_manifest(&manifest).unwrap();
    }

    #[test]
    fn rejects_invalid_manifest_declarations() {
        let mut manifest = valid_manifest();
        manifest.entry_points = vec![AddonEntryPointDeclaration {
            id: "metadata-action".to_owned(),
            kind: AddonEntryPointKind::ItemAction,
            label: "Suggest Metadata".to_owned(),
            path: "ui/metadata-action".to_owned(),
            hosted_page_id: None,
            required_scopes: vec![AddonScope::ItemMetadataSuggest],
        }];
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidDeclarationPath {
                declaration: "entry_point",
                ..
            })
        ));

        let mut manifest = valid_manifest();
        manifest.hosted_pages = vec![
            AddonHostedPageDeclaration {
                id: "diagnostics".to_owned(),
                title: "Diagnostics".to_owned(),
                path: "/ui/diagnostics".to_owned(),
                required_scopes: Vec::new(),
            },
            AddonHostedPageDeclaration {
                id: "diagnostics".to_owned(),
                title: "Diagnostics Duplicate".to_owned(),
                path: "/ui/diagnostics-2".to_owned(),
                required_scopes: Vec::new(),
            },
        ];
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::DuplicateDeclaration {
                declaration: "hosted_page",
                ..
            })
        ));

        let mut manifest = valid_manifest();
        manifest.entry_points = vec![AddonEntryPointDeclaration {
            id: "metadata-action".to_owned(),
            kind: AddonEntryPointKind::ItemAction,
            label: "Suggest Metadata".to_owned(),
            path: "/ui/metadata-action".to_owned(),
            hosted_page_id: Some("missing-page".to_owned()),
            required_scopes: vec![AddonScope::ItemMetadataSuggest],
        }];
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::UnknownHostedPageReference { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.tasks = vec![AddonTaskDeclaration {
            id: "bulk-metadata-scrape".to_owned(),
            name: "Bulk metadata scrape".to_owned(),
            path: "/tasks/bulk-metadata-scrape".to_owned(),
            description: None,
            required_scopes: vec![AddonScope::AutomationRun],
            timeout_ms: Some(30_000),
            max_attempts: Some(2),
        }];
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::MissingDeclaredScopeForDeclaration {
                declaration: "task",
                scope: AddonScope::AutomationRun,
            })
        ));

        let mut manifest = valid_manifest();
        manifest.configuration_schema = Some(AddonConfigurationSchema {
            schema_id: "nako.reference.metadata.config.v1".to_owned(),
            schema: serde_json::json!("not-an-object"),
        });
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidConfigurationSchema { .. })
        ));
    }

    #[test]
    fn denies_missing_scope_grants() {
        let manifest = valid_manifest();

        assert!(matches!(
            ensure_scope_grant(
                &manifest,
                AddonResource::Metadata,
                &[AddonScope::ItemMetadataRead]
            ),
            Err(AddonManifestError::MissingDeclaredScope { .. })
        ));
    }

    #[test]
    fn resource_envelopes_round_trip() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: "example".to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({"item_id":"018f0000-0000-7000-8000-000000000001"}),
        };
        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            serde_json::from_str::<AddonResourceRequest>(&json).unwrap(),
            request
        );
    }

    #[test]
    fn resource_search_protocol_vocabulary_uses_stable_wire_names() {
        assert_eq!(AddonResource::ResourceSearch.as_str(), "resource_search");
        assert_eq!(
            serde_json::to_value(AddonResource::ResourceSearch).unwrap(),
            serde_json::json!("resource_search")
        );
        assert_eq!(
            serde_json::from_value::<AddonResource>(serde_json::json!("resource_search")).unwrap(),
            AddonResource::ResourceSearch
        );

        assert_eq!(
            AddonScope::AcquisitionSearchRead.as_str(),
            "acquisition_search_read"
        );
        assert_eq!(
            serde_json::to_value(AddonScope::AcquisitionSearchRead).unwrap(),
            serde_json::json!("acquisition_search_read")
        );
        assert_eq!(
            serde_json::from_value::<AddonScope>(serde_json::json!("acquisition_search_read"))
                .unwrap(),
            AddonScope::AcquisitionSearchRead
        );

        assert_eq!(AddonResourceLinkType::OneOneFive.as_str(), "115");
        assert_eq!(
            serde_json::to_value(AddonResourceLinkType::OneTwoThree).unwrap(),
            serde_json::json!("123")
        );
        assert_eq!(AddonResourceSearchProviderStatus::Error.as_str(), "error");
        assert_eq!(
            AddonResourceSearchProviderFinality::Partial.as_str(),
            "partial"
        );
    }

    #[test]
    fn resource_search_manifest_requires_read_scope() {
        let manifest = resource_search_manifest();

        validate_manifest(&manifest).unwrap();
        ensure_scope_grant(
            &manifest,
            AddonResource::ResourceSearch,
            &[AddonScope::AcquisitionSearchRead],
        )
        .unwrap();

        assert!(matches!(
            ensure_scope_grant(&manifest, AddonResource::ResourceSearch, &[]),
            Err(AddonManifestError::MissingDeclaredScope {
                resource: AddonResource::ResourceSearch,
                scope: AddonScope::AcquisitionSearchRead,
            })
        ));
    }

    #[test]
    fn resource_search_payload_contracts_round_trip() {
        let request = AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: AddonResourceSearchIntent::MediaTitle {
                title: "Demo Movie".to_owned(),
                year: Some(2026),
                media_kind: Some("movie".to_owned()),
            },
            query: "Demo Movie 2026".to_owned(),
            limit: Some(20),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark, AddonResourceLinkType::Magnet],
            refresh: true,
            context: serde_json::json!({"library_id": "library-1"}),
        };
        let request_json = serde_json::to_value(&request).unwrap();
        assert_eq!(request_json["schema"], ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA);
        assert_eq!(request_json["intent"]["kind"], "media_title");
        assert_eq!(request_json["intent"]["year"], 2026);
        assert_eq!(
            request_json["link_types"],
            serde_json::json!(["quark", "magnet"])
        );
        assert_eq!(
            serde_json::from_value::<AddonResourceSearchRequest>(request_json).unwrap(),
            request
        );

        let link = AddonResourceLink {
            url: "magnet:?xt=urn:btih:ABCDEF".to_owned(),
            normalized_url: "magnet:?xt=urn:btih:abcdef".to_owned(),
            link_type: AddonResourceLinkType::Magnet,
            source: "pansou:movies".to_owned(),
            password: Some("secret-code".to_owned()),
            note: Some("disc 1".to_owned()),
        };
        let mut merged_by_type = BTreeMap::new();
        merged_by_type.insert(
            AddonResourceLinkType::Magnet,
            vec![AddonMergedResourceLink {
                url: link.url.clone(),
                normalized_url: link.normalized_url.clone(),
                link_type: AddonResourceLinkType::Magnet,
                password: Some("secret-code".to_owned()),
                note: Some("merged".to_owned()),
                sources: vec!["pansou:movies".to_owned()],
            }],
        );
        let response = AddonResourceSearchResponse {
            schema: ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned(),
            query: "Demo Movie 2026".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie 2026".to_owned(),
            },
            total: 1,
            results: vec![AddonResourceSearchResult {
                id: "result-1".to_owned(),
                title: "Demo Movie".to_owned(),
                source: "pansou:movies".to_owned(),
                content: Some("resource candidate".to_owned()),
                links: vec![link],
                tags: vec!["demo".to_owned()],
                images: Vec::new(),
                score: 700,
            }],
            merged_by_type,
            provider_executions: vec![AddonResourceSearchProviderExecution {
                provider_id: "pansou_compatible".to_owned(),
                status: AddonResourceSearchProviderStatus::Ok,
                result_count: 1,
                finality: AddonResourceSearchProviderFinality::Complete,
                safe_message: None,
            }],
        };
        let response_json = serde_json::to_value(&response).unwrap();

        assert_eq!(
            response_json["schema"],
            ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA
        );
        assert_eq!(
            response_json["merged_by_type"]["magnet"][0]["normalized_url"],
            "magnet:?xt=urn:btih:abcdef"
        );
        assert_eq!(
            response_json["provider_executions"][0]["finality"],
            "complete"
        );
        let manifest = resource_search_manifest();
        let envelope = AddonResourceResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: manifest.id.clone(),
            resource: AddonResource::ResourceSearch,
            request_id: "resource-search-1".to_owned(),
            payload: response_json.clone(),
            artifacts: Vec::new(),
        };
        validate_resource_response(
            &envelope,
            &manifest,
            AddonResource::ResourceSearch,
            "resource-search-1",
        )
        .unwrap();
        let debug = format!("{response:?}");
        for forbidden in ["secret-code", "ABCDEF", "abcdef"] {
            assert!(
                !debug.contains(forbidden),
                "resource search debug leaked forbidden term: {forbidden}"
            );
        }
        assert_eq!(
            serde_json::from_value::<AddonResourceSearchResponse>(response_json).unwrap(),
            response
        );
    }

    #[test]
    fn renderer_adapter_payload_contracts_round_trip_and_redact_debug() {
        let target = AddonRendererAdapterTarget {
            stable_device_id: "living-room-tv".to_owned(),
            target_kind: AddonRendererAdapterProtocol::Chromecast,
            display_name: "Living Room TV".to_owned(),
            network_scope: AddonRendererAdapterNetworkScope::Local,
            media_capabilities: AddonRendererAdapterMediaCapabilities {
                direct_play: true,
                containers: vec!["mp4".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned()],
            },
            control_capabilities: AddonRendererAdapterControlCapabilities::basic_playback(),
            discovered_at_ms: Some(1_779_814_400_000),
        };
        let targets = AddonRendererAdapterResponse::Targets {
            targets: vec![target],
        };
        assert_eq!(
            serde_json::to_value(&targets).unwrap(),
            serde_json::json!({
                "kind": "targets",
                "targets": [{
                    "stable_device_id": "living-room-tv",
                    "target_kind": "chromecast",
                    "display_name": "Living Room TV",
                    "network_scope": "local",
                    "media_capabilities": {
                        "direct_play": true,
                        "containers": ["mp4"],
                        "video_codecs": ["h264"],
                        "audio_codecs": ["aac"]
                    },
                    "control_capabilities": {
                        "play": true,
                        "pause": true,
                        "resume": true,
                        "seek": true,
                        "stop": true,
                        "set_volume": false
                    },
                    "discovered_at_ms": 1779814400000u64
                }]
            })
        );

        let envelope = AddonRendererAdapterCommandEnvelope {
            adapter_id: "nako.official.chromecast-renderer".to_owned(),
            stable_device_id: "living-room-tv".to_owned(),
            target_kind: AddonRendererAdapterProtocol::Chromecast,
            renderer_session_id: "018f0000-0000-7000-8000-000000000010".to_owned(),
            playback_session_id: "018f0000-0000-7000-8000-000000000011".to_owned(),
            source_id: "018f0000-0000-7000-8000-000000000012".to_owned(),
            command: AddonRendererAdapterCommand::Play,
            position_ms: Some(9_000),
            volume_percent: None,
            transport: AddonRendererAdapterTransport {
                mode: AddonRendererAdapterTransportMode::Direct,
                expires_at: "2026-05-27T12:00:00.000Z".to_owned(),
                urls: vec![AddonRendererAdapterTransportUrl {
                    kind: AddonRendererAdapterTransportUrlKind::Stream,
                    url: "https://nako.example/playback/stream?renderer_ticket=nako_rtt_secret"
                        .to_owned(),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                }],
            },
        };
        let request = AddonRendererAdapterRequest::DispatchCommand {
            protocol: AddonRendererAdapterProtocol::Chromecast,
            envelope,
        };
        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            serde_json::from_str::<AddonRendererAdapterRequest>(&json).unwrap(),
            request
        );
        assert!(json.contains("renderer_ticket=nako_rtt_secret"));

        let debug = format!("{request:?}").to_ascii_lowercase();
        for forbidden in ["renderer_ticket", "nako_rtt_secret", "bearer", "local://"] {
            assert!(
                !debug.contains(forbidden),
                "renderer adapter debug leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn health_check_envelopes_validate_manifest_facts() {
        let manifest = valid_manifest();
        let request = AddonHealthCheckRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: manifest.id.clone(),
            request_id: "health-1".to_owned(),
            expected_addon_version: manifest.version.clone(),
            expected_resource_count: manifest.resources.len(),
        };
        let response = AddonHealthCheckResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            manifest_id: manifest.id.clone(),
            status: AddonHealthStatus::Ok,
            checked_at: "2026-05-21T12:00:00.000Z".to_owned(),
            manifest: AddonHealthManifestFacts {
                addon_version: manifest.version.clone(),
                resource_count: manifest.resources.len(),
            },
            diagnostics: serde_json::json!({"safe_note": "ok"}),
        };

        assert_eq!(
            serde_json::from_str::<AddonHealthCheckRequest>(
                &serde_json::to_string(&request).unwrap()
            )
            .unwrap(),
            request
        );
        validate_health_check_response(&response, &manifest).unwrap();

        let mut invalid = response;
        invalid.manifest.resource_count += 1;
        assert!(matches!(
            validate_health_check_response(&invalid, &manifest),
            Err(AddonManifestError::InvalidEnvelope { .. })
        ));
    }

    #[test]
    fn protected_write_payload_contracts_keep_wire_shape() {
        let metadata = AddonMetadataPatch {
            title: Some("Demo".to_owned()),
            genres: Some(vec!["Drama".to_owned()]),
            ..AddonMetadataPatch::default()
        };
        assert_eq!(
            serde_json::to_value(&metadata).unwrap(),
            serde_json::json!({
                "title": "Demo",
                "original_title": null,
                "sort_title": null,
                "overview": null,
                "release_date": null,
                "runtime_minutes": null,
                "tagline": null,
                "genres": ["Drama"],
                "tags": null,
                "ratings": null,
                "images": null,
                "credits": null,
                "collections": null,
                "studios": null,
                "external_ids": null
            })
        );

        let artwork = AddonArtworkWritePayload {
            intent: AddonArtworkIntent::ProposeArtwork,
            kind: AddonArtworkKind::Poster,
            source: AddonArtworkSourcePayload {
                kind: AddonArtworkSourceKind::RemoteUrl,
                url: "https://addon.example/poster.jpg".to_owned(),
            },
            language: Some("en".to_owned()),
            width: Some(1000),
            height: Some(1500),
        };
        assert_eq!(
            serde_json::to_value(&artwork).unwrap(),
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "https://addon.example/poster.jpg"
                },
                "language": "en",
                "width": 1000,
                "height": 1500
            })
        );

        let file_write = AddonLibraryFileWritePayload {
            file_role: AddonLibraryFileRole::Nfo,
            policy: AddonLibraryFileWritePolicy::ReplaceExistingPreserving,
        };
        assert_eq!(
            serde_json::to_value(&file_write).unwrap(),
            serde_json::json!({
                "file_role": "nfo",
                "policy": "replace_existing_preserving"
            })
        );
    }

    #[test]
    fn addon_install_descriptor_generates_redacted_install_guide() {
        let mut manifest = valid_manifest();
        manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration {
            id: "metadata_api_key".to_owned(),
            label: "Metadata API key".to_owned(),
            description: Some("Resolved by Nako at runtime".to_owned()),
            required: true,
        }];
        manifest.tasks = vec![AddonTaskDeclaration {
            id: "bulk-metadata-scrape".to_owned(),
            name: "Bulk metadata scrape".to_owned(),
            path: "/tasks/bulk-metadata-scrape".to_owned(),
            description: None,
            required_scopes: vec![AddonScope::AutomationRun],
            timeout_ms: Some(30_000),
            max_attempts: Some(2),
        }];
        manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration {
            id: "library-scan-finished".to_owned(),
            event_kind: "library_scan.succeeded".to_owned(),
            path: "/events/library-scan-finished".to_owned(),
            required_scopes: vec![AddonScope::WebhookEventRead],
            filters: serde_json::json!({ "library_preset": "movies" }),
        }];
        manifest
            .scopes
            .extend([AddonScope::AutomationRun, AddonScope::WebhookEventRead]);
        let descriptor = AddonInstallDescriptor {
            manifest,
            runtime: AddonRuntimeRequirement {
                kind: AddonRuntimeKind::HttpSidecar,
                image: Some("ghcr.io/nako/example-metadata-addon:0.1.0".to_owned()),
                binary: None,
                command: None,
            },
            secret_reference_bindings: vec![AddonSecretReferenceBinding {
                field_id: "metadata_api_key".to_owned(),
                secret_ref: "env:NAKO_METADATA_ADDON_API_KEY".to_owned(),
            }],
            install_notes: vec!["Use a reverse proxy if exposing the sidecar remotely.".to_owned()],
        };

        validate_install_descriptor(&descriptor).unwrap();
        let guide = addon_install_guide(&descriptor);
        let body = serde_json::to_string(&guide).unwrap();

        assert_eq!(guide.manifest_id, "example");
        assert_eq!(guide.runtime_kind, AddonRuntimeKind::HttpSidecar);
        assert_eq!(
            guide.runtime_reference,
            AddonRuntimeReference {
                kind: AddonRuntimeReferenceKind::Image,
                value: "ghcr.io/nako/example-metadata-addon:0.1.0".to_owned(),
            }
        );
        assert_eq!(guide.base_url_scheme, "https");
        assert_eq!(guide.task_count, 1);
        assert_eq!(guide.event_subscription_count, 1);
        assert_eq!(guide.required_secret_fields[0].id, "metadata_api_key");
        assert!(guide.required_secret_fields[0].provided);
        assert!(guide.missing_required_secret_fields.is_empty());
        assert!(
            guide
                .install_steps
                .iter()
                .any(|step| step.kind == AddonInstallStepKind::ConfigureSecretReference)
        );
        assert!(!body.contains("NAKO_METADATA_ADDON_API_KEY="));
        assert!(!body.contains("secret-value"));
        assert!(!body.contains("Bearer "));
        assert!(!body.contains("C:\\"));
        assert!(!body.contains("file:///"));
    }

    #[test]
    fn addon_install_descriptor_rejects_secret_values_and_local_runtime_paths() {
        let mut descriptor = AddonInstallDescriptor {
            manifest: valid_manifest(),
            runtime: AddonRuntimeRequirement {
                kind: AddonRuntimeKind::HttpSidecar,
                image: None,
                binary: Some("C:\\addons\\metadata.exe".to_owned()),
                command: None,
            },
            secret_reference_bindings: Vec::new(),
            install_notes: Vec::new(),
        };
        assert!(matches!(
            validate_install_descriptor(&descriptor),
            Err(AddonManifestError::InvalidRuntimeReference)
        ));

        descriptor.runtime.binary = Some("C:/addons/metadata.exe".to_owned());
        assert!(matches!(
            validate_install_descriptor(&descriptor),
            Err(AddonManifestError::InvalidRuntimeReference)
        ));

        descriptor.runtime.binary = Some("nako-metadata-addon".to_owned());
        descriptor.runtime.command = Some("nako-metadata-addon --port 8080".to_owned());
        assert!(matches!(
            validate_install_descriptor(&descriptor),
            Err(AddonManifestError::InvalidRuntimeReference)
        ));

        descriptor.runtime.binary = Some("nako-metadata-addon".to_owned());
        descriptor.runtime.command = None;
        descriptor
            .manifest
            .secret_reference_fields
            .push(AddonSecretReferenceFieldDeclaration {
                id: "metadata_api_key".to_owned(),
                label: "Metadata API key".to_owned(),
                description: None,
                required: true,
            });
        descriptor.secret_reference_bindings = vec![AddonSecretReferenceBinding {
            field_id: "metadata_api_key".to_owned(),
            secret_ref: "secret-value-token".to_owned(),
        }];
        assert!(matches!(
            validate_install_descriptor(&descriptor),
            Err(AddonManifestError::SecretReferenceContainsValue { .. })
        ));

        descriptor.secret_reference_bindings = vec![AddonSecretReferenceBinding {
            field_id: "metadata_api_key".to_owned(),
            secret_ref: "env:NAKO_METADATA_ADDON_TOKEN".to_owned(),
        }];
        validate_install_descriptor(&descriptor).unwrap();
    }

    fn valid_manifest() -> AddonManifest {
        AddonManifest {
            id: "example".to_owned(),
            name: "Example".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            description: None,
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::Metadata,
                path: "/metadata".to_owned(),
                input_schema: Some("nako.metadata.request.v1".to_owned()),
                output_schema: Some("nako.metadata.response.v1".to_owned()),
                required_scopes: vec![
                    AddonScope::ItemMetadataRead,
                    AddonScope::ItemMetadataSuggest,
                ],
                timeout_ms: Some(5_000),
                max_attempts: Some(2),
            }],
            entry_points: Vec::new(),
            hosted_pages: Vec::new(),
            configuration_schema: None,
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::Bearer,
            default_timeout_ms: Some(10_000),
            default_max_attempts: Some(2),
            scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
        }
    }

    fn resource_search_manifest() -> AddonManifest {
        AddonManifest {
            id: "resource-search".to_owned(),
            name: "Resource Search".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            description: None,
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::ResourceSearch,
                path: "/resource-search".to_owned(),
                input_schema: Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![AddonScope::AcquisitionSearchRead],
                timeout_ms: Some(10_000),
                max_attempts: Some(1),
            }],
            entry_points: Vec::new(),
            hosted_pages: Vec::new(),
            configuration_schema: None,
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::Bearer,
            default_timeout_ms: Some(10_000),
            default_max_attempts: Some(1),
            scopes: vec![AddonScope::AcquisitionSearchRead],
        }
    }
}
