use super::*;
use axum::Json;
use axum::http::HeaderValue;
use std::sync::{
    Arc as StdArc,
    atomic::{AtomicUsize, Ordering},
};

fn tiny_png() -> Vec<u8> {
    png_with_size(1, 1)
}

fn png_with_size(width: u32, height: u32) -> Vec<u8> {
    let image = image::RgbaImage::from_fn(width, height, |x, y| {
        image::Rgba([
            (x.saturating_mul(40) % 255) as u8,
            (y.saturating_mul(80) % 255) as u8,
            128,
            255,
        ])
    });
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    cursor.into_inner()
}

pub(super) async fn tiny_artwork_server() -> (String, u64) {
    artwork_server(StatusCode::OK, "image/png", tiny_png()).await
}

async fn health_check_addon_server(
    status: StatusCode,
    response_status: AddonHealthStatus,
    manifest_id: &'static str,
    include_secret_echo: bool,
) -> (String, StdArc<AtomicUsize>) {
    let requests = StdArc::new(AtomicUsize::new(0));
    let request_counter = StdArc::clone(&requests);
    let router = Router::new().route(
        "/health",
        axum::routing::post(
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonHealthCheckRequest>| {
                let request_counter = StdArc::clone(&request_counter);
                async move {
                    request_counter.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(
                        headers
                            .get("x-taru-addon-id")
                            .and_then(|value| value.to_str().ok()),
                        Some(manifest_id)
                    );
                    assert_eq!(
                        headers
                            .get("x-taru-addon-protocol-version")
                            .and_then(|value| value.to_str().ok()),
                        Some(ADDON_PROTOCOL_VERSION)
                    );
                    assert!(headers.get(header::AUTHORIZATION).is_none());
                    assert!(headers.get("x-taru-addon-secret").is_none());
                    assert_eq!(request.manifest_id, manifest_id);
                    assert_eq!(request.protocol_version, ADDON_PROTOCOL_VERSION);

                    let diagnostics = if include_secret_echo {
                        serde_json::json!({
                            "safe_note": "ok",
                            "raw_network_error": "Bearer taru_at_should_not_echo"
                        })
                    } else {
                        serde_json::json!({ "safe_note": "ok" })
                    };

                    (
                        status,
                        Json(ProtocolAddonHealthCheckResponse {
                            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                            manifest_id: manifest_id.to_owned(),
                            status: response_status,
                            checked_at: "2026-05-21T12:00:00.000Z".to_owned(),
                            manifest: AddonHealthManifestFacts {
                                addon_version: "0.1.0".to_owned(),
                                resource_count: 1,
                            },
                            diagnostics,
                        }),
                    )
                }
            },
        ),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn failing_resource_addon_server(status: StatusCode, body: &'static str) -> String {
    let router = Router::new().route(
        "/metadata",
        axum::routing::post(move || async move { (status, body) }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    format!("http://{addr}")
}

async fn raw_health_addon_server(status: StatusCode, body: &'static str) -> String {
    let router = Router::new().route(
        "/health",
        axum::routing::post(move || async move { (status, body) }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    format!("http://{addr}")
}

async fn mismatched_health_addon_server(
    manifest_id: &'static str,
    version: &'static str,
) -> String {
    let router = Router::new().route(
        "/health",
        axum::routing::post(move || async move {
            Json(ProtocolAddonHealthCheckResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                manifest_id: manifest_id.to_owned(),
                status: AddonHealthStatus::Ok,
                checked_at: "2026-05-21T12:00:00.000Z".to_owned(),
                manifest: AddonHealthManifestFacts {
                    addon_version: version.to_owned(),
                    resource_count: 1,
                },
                diagnostics: serde_json::json!({
                    "raw_network_error": "Bearer taru_at_should_not_echo"
                }),
            })
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    format!("http://{addr}")
}

async fn artwork_server(
    status: StatusCode,
    content_type: &'static str,
    bytes: Vec<u8>,
) -> (String, u64) {
    let byte_len = bytes.len() as u64;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let served_bytes = bytes.clone();
    let router = Router::new().route(
        "/poster.png",
        axum::routing::get(move || {
            let bytes = served_bytes.clone();
            async move { (status, [(header::CONTENT_TYPE, content_type)], bytes) }
        }),
    );
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}/poster.png?token=secret"), byte_len)
}

async fn changing_artwork_server(
    first_status: StatusCode,
    first_content_type: &'static str,
    first_bytes: Vec<u8>,
    second_status: StatusCode,
    second_content_type: &'static str,
    second_bytes: Vec<u8>,
) -> (String, u64) {
    let second_byte_len = second_bytes.len() as u64;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = StdArc::new(AtomicUsize::new(0));
    let router = Router::new().route(
        "/poster.png",
        axum::routing::get({
            let requests = StdArc::clone(&requests);
            move || {
                let request_index = requests.fetch_add(1, Ordering::SeqCst);
                let bytes = if request_index == 0 {
                    first_bytes.clone()
                } else {
                    second_bytes.clone()
                };
                let status = if request_index == 0 {
                    first_status
                } else {
                    second_status
                };
                let content_type = if request_index == 0 {
                    first_content_type
                } else {
                    second_content_type
                };
                async move { (status, [(header::CONTENT_TYPE, content_type)], bytes) }
            }
        }),
    );
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (
        format!("http://{addr}/poster.png?token=secret"),
        second_byte_len,
    )
}

pub(super) async fn propose_and_accept_remote_artwork(
    router: &Router,
    library_id: LibraryId,
    item_id: MediaItemId,
    remote_url: &str,
    idempotency_key: &str,
) -> (
    String,
    taru_core::ArtworkCandidateId,
    AcceptManagedArtworkCandidateResponse,
) {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: item_id.to_string(),
        },
        idempotency_key: idempotency_key.to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "token": issued.raw_token.clone()
        }),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": remote_url
            },
            "language": "en"
        }),
    };
    let proposed = addon_side_effect(router, Some(&issued.raw_token), &request).await;
    assert_eq!(proposed.status(), StatusCode::OK);
    let proposed = body_json::<AddonSideEffectResponse>(proposed).await;
    let candidate_id: taru_core::ArtworkCandidateId =
        proposed.side_effect.apply_report.as_ref().unwrap()["candidate_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
    let accept_path = format!("/admin/v1/artwork/candidates/{candidate_id}/accept");
    let accepted =
        request_json::<AcceptManagedArtworkCandidateResponse>(router, Method::POST, &accept_path)
            .await;

    (issued.raw_token, candidate_id, accepted)
}

async fn register_artwork_addon(router: &Router, library_id: LibraryId) -> String {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    issued.raw_token
}

async fn propose_and_accept_remote_artwork_with_token(
    router: &Router,
    library_id: LibraryId,
    item_id: MediaItemId,
    remote_url: &str,
    idempotency_key: &str,
    raw_token: &str,
) -> (
    taru_core::ArtworkCandidateId,
    AcceptManagedArtworkCandidateResponse,
) {
    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: item_id.to_string(),
        },
        idempotency_key: idempotency_key.to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "token": raw_token
        }),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": remote_url
            },
            "language": "en"
        }),
    };
    let proposed = addon_side_effect(router, Some(raw_token), &request).await;
    assert_eq!(proposed.status(), StatusCode::OK);
    let proposed = body_json::<AddonSideEffectResponse>(proposed).await;
    let candidate_id: taru_core::ArtworkCandidateId =
        proposed.side_effect.apply_report.as_ref().unwrap()["candidate_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
    let accept_path = format!("/admin/v1/artwork/candidates/{candidate_id}/accept");
    let accepted =
        request_json::<AcceptManagedArtworkCandidateResponse>(router, Method::POST, &accept_path)
            .await;

    (candidate_id, accepted)
}

#[tokio::test]
async fn register_addon_routes_disabled_by_default_and_validate_contract() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = addon_manifest();

    let legacy_register = post_legacy_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: manifest.clone(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(legacy_register.status(), StatusCode::NOT_FOUND);

    let response = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: manifest.clone(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: None,
        },
    )
    .await;

    assert_eq!(response.addon.summary.manifest_id, manifest.id);
    assert_eq!(response.addon.summary.status, AddonStatus::Disabled);
    assert_eq!(
        response.addon.summary.granted_scopes,
        vec!["item_metadata_suggest", "item_metadata_read"]
    );
    let response_json = serde_json::to_value(&response).unwrap();
    assert!(response_json["addon"].get("manifest_json").is_none());
    assert!(!response_json.to_string().contains("token"));

    let disabled = request_json::<AdminAddonRegistrationsResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons?status=disabled",
    )
    .await;
    assert_eq!(disabled.addons, vec![response.addon.summary.clone()]);

    let enabled = request_json::<AdminAddonRegistrationsResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons?status=enabled",
    )
    .await;
    assert!(enabled.addons.is_empty());

    let detail_path = format!("/admin/v1/addons/{}", response.addon.summary.id);
    let detail =
        request_json::<AdminAddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    assert_eq!(detail, response);

    let legacy_list = response_for(&router, Method::GET, "/addons").await;
    assert_eq!(legacy_list.status(), StatusCode::NOT_FOUND);

    let legacy_detail = response_for(
        &router,
        Method::GET,
        &format!("/addons/{}", response.addon.summary.id),
    )
    .await;
    assert_eq!(legacy_detail.status(), StatusCode::NOT_FOUND);

    let mut invalid_manifest = addon_manifest();
    invalid_manifest.resources[0].path = "metadata".to_owned();
    let invalid = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: invalid_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let invalid_error = body_json::<ErrorResponse>(invalid).await;
    assert_eq!(invalid_error.code, "invalid_input");

    let missing_scope = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![AddonScope::ItemMetadataRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(missing_scope.status(), StatusCode::BAD_REQUEST);
    let missing_scope_error = body_json::<ErrorResponse>(missing_scope).await;
    assert_eq!(missing_scope_error.code, "invalid_input");
}

#[tokio::test]
async fn admin_addon_install_guide_preview_redacts_package_and_secret_material() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = addon_manifest();
    manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration::new(
        "metadata_api_key",
        "Metadata API key",
        Some("Resolved by Taru at runtime".to_owned()),
        true,
    )];
    manifest.tasks = vec![AddonTaskDeclaration::new(
        "bulk-metadata-scrape",
        "Bulk Metadata Scrape",
        "/tasks/bulk-metadata-scrape",
        vec![AddonScope::ItemMetadataSuggest],
    )];
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scan-finished",
        "library_scan.succeeded",
        "/events/library-scan-finished",
        vec![AddonScope::ItemMetadataRead],
        serde_json::json!({"library_preset":"movies"}),
    )];
    let descriptor = AddonInstallDescriptor {
        manifest,
        runtime: AddonRuntimeRequirement {
            kind: AddonRuntimeKind::HttpSidecar,
            image: Some("ghcr.io/taru/example-metadata-addon:0.1.0".to_owned()),
            binary: None,
            command: None,
        },
        secret_reference_bindings: vec![AddonSecretReferenceBinding {
            field_id: "metadata_api_key".to_owned(),
            secret_ref: "env:TARU_METADATA_ADDON_TOKEN".to_owned(),
        }],
        install_notes: vec![
            "Do not include TARU_METADATA_ADDON_TOKEN=secret-value in the guide".to_owned(),
        ],
    };

    let response = request_body_json::<AdminAddonInstallGuidePreviewResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons/install-guide-preview",
        &AdminAddonInstallGuidePreviewRequest { descriptor },
    )
    .await;
    let text = serde_json::to_string(&response).unwrap();

    assert_eq!(response.guide.manifest_id, "example.metadata");
    assert_eq!(response.guide.runtime_kind, AddonRuntimeKind::HttpSidecar);
    assert_eq!(
        response.guide.runtime_reference.value,
        "ghcr.io/taru/example-metadata-addon:0.1.0"
    );
    assert_eq!(response.guide.required_secret_fields.len(), 1);
    assert!(response.guide.required_secret_fields[0].provided);
    assert!(response.guide.missing_required_secret_fields.is_empty());
    assert_eq!(response.guide.task_count, 1);
    assert_eq!(response.guide.event_subscription_count, 1);
    assert!(!text.contains("secret-value"));
    assert!(!text.contains("TARU_METADATA_ADDON_TOKEN="));
    assert!(!text.contains("Bearer "));
    assert!(!text.contains("taru_at_"));
    assert!(!text.contains("C:\\"));
    assert!(!text.contains("file:///"));
}

#[tokio::test]
async fn admin_addon_install_guide_preview_rejects_raw_secret_and_local_runtime_paths() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = addon_manifest();
    manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration::new(
        "metadata_api_key",
        "Metadata API key",
        Some("Resolved by Taru at runtime".to_owned()),
        true,
    )];
    let mut descriptor = AddonInstallDescriptor {
        manifest,
        runtime: AddonRuntimeRequirement {
            kind: AddonRuntimeKind::HttpSidecar,
            image: None,
            binary: Some("C:\\addons\\metadata.exe".to_owned()),
            command: None,
        },
        secret_reference_bindings: Vec::new(),
        install_notes: Vec::new(),
    };

    let local_runtime = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons/install-guide-preview",
        &AdminAddonInstallGuidePreviewRequest {
            descriptor: descriptor.clone(),
        },
    )
    .await;
    assert_eq!(local_runtime.status(), StatusCode::BAD_REQUEST);
    let local_runtime_text =
        serde_json::to_string(&body_json::<ErrorResponse>(local_runtime).await).unwrap();
    assert!(!local_runtime_text.contains("C:\\addons\\metadata.exe"));

    descriptor.runtime.binary = Some("taru-metadata-addon".to_owned());
    descriptor.secret_reference_bindings = vec![AddonSecretReferenceBinding {
        field_id: "metadata_api_key".to_owned(),
        secret_ref: "metadata-secret-token-value".to_owned(),
    }];
    let raw_secret = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons/install-guide-preview",
        &AdminAddonInstallGuidePreviewRequest { descriptor },
    )
    .await;
    assert_eq!(raw_secret.status(), StatusCode::BAD_REQUEST);
    let raw_secret_text =
        serde_json::to_string(&body_json::<ErrorResponse>(raw_secret).await).unwrap();
    assert!(!raw_secret_text.contains("metadata-secret-token-value"));

    let mut descriptor = AddonInstallDescriptor {
        manifest: addon_manifest(),
        runtime: AddonRuntimeRequirement {
            kind: AddonRuntimeKind::HttpSidecar,
            image: Some("ghcr.io/taru/example-metadata-addon:0.1.0".to_owned()),
            binary: None,
            command: None,
        },
        secret_reference_bindings: Vec::new(),
        install_notes: Vec::new(),
    };
    descriptor.manifest.resources[0].path = "C:\\secret\\metadata".to_owned();
    let invalid_manifest = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons/install-guide-preview",
        &AdminAddonInstallGuidePreviewRequest { descriptor },
    )
    .await;
    assert_eq!(invalid_manifest.status(), StatusCode::BAD_REQUEST);
    let invalid_manifest_text =
        serde_json::to_string(&body_json::<ErrorResponse>(invalid_manifest).await).unwrap();
    assert!(!invalid_manifest_text.contains("C:\\secret\\metadata"));
}

