use super::*;

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
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
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
