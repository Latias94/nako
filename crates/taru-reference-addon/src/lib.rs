use axum::{Json, Router, routing::post};
use taru_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonAuth, AddonConfigurationSchema,
    AddonEntryPointDeclaration, AddonEntryPointKind, AddonHealthCheckRequest,
    AddonHealthCheckResponse, AddonHealthManifestFacts, AddonHealthStatus,
    AddonHostedPageDeclaration, AddonLibraryFileRole, AddonLibraryFileWritePayload,
    AddonLibraryFileWritePolicy, AddonManifest, AddonMetadataPatch, AddonResource,
    AddonResourceDeclaration, AddonResourceRequest, AddonResourceResponse, AddonScope,
};

pub const REFERENCE_ADDON_ID: &str = "taru.reference.metadata";

#[must_use]
pub fn reference_manifest(base_url: impl Into<String>) -> AddonManifest {
    AddonManifest {
        id: REFERENCE_ADDON_ID.to_owned(),
        name: "Taru Reference Metadata Addon".to_owned(),
        version: "0.1.0".to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: base_url.into(),
        description: Some("Minimal metadata suggestion addon for Taru protocol tests".to_owned()),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Metadata,
            path: "/metadata".to_owned(),
            input_schema: Some("taru.metadata.request.v1".to_owned()),
            output_schema: Some("taru.metadata.response.v1".to_owned()),
            required_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            timeout_ms: Some(5_000),
            max_attempts: Some(2),
        }],
        entry_points: vec![AddonEntryPointDeclaration::hosted_page(
            "suggest-metadata",
            AddonEntryPointKind::ItemAction,
            "Suggest Metadata",
            "/ui/suggest-metadata",
            "diagnostics",
            vec![AddonScope::ItemMetadataSuggest],
        )],
        hosted_pages: vec![AddonHostedPageDeclaration {
            id: "diagnostics".to_owned(),
            title: "Reference Addon Diagnostics".to_owned(),
            path: "/ui/diagnostics".to_owned(),
            required_scopes: vec![AddonScope::ItemMetadataRead],
        }],
        configuration_schema: Some(AddonConfigurationSchema {
            schema_id: "taru.reference.metadata.config.v1".to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "preferred_language": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }),
        secret_reference_fields: Vec::new(),
        event_subscriptions: Vec::new(),
        tasks: Vec::new(),
        auth: AddonAuth::None,
        default_timeout_ms: Some(10_000),
        default_max_attempts: Some(2),
        scopes: vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
        ],
    }
}

#[must_use]
pub fn build_router() -> Router {
    Router::new()
        .route("/health", post(health))
        .route("/metadata", post(metadata))
}

#[must_use]
pub fn demo_metadata_patch(title: impl Into<String>) -> AddonMetadataPatch {
    AddonMetadataPatch {
        title: Some(title.into()),
        overview: Some("Reference addon metadata suggestion".to_owned()),
        tags: Some(vec!["reference-addon".to_owned()]),
        ..AddonMetadataPatch::default()
    }
}

#[must_use]
pub fn demo_nfo_export_payload() -> AddonLibraryFileWritePayload {
    AddonLibraryFileWritePayload {
        file_role: AddonLibraryFileRole::Nfo,
        policy: AddonLibraryFileWritePolicy::CreateMissing,
    }
}

async fn metadata(Json(request): Json<AddonResourceRequest>) -> Json<AddonResourceResponse> {
    let requested_title = request
        .payload
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown Title");
    let suggestion = serde_json::json!({
        "title": requested_title,
        "summary": "Reference addon metadata suggestion",
        "source": REFERENCE_ADDON_ID
    });

    Json(AddonResourceResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: request.addon_id,
        resource: request.resource,
        request_id: request.request_id,
        payload: suggestion.clone(),
        artifacts: vec![AddonArtifact {
            kind: "metadata_suggestion".to_owned(),
            payload: suggestion,
        }],
    })
}

async fn health(Json(request): Json<AddonHealthCheckRequest>) -> Json<AddonHealthCheckResponse> {
    Json(AddonHealthCheckResponse {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: request.manifest_id,
        status: AddonHealthStatus::Ok,
        checked_at: "2026-05-21T00:00:00.000Z".to_owned(),
        manifest: AddonHealthManifestFacts {
            addon_version: "0.1.0".to_owned(),
            resource_count: 1,
        },
        diagnostics: serde_json::json!({"fixture": "taru-reference-addon"}),
    })
}

#[cfg(test)]
mod tests {
    use taru_addon_protocol::{
        AddonLibraryFileRole, AddonLibraryFileWritePolicy, validate_manifest,
    };

    use super::*;

    #[test]
    fn reference_manifest_is_valid() {
        let manifest = reference_manifest("http://127.0.0.1:3000");

        validate_manifest(&manifest).unwrap();
        assert_eq!(manifest.entry_points[0].id, "suggest-metadata");
        assert_eq!(manifest.hosted_pages[0].id, "diagnostics");
        assert_eq!(
            manifest.configuration_schema.as_ref().unwrap().schema_id,
            "taru.reference.metadata.config.v1"
        );
    }

    #[test]
    fn reference_protected_write_payloads_match_protocol_contracts() {
        let patch = demo_metadata_patch("The Matrix");
        let patch_json = serde_json::to_value(&patch).unwrap();
        assert_eq!(patch_json["title"], "The Matrix");
        assert_eq!(patch_json["tags"][0], "reference-addon");

        let nfo = demo_nfo_export_payload();
        assert_eq!(nfo.file_role, AddonLibraryFileRole::Nfo);
        assert_eq!(nfo.policy, AddonLibraryFileWritePolicy::CreateMissing);
        assert_eq!(
            serde_json::to_value(&nfo).unwrap(),
            serde_json::json!({
                "file_role": "nfo",
                "policy": "create_missing"
            })
        );
    }
}