#[tokio::test]
async fn admin_addon_status_patch_enables_and_disables_runtime_access() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: None,
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    assert_eq!(registered.addon.summary.status, AddonStatus::Disabled);

    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let disabled_runtime = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(disabled_runtime.status(), StatusCode::FORBIDDEN);
    let tokens_after_disabled_attempt =
        request_json::<AddonTokensResponse>(&router, Method::GET, &token_path).await;
    assert_eq!(tokens_after_disabled_attempt.tokens.len(), 1);
    assert!(
        tokens_after_disabled_attempt.tokens[0]
            .last_used_at
            .is_none()
    );

    let enabled = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::PATCH,
        &format!("/admin/v1/addons/{addon_id}/status"),
        &UpdateAddonStatusRequest {
            status: AddonStatus::Enabled,
        },
    )
    .await;
    assert_eq!(enabled.addon.summary.status, AddonStatus::Enabled);
    assert_eq!(enabled.addon.manifest.id, registered.addon.manifest.id);
    let enabled_json = serde_json::to_value(&enabled).unwrap();
    assert!(enabled_json["addon"].get("manifest_json").is_none());
    assert!(!enabled_json.to_string().contains(&issued.raw_token));
    assert!(!enabled_json.to_string().contains("token_hash"));

    let enabled_filter = request_json::<AdminAddonRegistrationsResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons?status=enabled",
    )
    .await;
    assert_eq!(enabled_filter.addons.len(), 1);
    assert_eq!(enabled_filter.addons[0].id, addon_id);

    let allowed_runtime = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(allowed_runtime.status(), StatusCode::OK);

    let disabled = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::PATCH,
        &format!("/admin/v1/addons/{addon_id}/status"),
        &UpdateAddonStatusRequest {
            status: AddonStatus::Disabled,
        },
    )
    .await;
    assert_eq!(disabled.addon.summary.status, AddonStatus::Disabled);

    let disabled_again = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(disabled_again.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_addon_unregister_revokes_tokens_clears_grants_and_preserves_audit() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let allowed_before_unregister = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(allowed_before_unregister.status(), StatusCode::OK);

    let unregistered = request_json::<AdminAddonRegistrationResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/unregister"),
    )
    .await;
    assert_eq!(unregistered.addon.summary.status, AddonStatus::Unregistered);
    assert_eq!(unregistered.addon.summary.id, addon_id);
    assert_eq!(unregistered.addon.manifest.id, registered.addon.manifest.id);
    let unregistered_json = serde_json::to_value(&unregistered).unwrap();
    assert!(unregistered_json["addon"].get("manifest_json").is_none());
    assert!(!unregistered_json.to_string().contains(&issued.raw_token));
    assert!(!unregistered_json.to_string().contains("token_hash"));

    let detail_path = format!("/admin/v1/addons/{addon_id}");
    let detail =
        request_json::<AdminAddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    assert_eq!(detail.addon.summary.status, AddonStatus::Unregistered);

    let tokens = request_json::<AddonTokensResponse>(&router, Method::GET, &token_path).await;
    assert_eq!(tokens.tokens.len(), 1);
    assert_eq!(tokens.tokens[0].id, issued.token.id);
    assert_eq!(tokens.tokens[0].status, AddonTokenStatus::Revoked);
    assert!(tokens.tokens[0].revoked_at.is_some());

    let grants = request_json::<AddonGrantsResponse>(&router, Method::GET, &grants_path).await;
    assert!(grants.grants.is_empty());

    let runtime_after_unregister = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(runtime_after_unregister.status(), StatusCode::UNAUTHORIZED);

    let enable_after_unregister = response_body_json(
        &router,
        Method::PATCH,
        &format!("/admin/v1/addons/{addon_id}/status"),
        &UpdateAddonStatusRequest {
            status: AddonStatus::Enabled,
        },
    )
    .await;
    assert_eq!(enable_after_unregister.status(), StatusCode::CONFLICT);

    let re_register_without_new_id = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: registered.addon.manifest.clone(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: None,
        },
    )
    .await;
    assert_eq!(re_register_without_new_id.status(), StatusCode::OK);
    let re_registered_without_new_id =
        body_json::<AdminAddonRegistrationResponse>(re_register_without_new_id).await;
    assert_ne!(re_registered_without_new_id.addon.summary.id, addon_id);
    assert_eq!(
        re_registered_without_new_id.addon.summary.status,
        AddonStatus::Disabled
    );

    let re_register_enabled = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: Some(taru_core::AddonId::new()),
            manifest: registered.addon.manifest.clone(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(re_register_enabled.status(), StatusCode::BAD_REQUEST);

    let delete_unregistered = response_for(&router, Method::DELETE, &detail_path).await;
    assert_eq!(delete_unregistered.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn register_addon_routes_accept_manifest_declarations_and_reject_invalid_ones() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = addon_manifest();
    manifest.entry_points = vec![AddonEntryPointDeclaration::hosted_page(
        "metadata-action",
        AddonEntryPointKind::ItemAction,
        "Suggest Metadata",
        "/ui/metadata-action",
        "diagnostics",
        vec![AddonScope::ItemMetadataSuggest],
    )];
    manifest.hosted_pages = vec![AddonHostedPageDeclaration {
        id: "diagnostics".to_owned(),
        title: "Addon Diagnostics".to_owned(),
        path: "/ui/diagnostics".to_owned(),
        required_scopes: vec![AddonScope::ItemMetadataRead],
    }];
    manifest.configuration_schema = Some(AddonConfigurationSchema {
        schema_id: "taru.example.metadata.config.v1".to_owned(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "language": { "type": "string" }
            },
            "additionalProperties": false
        }),
    });
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

    let response = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: manifest.clone(),
            granted_scopes: manifest.scopes.clone(),
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let stored_manifest = response.addon.manifest;
    assert_eq!(stored_manifest.entry_points[0].id, "metadata-action");
    assert_eq!(stored_manifest.hosted_pages[0].id, "diagnostics");
    assert_eq!(stored_manifest.tasks[0].id, "bulk-metadata-scrape");
    assert_eq!(
        stored_manifest
            .configuration_schema
            .as_ref()
            .unwrap()
            .schema_id,
        "taru.example.metadata.config.v1"
    );

    let mut invalid_manifest = manifest.clone();
    invalid_manifest.tasks[0].required_scopes = vec![AddonScope::RecommendationWrite];
    let invalid = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: invalid_manifest,
            granted_scopes: manifest.scopes,
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let invalid_error = body_json::<ErrorResponse>(invalid).await;
    assert_eq!(invalid_error.code, "invalid_input");
    assert!(invalid_error.message.contains("task"));
    assert!(invalid_error.message.contains("recommendation_write"));
}

#[tokio::test]
async fn reference_addon_registers_queries_and_handles_resource_call() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addon_base_url = format!("http://{}", listener.local_addr().unwrap());
    let addon_server = tokio::spawn(async move {
        axum::serve(listener, taru_reference_addon::build_router())
            .await
            .unwrap();
    });
    yield_now().await;

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = taru_reference_addon::reference_manifest(addon_base_url);

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(registered.addon.summary.status, AddonStatus::Enabled);
    assert_eq!(
        registered.addon.summary.manifest_id,
        taru_reference_addon::REFERENCE_ADDON_ID
    );

    let detail_path = format!("/admin/v1/addons/{}", registered.addon.summary.id);
    let detail =
        request_json::<AdminAddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    let stored_manifest = detail.addon.manifest;
    let granted_scopes = [
        AddonScope::ItemMetadataRead,
        AddonScope::ItemMetadataSuggest,
    ];

    let response = call_addon_resource(
        &ReqwestAddonTransport::default(),
        &stored_manifest,
        AddonResource::Metadata,
        &granted_scopes,
        "reference-addon-e2e-1",
        serde_json::json!({"title":"The Matrix"}),
        None,
    )
    .await
    .unwrap();

    assert_eq!(response.payload["title"], "The Matrix");
    assert_eq!(
        response.payload["source"],
        taru_reference_addon::REFERENCE_ADDON_ID
    );
    assert_eq!(response.artifacts[0].kind, "metadata_suggestion");

    addon_server.abort();
}

