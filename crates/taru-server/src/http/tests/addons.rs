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
