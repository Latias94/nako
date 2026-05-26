use super::*;

fn context_link<'a>(
    response: &'a ManagementContextLinksResponse,
    route_name: &str,
) -> &'a ManagementContextLinkDto {
    response
        .links
        .iter()
        .find(|link| link.route_name == route_name)
        .unwrap_or_else(|| panic!("missing management context link {route_name}"))
}

#[tokio::test]
async fn management_context_links_enable_bootstrap_admin_actions_without_admin_urls() {
    let (_temp, router, source, _store) = router_with_media_source("Demo.mkv", b"media").await;
    let uri = format!("/management/context-links?source_id={}", source.id);

    let response = request_json::<ManagementContextLinksResponse>(&router, Method::GET, &uri).await;

    assert_eq!(
        response.context.library_id,
        Some(source.library_id.to_string())
    );
    assert_eq!(response.context.item_id, Some(source.item_id.to_string()));
    assert_eq!(response.context.source_id, Some(source.id.to_string()));
    for route in [
        "library.scan",
        "library.metadata_profile",
        "item.metadata_refresh",
        "jobs.filtered",
        "playback.support",
        "playback.runtime",
        "access.library_policies",
    ] {
        let link = context_link(&response, route);
        assert!(
            link.enabled,
            "{route} should be enabled for bootstrap admin"
        );
        assert_eq!(link.disabled_reason, None);
    }
    assert_eq!(
        context_link(&response, "library.scan").method.wire_value(),
        "POST"
    );
    assert_eq!(
        context_link(&response, "playback.runtime")
            .method
            .wire_value(),
        "GET"
    );

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(!serialized.contains("/admin/v1"));
    assert!(!serialized.contains("local:///"));
    assert!(!serialized.contains("Demo.mkv"));
}

#[tokio::test]
async fn management_context_links_allow_library_manager_scoped_operations_only() {
    let (_temp, app, source, store) =
        app_with_media_source_config("Manager Demo.mkv", b"media", |_| {}).await;
    let principal = local_principal_with_library_access(
        &store,
        source.library_id,
        UserRole::LibraryManager,
        LibraryAccessLevel::Manage,
    )
    .await;
    let router = public_client_router_with_principal(app, principal);
    let uri = format!("/management/context-links?source_id={}", source.id);

    let response = request_json::<ManagementContextLinksResponse>(&router, Method::GET, &uri).await;

    assert!(context_link(&response, "library.scan").enabled);
    assert!(context_link(&response, "item.metadata_refresh").enabled);
    let runtime = context_link(&response, "playback.runtime");
    assert!(!runtime.enabled);
    assert_eq!(
        runtime
            .disabled_reason
            .as_ref()
            .map(|reason| reason.wire_value()),
        Some("insufficient_permission")
    );
    let access = context_link(&response, "access.library_policies");
    assert!(!access.enabled);
    assert_eq!(
        access
            .disabled_reason
            .as_ref()
            .map(|reason| reason.wire_value()),
        Some("insufficient_permission")
    );
}

#[tokio::test]
async fn management_context_links_disable_viewer_management_actions() {
    let (_temp, app, source, store) =
        app_with_media_source_config("Viewer Demo.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    let router = public_client_router_with_principal(app, principal);
    let uri = format!("/management/context-links?source_id={}", source.id);

    let response = request_json::<ManagementContextLinksResponse>(&router, Method::GET, &uri).await;

    for route in ["library.scan", "item.metadata_refresh", "playback.runtime"] {
        let link = context_link(&response, route);
        assert!(!link.enabled, "{route} should be disabled for viewer");
        assert_eq!(
            link.disabled_reason
                .as_ref()
                .map(|reason| reason.wire_value()),
            Some("insufficient_permission")
        );
    }

    let refresh = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/items/{}/metadata/refresh", source.item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn management_context_links_hide_no_access_source_context() {
    let (_temp, app, source, store) =
        app_with_media_source_config("Hidden Demo.mkv", b"media", |_| {}).await;
    let principal = local_principal_with_library_access(
        &store,
        source.library_id,
        UserRole::Viewer,
        LibraryAccessLevel::None,
    )
    .await;
    let router = public_client_router_with_principal(app, principal);
    let uri = format!("/management/context-links?source_id={}", source.id);

    let response = response_for(&router, Method::GET, &uri).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = serde_json::to_string(&body_json::<ErrorResponse>(response).await).unwrap();
    assert!(!body.contains(&source.id.to_string()));
    assert!(!body.contains("Hidden Demo.mkv"));
}

#[tokio::test]
async fn management_context_links_reject_disabled_user_sessions_and_admin_viewer_access() {
    let (_temp, app, source, _store) =
        app_with_media_source_config("Disabled Demo.mkv", b"media", |_| {}).await;
    let admin_token = "test-admin-token";
    let router = build_router_with_auth(app, auth::InboundAuthState::bearer_token(admin_token));

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "disabled-context-viewer".to_owned(),
            display_name: "Disabled Context Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        admin_token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        admin_token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "disabled-context-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    let admin_runtime = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/playback/runtime")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_runtime.status(), StatusCode::FORBIDDEN);

    let status_path = format!("/admin/v1/access/users/{}/status", created.user.user_id);
    request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::PATCH,
        &status_path,
        &AdminUpdateUserStatusRequest {
            status: UserStatus::Disabled,
        },
        admin_token,
    )
    .await;

    let context_after_disable = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/management/context-links?source_id={}", source.id))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(context_after_disable.status(), StatusCode::UNAUTHORIZED);
}