#[tokio::test]
async fn admin_addon_health_check_reports_safe_reachability_without_tokens() {
    let (addon_base_url, requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "taru.health.metadata",
        true,
    )
    .await;
    let mut manifest = taru_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "taru.health.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;

    let path = format!("/admin/v1/addons/{addon_id}/health-check");
    let response =
        request_json::<AdminAddonHealthCheckResponse>(&router, Method::POST, &path).await;

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.manifest_id, "taru.health.metadata");
    assert_eq!(response.status, AdminAddonHealthCheckStatus::Reachable);
    assert_eq!(
        response.protocol_version.as_deref(),
        Some(ADDON_PROTOCOL_VERSION)
    );
    assert_eq!(response.addon_version.as_deref(), Some("0.1.0"));
    assert_eq!(response.resource_count, Some(1));
    assert!(response.safe_error_code.is_none());
    assert_eq!(
        response.protocol_checked_at.as_deref(),
        Some("2026-05-21T12:00:00.000Z")
    );
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    let raw = response_for(&router, Method::POST, &path).await;
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(!text.contains("taru_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
    assert!(!text.contains("authorization"));
}

#[tokio::test]
async fn admin_addon_health_check_classifies_unreachable_without_raw_error_leak() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let unreachable_base_url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);

    let mut manifest = taru_reference_addon::reference_manifest(unreachable_base_url);
    manifest.id = "taru.unreachable.metadata".to_owned();
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/health-check");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonHealthCheckResponse>(&text).unwrap();

    assert_eq!(response.status, AdminAddonHealthCheckStatus::Unreachable);
    assert_eq!(
        response.safe_error_code.as_deref(),
        Some("transport_failure")
    );
    assert!(response.protocol_version.is_none());
    assert!(!text.contains("Connection refused"));
    assert!(!text.contains("os error"));
    assert!(!text.contains("127.0.0.1"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_reports_ready_sidecar_without_token_or_payload_echo() {
    let (addon_base_url, requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "taru.ready.metadata",
        true,
    )
    .await;
    let mut manifest = taru_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "taru.ready.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/runtime-readiness");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.manifest_id, "taru.ready.metadata");
    assert_eq!(
        response.readiness.status,
        AdminAddonRuntimeReadinessStatus::Ready
    );
    assert_eq!(
        response.readiness.reason,
        AdminAddonRuntimeReadinessReason::Ready
    );
    assert!(response.readiness.checks.iter().any(|check| {
        check.name == taru_api::extension::AdminAddonRuntimeReadinessCheckName::Reachability
            && check.status == AdminAddonRuntimeReadinessStatus::Ready
    }));
    assert!(response.readiness.checks.iter().any(|check| {
        check.name == taru_api::extension::AdminAddonRuntimeReadinessCheckName::Network
            && check.status == AdminAddonRuntimeReadinessStatus::Ready
    }));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(!text.contains("taru_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
    assert!(!text.contains("authorization"));
    assert!(!text.contains("127.0.0.1"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_preserves_sidecar_degraded_status() {
    let (addon_base_url, _) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Degraded,
        "taru.degraded.metadata",
        true,
    )
    .await;
    let mut manifest = taru_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "taru.degraded.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/runtime-readiness");

    let response =
        request_json::<AdminAddonRuntimeReadinessResponse>(&router, Method::POST, &path).await;

    assert_eq!(
        response.readiness.status,
        AdminAddonRuntimeReadinessStatus::Degraded
    );
    assert_eq!(
        response.readiness.reason,
        AdminAddonRuntimeReadinessReason::SidecarDegraded
    );
    assert!(
        response
            .readiness
            .checks
            .iter()
            .any(|check| { check.safe_error_code.as_deref() == Some("sidecar_degraded") })
    );
}

#[tokio::test]
async fn admin_addon_runtime_readiness_classifies_local_gaps_without_sidecar_call() {
    let (addon_base_url, requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "taru.gapped.metadata",
        true,
    )
    .await;
    let mut manifest = taru_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "taru.gapped.metadata".to_owned();
    manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration::new(
        "provider_token",
        "Provider token",
        Some("Resolved by Taru".to_owned()),
        true,
    )];

    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    let addon_id = taru_core::AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            protocol_version: manifest.protocol_version.clone(),
            base_url: manifest.base_url.clone(),
            manifest_json,
            granted_scopes: vec![AddonScope::ItemMetadataRead.as_str().to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let router = build_router(app);
    let path = format!("/admin/v1/addons/{addon_id}/runtime-readiness");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&text).unwrap();

    assert_eq!(
        response.readiness.status,
        AdminAddonRuntimeReadinessStatus::Degraded
    );
    assert!(response.readiness.checks.iter().any(|check| {
        check.reason == AdminAddonRuntimeReadinessReason::MissingGrant
            && check.safe_error_code.as_deref() == Some("missing_grant")
    }));
    assert!(response.readiness.checks.iter().any(|check| {
        check.reason == AdminAddonRuntimeReadinessReason::MissingSecretReference
            && check.safe_error_code.as_deref() == Some("missing_secret_reference")
    }));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert!(!text.contains("provider_token"));
    assert!(!text.contains("Provider token"));
    assert!(!text.contains("taru_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_classifies_network_policy_blockers_without_echoing_url() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest =
        taru_reference_addon::reference_manifest("http://user:secret@addon.example.test/base");
    manifest.id = "taru.blocked-network.metadata".to_owned();
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/runtime-readiness");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&text).unwrap();

    assert_eq!(
        response.readiness.status,
        AdminAddonRuntimeReadinessStatus::Unavailable
    );
    assert_eq!(
        response.readiness.reason,
        AdminAddonRuntimeReadinessReason::NetworkPolicyBlocked
    );
    assert!(
        response
            .readiness
            .checks
            .iter()
            .any(|check| { check.safe_error_code.as_deref() == Some("network_policy_blocked") })
    );
    assert!(!text.contains("addon.example.test"));
    assert!(!text.contains("user:secret"));
    assert!(!text.contains("http://"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_classifies_protocol_manifest_and_unsafe_responses_safely() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let protocol_base_url = raw_health_addon_server(
        StatusCode::OK,
        r#"{"protocol_version":"2020-01-01","manifest_id":"taru.protocol.metadata","status":"ok","checked_at":"2026-05-21T12:00:00.000Z","manifest":{"addon_version":"0.1.0","resource_count":1},"diagnostics":{"secret":"taru_at_should_not_echo"}}"#,
    )
    .await;
    let mut protocol_manifest = taru_reference_addon::reference_manifest(protocol_base_url);
    protocol_manifest.id = "taru.protocol.metadata".to_owned();
    let protocol_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: protocol_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let protocol_raw = response_for(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/addons/{}/runtime-readiness",
            protocol_addon.addon.summary.id
        ),
    )
    .await;
    let protocol_text = String::from_utf8(
        to_bytes(protocol_raw.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let protocol_response =
        serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&protocol_text).unwrap();
    assert_eq!(
        protocol_response.readiness.reason,
        AdminAddonRuntimeReadinessReason::ProtocolMismatch
    );
    assert!(
        protocol_response
            .readiness
            .checks
            .iter()
            .any(|check| { check.safe_error_code.as_deref() == Some("protocol_mismatch") })
    );
    assert!(!protocol_text.contains("2020-01-01"));
    assert!(!protocol_text.contains("taru_at_should_not_echo"));

    let manifest_base_url = mismatched_health_addon_server("wrong-manifest", "0.1.0").await;
    let mut manifest = taru_reference_addon::reference_manifest(manifest_base_url);
    manifest.id = "taru.manifest.metadata".to_owned();
    let manifest_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let manifest_raw = response_for(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/addons/{}/runtime-readiness",
            manifest_addon.addon.summary.id
        ),
    )
    .await;
    let manifest_text = String::from_utf8(
        to_bytes(manifest_raw.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let manifest_response =
        serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&manifest_text).unwrap();
    assert_eq!(
        manifest_response.readiness.reason,
        AdminAddonRuntimeReadinessReason::ManifestMismatch
    );
    assert!(
        manifest_response
            .readiness
            .checks
            .iter()
            .any(|check| { check.safe_error_code.as_deref() == Some("manifest_mismatch") })
    );
    assert!(!manifest_text.contains("wrong-manifest"));
    assert!(!manifest_text.contains("raw_network_error"));

    let unsafe_base_url = raw_health_addon_server(StatusCode::OK, "not-json-secret").await;
    let mut unsafe_manifest = taru_reference_addon::reference_manifest(unsafe_base_url);
    unsafe_manifest.id = "taru.unsafe-health.metadata".to_owned();
    let unsafe_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: unsafe_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let unsafe_raw = response_for(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/addons/{}/runtime-readiness",
            unsafe_addon.addon.summary.id
        ),
    )
    .await;
    let unsafe_text = String::from_utf8(
        to_bytes(unsafe_raw.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let unsafe_response =
        serde_json::from_str::<AdminAddonRuntimeReadinessResponse>(&unsafe_text).unwrap();
    assert_eq!(
        unsafe_response.readiness.reason,
        AdminAddonRuntimeReadinessReason::UnsafeResponse
    );
    assert!(
        unsafe_response
            .readiness
            .checks
            .iter()
            .any(|check| { check.safe_error_code.as_deref() == Some("unsafe_response") })
    );
    assert!(!unsafe_text.contains("not-json-secret"));
}

#[tokio::test]
async fn admin_addon_surfaces_returns_manifest_declarations_without_launch_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = taru_reference_addon::reference_manifest("https://addon.example.test/base");
    manifest.secret_reference_fields = vec![
        taru_addon_protocol::AddonSecretReferenceFieldDeclaration::new(
            "api_key",
            "API Key",
            Some("Resolved by Taru at runtime".to_owned()),
            true,
        ),
    ];
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scan-finished",
        "library_scan.succeeded",
        "/events/library-scan-finished",
        vec![AddonScope::WebhookEventRead],
        serde_json::json!({"library_preset":"movies"}),
    )];
    manifest.tasks = vec![
        AddonTaskDeclaration::new(
            "bulk-metadata-scrape",
            "Bulk Metadata Scrape",
            "/tasks/bulk-metadata-scrape",
            vec![AddonScope::AutomationRun],
        )
        .with_description("Suggests metadata for selected items")
        .with_execution_bounds(Some(30_000), Some(2)),
    ];
    manifest
        .scopes
        .extend([AddonScope::WebhookEventRead, AddonScope::AutomationRun]);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;

    let path = format!("/admin/v1/addons/{addon_id}/surfaces");
    let raw = response_for(&router, Method::GET, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonSurfacesResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(
        response.manifest_id,
        taru_reference_addon::REFERENCE_ADDON_ID
    );
    assert_eq!(response.entry_points.len(), 1);
    assert_eq!(response.entry_points[0].id, "suggest-metadata");
    assert_eq!(
        response.entry_points[0].hosted_page_id.as_deref(),
        Some("diagnostics")
    );
    assert_eq!(response.hosted_pages.len(), 1);
    assert_eq!(response.hosted_pages[0].id, "diagnostics");
    assert_eq!(
        response.hosted_pages[0].url,
        "https://addon.example.test/base/ui/diagnostics"
    );
    assert_eq!(
        response.configuration_schema.as_ref().unwrap().schema_id,
        "taru.reference.metadata.config.v1"
    );
    assert_eq!(response.secret_reference_fields.len(), 1);
    assert_eq!(response.secret_reference_fields[0].id, "api_key");
    assert_eq!(response.secret_reference_fields[0].label, "API Key");
    assert!(response.secret_reference_fields[0].required);
    assert_eq!(response.tasks.len(), 1);
    assert_eq!(response.tasks[0].id, "bulk-metadata-scrape");
    assert_eq!(response.tasks[0].timeout_ms, Some(30_000));
    assert_eq!(response.tasks[0].max_attempts, Some(2));
    assert_eq!(response.event_subscriptions.len(), 1);
    assert_eq!(
        response.event_subscriptions[0].event_kind,
        "library_scan.succeeded"
    );
    assert_eq!(
        response.event_subscriptions[0].filters["library_preset"],
        "movies"
    );
    assert!(!text.contains("Bearer"));
    assert!(!text.contains("taru_at_"));
    assert!(!text.contains("launch_token"));
    assert!(!text.contains("secret_value"));
}

#[tokio::test]
async fn admin_addon_routing_plans_syncs_manifest_declarations_without_hidden_work() {
    let (_temp, router, _source, store) =
        router_with_media_source_config("routing-plan.mkv", b"media", |_| {}).await;

    let mut manifest = taru_reference_addon::reference_manifest("https://addon.example.test/base");
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scanned",
        DomainEventKind::LibraryScanned.as_str(),
        "/events/library-scanned",
        vec![AddonScope::WebhookEventRead],
        serde_json::json!({"library_preset":"movies","token":"taru_at_should_not_echo"}),
    )];
    manifest.tasks = vec![
        AddonTaskDeclaration::new(
            "bulk-metadata-scrape",
            "Bulk Metadata Scrape",
            "/tasks/bulk-metadata-scrape",
            vec![AddonScope::AutomationRun],
        )
        .with_execution_bounds(Some(30_000), Some(2)),
    ];
    manifest
        .scopes
        .extend([AddonScope::WebhookEventRead, AddonScope::AutomationRun]);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::WebhookEventRead,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/routing-plans");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRoutingPlansResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.executable, 2);
    assert_eq!(response.deferred, 0);
    assert!(response.manifest_fingerprint.starts_with("sha256:"));
    let task = response
        .plans
        .iter()
        .find(|plan| plan.declaration_id == "bulk-metadata-scrape")
        .unwrap();
    assert_eq!(task.status, AddonRoutingPlanStatus::Executable);
    assert_eq!(task.target, AddonRoutingPlanTarget::AddonTaskJob);
    assert_eq!(task.job_kind, Some(JobKind::AddonTask));
    assert_eq!(task.required_scope_count, 1);
    assert_eq!(task.timeout_ms, Some(30_000));
    assert_eq!(task.max_attempts, Some(2));
    let event = response
        .plans
        .iter()
        .find(|plan| plan.declaration_id == "library-scanned")
        .unwrap();
    assert_eq!(event.status, AddonRoutingPlanStatus::Executable);
    assert_eq!(event.target, AddonRoutingPlanTarget::EventOutbox);
    assert_eq!(
        event.event_kind.as_deref(),
        Some(DomainEventKind::LibraryScanned.as_str())
    );
    assert!(event.filter_configured);
    assert!(!text.contains("taru_at_should_not_echo"));
    assert!(!text.contains("library_preset"));

    assert!(
        store
            .list_jobs(
                taru_core::JobListFilter::default(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_outbox_events(
                taru_core::OutboxEventListFilter::default(),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );

    let repeated =
        request_json::<AdminAddonRoutingPlansResponse>(&router, Method::POST, &path).await;
    assert_eq!(repeated.plans.len(), 2);
    assert_eq!(
        store
            .list_addon_routing_plans(addon_id)
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn admin_addon_routing_plans_defers_missing_grants_and_unsupported_events_safely() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = addon_manifest();
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "unknown-event",
        "library_scan.succeeded",
        "/events/unknown",
        vec![AddonScope::WebhookEventRead],
        serde_json::json!({"token":"taru_at_should_not_echo"}),
    )];
    manifest.tasks = vec![AddonTaskDeclaration::new(
        "bulk-task",
        "Bulk Task",
        "/tasks/bulk",
        vec![AddonScope::AutomationRun],
    )];
    manifest
        .scopes
        .extend([AddonScope::WebhookEventRead, AddonScope::AutomationRun]);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::WebhookEventRead,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/routing-plans");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRoutingPlansResponse>(&text).unwrap();

    assert_eq!(response.executable, 0);
    assert_eq!(response.deferred, 2);
    let task = response
        .plans
        .iter()
        .find(|plan| plan.declaration_id == "bulk-task")
        .unwrap();
    assert_eq!(task.status, AddonRoutingPlanStatus::Deferred);
    assert_eq!(task.target, AddonRoutingPlanTarget::None);
    assert_eq!(task.safe_reason_code.as_deref(), Some("missing_grant"));
    assert_eq!(task.job_kind, None);
    let event = response
        .plans
        .iter()
        .find(|plan| plan.declaration_id == "unknown-event")
        .unwrap();
    assert_eq!(event.status, AddonRoutingPlanStatus::Deferred);
    assert_eq!(event.target, AddonRoutingPlanTarget::None);
    assert_eq!(
        event.safe_reason_code.as_deref(),
        Some("unsupported_event_kind")
    );
    assert_eq!(event.event_kind, None);
    assert!(!text.contains("library_scan.succeeded"));
    assert!(!text.contains("taru_at_should_not_echo"));
}

#[tokio::test]
async fn admin_addon_routing_plans_defers_disabled_addons_without_runtime_targets() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = addon_manifest();
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scanned",
        DomainEventKind::LibraryScanned.as_str(),
        "/events/library-scanned",
        vec![AddonScope::WebhookEventRead],
        serde_json::json!({"token":"taru_at_should_not_echo"}),
    )];
    manifest.tasks = vec![AddonTaskDeclaration::new(
        "bulk-task",
        "Bulk Task",
        "/tasks/bulk",
        vec![AddonScope::AutomationRun],
    )];
    manifest
        .scopes
        .extend([AddonScope::WebhookEventRead, AddonScope::AutomationRun]);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::WebhookEventRead,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Disabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/routing-plans");

    let raw = response_for(&router, Method::POST, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonRoutingPlansResponse>(&text).unwrap();

    assert_eq!(response.executable, 0);
    assert_eq!(response.deferred, 2);
    for plan in &response.plans {
        assert_eq!(plan.status, AddonRoutingPlanStatus::Deferred);
        assert_eq!(plan.target, AddonRoutingPlanTarget::None);
        assert_eq!(plan.safe_reason_code.as_deref(), Some("addon_disabled"));
        assert_eq!(plan.job_kind, None);
    }
    let event = response
        .plans
        .iter()
        .find(|plan| plan.declaration_id == "library-scanned")
        .unwrap();
    assert_eq!(
        event.event_kind.as_deref(),
        Some(DomainEventKind::LibraryScanned.as_str())
    );
    assert!(!text.contains("taru_at_should_not_echo"));
}

#[tokio::test]
async fn admin_addon_resource_call_diagnostic_classifies_safe_success_without_payload_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addon_base_url = format!("http://{}", listener.local_addr().unwrap());
    let addon_server = tokio::spawn(async move {
        axum::serve(listener, taru_reference_addon::build_router())
            .await
            .unwrap();
    });
    yield_now().await;

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = taru_reference_addon::reference_manifest(addon_base_url);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/diagnostics/resource-call");
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonResourceCallDiagnosticRequest {
            resource: AddonResource::Metadata,
            payload: serde_json::json!({
                "title": "The Matrix",
                "source_locator": "local:///secret/movie.mkv",
                "token": "taru_at_should_not_echo"
            }),
        },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonResourceCallDiagnosticResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.resource, AddonResource::Metadata);
    assert_eq!(
        response.status,
        AdminAddonResourceCallDiagnosticStatus::Succeeded
    );
    assert_eq!(response.safe_error_code, None);
    assert_eq!(response.http_status, Some(200));
    assert!(!text.contains("The Matrix"));
    assert!(!text.contains("Reference addon metadata suggestion"));
    assert!(!text.contains("local:///secret"));
    assert!(!text.contains("taru_at_should_not_echo"));
    assert!(!text.contains("metadata_suggestion"));

    addon_server.abort();
}

#[tokio::test]
async fn admin_addon_resource_call_diagnostic_classifies_safe_failures() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let mut auth_gap_manifest =
        taru_reference_addon::reference_manifest("https://auth-gap.example.test/addon");
    auth_gap_manifest.id = "taru.auth-gap.metadata".to_owned();
    auth_gap_manifest.auth = AddonAuth::Bearer;
    let auth_gap = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: auth_gap_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let auth_gap_path = format!(
        "/admin/v1/addons/{}/diagnostics/resource-call",
        auth_gap.addon.summary.id
    );
    let auth_gap_response = request_body_json::<AdminAddonResourceCallDiagnosticResponse, _>(
        &router,
        Method::POST,
        &auth_gap_path,
        &AdminAddonResourceCallDiagnosticRequest {
            resource: AddonResource::Metadata,
            payload: serde_json::json!({"secret":"taru_at_should_not_echo"}),
        },
    )
    .await;
    assert_eq!(
        auth_gap_response.status,
        AdminAddonResourceCallDiagnosticStatus::AuthorizationGap
    );
    assert_eq!(
        auth_gap_response.safe_error_code.as_deref(),
        Some("authorization_gap")
    );
    assert_eq!(auth_gap_response.attempts, 0);

    let missing_resource_response =
        request_body_json::<AdminAddonResourceCallDiagnosticResponse, _>(
            &router,
            Method::POST,
            &auth_gap_path,
            &AdminAddonResourceCallDiagnosticRequest {
                resource: AddonResource::Image,
                payload: serde_json::json!({"secret":"taru_at_should_not_echo"}),
            },
        )
        .await;
    assert_eq!(
        missing_resource_response.status,
        AdminAddonResourceCallDiagnosticStatus::MissingResource
    );
    assert_eq!(
        missing_resource_response.safe_error_code.as_deref(),
        Some("missing_resource")
    );
    assert_eq!(missing_resource_response.attempts, 0);

    let retryable_base_url =
        failing_resource_addon_server(StatusCode::INTERNAL_SERVER_ERROR, "token=secret").await;
    let mut retryable_manifest = taru_reference_addon::reference_manifest(retryable_base_url);
    retryable_manifest.id = "taru.retryable.metadata".to_owned();
    let retryable = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: retryable_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let retryable_path = format!(
        "/admin/v1/addons/{}/diagnostics/resource-call",
        retryable.addon.summary.id
    );
    let retryable_raw = response_body_json(
        &router,
        Method::POST,
        &retryable_path,
        &AdminAddonResourceCallDiagnosticRequest {
            resource: AddonResource::Metadata,
            payload: serde_json::json!({"title":"Hidden"}),
        },
    )
    .await;
    assert_eq!(retryable_raw.status(), StatusCode::OK);
    let retryable_bytes = to_bytes(retryable_raw.into_body(), usize::MAX)
        .await
        .unwrap();
    let retryable_text = String::from_utf8(retryable_bytes.to_vec()).unwrap();
    let retryable_response =
        serde_json::from_str::<AdminAddonResourceCallDiagnosticResponse>(&retryable_text).unwrap();
    assert_eq!(
        retryable_response.status,
        AdminAddonResourceCallDiagnosticStatus::RetryableHttpFailure
    );
    assert_eq!(retryable_response.http_status, Some(500));
    assert_eq!(retryable_response.attempts, 2);
    assert_eq!(
        retryable_response.safe_error_code.as_deref(),
        Some("retryable_http_failure")
    );
    assert!(!retryable_text.contains("token=secret"));
    assert!(!retryable_text.contains("Hidden"));

    let unsafe_base_url = failing_resource_addon_server(StatusCode::OK, "not-json-secret").await;
    let mut unsafe_manifest = taru_reference_addon::reference_manifest(unsafe_base_url);
    unsafe_manifest.id = "taru.unsafe.metadata".to_owned();
    let unsafe_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: unsafe_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let unsafe_path = format!(
        "/admin/v1/addons/{}/diagnostics/resource-call",
        unsafe_addon.addon.summary.id
    );
    let unsafe_raw = response_body_json(
        &router,
        Method::POST,
        &unsafe_path,
        &AdminAddonResourceCallDiagnosticRequest {
            resource: AddonResource::Metadata,
            payload: serde_json::json!({"title":"Hidden"}),
        },
    )
    .await;
    assert_eq!(unsafe_raw.status(), StatusCode::OK);
    let unsafe_bytes = to_bytes(unsafe_raw.into_body(), usize::MAX).await.unwrap();
    let unsafe_text = String::from_utf8(unsafe_bytes.to_vec()).unwrap();
    let unsafe_response =
        serde_json::from_str::<AdminAddonResourceCallDiagnosticResponse>(&unsafe_text).unwrap();
    assert_eq!(
        unsafe_response.status,
        AdminAddonResourceCallDiagnosticStatus::UnsafeResponse
    );
    assert_eq!(
        unsafe_response.safe_error_code.as_deref(),
        Some("unsafe_response")
    );
    assert_eq!(unsafe_response.attempts, 1);
    assert!(!unsafe_text.contains("not-json-secret"));
    assert!(!unsafe_text.contains("Hidden"));
}

