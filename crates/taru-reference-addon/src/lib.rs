use axum::{Json, Router, routing::post};
use taru_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonArtifact, AddonAuth, AddonManifest, AddonResource,
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
    Router::new().route("/metadata", post(metadata))
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

#[cfg(test)]
mod tests {
    use taru_addon_protocol::validate_manifest;

    use super::*;

    #[test]
    fn reference_manifest_is_valid() {
        let manifest = reference_manifest("http://127.0.0.1:3000");

        validate_manifest(&manifest).unwrap();
    }
}
