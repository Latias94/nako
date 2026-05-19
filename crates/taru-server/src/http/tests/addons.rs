use super::*;
use axum::http::HeaderValue;

fn tiny_png() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    cursor.into_inner()
}

async fn tiny_artwork_server() -> (String, u64) {
    artwork_server(StatusCode::OK, "image/png", tiny_png()).await
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

async fn propose_and_accept_remote_artwork(
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
    let registered = request_body_json::<AddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
    let registered = request_body_json::<AddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
async fn addon_routes_register_disabled_by_default_and_validate_contract() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = addon_manifest();

    let response = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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

    assert_eq!(response.addon.manifest_id, manifest.id);
    assert_eq!(response.addon.status, AddonStatus::Disabled);
    assert_eq!(
        response.addon.granted_scopes,
        vec!["item_metadata_suggest", "item_metadata_read"]
    );
    assert!(!response.addon.manifest_json.contains("token"));

    let disabled =
        request_json::<AddonRegistrationsResponse>(&router, Method::GET, "/addons?status=disabled")
            .await;
    assert_eq!(disabled.addons, vec![response.addon.clone()]);

    let enabled =
        request_json::<AddonRegistrationsResponse>(&router, Method::GET, "/addons?status=enabled")
            .await;
    assert!(enabled.addons.is_empty());

    let detail_path = format!("/addons/{}", response.addon.id);
    let detail =
        request_json::<AddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    assert_eq!(detail, response);

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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    assert_eq!(registered.addon.status, AddonStatus::Enabled);
    assert_eq!(
        registered.addon.manifest_id,
        taru_reference_addon::REFERENCE_ADDON_ID
    );

    let detail_path = format!("/addons/{}", registered.addon.id);
    let detail =
        request_json::<AddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    let stored_manifest =
        serde_json::from_str::<AddonManifest>(&detail.addon.manifest_json).unwrap();
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
async fn addon_admin_routes_issue_rotate_revoke_tokens_and_replace_grants_without_leaking_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;

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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
    let addon_id = registered.addon.id;
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
        .search(SearchQuery {
            query: "safe metadata".to_owned(),
            facets: vec!["tag:sidecar".to_owned()],
            limit: 10,
            offset: 0,
        })
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
    assert_eq!(report["file_role"], "nfo");
    assert_eq!(report["policy"], "create_missing");
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
    assert_eq!(
        accepted.job.input.as_ref().unwrap()["candidate_id"],
        candidate_id.to_string()
    );
    assert_eq!(accepted.job.input.as_ref().unwrap()["image_kind"], "poster");

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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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

    let response_text = String::from_utf8_lossy(&response_body);
    assert!(!response_text.contains(&remote_url));
    assert!(!response_text.contains("token=secret"));
    assert!(!response_text.contains(&raw_token));
    assert!(!response_text.contains("source_uri"));
    assert!(!response_text.contains("cache_uri"));
    assert!(!response_text.contains("storage_uri"));
    assert!(!response_text.contains("managed-artwork://"));
    assert!(!response_text.contains(temp.path().to_string_lossy().as_ref()));

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
async fn public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks() {
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let expected_bytes = tiny_png();
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
    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;

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
    let images: taru_api::ImagesResponse = serde_json::from_slice(&images_body).unwrap();
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
    let item_detail: taru_api::ItemDetailResponse =
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
    assert!(image_response.headers().get(header::ETAG).is_some());
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
    let head_body = to_bytes(head_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(head_body.is_empty());

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
    if let Some(content_hash) = first_artifact.content_hash.as_ref() {
        assert!(!response_text.contains(content_hash));
    }
    if let Some(content_hash) = second_artifact.content_hash.as_ref() {
        assert!(!response_text.contains(content_hash));
    }
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
    assert!(!cleanup_text.contains("content_hash"));
    if let Some(content_hash) = selected_artifact.content_hash.as_ref() {
        assert!(!cleanup_text.contains(content_hash));
    }
    if let Some(content_hash) = orphan_artifact.content_hash.as_ref() {
        assert!(!cleanup_text.contains(content_hash));
    }
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
    assert!(processed.artifact.is_none());
    assert_eq!(job.id, accepted.job.id);
    assert_eq!(job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error.as_deref(), Some("unsupported_media_type"));
    assert_eq!(
        job.summary.as_ref().unwrap()["failure_code"],
        "unsupported_media_type"
    );
    assert_eq!(job.summary.as_ref().unwrap()["status"], "failed");

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
    assert!(processed.artifact.is_none());
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error.as_deref(), Some("invalid_image"));
    assert_eq!(
        job.summary.as_ref().unwrap()["failure_code"],
        "invalid_image"
    );
    assert_eq!(job.summary.as_ref().unwrap()["status"], "failed");

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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
        .upsert(SearchDocument {
            item_id: source.item_id,
            title: "demo.mkv".to_owned(),
            body: "demo.mkv Existing Genre existing-tag".to_owned(),
            facets: vec![
                "genre:Existing Genre".to_owned(),
                "tag:existing-tag".to_owned(),
            ],
        })
        .await
        .unwrap();

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
        .search(SearchQuery {
            query: "Scalar-only".to_owned(),
            facets: vec![
                "genre:Existing Genre".to_owned(),
                "tag:existing-tag".to_owned(),
            ],
            limit: 10,
            offset: 0,
        })
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
        .search(SearchQuery {
            query: "addon-tag".to_owned(),
            facets: vec!["genre:Existing Genre".to_owned()],
            limit: 10,
            offset: 0,
        })
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
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
    let addon_id = registered.addon.id;
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
) -> AddonRegistrationResponse {
    request_body_json_with_bearer(
        router,
        Method::POST,
        "/addons",
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