#[tokio::test]
async fn addon_admin_routes_issue_rotate_revoke_tokens_and_replace_grants_without_leaking_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;

    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&token_path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&IssueAddonTokenRequest {
                        label: Some("metadata sidecar".to_owned()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(issued_response.status(), StatusCode::OK);
    let issued_body = to_bytes(issued_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let issued: AddonTokenIssuedResponse = serde_json::from_slice(&issued_body).unwrap();
    assert!(issued.raw_token.starts_with("taru_at_"));
    assert_eq!(issued.token.label, "metadata sidecar");
    assert_eq!(issued.token.status, AddonTokenStatus::Active);
    assert!(!String::from_utf8_lossy(&issued_body).contains("token_hash"));

    let tokens_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&token_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tokens_response.status(), StatusCode::OK);
    let tokens_body = to_bytes(tokens_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let tokens: AddonTokensResponse = serde_json::from_slice(&tokens_body).unwrap();
    assert_eq!(tokens.tokens, vec![issued.token.clone()]);
    let tokens_body = String::from_utf8_lossy(&tokens_body);
    assert!(!tokens_body.contains(&issued.raw_token));
    assert!(!tokens_body.contains("raw_token"));
    assert!(!tokens_body.contains("token_hash"));

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    let grants = request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![
                AddonGrantAssignment {
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library_id),
                },
                AddonGrantAssignment {
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library_id),
                },
                AddonGrantAssignment {
                    permission: AddonPermission::ArtworkWrite,
                    library_id: None,
                },
            ],
        },
    )
    .await;
    assert_eq!(grants.grants.len(), 2);
    assert!(
        grants
            .grants
            .iter()
            .any(|grant| grant.permission == AddonPermission::MetadataWrite
                && grant.library_id == Some(library_id))
    );
    assert!(grants.grants.iter().any(
        |grant| grant.permission == AddonPermission::ArtworkWrite && grant.library_id.is_none()
    ));
    assert_eq!(
        request_json::<AddonGrantsResponse>(&router, Method::GET, &grants_path).await,
        grants
    );

    let rotate_path = format!(
        "/admin/v1/addons/{addon_id}/tokens/{}/rotate",
        issued.token.id
    );
    let rotation = request_body_json::<AddonTokenRotationResponse, _>(
        &router,
        Method::POST,
        &rotate_path,
        &IssueAddonTokenRequest {
            label: Some("metadata sidecar rotated".to_owned()),
        },
    )
    .await;
    assert_eq!(rotation.rotated.id, issued.token.id);
    assert_eq!(rotation.rotated.status, AddonTokenStatus::Rotated);
    assert!(rotation.raw_token.starts_with("taru_at_"));
    assert_ne!(rotation.raw_token, issued.raw_token);
    assert_eq!(rotation.token.status, AddonTokenStatus::Active);

    let revoke_path = format!(
        "/admin/v1/addons/{addon_id}/tokens/{}/revoke",
        rotation.token.id
    );
    let revoked = request_json::<AddonTokenResponse>(&router, Method::POST, &revoke_path).await;
    assert_eq!(revoked.token.status, AddonTokenStatus::Revoked);
    assert_eq!(revoked.token.id, rotation.token.id);
}

#[tokio::test]
async fn addon_runtime_access_check_enforces_token_permission_and_library_scope() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![
                AddonGrantAssignment {
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library_id),
                },
                AddonGrantAssignment {
                    permission: AddonPermission::ArtworkWrite,
                    library_id: None,
                },
            ],
        },
    )
    .await;

    let allowed = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(allowed.status(), StatusCode::OK);
    let allowed = body_json::<AddonAccessCheckResponse>(allowed).await;
    assert_eq!(allowed.addon_id, addon_id);
    assert_eq!(allowed.token_id, issued.token.id);
    assert_eq!(allowed.permission, AddonPermission::MetadataWrite);
    assert_eq!(allowed.library_id, Some(library_id));
    assert!(allowed.allowed);

    let global_grant_allows_library_target = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::ArtworkWrite,
            library_id: Some(other_library_id),
        },
    )
    .await;
    assert_eq!(global_grant_allows_library_target.status(), StatusCode::OK);

    let wrong_library = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(other_library_id),
        },
    )
    .await;
    assert_eq!(wrong_library.status(), StatusCode::FORBIDDEN);
    let wrong_library = body_json::<ErrorResponse>(wrong_library).await;
    assert_eq!(wrong_library.code, "forbidden");

    let missing_permission = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::SubtitleWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(missing_permission.status(), StatusCode::FORBIDDEN);

    let missing_token = addon_access_check(
        &router,
        None,
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(missing_token.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(missing_token.headers()[header::WWW_AUTHENTICATE], "Bearer");
    assert_eq!(
        body_json::<ErrorResponse>(missing_token).await.code,
        "unauthorized"
    );

    let invalid_token = addon_access_check(
        &router,
        Some("taru_at_invalid"),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(invalid_token.status(), StatusCode::UNAUTHORIZED);

    let revoke_path = format!(
        "/admin/v1/addons/{addon_id}/tokens/{}/revoke",
        issued.token.id
    );
    request_json::<AddonTokenResponse>(&router, Method::POST, &revoke_path).await;
    let revoked_token = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(revoked_token.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn addon_token_cannot_authenticate_admin_routes() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let admin_token = "admin-token";
    let router =
        test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, admin_token).await;

    let registered = register_addon_with_admin_token(&router, admin_token).await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json_with_bearer::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        admin_token,
        &IssueAddonTokenRequest {
            label: Some("runtime".to_owned()),
        },
    )
    .await;

    let addon_token_on_admin_route = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", issued.raw_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        addon_token_on_admin_route.status(),
        StatusCode::UNAUTHORIZED
    );
    let error = body_json::<ErrorResponse>(addon_token_on_admin_route).await;
    assert_eq!(error.code, "unauthorized");

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json_with_bearer::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        admin_token,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let addon_runtime_route = addon_access_check(
        &router,
        Some(&issued.raw_token),
        AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        },
    )
    .await;
    assert_eq!(addon_runtime_route.status(), StatusCode::OK);
}

#[tokio::test]
async fn addon_side_effect_intake_accepts_authorized_metadata_write_without_echoing_payload() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-demo-1".to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "request_id": "request-1",
            "raw_path": "local:///Movies/demo.mkv",
            "token": issued.raw_token
        }),
        payload: serde_json::json!({
            "title": "Demo From Addon",
            "overview": "A safe metadata update.",
            "genres": ["Addon Genre", "Addon Genre"],
            "tags": ["sidecar", "metadata"]
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let body: AddonSideEffectResponse = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(body.side_effect.addon_id, addon_id);
    assert_eq!(body.side_effect.token_id, issued.token.id);
    assert_eq!(body.side_effect.permission, AddonPermission::MetadataWrite);
    assert_eq!(body.side_effect.library_id, library_id);
    assert_eq!(
        body.side_effect.target.kind,
        AddonSideEffectTargetKind::MediaSource
    );
    assert_eq!(body.side_effect.target.id, source.id.to_string());
    assert_eq!(body.side_effect.idempotency_key, "metadata-demo-1");
    assert_eq!(
        body.side_effect.validation_status,
        AddonSideEffectValidationStatus::Accepted
    );
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    assert_eq!(body.side_effect.apply_error_code, None);
    assert_eq!(body.side_effect.applied_item_id, Some(source.item_id));
    assert_eq!(
        body.side_effect.applied_source,
        Some(format!("addon:{addon_id}"))
    );
    assert!(body.side_effect.applied_at.is_some());
    assert!(!body.idempotent_replay);

    let response_body = String::from_utf8_lossy(&response_body);
    assert!(!response_body.contains("token_hash"));
    assert!(!response_body.contains("raw_token"));
    assert!(!response_body.contains(&issued.raw_token));
    assert!(!response_body.contains("local:///Movies/demo.mkv"));
    assert!(!response_body.contains("Demo From Addon"));

    let updated = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item was applied");
    assert_eq!(updated.metadata.title, "Demo From Addon");
    assert_eq!(
        updated.metadata.overview.as_deref(),
        Some("A safe metadata update.")
    );
    assert_eq!(updated.metadata.genres, vec!["Addon Genre"]);
    assert_eq!(
        updated.metadata.tags,
        vec!["sidecar".to_owned(), "metadata".to_owned()]
    );
    let tags = store
        .list_tags(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(tags.len(), 2);
    assert!(
        tags.iter()
            .all(|tag| tag.source == MetadataSource::Addon(addon_id))
    );
    let genres = store
        .list_genres(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(genres.len(), 1);
    assert_eq!(genres[0].source, MetadataSource::Addon(addon_id));
    let hits = store
        .search(
            SearchQuery::from_facet_labels("safe metadata", vec!["tag:sidecar".to_owned()], 10, 0)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hits[0].item_id, source.item_id);

    let duplicate = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate = body_json::<AddonSideEffectResponse>(duplicate).await;
    assert_eq!(duplicate.side_effect.id, body.side_effect.id);
    assert!(duplicate.idempotent_replay);
    assert_eq!(
        duplicate.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );

    let mut conflicting_payload = request.clone();
    conflicting_payload.payload = serde_json::json!({
        "title": "Conflicting Addon Title"
    });
    let conflict = addon_side_effect(&router, Some(&issued.raw_token), &conflicting_payload).await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conflict_body = to_bytes(conflict.into_body(), usize::MAX).await.unwrap();
    let conflict_error: ErrorResponse = serde_json::from_slice(&conflict_body).unwrap();
    assert_eq!(conflict_error.code, "conflict");
    let conflict_text = String::from_utf8_lossy(&conflict_body);
    assert!(!conflict_text.contains("Conflicting Addon Title"));
    assert!(!conflict_text.contains("local:///Movies/demo.mkv"));
    assert!(!conflict_text.contains(&issued.raw_token));

    let mut conflicting_provenance = request.clone();
    conflicting_provenance.provenance = serde_json::json!({
        "origin": "reference-addon",
        "request_id": "request-2"
    });
    let conflict =
        addon_side_effect(&router, Some(&issued.raw_token), &conflicting_provenance).await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn addon_side_effect_library_file_write_exports_missing_nfo_without_echoing_paths() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut library = store
        .get_library(library_id)
        .await
        .unwrap()
        .expect("library exists");
    library.options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::WriteSidecar;
    store.upsert_library(&library).await.unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("nfo runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::LibraryFileWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::LibraryFileWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "nfo-create-missing-demo".to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "raw_path": "local:///demo.mkv",
            "token": issued.raw_token
        }),
        payload: serde_json::json!({
            "file_role": "nfo",
            "policy": "create_missing"
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let body: AddonSideEffectResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(
        body.side_effect.permission,
        AddonPermission::LibraryFileWrite
    );
    assert_eq!(
        body.side_effect.target.kind,
        AddonSideEffectTargetKind::MediaSource
    );
    assert_eq!(body.side_effect.target.id, source.id.to_string());
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    assert_eq!(body.side_effect.apply_error_code, None);
    assert_eq!(body.side_effect.applied_item_id, Some(source.item_id));
    assert_eq!(
        body.side_effect.applied_source,
        Some("nfo_export".to_owned())
    );

    let report = body
        .side_effect
        .apply_report
        .as_ref()
        .expect("NFO export side effect report");
    assert_eq!(report["kind"], "nfo_export");
    assert_eq!(report["target_kind"], "media_source");
    assert_eq!(report["file_role"], "nfo");
    assert_eq!(report["policy"], "create_missing");
    assert_eq!(report["write_mode"], "create_missing");
    assert_eq!(report["backup_policy"], "none");
    assert_eq!(report["library_id"], library_id.to_string());
    assert_eq!(report["source_id"], source.id.to_string());
    assert_eq!(report["item_id"], source.item_id.to_string());
    assert_eq!(report["exported_items"], 1);
    assert_eq!(report["skipped_items"], 0);
    assert_eq!(report["failed_items"], 0);
    assert_eq!(report["backed_up_items"], 0);

    let response_body = String::from_utf8_lossy(&response_body);
    assert!(!response_body.contains("local:///demo.mkv"));
    assert!(!response_body.contains(temp.path().to_string_lossy().as_ref()));

    let nfo = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    assert!(nfo.contains("<title>demo.mkv</title>"));

    let duplicate = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate = body_json::<AddonSideEffectResponse>(duplicate).await;
    assert_eq!(duplicate.side_effect.id, body.side_effect.id);
    assert!(duplicate.idempotent_replay);
    assert_eq!(
        duplicate.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    assert_eq!(
        duplicate.side_effect.apply_report.as_ref(),
        body.side_effect.apply_report.as_ref()
    );
}

#[tokio::test]
async fn addon_side_effect_library_file_write_replaces_existing_nfo_with_backup_report() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut library = store
        .get_library(library_id)
        .await
        .unwrap()
        .expect("library exists");
    library.options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::WriteSidecar;
    store.upsert_library(&library).await.unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Old Sidecar Title</title>
  <customrating system="local">five stars</customrating>
</movie>"#,
    )
    .unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("nfo runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::LibraryFileWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::LibraryFileWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "nfo-replace-existing-demo".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "file_role": "nfo",
            "policy": "replace_existing_preserving"
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let body: AddonSideEffectResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    let report = body
        .side_effect
        .apply_report
        .as_ref()
        .expect("NFO export side effect report");
    assert_eq!(report["kind"], "nfo_export");
    assert_eq!(report["policy"], "replace_existing_preserving");
    assert_eq!(report["write_mode"], "atomic_replace");
    assert_eq!(report["backup_policy"], "existing_file_keep_latest");
    assert_eq!(report["library_id"], library_id.to_string());
    assert_eq!(report["source_id"], source.id.to_string());
    assert_eq!(report["item_id"], source.item_id.to_string());
    assert_eq!(report["exported_items"], 1);
    assert_eq!(report["backed_up_items"], 1);
    assert_eq!(report["failed_items"], 0);

    let response_body = String::from_utf8_lossy(&response_body);
    assert!(!response_body.contains("local:///demo.nfo"));
    assert!(!response_body.contains("taru-backup"));
    assert!(!response_body.contains(temp.path().to_string_lossy().as_ref()));

    let nfo = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    assert!(nfo.contains("<title>demo.mkv</title>"));
    assert!(nfo.contains(r#"<customrating system="local">five stars</customrating>"#));
    assert!(!nfo.contains("<title>Old Sidecar Title</title>"));

    let backups = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("taru-backup"))
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
}

#[tokio::test]
async fn addon_side_effect_library_file_write_rejects_raw_payload_and_media_item_target() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut library = store
        .get_library(library_id)
        .await
        .unwrap()
        .expect("library exists");
    library.options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::WriteSidecar;
    store.upsert_library(&library).await.unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("nfo runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::LibraryFileWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let raw_payload_request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::LibraryFileWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "nfo-raw-payload-denied".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "file_role": "nfo",
            "policy": "create_missing",
            "raw_nfo": "<movie><title>Should Not Apply</title></movie>"
        }),
    };
    let raw_payload =
        addon_side_effect(&router, Some(&issued.raw_token), &raw_payload_request).await;
    assert_eq!(raw_payload.status(), StatusCode::BAD_REQUEST);
    let raw_payload_body = to_bytes(raw_payload.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&raw_payload_body).unwrap();
    assert_eq!(error.code, "invalid_input");
    let raw_payload_body = String::from_utf8_lossy(&raw_payload_body);
    assert!(!raw_payload_body.contains("Should Not Apply"));
    assert!(!temp.path().join("demo.nfo").exists());
    let replay = addon_side_effect(&router, Some(&issued.raw_token), &raw_payload_request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json::<AddonSideEffectResponse>(replay).await;
    assert!(replay.idempotent_replay);
    assert_eq!(
        replay.side_effect.apply_status,
        AddonSideEffectApplyStatus::Failed
    );
    assert_eq!(
        replay.side_effect.apply_error_code.as_deref(),
        Some("invalid_payload")
    );

    let media_item_request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::LibraryFileWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: source.item_id.to_string(),
        },
        idempotency_key: "nfo-media-item-denied".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "file_role": "nfo",
            "policy": "create_missing"
        }),
    };
    let media_item = addon_side_effect(&router, Some(&issued.raw_token), &media_item_request).await;
    assert_eq!(media_item.status(), StatusCode::BAD_REQUEST);
    let media_item_body = to_bytes(media_item.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&media_item_body).unwrap();
    assert_eq!(error.code, "invalid_input");
    assert!(!temp.path().join("demo.nfo").exists());
    let replay = addon_side_effect(&router, Some(&issued.raw_token), &media_item_request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json::<AddonSideEffectResponse>(replay).await;
    assert!(replay.idempotent_replay);
    assert_eq!(
        replay.side_effect.validation_status,
        AddonSideEffectValidationStatus::Rejected
    );
    assert_eq!(
        replay.side_effect.apply_status,
        AddonSideEffectApplyStatus::Skipped
    );
    assert_eq!(
        replay.side_effect.safe_error_code.as_deref(),
        Some("invalid_target")
    );
}

#[tokio::test]
async fn addon_side_effect_artwork_write_proposes_candidate_without_public_artwork_or_url_echo() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let remote_url = "https://artwork.example.test/posters/demo.jpg?token=secret";

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: source.item_id.to_string(),
        },
        idempotency_key: "artwork-candidate-poster".to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "raw_path": "local:///Movies/demo.mkv",
            "token": issued.raw_token
        }),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": remote_url
            },
            "language": "EN",
            "width": 1000,
            "height": 1500
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let body: AddonSideEffectResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(body.side_effect.permission, AddonPermission::ArtworkWrite);
    assert_eq!(
        body.side_effect.target.kind,
        AddonSideEffectTargetKind::MediaItem
    );
    assert_eq!(body.side_effect.target.id, source.item_id.to_string());
    assert_eq!(
        body.side_effect.validation_status,
        AddonSideEffectValidationStatus::Accepted
    );
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    assert_eq!(body.side_effect.apply_error_code, None);
    assert_eq!(body.side_effect.applied_item_id, Some(source.item_id));
    assert_eq!(
        body.side_effect.applied_source.as_deref(),
        Some("artwork_candidate")
    );

    let report = body
        .side_effect
        .apply_report
        .as_ref()
        .expect("artwork candidate report");
    let candidate_id = report["candidate_id"]
        .as_str()
        .expect("redacted candidate id");
    assert_eq!(report["kind"], "artwork_candidate");
    assert_eq!(report["image_kind"], "poster");
    assert_eq!(report["status"], "proposed");
    assert_eq!(report["candidate_created"], 1);
    assert_eq!(report["candidate_existing"], 0);

    let response_body = String::from_utf8_lossy(&response_body);
    assert!(!response_body.contains(remote_url));
    assert!(!response_body.contains("token=secret"));
    assert!(!response_body.contains("local:///Movies/demo.mkv"));
    assert!(!response_body.contains(&issued.raw_token));
    assert!(!response_body.contains("source_uri"));
    assert!(!response_body.contains("cache_uri"));

    let candidates = store
        .list_artwork_candidates_for_item(source.item_id, taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];
    assert_eq!(candidate.id.to_string(), candidate_id);
    assert_eq!(candidate.addon_id, addon_id);
    assert_eq!(candidate.side_effect_id, body.side_effect.id);
    assert_eq!(candidate.library_id, library_id);
    assert_eq!(candidate.item_id, source.item_id);
    assert_eq!(candidate.kind, ImageKind::Poster);
    assert_eq!(candidate.source_kind, ArtworkCandidateSourceKind::RemoteUrl);
    assert_eq!(candidate.source_uri, remote_url);
    assert_eq!(candidate.width, Some(1000));
    assert_eq!(candidate.height, Some(1500));
    assert_eq!(candidate.language.as_deref(), Some("en"));
    assert_eq!(candidate.status, ArtworkCandidateStatus::Proposed);
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );

    let duplicate = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate_body = to_bytes(duplicate.into_body(), usize::MAX).await.unwrap();
    let duplicate = serde_json::from_slice::<AddonSideEffectResponse>(&duplicate_body).unwrap();
    assert!(duplicate.idempotent_replay);
    assert_eq!(duplicate.side_effect.id, body.side_effect.id);
    assert_eq!(
        duplicate.side_effect.apply_report.as_ref(),
        body.side_effect.apply_report.as_ref()
    );
    assert_eq!(
        store
            .list_artwork_candidates_for_item(source.item_id, taru_core::PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(!String::from_utf8_lossy(&duplicate_body).contains(remote_url));
}

#[tokio::test]
async fn admin_accept_artwork_candidate_queues_managed_ingest_without_public_artwork_or_url_echo() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let remote_url = "https://artwork.example.test/posters/demo.jpg?token=secret";

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: source.item_id.to_string(),
        },
        idempotency_key: "artwork-candidate-ingest".to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "token": issued.raw_token
        }),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": remote_url
            },
            "language": "en",
            "width": 1000,
            "height": 1500
        }),
    };
    let proposed = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(proposed.status(), StatusCode::OK);
    let proposed = body_json::<AddonSideEffectResponse>(proposed).await;
    let candidate_id: taru_core::ArtworkCandidateId =
        proposed.side_effect.apply_report.as_ref().unwrap()["candidate_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();

    let accept_path = format!("/admin/v1/artwork/candidates/{candidate_id}/accept");
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&accept_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let accepted: AcceptManagedArtworkCandidateResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert_eq!(accepted.candidate_id, candidate_id);
    assert_eq!(accepted.candidate_status, ArtworkCandidateStatus::Accepted);
    assert_eq!(accepted.ingest.candidate_id, candidate_id);
    assert_eq!(accepted.ingest.library_id, library_id);
    assert_eq!(accepted.ingest.item_id, source.item_id);
    assert_eq!(accepted.ingest.kind, ImageKind::Poster);
    assert_eq!(accepted.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(accepted.job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(accepted.job.status, JobStatus::Queued);
    assert_eq!(accepted.job.resource_class, "artwork.ingest");
    assert!(accepted.job.has_input);
    assert!(!accepted.job.has_summary);
    assert!(!accepted.job.has_error);

    let response_body = String::from_utf8_lossy(&response_body);
    assert!(!response_body.contains(remote_url));
    assert!(!response_body.contains("token=secret"));
    assert!(!response_body.contains(&issued.raw_token));
    assert!(!response_body.contains("source_uri"));
    assert!(!response_body.contains("cache_uri"));
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );

    let candidate = store
        .get_artwork_candidate(candidate_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(candidate.status, ArtworkCandidateStatus::Accepted);
    assert_eq!(candidate.source_uri, remote_url);
    let ingest = store
        .find_managed_artwork_ingest_by_candidate(candidate_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ingest.id, accepted.ingest.id);
    assert_eq!(ingest.job_id, accepted.job.id);

    let replay =
        request_json::<AcceptManagedArtworkCandidateResponse>(&router, Method::POST, &accept_path)
            .await;
    assert_eq!(replay.ingest.id, accepted.ingest.id);
    assert_eq!(replay.job.id, accepted.job.id);
}

#[tokio::test]
async fn admin_process_next_managed_artwork_ingest_stores_internal_artifact_without_public_artwork()
{
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (remote_url, expected_byte_len) = tiny_artwork_server().await;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaItem,
            id: source.item_id.to_string(),
        },
        idempotency_key: "artwork-candidate-process-next".to_owned(),
        provenance: serde_json::json!({
            "origin": "reference-addon",
            "token": issued.raw_token
        }),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": remote_url
            },
            "language": "en"
        }),
    };
    let proposed = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(proposed.status(), StatusCode::OK);
    let proposed = body_json::<AddonSideEffectResponse>(proposed).await;
    let candidate_id: taru_core::ArtworkCandidateId =
        proposed.side_effect.apply_report.as_ref().unwrap()["candidate_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();

    let accept_path = format!("/admin/v1/artwork/candidates/{candidate_id}/accept");
    let accepted =
        request_json::<AcceptManagedArtworkCandidateResponse>(&router, Method::POST, &accept_path)
            .await;
    assert_eq!(accepted.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert!(!accepted.ingest.has_artifact);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/ingests/process-next")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let processed: ProcessManagedArtworkIngestResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(processed.processed);
    let ingest = processed.ingest.as_ref().unwrap();
    let artifact = processed
        .artifact
        .as_ref()
        .unwrap_or_else(|| panic!("{}", String::from_utf8_lossy(&response_body)));
    let job = processed.job.as_ref().unwrap();
    assert_eq!(ingest.id, accepted.ingest.id);
    assert_eq!(ingest.status, ManagedArtworkIngestStatus::Stored);
    assert!(ingest.has_artifact);
    assert_eq!(artifact.ingest_id, ingest.id);
    assert_eq!(artifact.library_id, library_id);
    assert_eq!(artifact.item_id, source.item_id);
    assert_eq!(artifact.kind, ImageKind::Poster);
    assert_eq!(artifact.media_type.as_deref(), Some("image/png"));
    assert_eq!(artifact.byte_len, Some(expected_byte_len));
    assert_eq!(artifact.width, Some(1));
    assert_eq!(artifact.height, Some(1));
    assert_eq!(job.id, accepted.job.id);
    assert_eq!(job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(job.status, JobStatus::Succeeded);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&issued.raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));

    let stored_ingest = store
        .get_managed_artwork_ingest(ingest.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_ingest.status, ManagedArtworkIngestStatus::Stored);
    assert_eq!(stored_ingest.artifact_id, Some(artifact.id));
    assert_eq!(stored_ingest.failure_code, None);

    let stored_artifact = store
        .get_managed_artwork_artifact(artifact.id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        stored_artifact
            .storage_uri
            .starts_with("managed-artwork://")
    );
    assert!(
        !stored_artifact
            .storage_uri
            .contains(temp.path().to_string_lossy().as_ref())
    );
    assert!(temp.path().join("taru-cache").join("artwork").exists());
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn job_runtime_worker_stores_managed_artwork_artifact_without_admin_process_next() {
    let (temp, router, source, store) =
        router_with_media_source_config("demo.mkv", b"media", |config| {
            config.artwork.ingest_worker_enabled = true;
            config.artwork.ingest_worker_idle_ms = 10;
        })
        .await;
    let library_id = source.library_id;
    let (remote_url, expected_byte_len) = tiny_artwork_server().await;
    let (raw_token, _candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-worker-store",
    )
    .await;

    let mut stored_ingest = None;
    for _ in 0..100 {
        let ingest = store
            .get_managed_artwork_ingest(accepted.ingest.id)
            .await
            .unwrap()
            .unwrap();
        if ingest.status == ManagedArtworkIngestStatus::Stored {
            stored_ingest = Some(ingest);
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    let stored_ingest = stored_ingest.expect("managed artwork worker should store the ingest");
    let artifact_id = stored_ingest
        .artifact_id
        .expect("stored managed artwork ingest should reference an artifact");
    let artifact = store
        .get_managed_artwork_artifact(artifact_id)
        .await
        .unwrap()
        .unwrap();
    let job = store.get_job(accepted.job.id).await.unwrap().unwrap();

    assert_eq!(stored_ingest.id, accepted.ingest.id);
    assert_eq!(stored_ingest.status, ManagedArtworkIngestStatus::Stored);
    assert_eq!(stored_ingest.failure_code, None);
    assert_eq!(artifact.ingest_id, accepted.ingest.id);
    assert_eq!(artifact.library_id, library_id);
    assert_eq!(artifact.item_id, source.item_id);
    assert_eq!(artifact.kind, ImageKind::Poster);
    assert_eq!(artifact.byte_len, Some(expected_byte_len));
    assert_eq!(artifact.media_type.as_deref(), Some("image/png"));
    assert_eq!(job.status, JobStatus::Succeeded);
    assert!(job.started_at.is_some());
    assert!(job.completed_at.is_some());
    assert!(job.error.is_none());

    assert!(artifact.storage_uri.starts_with("managed-artwork://"));
    assert!(
        !artifact
            .storage_uri
            .contains(temp.path().to_string_lossy().as_ref())
    );
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );

    let empty = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    assert!(!empty.processed);

    let overview =
        request_json::<AdminOverviewResponse>(&router, Method::GET, "/admin/v1/overview").await;
    assert!(overview.startup.artwork_ingest_worker_started);

    let response_text = serde_json::to_string(&overview).unwrap();
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));
}

#[tokio::test]
async fn admin_publish_managed_artwork_artifact_creates_selected_artwork_without_locator_leaks() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (remote_url, _) = tiny_artwork_server().await;
    let (raw_token, _candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-publish-selected",
    )
    .await;

    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap();
    assert_eq!(processed.ingest.as_ref().unwrap().id, accepted.ingest.id);
    assert_eq!(artifact.kind, ImageKind::Poster);

    let publish_path = format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&publish_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let published: PublishSelectedArtworkResponse = serde_json::from_slice(&response_body).unwrap();

    assert!(published.changed);
    assert_eq!(published.selected_artwork.library_id, library_id);
    assert_eq!(published.selected_artwork.item_id, source.item_id);
    assert_eq!(published.selected_artwork.kind, ImageKind::Poster);
    assert_eq!(published.selected_artwork.artifact_id, artifact.id);
    assert_eq!(
        published.image.id,
        published.selected_artwork.id.to_string()
    );
    assert_eq!(
        published.image.url,
        format!("/images/{}", published.selected_artwork.id)
    );
    assert_eq!(published.image.media_type.as_deref(), Some("image/png"));
    assert_eq!(published.image.width, Some(1));
    assert_eq!(published.image.height, Some(1));
    assert_eq!(published.image.etag, None);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));
    assert!(artifact.has_content_hash);
    assert!(!response_text.contains("\"content_hash\""));

    let replay =
        request_json::<PublishSelectedArtworkResponse>(&router, Method::POST, &publish_path).await;
    assert_eq!(replay.selected_artwork.id, published.selected_artwork.id);
    assert_eq!(replay.selected_artwork.artifact_id, artifact.id);
    assert!(!replay.changed);

    let selected = store
        .get_selected_artwork(published.selected_artwork.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(selected.artifact_id, artifact.id);
}

#[tokio::test]
async fn admin_managed_artwork_gallery_lists_candidates_artifacts_and_selection_without_locator_leaks()
 {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (remote_url, _) = tiny_artwork_server().await;
    let (raw_token, candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-admin-gallery",
    )
    .await;

    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap();
    assert!(artifact.has_content_hash);
    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/items/{}/artwork?limit=50&offset=0",
                    source.item_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let gallery: AdminManagedArtworkGalleryResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert_eq!(gallery.item_id, source.item_id);
    assert_eq!(gallery.summary.candidates, 1);
    assert_eq!(gallery.summary.artifacts, 1);
    assert_eq!(gallery.summary.selected, 1);
    assert_eq!(gallery.page.returned, 1);
    assert_eq!(gallery.candidates[0].id, candidate_id);
    assert_eq!(
        gallery.candidates[0].ingest.as_ref().unwrap().id,
        accepted.ingest.id
    );
    assert_eq!(gallery.candidates[0].artifact_id, Some(artifact.id));
    assert!(gallery.candidates[0].has_stored_artifact);
    assert!(gallery.candidates[0].selected);
    assert_eq!(gallery.artifacts[0].id, artifact.id);
    assert_eq!(gallery.artifacts[0].candidate_id, candidate_id);
    assert!(gallery.artifacts[0].selected);
    assert!(gallery.artifacts[0].has_content_hash);
    assert_eq!(
        gallery.selected[0].selected_artwork.id,
        published.selected_artwork.id
    );
    assert_eq!(gallery.selected[0].image, published.image);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));
    assert!(!response_text.contains("\"content_hash\""));
}

#[tokio::test]
async fn admin_managed_artwork_gallery_selects_item_kind_artifact_with_guards() {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (first_remote_url, _) = tiny_artwork_server().await;
    let (raw_token, _first_candidate_id, _accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &first_remote_url,
        "artwork-candidate-select-first",
    )
    .await;
    let first_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let first_artifact = first_processed.artifact.as_ref().unwrap().clone();

    let (second_remote_url, _) = tiny_artwork_server().await;
    let (_second_candidate_id, _second_accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &second_remote_url,
        "artwork-candidate-select-second",
        &raw_token,
    )
    .await;
    let second_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let second_artifact = second_processed.artifact.as_ref().unwrap().clone();

    let select_path = format!("/admin/v1/items/{}/artwork/poster/select", source.item_id);
    let first_selected = request_body_json::<PublishSelectedArtworkResponse, _>(
        &router,
        Method::POST,
        &select_path,
        &serde_json::json!({ "artifact_id": first_artifact.id }),
    )
    .await;
    assert!(first_selected.changed);
    assert_eq!(first_selected.selected_artwork.item_id, source.item_id);
    assert_eq!(first_selected.selected_artwork.kind, ImageKind::Poster);
    assert_eq!(
        first_selected.selected_artwork.artifact_id,
        first_artifact.id
    );

    let second_selected = request_body_json::<PublishSelectedArtworkResponse, _>(
        &router,
        Method::POST,
        &select_path,
        &serde_json::json!({ "artifact_id": second_artifact.id }),
    )
    .await;
    assert!(second_selected.changed);
    assert_eq!(
        second_selected.selected_artwork.id,
        first_selected.selected_artwork.id
    );
    assert_eq!(
        second_selected.selected_artwork.artifact_id,
        second_artifact.id
    );

    let replay = request_body_json::<PublishSelectedArtworkResponse, _>(
        &router,
        Method::POST,
        &select_path,
        &serde_json::json!({ "artifact_id": second_artifact.id }),
    )
    .await;
    assert_eq!(
        replay.selected_artwork.id,
        second_selected.selected_artwork.id
    );
    assert!(!replay.changed);

    let images = request_json::<taru_api::public_client::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", source.item_id),
    )
    .await;
    assert_eq!(images.images, vec![second_selected.image.clone()]);

    let wrong_kind = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/items/{}/artwork/backdrop/select",
                    source.item_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "artifact_id": second_artifact.id
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let wrong_kind_status = wrong_kind.status();
    let wrong_kind_body = to_bytes(wrong_kind.into_body(), usize::MAX).await.unwrap();
    assert_eq!(wrong_kind_status, StatusCode::CONFLICT);
    let wrong_kind_text = String::from_utf8_lossy(&wrong_kind_body);
    assert!(!wrong_kind_text.contains(&first_remote_url));
    assert!(!wrong_kind_text.contains(&second_remote_url));
    assert!(!wrong_kind_text.contains("token=secret"));
    assert!(!wrong_kind_text.contains(&raw_token));
    assert!(!wrong_kind_text.contains("source_uri"));
    assert!(!wrong_kind_text.contains("storage_uri"));
    assert!(!wrong_kind_text.contains("managed-artwork://"));
    assert!(!wrong_kind_text.contains(temp.path().to_string_lossy().as_ref()));
    assert!(!wrong_kind_text.contains("\"content_hash\""));

    let unknown_kind = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/items/{}/artwork/custom/select",
                    source.item_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "artifact_id": second_artifact.id
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_kind.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks() {
    assert_selected_artwork_variant_serving_without_locator_or_hash_leaks().await;
}

#[tokio::test]
async fn managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks() {
    assert_selected_artwork_variant_serving_without_locator_or_hash_leaks().await;
}

#[tokio::test]
async fn admin_selected_artwork_unpublish_hides_public_image_without_deleting_artifact() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let expected_bytes = png_with_size(4, 2);
    let (remote_url, _) = artwork_server(StatusCode::OK, "image/png", expected_bytes).await;
    let (raw_token, _candidate_id, _accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-unpublish",
    )
    .await;

    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap().clone();
    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;
    assert_eq!(
        request_json::<taru_api::public_client::ImagesResponse>(
            &router,
            Method::GET,
            &format!("/items/{}/images", source.item_id),
        )
        .await
        .images,
        vec![published.image.clone()]
    );

    let unpublish_path = format!(
        "/admin/v1/items/{}/artwork/poster/selection",
        source.item_id
    );
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&unpublish_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let unpublished: UnpublishSelectedArtworkResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(unpublished.changed);
    assert_eq!(unpublished.item_id, source.item_id);
    assert_eq!(unpublished.kind, ImageKind::Poster);
    let previous = unpublished.unpublished.as_ref().unwrap();
    assert_eq!(previous.selected_artwork.id, published.selected_artwork.id);
    assert_eq!(previous.selected_artwork.artifact_id, artifact.id);
    assert_eq!(previous.previous_image, published.image);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));
    assert!(artifact.has_content_hash);
    assert!(!response_text.contains("\"content_hash\""));

    let images = request_json::<taru_api::public_client::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", source.item_id),
    )
    .await;
    assert!(images.images.is_empty());

    for method in [Method::GET, Method::HEAD] {
        let image_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(&published.image.url)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(image_response.status(), StatusCode::NOT_FOUND);
    }

    assert!(
        store
            .get_managed_artwork_artifact(artifact.id)
            .await
            .unwrap()
            .is_some()
    );
    let lifecycle = request_json::<AdminManagedArtworkArtifactLifecycleResponse>(
        &router,
        Method::GET,
        "/admin/v1/artwork/artifacts/lifecycle?cleanup_candidates_only=true",
    )
    .await;
    assert!(
        lifecycle
            .artifacts
            .iter()
            .any(|candidate| candidate.id == artifact.id
                && candidate.cleanup_candidate
                && candidate.selected_artwork_count == 0)
    );

    let replay =
        request_json::<UnpublishSelectedArtworkResponse>(&router, Method::DELETE, &unpublish_path)
            .await;
    assert!(!replay.changed);
    assert!(replay.unpublished.is_none());

    let unknown_kind = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "/admin/v1/items/{}/artwork/custom/selection",
                    source.item_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_kind.status(), StatusCode::BAD_REQUEST);
}

async fn assert_selected_artwork_variant_serving_without_locator_or_hash_leaks() {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let expected_bytes = png_with_size(4, 2);
    let expected_byte_len = expected_bytes.len();
    let (remote_url, _) = artwork_server(StatusCode::OK, "image/png", expected_bytes.clone()).await;
    let (raw_token, _candidate_id, _accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-public-image-serving",
    )
    .await;

    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap();
    assert!(artifact.has_content_hash);
    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;
    assert_eq!(published.image.width, Some(4));
    assert_eq!(published.image.height, Some(2));
    assert_eq!(published.image.etag, None);

    let images_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/items/{}/images", source.item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let images_status = images_response.status();
    let images_body = to_bytes(images_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        images_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&images_body)
    );
    let images: taru_api::public_client::ImagesResponse =
        serde_json::from_slice(&images_body).unwrap();
    assert_eq!(images.item_id, source.item_id.to_string());
    assert_eq!(images.images, vec![published.image.clone()]);

    let item_detail_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/items/{}", source.item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let item_detail_body = to_bytes(item_detail_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let item_detail: taru_api::public_client::ItemDetailResponse =
        serde_json::from_slice(&item_detail_body).unwrap();
    assert_eq!(item_detail.images, vec![published.image.clone()]);

    for body in [&images_body, &item_detail_body] {
        let text = String::from_utf8_lossy(body);
        assert!(!text.contains(&remote_url));
        assert!(!text.contains("token=secret"));
        assert!(!text.contains(&raw_token));
        assert!(!text.contains("source_uri"));
        assert!(!text.contains("cache_uri"));
        assert!(!text.contains("storage_uri"));
        assert!(!text.contains("managed-artwork://"));
        assert!(!text.contains(temp.path().to_string_lossy().as_ref()));
        assert!(!text.contains("\"content_hash\""));
    }

    let image_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(image_response.status(), StatusCode::OK);
    assert_eq!(
        image_response.headers()[header::CONTENT_TYPE],
        HeaderValue::from_static("image/png")
    );
    assert_eq!(
        image_response.headers()[header::CONTENT_LENGTH],
        HeaderValue::from_str(&expected_byte_len.to_string()).unwrap()
    );
    let original_etag = image_response
        .headers()
        .get(header::ETAG)
        .expect("selected image has an ETag")
        .to_str()
        .unwrap()
        .to_owned();
    assert!(original_etag.contains("taru-img-v1-"));
    let image_bytes = to_bytes(image_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(image_bytes.as_ref(), expected_bytes.as_slice());

    let head_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&published.image.url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(head_response.status(), StatusCode::OK);
    assert_eq!(
        head_response.headers()[header::CONTENT_TYPE],
        HeaderValue::from_static("image/png")
    );
    assert_eq!(
        head_response.headers()[header::CONTENT_LENGTH],
        HeaderValue::from_str(&expected_byte_len.to_string()).unwrap()
    );
    assert_eq!(
        head_response.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&original_etag).unwrap()
    );
    let head_body = to_bytes(head_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(head_body.is_empty());

    let variant_url = format!("{}?width=2", published.image.url);
    let variant_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(variant_response.status(), StatusCode::OK);
    assert_eq!(
        variant_response.headers()[header::CONTENT_TYPE],
        HeaderValue::from_static("image/png")
    );
    let variant_etag = variant_response
        .headers()
        .get(header::ETAG)
        .expect("selected image variant has an ETag")
        .to_str()
        .unwrap()
        .to_owned();
    assert_ne!(variant_etag, original_etag);
    assert!(variant_etag.contains("taru-img-v1-"));
    let variant_content_length = variant_response.headers()[header::CONTENT_LENGTH]
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let variant_bytes = to_bytes(variant_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(variant_content_length, variant_bytes.len());
    let variant_image = image::load_from_memory(&variant_bytes).unwrap();
    assert_eq!(variant_image.width(), 2);
    assert_eq!(variant_image.height(), 1);
    assert_ne!(variant_bytes.as_ref(), expected_bytes.as_slice());

    let variant_head_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&variant_url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(variant_head_response.status(), StatusCode::OK);
    assert_eq!(
        variant_head_response.headers()[header::CONTENT_TYPE],
        HeaderValue::from_static("image/png")
    );
    assert_eq!(
        variant_head_response.headers()[header::CONTENT_LENGTH],
        HeaderValue::from_str(&variant_content_length.to_string()).unwrap()
    );
    assert_eq!(
        variant_head_response.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&variant_etag).unwrap()
    );
    let variant_head_body = to_bytes(variant_head_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(variant_head_body.is_empty());

    for invalid_url in [
        format!("{}?width=0", published.image.url),
        format!("{}?width=20001", published.image.url),
    ] {
        let invalid = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&invalid_url)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let invalid_status = invalid.status();
        let invalid_body = to_bytes(invalid.into_body(), usize::MAX).await.unwrap();
        assert_eq!(invalid_status, StatusCode::BAD_REQUEST);
        let invalid_text = String::from_utf8_lossy(&invalid_body);
        assert!(!invalid_text.contains(&remote_url));
        assert!(!invalid_text.contains("token=secret"));
        assert!(!invalid_text.contains(&raw_token));
        assert!(!invalid_text.contains("source_uri"));
        assert!(!invalid_text.contains("cache_uri"));
        assert!(!invalid_text.contains("storage_uri"));
        assert!(!invalid_text.contains("managed-artwork://"));
        assert!(!invalid_text.contains(temp.path().to_string_lossy().as_ref()));
        assert!(!invalid_text.contains("\"content_hash\""));
    }

    let missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/images/{}", taru_core::SelectedArtworkId::new()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let missing_status = missing.status();
    let missing_body = to_bytes(missing.into_body(), usize::MAX).await.unwrap();
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    let missing_text = String::from_utf8_lossy(&missing_body);
    assert!(!missing_text.contains("managed-artwork://"));
    assert!(!missing_text.contains(temp.path().to_string_lossy().as_ref()));
}

#[tokio::test]
async fn admin_managed_artwork_lifecycle_dry_run_protects_selected_artwork_and_redacts_locators() {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let raw_token = register_artwork_addon(&router, library_id).await;

    let first_png = tiny_png();
    let (first_remote_url, _) =
        artwork_server(StatusCode::OK, "image/png", first_png.clone()).await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &first_remote_url,
        "artwork-candidate-lifecycle-selected",
        &raw_token,
    )
    .await;
    let first_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let first_artifact = first_processed.artifact.as_ref().unwrap();
    request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", first_artifact.id),
    )
    .await;

    let second_png = tiny_png();
    let (second_remote_url, _) = artwork_server(StatusCode::OK, "image/png", second_png).await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &second_remote_url,
        "artwork-candidate-lifecycle-orphan",
        &raw_token,
    )
    .await;
    let second_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let second_artifact = second_processed.artifact.as_ref().unwrap();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/artwork/artifacts/lifecycle")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let lifecycle: AdminManagedArtworkArtifactLifecycleResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(lifecycle.dry_run);
    assert_eq!(lifecycle.summary.total_artifacts, 2);
    assert_eq!(lifecycle.summary.protected_artifacts, 1);
    assert_eq!(lifecycle.summary.cleanup_candidate_artifacts, 1);
    assert!(
        lifecycle
            .artifacts
            .iter()
            .any(|artifact| artifact.id == first_artifact.id
                && artifact.selected_artwork_count == 1
                && !artifact.cleanup_candidate)
    );
    assert!(
        lifecycle
            .artifacts
            .iter()
            .any(|artifact| artifact.id == second_artifact.id
                && artifact.selected_artwork_count == 0
                && artifact.cleanup_candidate)
    );

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&first_remote_url));
    assert!(!response_text.contains(&second_remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(first_artifact.has_content_hash);
    assert!(second_artifact.has_content_hash);
    assert!(!response_text.contains("\"content_hash\""));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));

    let cleanup_only = request_json::<AdminManagedArtworkArtifactLifecycleResponse>(
        &router,
        Method::GET,
        "/admin/v1/artwork/artifacts/lifecycle?cleanup_candidates_only=true",
    )
    .await;
    assert_eq!(cleanup_only.artifacts.len(), 1);
    assert_eq!(cleanup_only.artifacts[0].id, second_artifact.id);
    assert!(cleanup_only.artifacts[0].cleanup_candidate);
}

#[tokio::test]
async fn admin_managed_artwork_cleanup_removes_only_unselected_artifacts_without_locator_leaks() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let raw_token = register_artwork_addon(&router, library_id).await;

    let (selected_remote_url, _) = tiny_artwork_server().await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &selected_remote_url,
        "artwork-candidate-cleanup-selected",
        &raw_token,
    )
    .await;
    let selected_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let selected_artifact = selected_processed.artifact.as_ref().unwrap();
    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/artwork/artifacts/{}/publish",
            selected_artifact.id
        ),
    )
    .await;

    let (orphan_remote_url, _) = tiny_artwork_server().await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &orphan_remote_url,
        "artwork-candidate-cleanup-orphan",
        &raw_token,
    )
    .await;
    let orphan_processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let orphan_artifact = orphan_processed.artifact.as_ref().unwrap();

    let cleanup_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/artifacts/cleanup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let cleanup_status = cleanup_response.status();
    let cleanup_body = to_bytes(cleanup_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        cleanup_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&cleanup_body)
    );
    let cleanup: AdminManagedArtworkArtifactCleanupResponse =
        serde_json::from_slice(&cleanup_body).unwrap();

    assert!(!cleanup.dry_run);
    assert_eq!(cleanup.examined_artifacts, 1);
    assert_eq!(cleanup.cleanup_candidate_artifacts, 1);
    assert_eq!(cleanup.cleaned_artifacts.len(), 1);
    assert_eq!(cleanup.cleaned_artifacts[0].id, orphan_artifact.id);
    assert_eq!(cleanup.file_deleted_artifacts, 1);
    assert_eq!(cleanup.file_missing_artifacts, 0);
    assert_eq!(cleanup.file_delete_failed_artifacts, 0);
    assert!(
        store
            .get_managed_artwork_artifact(selected_artifact.id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .get_managed_artwork_artifact(orphan_artifact.id)
            .await
            .unwrap()
            .is_none()
    );

    let selected_image = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(published.image.url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(selected_image.status(), StatusCode::OK);

    let cleanup_text = String::from_utf8_lossy(&cleanup_body);
    assert!(!cleanup_text.contains(&selected_remote_url));
    assert!(!cleanup_text.contains(&orphan_remote_url));
    assert!(!cleanup_text.contains("token=secret"));
    assert!(!cleanup_text.contains(&raw_token));
    assert!(!cleanup_text.contains("source_uri"));
    assert!(!cleanup_text.contains("cache_uri"));
    assert!(!cleanup_text.contains("storage_uri"));
    assert!(!cleanup_text.contains("managed-artwork://"));
    assert!(!cleanup_text.contains("\"content_hash\""));
    assert!(selected_artifact.has_content_hash);
    assert!(orphan_artifact.has_content_hash);
    assert!(!cleanup_text.contains(temp.path().to_string_lossy().as_ref()));

    let lifecycle = request_json::<AdminManagedArtworkArtifactLifecycleResponse>(
        &router,
        Method::GET,
        "/admin/v1/artwork/artifacts/lifecycle",
    )
    .await;
    assert_eq!(lifecycle.summary.total_artifacts, 1);
    assert_eq!(lifecycle.summary.protected_artifacts, 1);
    assert_eq!(lifecycle.summary.cleanup_candidate_artifacts, 0);
}

#[tokio::test]
async fn admin_managed_artwork_storage_drift_reports_missing_and_stray_files_without_locator_leaks()
{
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let raw_token = register_artwork_addon(&router, library_id).await;

    let (remote_url, _) = tiny_artwork_server().await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-storage-drift",
        &raw_token,
    )
    .await;
    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap();
    request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;

    let missing_path = managed_artwork_artifact_test_path(&temp, artifact.id, "png");
    fs::remove_file(&missing_path).unwrap();

    let stray_artifact_id = ManagedArtworkArtifactId::new();
    let stray_path = managed_artwork_artifact_test_path(&temp, stray_artifact_id, "png");
    fs::create_dir_all(stray_path.parent().unwrap()).unwrap();
    fs::write(&stray_path, b"stray-private-file-token").unwrap();

    let private_filename = "private-filename-token.txt";
    let unrecognized_path = temp
        .path()
        .join("taru-cache")
        .join("artwork")
        .join("zz")
        .join(private_filename);
    fs::create_dir_all(unrecognized_path.parent().unwrap()).unwrap();
    fs::write(&unrecognized_path, b"private-unrecognized-file-token").unwrap();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/artwork/artifacts/storage-drift?file_scan_limit=50")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let drift: AdminManagedArtworkArtifactStorageDriftResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(drift.dry_run);
    assert_eq!(drift.summary.scanned_db_artifacts, 1);
    assert_eq!(drift.summary.db_backed_present_artifacts, 0);
    assert_eq!(drift.summary.db_backed_missing_artifacts, 1);
    assert_eq!(drift.summary.db_backed_unresolvable_artifacts, 0);
    assert_eq!(drift.summary.db_backed_metadata_read_failed_artifacts, 0);
    assert_eq!(drift.missing_artifacts.len(), 1);
    assert_eq!(drift.missing_artifacts[0].id, artifact.id);
    assert_eq!(drift.missing_artifacts[0].selected_artwork_count, 1);
    assert!(!drift.missing_artifacts[0].cleanup_candidate);
    assert_eq!(
        drift.missing_artifacts[0].issue,
        AdminManagedArtworkArtifactStorageDriftArtifactIssue::MissingFile
    );

    assert_eq!(drift.summary.file_scan_limit, 50);
    assert_eq!(drift.summary.scanned_files, 2);
    assert_eq!(drift.summary.stray_files, 2);
    assert_eq!(drift.summary.untracked_artifact_files, 1);
    assert_eq!(drift.summary.unrecognized_layout_files, 1);
    assert!(!drift.summary.file_scan_truncated);
    assert!(drift.stray_files.iter().any(|file| {
        file.reason == AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile
            && file.recognized_artifact_id == Some(stray_artifact_id)
            && file.extension.as_deref() == Some("png")
    }));
    assert!(drift.stray_files.iter().any(|file| {
        file.reason == AdminManagedArtworkArtifactStorageDriftFileReason::UnrecognizedLayout
            && file.recognized_artifact_id.is_none()
            && file.extension.is_none()
    }));

    let body = String::from_utf8_lossy(&response_body);
    assert!(!body.contains(&remote_url));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains(&raw_token));
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("cache_uri"));
    assert!(!body.contains("storage_uri"));
    assert!(!body.contains("managed-artwork://"));
    assert!(!body.contains("\"content_hash\""));
    assert!(artifact.has_content_hash);
    assert!(!body.contains(temp.path().to_string_lossy().as_ref()));
    assert!(!body.contains(private_filename));
    assert!(!body.contains(&format!("{stray_artifact_id}.png")));
    assert!(!body.contains("stray-private-file-token"));
    assert!(!body.contains("private-unrecognized-file-token"));
}

#[tokio::test]
async fn admin_managed_artwork_remediation_requires_confirmation_and_deletes_only_untracked_artifact_files()
 {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let raw_token = register_artwork_addon(&router, library_id).await;

    let (remote_url, _) = tiny_artwork_server().await;
    let (_candidate_id, _accepted) = propose_and_accept_remote_artwork_with_token(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-remediation",
        &raw_token,
    )
    .await;
    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let artifact = processed.artifact.as_ref().unwrap();
    request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;

    let expected_path = managed_artwork_artifact_test_path(&temp, artifact.id, "png");
    fs::remove_file(&expected_path).unwrap();
    let unexpected_active_path = managed_artwork_artifact_test_path(&temp, artifact.id, "jpg");
    fs::write(
        &unexpected_active_path,
        b"active-artifact-private-file-token",
    )
    .unwrap();

    let stray_artifact_id = ManagedArtworkArtifactId::new();
    let stray_path = managed_artwork_artifact_test_path(&temp, stray_artifact_id, "png");
    fs::create_dir_all(stray_path.parent().unwrap()).unwrap();
    fs::write(&stray_path, b"stray-remediation-private-file-token").unwrap();

    let private_filename = "manual-private-remediation-token.txt";
    let manual_path = temp
        .path()
        .join("taru-cache")
        .join("artwork")
        .join("yy")
        .join(private_filename);
    fs::create_dir_all(manual_path.parent().unwrap()).unwrap();
    fs::write(&manual_path, b"manual-private-remediation-token").unwrap();

    let plan = request_json::<AdminManagedArtworkArtifactRemediationPlanResponse>(
        &router,
        Method::GET,
        "/admin/v1/artwork/artifacts/remediation-plan?file_scan_limit=50",
    )
    .await;
    assert!(plan.dry_run);
    assert_eq!(plan.summary.scanned_db_artifacts, 1);
    assert_eq!(plan.summary.missing_db_backed_artifacts, 1);
    assert_eq!(plan.summary.selected_missing_artifacts, 1);
    assert_eq!(plan.summary.cleanup_candidate_missing_artifacts, 0);
    assert_eq!(plan.summary.cleanable_stray_files, 1);
    assert_eq!(plan.summary.blocked_stray_files, 2);
    assert_eq!(plan.missing_artifacts.len(), 1);
    assert_eq!(plan.missing_artifacts[0].id, artifact.id);
    assert_eq!(
        plan.missing_artifacts[0].recommendation,
        taru_api::admin::AdminManagedArtworkArtifactMissingRemediationRecommendation::RestoreOrRepublishSelectedArtwork
    );
    assert!(plan.stray_files.iter().any(|file| {
        file.action == AdminManagedArtworkArtifactStrayFileRemediationAction::DeleteStrayFile
            && file.recognized_artifact_id == Some(stray_artifact_id)
    }));
    assert!(plan.stray_files.iter().any(|file| {
        file.action == AdminManagedArtworkArtifactStrayFileRemediationAction::InspectManually
            && file.recognized_artifact_id == Some(artifact.id)
    }));

    let unconfirmed = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/artifacts/remediate-stray-files?file_scan_limit=50")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unconfirmed.status(), StatusCode::BAD_REQUEST);
    assert!(stray_path.exists());

    let confirmed_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/artifacts/remediate-stray-files?confirm=true&file_scan_limit=50")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let confirmed_status = confirmed_response.status();
    let confirmed_body = to_bytes(confirmed_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        confirmed_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&confirmed_body)
    );
    let cleanup: AdminManagedArtworkArtifactStrayFileCleanupResponse =
        serde_json::from_slice(&confirmed_body).unwrap();

    assert!(!cleanup.dry_run);
    assert_eq!(cleanup.summary.cleanable_stray_files, 1);
    assert_eq!(cleanup.summary.blocked_stray_files, 2);
    assert_eq!(cleanup.summary.deleted_files, 1);
    assert_eq!(cleanup.summary.missing_files, 0);
    assert_eq!(cleanup.summary.failed_files, 0);
    assert_eq!(cleanup.cleaned_files.len(), 1);
    assert_eq!(
        cleanup.cleaned_files[0].recognized_artifact_id,
        stray_artifact_id
    );
    assert_eq!(
        cleanup.cleaned_files[0].status,
        AdminManagedArtworkArtifactStrayFileCleanupStatus::Deleted
    );
    assert_eq!(cleanup.blocked_files.len(), 2);
    assert!(!stray_path.exists());
    assert!(unexpected_active_path.exists());
    assert!(manual_path.exists());

    let body = String::from_utf8_lossy(&confirmed_body);
    assert!(!body.contains(&remote_url));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains(&raw_token));
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("cache_uri"));
    assert!(!body.contains("storage_uri"));
    assert!(!body.contains("managed-artwork://"));
    assert!(!body.contains("\"content_hash\""));
    assert!(artifact.has_content_hash);
    assert!(!body.contains(temp.path().to_string_lossy().as_ref()));
    assert!(!body.contains(private_filename));
    assert!(!body.contains(&format!("{stray_artifact_id}.png")));
    assert!(!body.contains("stray-remediation-private-file-token"));
    assert!(!body.contains("active-artifact-private-file-token"));
    assert!(!body.contains("manual-private-remediation-token"));
}

fn managed_artwork_artifact_test_path(
    temp: &tempfile::TempDir,
    artifact_id: ManagedArtworkArtifactId,
    extension: &str,
) -> PathBuf {
    let artifact_id_text = artifact_id.to_string();
    temp.path()
        .join("taru-cache")
        .join("artwork")
        .join(artifact_id_text.get(0..2).unwrap())
        .join(format!("{artifact_id_text}.{extension}"))
}

#[tokio::test]
async fn admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_unsupported_media_type()
 {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (remote_url, _) = artwork_server(
        StatusCode::OK,
        "text/plain",
        b"not-an-image token=secret".to_vec(),
    )
    .await;
    let (raw_token, _candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-process-next-unsupported-media-type",
    )
    .await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/ingests/process-next")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let processed: ProcessManagedArtworkIngestResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(processed.processed);
    let ingest = processed.ingest.as_ref().unwrap();
    let job = processed.job.as_ref().unwrap();
    assert_eq!(ingest.id, accepted.ingest.id);
    assert_eq!(ingest.status, ManagedArtworkIngestStatus::Failed);
    assert!(!ingest.has_artifact);
    assert!(ingest.has_failure);
    assert_eq!(
        ingest.failure_code.as_deref(),
        Some("unsupported_media_type")
    );
    assert!(processed.artifact.is_none());
    assert_eq!(job.id, accepted.job.id);
    assert_eq!(job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(job.status, JobStatus::Failed);
    assert!(job.has_input);
    assert!(job.has_summary);
    assert!(job.has_error);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("not-an-image"));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));

    let stored_ingest = store
        .get_managed_artwork_ingest(ingest.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_ingest.status, ManagedArtworkIngestStatus::Failed);
    assert_eq!(stored_ingest.artifact_id, None);
    assert_eq!(
        stored_ingest.failure_code.as_deref(),
        Some("unsupported_media_type")
    );
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn admin_managed_artwork_ingest_requeue_retries_failed_ingest_without_leaks() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let expected_png = tiny_png();
    let (remote_url, expected_byte_len) = changing_artwork_server(
        StatusCode::OK,
        "text/plain",
        b"not-an-image token=secret".to_vec(),
        StatusCode::OK,
        "image/png",
        expected_png,
    )
    .await;
    let (raw_token, _candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-requeue-retry",
    )
    .await;

    let failed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let failed_ingest = failed.ingest.as_ref().unwrap();
    let failed_job = failed.job.as_ref().unwrap();
    assert!(failed.processed);
    assert_eq!(failed_ingest.id, accepted.ingest.id);
    assert_eq!(failed_ingest.status, ManagedArtworkIngestStatus::Failed);
    assert!(failed_ingest.has_failure);
    assert_eq!(
        failed_ingest.failure_code.as_deref(),
        Some("unsupported_media_type")
    );
    assert!(!failed_ingest.has_artifact);
    assert_eq!(failed_job.id, accepted.job.id);
    assert_eq!(failed_job.status, JobStatus::Failed);
    assert!(failed_job.has_input);
    assert!(failed_job.has_summary);
    assert!(failed_job.has_error);
    assert!(failed.artifact.is_none());

    let requeue_path = format!("/admin/v1/artwork/ingests/{}/requeue", accepted.ingest.id);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&requeue_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let requeued: RequeueManagedArtworkIngestResponse =
        serde_json::from_slice(&response_body).unwrap();
    assert!(requeued.requeued);
    assert!(requeued.had_failure);
    assert_eq!(requeued.ingest.id, accepted.ingest.id);
    assert_eq!(requeued.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert!(!requeued.ingest.has_failure);
    assert!(!requeued.ingest.has_artifact);
    assert_eq!(requeued.job.id, accepted.job.id);
    assert_eq!(requeued.job.status, JobStatus::Queued);
    assert!(requeued.job.has_input);
    assert!(!requeued.job.has_summary);
    assert!(!requeued.job.has_error);
    assert_eq!(requeued.job.started_at, None);
    assert_eq!(requeued.job.completed_at, None);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("not-an-image"));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains("summary_json"));
    assert!(!response_text.contains("input_json"));

    let replay =
        request_json::<RequeueManagedArtworkIngestResponse>(&router, Method::POST, &requeue_path)
            .await;
    assert!(!replay.requeued);
    assert!(!replay.had_failure);
    assert_eq!(replay.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(replay.job.status, JobStatus::Queued);

    let stored = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    let stored_ingest = stored.ingest.as_ref().unwrap();
    let artifact = stored.artifact.as_ref().unwrap();
    let stored_job = stored.job.as_ref().unwrap();
    assert!(stored.processed);
    assert_eq!(stored_ingest.id, accepted.ingest.id);
    assert_eq!(stored_ingest.status, ManagedArtworkIngestStatus::Stored);
    assert!(stored_ingest.has_artifact);
    assert!(!stored_ingest.has_failure);
    assert_eq!(stored_job.id, accepted.job.id);
    assert_eq!(stored_job.status, JobStatus::Succeeded);
    assert_eq!(artifact.ingest_id, accepted.ingest.id);
    assert_eq!(artifact.byte_len, Some(expected_byte_len));
    assert_eq!(artifact.media_type.as_deref(), Some("image/png"));

    let persisted_ingest = store
        .get_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted_ingest.status, ManagedArtworkIngestStatus::Stored);
    assert_eq!(persisted_ingest.artifact_id, Some(artifact.id));
    assert_eq!(persisted_ingest.failure_code, None);

    let stored_requeue_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&requeue_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stored_requeue_response.status(), StatusCode::CONFLICT);
    let stored_requeue_body = to_bytes(stored_requeue_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stored_requeue_text = String::from_utf8_lossy(&stored_requeue_body);
    assert!(!stored_requeue_text.contains(&remote_url));
    assert!(!stored_requeue_text.contains("token=secret"));
    assert!(!stored_requeue_text.contains(&raw_token));
}

#[tokio::test]
async fn admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_invalid_image()
 {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let (remote_url, _) = artwork_server(
        StatusCode::OK,
        "image/png",
        b"not-an-image token=secret".to_vec(),
    )
    .await;
    let (raw_token, _candidate_id, accepted) = propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "artwork-candidate-process-next-invalid-image",
    )
    .await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/ingests/process-next")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let processed: ProcessManagedArtworkIngestResponse =
        serde_json::from_slice(&response_body).unwrap();

    assert!(processed.processed);
    let ingest = processed.ingest.as_ref().unwrap();
    let job = processed.job.as_ref().unwrap();
    assert_eq!(ingest.id, accepted.ingest.id);
    assert_eq!(ingest.status, ManagedArtworkIngestStatus::Failed);
    assert!(!ingest.has_artifact);
    assert!(ingest.has_failure);
    assert_eq!(ingest.failure_code.as_deref(), Some("invalid_image"));
    assert!(processed.artifact.is_none());
    assert_eq!(job.status, JobStatus::Failed);
    assert!(job.has_input);
    assert!(job.has_summary);
    assert!(job.has_error);

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("not-an-image"));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));

    let stored_ingest = store
        .get_managed_artwork_ingest(ingest.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_ingest.status, ManagedArtworkIngestStatus::Failed);
    assert_eq!(stored_ingest.artifact_id, None);
    assert_eq!(stored_ingest.failure_code.as_deref(), Some("invalid_image"));
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn addon_side_effect_artwork_write_rejects_unsafe_payloads_and_media_source_targets() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("artwork runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::ArtworkWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    for (idempotency_key, payload, leaked) in [
        (
            "artwork-cache-uri-denied",
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "https://artwork.example.test/poster.jpg"
                },
                "cache_uri": "local:///cache/poster.webp"
            }),
            "local:///cache/poster.webp",
        ),
        (
            "artwork-public-flag-denied",
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "https://artwork.example.test/poster.jpg"
                },
                "selected": true
            }),
            "selected",
        ),
        (
            "artwork-file-url-denied",
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "file:///Movies/poster.jpg"
                }
            }),
            "file:///Movies/poster.jpg",
        ),
        (
            "artwork-data-uri-denied",
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "data:image/png;base64,AAAA"
                }
            }),
            "data:image/png",
        ),
        (
            "artwork-source-locator-denied",
            serde_json::json!({
                "intent": "propose_artwork",
                "kind": "poster",
                "source": {
                    "kind": "remote_url",
                    "url": "local:///Movies/poster.jpg"
                }
            }),
            "local:///Movies/poster.jpg",
        ),
    ] {
        let request = SubmitAddonSideEffectRequest {
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTargetRequest {
                kind: AddonSideEffectTargetKind::MediaItem,
                id: source.item_id.to_string(),
            },
            idempotency_key: idempotency_key.to_owned(),
            provenance: serde_json::json!({"origin": "reference-addon"}),
            payload,
        };

        let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<ErrorResponse>(&response_body)
                .unwrap()
                .code,
            "invalid_input"
        );
        assert!(
            !String::from_utf8_lossy(&response_body).contains(leaked),
            "unsafe payload detail leaked for {idempotency_key}"
        );

        let replay = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
        assert_eq!(replay.status(), StatusCode::OK);
        let replay_body = to_bytes(replay.into_body(), usize::MAX).await.unwrap();
        let replay = serde_json::from_slice::<AddonSideEffectResponse>(&replay_body).unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(
            replay.side_effect.validation_status,
            AddonSideEffectValidationStatus::Accepted
        );
        assert_eq!(
            replay.side_effect.apply_status,
            AddonSideEffectApplyStatus::Failed
        );
        assert_eq!(
            replay.side_effect.apply_error_code.as_deref(),
            Some("invalid_payload")
        );
        assert!(
            !String::from_utf8_lossy(&replay_body).contains(leaked),
            "unsafe replay detail leaked for {idempotency_key}"
        );
    }

    let media_source_request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::ArtworkWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "artwork-media-source-denied".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "intent": "propose_artwork",
            "kind": "poster",
            "source": {
                "kind": "remote_url",
                "url": "https://artwork.example.test/poster.jpg"
            }
        }),
    };
    let media_source =
        addon_side_effect(&router, Some(&issued.raw_token), &media_source_request).await;
    assert_eq!(media_source.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(media_source).await;
    assert_eq!(error.code, "invalid_input");

    let replay = addon_side_effect(&router, Some(&issued.raw_token), &media_source_request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json::<AddonSideEffectResponse>(replay).await;
    assert!(replay.idempotent_replay);
    assert_eq!(
        replay.side_effect.validation_status,
        AddonSideEffectValidationStatus::Rejected
    );
    assert_eq!(
        replay.side_effect.apply_status,
        AddonSideEffectApplyStatus::Skipped
    );
    assert_eq!(
        replay.side_effect.safe_error_code.as_deref(),
        Some("invalid_target")
    );

    assert!(
        store
            .list_artwork_candidates_for_item(source.item_id, taru_core::PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_item_images(source.item_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn addon_side_effect_metadata_write_scalar_patch_preserves_catalog_graph_sources() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let tmdb_genre = Genre {
        id: GenreId::new(),
        name: "Existing Genre".to_owned(),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    let tmdb_tag = Tag {
        id: TagId::new(),
        name: "existing-tag".to_owned(),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    store.upsert_genre(&tmdb_genre).await.unwrap();
    store
        .upsert_item_genre(&ItemGenre {
            item_id: source.item_id,
            genre_id: tmdb_genre.id,
        })
        .await
        .unwrap();
    store.upsert_tag(&tmdb_tag).await.unwrap();
    store
        .upsert_item_tag(&ItemTag {
            item_id: source.item_id,
            tag_id: tmdb_tag.id,
        })
        .await
        .unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                source.item_id,
                "demo.mkv",
                "demo.mkv Existing Genre existing-tag",
                vec![
                    "genre:Existing Genre".to_owned(),
                    "tag:existing-tag".to_owned(),
                ],
            )
            .unwrap(),
        )
        .await
        .unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-scalar-only".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Scalar Addon Title",
            "overview": "Scalar-only update."
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json::<AddonSideEffectResponse>(response).await;
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );

    let item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item was updated");
    assert_eq!(item.metadata.title, "Scalar Addon Title");
    let item_genres = store.list_item_genres(source.item_id).await.unwrap();
    let item_tags = store.list_item_tags(source.item_id).await.unwrap();
    assert_eq!(
        item_genres,
        vec![ItemGenre {
            item_id: source.item_id,
            genre_id: tmdb_genre.id,
        }]
    );
    assert_eq!(
        item_tags,
        vec![ItemTag {
            item_id: source.item_id,
            tag_id: tmdb_tag.id,
        }]
    );
    let genres = store
        .list_genres(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    let tags = store
        .list_tags(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(genres, vec![tmdb_genre]);
    assert_eq!(tags, vec![tmdb_tag]);
    let hits = store
        .search(
            SearchQuery::from_facet_labels(
                "Scalar-only",
                vec![
                    "genre:Existing Genre".to_owned(),
                    "tag:existing-tag".to_owned(),
                ],
                10,
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hits[0].item_id, source.item_id);
}

#[tokio::test]
async fn addon_side_effect_metadata_write_label_patch_only_replaces_touched_catalog_labels() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let tmdb_genre = Genre {
        id: GenreId::new(),
        name: "Existing Genre".to_owned(),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    store.upsert_genre(&tmdb_genre).await.unwrap();
    store
        .upsert_item_genre(&ItemGenre {
            item_id: source.item_id,
            genre_id: tmdb_genre.id,
        })
        .await
        .unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-tags-only".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "tags": ["addon-tag"]
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json::<AddonSideEffectResponse>(response).await;
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );

    let item_genres = store.list_item_genres(source.item_id).await.unwrap();
    assert_eq!(
        item_genres,
        vec![ItemGenre {
            item_id: source.item_id,
            genre_id: tmdb_genre.id,
        }]
    );
    let genres = store
        .list_genres(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(genres, vec![tmdb_genre]);
    let tags = store
        .list_tags(taru_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "addon-tag");
    assert_eq!(tags[0].source, MetadataSource::Addon(addon_id));
    let hits = store
        .search(
            SearchQuery::from_facet_labels(
                "addon-tag",
                vec!["genre:Existing Genre".to_owned()],
                10,
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hits[0].item_id, source.item_id);
}

#[tokio::test]
async fn addon_side_effect_intake_rejects_unauthorized_scope_revoked_token_and_bad_targets() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let ungranted_library_id = LibraryId::new();
    store
        .upsert_library(&taru_core::Library {
            id: ungranted_library_id,
            name: "Other Movies".to_owned(),
            roots: vec!["local:///Other".to_owned()],
            options: taru_core::LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let token_path = format!("/admin/v1/addons/{addon_id}/tokens");
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &token_path,
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;

    let grants_path = format!("/admin/v1/addons/{addon_id}/grants");
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(ungranted_library_id),
            }],
        },
    )
    .await;

    let denied = AddonSideEffectTargetRequest {
        kind: AddonSideEffectTargetKind::MediaSource,
        id: source.id.to_string(),
    };
    let wrong_library_request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: denied.clone(),
        idempotency_key: "metadata-wrong-library".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({"title": "Denied"}),
    };

    let wrong_library =
        addon_side_effect(&router, Some(&issued.raw_token), &wrong_library_request).await;
    assert_eq!(wrong_library.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        body_json::<ErrorResponse>(wrong_library).await.code,
        "forbidden"
    );

    let wrong_library_replay =
        addon_side_effect(&router, Some(&issued.raw_token), &wrong_library_request).await;
    assert_eq!(wrong_library_replay.status(), StatusCode::OK);
    let wrong_library_replay = body_json::<AddonSideEffectResponse>(wrong_library_replay).await;
    assert!(wrong_library_replay.idempotent_replay);
    assert_eq!(
        wrong_library_replay.side_effect.validation_status,
        AddonSideEffectValidationStatus::Rejected
    );
    assert_eq!(
        wrong_library_replay.side_effect.apply_status,
        AddonSideEffectApplyStatus::Skipped
    );
    assert_eq!(
        wrong_library_replay.side_effect.safe_error_code.as_deref(),
        Some("forbidden")
    );

    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &grants_path,
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let missing_permission = SubmitAddonSideEffectRequest {
        permission: AddonPermission::SubtitleWrite,
        library_id,
        target: denied.clone(),
        idempotency_key: "metadata-missing-permission".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({"title": "Denied"}),
    };
    let missing_permission_response =
        addon_side_effect(&router, Some(&issued.raw_token), &missing_permission).await;
    assert_eq!(missing_permission_response.status(), StatusCode::FORBIDDEN);

    let malformed_target = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: "not-a-uuid".to_owned(),
        },
        idempotency_key: "metadata-bad-target".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({"title": "Denied"}),
    };
    let malformed_target_response =
        addon_side_effect(&router, Some(&issued.raw_token), &malformed_target).await;
    assert_eq!(malformed_target_response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json::<ErrorResponse>(malformed_target_response)
            .await
            .code,
        "invalid_input"
    );

    let revoked_path = format!(
        "/admin/v1/addons/{addon_id}/tokens/{}/revoke",
        issued.token.id
    );
    request_json::<AddonTokenResponse>(&router, Method::POST, &revoked_path).await;
    let revoked_token =
        addon_side_effect(&router, Some(&issued.raw_token), &malformed_target).await;
    assert_eq!(revoked_token.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn addon_side_effect_metadata_write_records_apply_failure_without_leaking_payload() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![AddonGrantAssignment {
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library_id),
            }],
        },
    )
    .await;

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-bad-payload".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Should Not Apply",
            "raw_path": "local:///Movies/demo.mkv"
        }),
    };

    let response = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json::<ErrorResponse>(response).await.code,
        "invalid_input"
    );

    let replay = addon_side_effect(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay_body = to_bytes(replay.into_body(), usize::MAX).await.unwrap();
    let replay = serde_json::from_slice::<AddonSideEffectResponse>(&replay_body).unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(
        replay.side_effect.validation_status,
        AddonSideEffectValidationStatus::Accepted
    );
    assert_eq!(
        replay.side_effect.apply_status,
        AddonSideEffectApplyStatus::Failed
    );
    assert_eq!(
        replay.side_effect.apply_error_code.as_deref(),
        Some("invalid_payload")
    );
    let replay_body = String::from_utf8_lossy(&replay_body);
    assert!(!replay_body.contains("Should Not Apply"));
    assert!(!replay_body.contains("local:///Movies/demo.mkv"));

    let item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item still exists");
    assert_eq!(item.metadata.title, "demo.mkv");
}

async fn register_addon_with_admin_token(
    router: &Router,
    admin_token: &str,
) -> AdminAddonRegistrationResponse {
    request_body_json_with_bearer(
        router,
        Method::POST,
        "/admin/v1/addons",
        admin_token,
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await
}

async fn request_body_json_with_bearer<T, B>(
    router: &Router,
    method: Method,
    uri: &str,
    bearer_token: &str,
    body: &B,
) -> T
where
    T: DeserializeOwned,
    B: Serialize,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn addon_access_check(
    router: &Router,
    raw_token: Option<&str>,
    request: AddonAccessCheckRequest,
) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/addon/v1/access-check")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(raw_token) = raw_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {raw_token}"));
    }

    router
        .clone()
        .oneshot(
            builder
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn addon_side_effect(
    router: &Router,
    raw_token: Option<&str>,
    request: &SubmitAddonSideEffectRequest,
) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/addon/v1/side-effects")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(raw_token) = raw_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {raw_token}"));
    }

    router
        .clone()
        .oneshot(
            builder
                .body(Body::from(serde_json::to_vec(request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}
