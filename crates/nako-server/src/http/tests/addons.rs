use super::*;
use axum::Json;
use axum::http::{HeaderMap, HeaderValue};
use nako_addon_client::{NakoRuntimeClient, NakoRuntimeClientConfig};
use nako_addon_protocol::{
    ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA,
    ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA,
    ADDON_RUNTIME_EXTERNAL_ACQUISITION_MATERIALIZE_PATH, ADDON_SUBTITLE_REQUEST_SCHEMA,
    ADDON_SUBTITLE_RESPONSE_SCHEMA, AddonExternalAcquisitionActionRequest,
    AddonExternalAcquisitionActionResponse, AddonExternalAcquisitionActionStatus,
    AddonExternalAcquisitionMaterializationPurpose, AddonExternalAcquisitionMaterializationRequest,
    AddonExternalAcquisitionMaterializationResponse, AddonExternalAcquisitionOperation,
    AddonExternalAcquisitionProgress, AddonExternalAcquisitionRunnerState,
    AddonExternalAcquisitionTargetRef, AddonSubtitleCandidate, AddonSubtitleDelivery,
    AddonSubtitleFormat, AddonSubtitleProviderExecution, AddonSubtitleProviderStatus,
    AddonSubtitleSearchResponse,
};
use nako_api::extension::{
    AdminAddonSubtitleDeliveryKind, AdminAddonSubtitleImportApplyRequest,
    AdminAddonSubtitleImportApplyResponse, AdminAddonSubtitleImportPlanRequest,
    AdminAddonSubtitleImportPlanResponse, AdminAddonSubtitleSearchRequest,
    AdminAddonSubtitleSearchResponse, AdminAddonSubtitleSelectionRequest,
    AdminAddonSubtitleSelectionResponse, AdminSubtitleImportApplyStatus,
    AdminSubtitleImportBackupPolicy, AdminSubtitleImportConflictPolicy,
    AdminSubtitleImportPlanReason, AdminSubtitleImportPlanStatus, AdminSubtitleSidecarRole,
};
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateState, AcquisitionIntakeRepository,
    AcquisitionIntakeSourceKind, NewAcquisitionIntakeCandidate,
};
use nako_official_addon_catalog::{
    chromecast_renderer, dlna_renderer, external_acquisition_runner, metadata_scraper,
    notification_bridge, resource_search, subtitle_provider,
};
use std::collections::VecDeque;
use std::sync::{
    Arc as StdArc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::{Mutex as TokioMutex, Notify};

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

fn assert_selected_artwork_cache_control(headers: &HeaderMap) {
    assert_eq!(
        headers.get(header::CACHE_CONTROL).unwrap(),
        HeaderValue::from_static("private, max-age=86400")
    );
}

async fn assert_selected_artwork_not_modified(response: Response, expected_etag: &str) {
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_selected_artwork_cache_control(response.headers());
    assert_eq!(
        response.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(expected_etag).unwrap()
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(body.is_empty());
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
                            .get("x-nako-addon-id")
                            .and_then(|value| value.to_str().ok()),
                        Some(manifest_id)
                    );
                    assert_eq!(
                        headers
                            .get("x-nako-addon-protocol-version")
                            .and_then(|value| value.to_str().ok()),
                        Some(ADDON_PROTOCOL_VERSION)
                    );
                    assert!(headers.get(header::AUTHORIZATION).is_none());
                    assert!(headers.get("x-nako-addon-secret").is_none());
                    assert_eq!(request.manifest_id, manifest_id);
                    assert_eq!(request.protocol_version, ADDON_PROTOCOL_VERSION);

                    let diagnostics = if include_secret_echo {
                        serde_json::json!({
                            "safe_note": "ok",
                            "raw_network_error": "Bearer nako_at_should_not_echo"
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

async fn resource_search_addon_server() -> (String, StdArc<TokioMutex<Vec<AddonResourceRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let router = Router::new()
        .route(
            "/resource-search",
            axum::routing::post({
                let requests = StdArc::clone(&requests);
                move |Json(request): Json<AddonResourceRequest>| {
                    let requests = StdArc::clone(&requests);
                    async move {
                        requests.lock().await.push(request.clone());
                        let link = AddonResourceLink {
                            url: "https://pan.quark.cn/s/raw-secret-link".to_owned(),
                            normalized_url: "https://pan.quark.cn/s/raw-secret-link".to_owned(),
                            link_type: AddonResourceLinkType::Quark,
                            source: "pansou:movies".to_owned(),
                            password: Some("secret-code".to_owned()),
                            note: Some("private-note".to_owned()),
                        };
                        let mut merged_by_type = std::collections::BTreeMap::new();
                        merged_by_type.insert(
                            AddonResourceLinkType::Quark,
                            vec![nako_addon_protocol::AddonMergedResourceLink {
                                url: link.url.clone(),
                                normalized_url: link.normalized_url.clone(),
                                link_type: AddonResourceLinkType::Quark,
                                password: link.password.clone(),
                                note: link.note.clone(),
                                sources: vec!["pansou:movies".to_owned()],
                            }],
                        );

                        Json(AddonResourceResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            resource: request.resource,
                            request_id: request.request_id,
                            payload: serde_json::to_value(AddonResourceSearchResponse {
                                schema: ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned(),
                                query: "Demo Movie".to_owned(),
                                intent: AddonResourceSearchIntent::FreeText {
                                    text: "Demo Movie".to_owned(),
                                },
                                total: 1,
                                results: vec![nako_addon_protocol::AddonResourceSearchResult {
                                    id: "result-1".to_owned(),
                                    title: "Secret Result Title".to_owned(),
                                    source: "pansou:movies".to_owned(),
                                    content: Some("private content".to_owned()),
                                    links: vec![link],
                                    tags: vec!["private-tag".to_owned()],
                                    images: vec!["https://image.example/private.jpg".to_owned()],
                                    score: 900,
                                }],
                                merged_by_type,
                                provider_executions: vec![AddonResourceSearchProviderExecution {
                                    provider_id: "pansou_compatible".to_owned(),
                                    status: AddonResourceSearchProviderStatus::Ok,
                                    result_count: 1,
                                    finality: AddonResourceSearchProviderFinality::Complete,
                                    safe_message: Some("provider secret message".to_owned()),
                                }],
                            })
                            .unwrap(),
                            artifacts: Vec::new(),
                        })
                    }
                }
            }),
        )
        .route(
            "/resource-link-check",
            axum::routing::post({
                let requests = StdArc::clone(&requests);
                move |Json(request): Json<AddonResourceRequest>| {
                    let requests = StdArc::clone(&requests);
                    async move {
                        requests.lock().await.push(request.clone());
                        Json(AddonResourceResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            resource: request.resource,
                            request_id: request.request_id,
                            payload: serde_json::to_value(AddonResourceLinkCheckResponse {
                                schema: ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA.to_owned(),
                                link_type: AddonResourceLinkType::Quark,
                                status: AddonResourceLinkCheckStatus::Reachable,
                                checked_at_ms: 1_779_814_400_000,
                                requires_password: false,
                                retryable: false,
                                retry_after_ms: None,
                                safe_message: Some("safe reachable".to_owned()),
                                safe_facts: std::collections::BTreeMap::from([(
                                    "http_status_class".to_owned(),
                                    "2xx".to_owned(),
                                )]),
                            })
                            .unwrap(),
                            artifacts: Vec::new(),
                        })
                    }
                }
            }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
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
                    "raw_network_error": "Bearer nako_at_should_not_echo"
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

#[derive(Clone, Debug)]
struct CapturedAddonTaskRequest {
    headers: Vec<(String, String)>,
    request: AddonTaskRequest,
}

#[derive(Clone, Debug)]
struct CapturedExternalAcquisitionMaterialization {
    request: AddonExternalAcquisitionMaterializationRequest,
    response: AddonExternalAcquisitionMaterializationResponse,
}

#[derive(Clone, Debug)]
struct CapturedAddonEventRequest {
    headers: Vec<(String, String)>,
    request: AddonEventRequest,
}

type CapturedAddonEventRequests = StdArc<TokioMutex<Vec<CapturedAddonEventRequest>>>;

async fn event_path_addon_server(
    status: StatusCode,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonEventRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let router = Router::new().route(
        "/events/library-scanned",
        axum::routing::post({
            let requests = StdArc::clone(&requests);
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonEventRequest>| {
                let requests = StdArc::clone(&requests);
                async move {
                    requests.lock().await.push(CapturedAddonEventRequest {
                        headers: headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
                            })
                            .collect(),
                        request: request.clone(),
                    });
                    if !status.is_success() {
                        return (status, "sidecar failed").into_response();
                    }

                    (
                        status,
                        Json(AddonEventResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            subscription_id: request.subscription_id,
                            event_id: request.event_id,
                            output: serde_json::json!({
                                "accepted": true,
                                "attempt": request.attempt,
                            }),
                        }),
                    )
                        .into_response()
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn notification_bridge_server() -> (String, CapturedAddonEventRequests) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let router = Router::new()
        .route(
            "/health",
            axum::routing::post(
                move |Json(request): Json<AddonHealthCheckRequest>| async move {
                    assert_eq!(request.manifest_id, notification_bridge::ADDON_ID);
                    assert_eq!(
                        request.expected_addon_version,
                        notification_bridge::ADDON_VERSION
                    );
                    assert_eq!(request.expected_resource_count, 1);

                    Json(ProtocolAddonHealthCheckResponse {
                        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                        manifest_id: request.manifest_id,
                        status: AddonHealthStatus::Ok,
                        checked_at: "2026-05-25T00:00:00.000Z".to_owned(),
                        manifest: AddonHealthManifestFacts {
                            addon_version: notification_bridge::ADDON_VERSION.to_owned(),
                            resource_count: 1,
                        },
                        diagnostics: serde_json::json!({
                            "mode": "ack_only",
                            "provider_fan_out": false
                        }),
                    })
                },
            ),
        )
        .route(
            notification_bridge::LIBRARY_SCANNED_EVENT_PATH,
            axum::routing::post({
                let requests = StdArc::clone(&requests);
                move |headers: axum::http::HeaderMap, Json(request): Json<AddonEventRequest>| {
                    let requests = StdArc::clone(&requests);
                    async move {
                        requests.lock().await.push(CapturedAddonEventRequest {
                            headers: headers
                                .iter()
                                .filter_map(|(name, value)| {
                                    value
                                        .to_str()
                                        .ok()
                                        .map(|value| (name.as_str().to_owned(), value.to_owned()))
                                })
                                .collect(),
                            request: request.clone(),
                        });
                        if request.protocol_version != ADDON_PROTOCOL_VERSION
                            || request.addon_id != notification_bridge::ADDON_ID
                            || request.subscription_id
                                != notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
                            || request.event_kind != notification_bridge::LIBRARY_SCANNED_EVENT_KIND
                        {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(serde_json::json!({
                                    "safe_error_code": "invalid_event_envelope"
                                })),
                            )
                                .into_response();
                        }

                        let payload_keys = request
                            .payload
                            .as_object()
                            .map(|object| {
                                let mut keys = object.keys().cloned().collect::<Vec<_>>();
                                keys.sort();
                                keys
                            })
                            .unwrap_or_default();

                        Json(AddonEventResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            subscription_id: request.subscription_id,
                            event_id: request.event_id,
                            output: serde_json::json!({
                                "schema": notification_bridge::WEBHOOK_RESPONSE_SCHEMA,
                                "accepted": true,
                                "mode": "ack_only",
                                "attempt": request.attempt,
                                "subject_kind": request.subject_kind,
                                "subject_id": request.subject_id,
                                "payload_keys": payload_keys
                            }),
                        })
                        .into_response()
                    }
                }
            }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn blocking_event_path_addon_server(
    status: StatusCode,
    gate: StdArc<Notify>,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonEventRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let router = Router::new().route(
        "/events/library-scanned",
        axum::routing::post({
            let requests = StdArc::clone(&requests);
            let gate = StdArc::clone(&gate);
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonEventRequest>| {
                let requests = StdArc::clone(&requests);
                let gate = StdArc::clone(&gate);
                async move {
                    requests.lock().await.push(CapturedAddonEventRequest {
                        headers: headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
                            })
                            .collect(),
                        request: request.clone(),
                    });
                    gate.notified().await;
                    if !status.is_success() {
                        return (status, "sidecar failed").into_response();
                    }

                    (
                        status,
                        Json(AddonEventResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            subscription_id: request.subscription_id,
                            event_id: request.event_id,
                            output: serde_json::json!({
                                "accepted": true,
                                "attempt": request.attempt,
                            }),
                        }),
                    )
                        .into_response()
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn wait_for_event_requests(
    requests: &StdArc<TokioMutex<Vec<CapturedAddonEventRequest>>>,
    expected: usize,
) {
    for _ in 0..100 {
        if requests.lock().await.len() >= expected {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }

    panic!("timed out waiting for {expected} addon event request(s)");
}

async fn task_path_addon_server(
    statuses: Vec<StatusCode>,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonTaskRequest>>>) {
    task_path_addon_server_with_gate(statuses, None).await
}

async fn task_path_addon_server_with_gate(
    statuses: Vec<StatusCode>,
    gate: Option<StdArc<Notify>>,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonTaskRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let statuses = StdArc::new(TokioMutex::new(VecDeque::from(statuses)));
    let router = Router::new().route(
        "/tasks/bulk",
        axum::routing::post({
            let requests = StdArc::clone(&requests);
            let statuses = StdArc::clone(&statuses);
            let gate = gate.clone();
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonTaskRequest>| {
                let requests = StdArc::clone(&requests);
                let statuses = StdArc::clone(&statuses);
                let gate = gate.clone();
                async move {
                    requests.lock().await.push(CapturedAddonTaskRequest {
                        headers: headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
                            })
                            .collect(),
                        request: request.clone(),
                    });
                    if let Some(gate) = gate {
                        gate.notified().await;
                    }
                    let status = statuses.lock().await.pop_front().unwrap_or(StatusCode::OK);
                    if !status.is_success() {
                        return (status, "sidecar failed").into_response();
                    }

                    (
                        status,
                        Json(AddonTaskResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            task_id: request.task_id,
                            job_id: request.job_id,
                            request_id: request.request_id,
                            output: serde_json::json!({
                                "accepted": true,
                                "attempt": request.attempt,
                                "mode": request.payload["mode"].clone(),
                            }),
                        }),
                    )
                        .into_response()
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn external_acquisition_action_addon_server(
    action_status: AddonExternalAcquisitionActionStatus,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonTaskRequest>>>) {
    external_acquisition_action_addon_server_with_gate(action_status, None).await
}

async fn external_acquisition_action_addon_server_with_gate(
    action_status: AddonExternalAcquisitionActionStatus,
    gate: Option<StdArc<Notify>>,
) -> (String, StdArc<TokioMutex<Vec<CapturedAddonTaskRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let router = Router::new().route(
        external_acquisition_runner::ACTION_TASK_PATH,
        axum::routing::post({
            let requests = StdArc::clone(&requests);
            let gate = gate.clone();
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonTaskRequest>| {
                let requests = StdArc::clone(&requests);
                let gate = gate.clone();
                async move {
                    requests.lock().await.push(CapturedAddonTaskRequest {
                        headers: headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
                            })
                            .collect(),
                        request: request.clone(),
                    });
                    if let Some(gate) = gate {
                        gate.notified().await;
                    }

                    let state = match action_status {
                        AddonExternalAcquisitionActionStatus::Accepted
                        | AddonExternalAcquisitionActionStatus::AlreadyExists => {
                            AddonExternalAcquisitionRunnerState::Running
                        }
                        AddonExternalAcquisitionActionStatus::Rejected
                        | AddonExternalAcquisitionActionStatus::NotFound
                        | AddonExternalAcquisitionActionStatus::Failed => {
                            AddonExternalAcquisitionRunnerState::Unknown
                        }
                    };
                    let runner_job_ref = matches!(
                        action_status,
                        AddonExternalAcquisitionActionStatus::Accepted
                            | AddonExternalAcquisitionActionStatus::AlreadyExists
                    )
                    .then(|| "fixture-job-1".to_owned());
                    Json(AddonTaskResponse {
                        protocol_version: request.protocol_version,
                        addon_id: request.addon_id,
                        task_id: request.task_id,
                        job_id: request.job_id,
                        request_id: request.request_id,
                        output: serde_json::to_value(AddonExternalAcquisitionActionResponse {
                            schema: ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA.to_owned(),
                            status: action_status,
                            state,
                            runner_job_ref,
                            progress: Some(AddonExternalAcquisitionProgress {
                                percent_milli: Some(0),
                                downloaded_bytes: Some(0),
                                total_bytes: None,
                            }),
                            retryable: false,
                            retry_after_ms: None,
                            safe_message: Some("runner_profile_unavailable".to_owned()),
                            safe_facts: std::collections::BTreeMap::new(),
                        })
                        .unwrap(),
                    })
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn start_nako_test_http_server(router: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    format!("http://{addr}")
}

async fn external_acquisition_materializing_addon_server(
    nako_base_url: String,
) -> (
    String,
    StdArc<TokioMutex<Vec<CapturedAddonTaskRequest>>>,
    StdArc<TokioMutex<Vec<CapturedExternalAcquisitionMaterialization>>>,
    StdArc<TokioMutex<Option<String>>>,
) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let materializations = StdArc::new(TokioMutex::new(Vec::new()));
    let raw_token = StdArc::new(TokioMutex::new(None::<String>));
    let router = Router::new().route(
        external_acquisition_runner::ACTION_TASK_PATH,
        axum::routing::post({
            let requests = StdArc::clone(&requests);
            let materializations = StdArc::clone(&materializations);
            let raw_token = StdArc::clone(&raw_token);
            move |headers: axum::http::HeaderMap, Json(request): Json<AddonTaskRequest>| {
                let requests = StdArc::clone(&requests);
                let materializations = StdArc::clone(&materializations);
                let raw_token = StdArc::clone(&raw_token);
                let nako_base_url = nako_base_url.clone();
                async move {
                    requests.lock().await.push(CapturedAddonTaskRequest {
                        headers: headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
                            })
                            .collect(),
                        request: request.clone(),
                    });
                    let action =
                        serde_json::from_value::<AddonExternalAcquisitionActionRequest>(
                            request.payload.clone(),
                        )
                        .expect("external acquisition task payload should decode");
                    let materialization_request = AddonExternalAcquisitionMaterializationRequest {
                        schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA
                            .to_owned(),
                        job_id: request.job_id.clone(),
                        declaration_id: request.task_id.clone(),
                        target_ref: action.target_ref.clone(),
                        runner_profile_id: action.runner_profile_id.clone(),
                        idempotency_key: action.idempotency_key.clone(),
                        operation: action.operation,
                        audit_ref: action
                            .audit_ref
                            .clone()
                            .expect("external acquisition action should carry audit ref"),
                        purpose: AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue,
                    };
                    let raw_token = raw_token
                        .lock()
                        .await
                        .clone()
                        .expect("addon raw token must be configured before action dispatch");
                    let materialized = NakoRuntimeClient::new(NakoRuntimeClientConfig {
                        base_url: nako_base_url,
                        addon_token: raw_token,
                        timeout_ms: 5_000,
                    })
                    .materialize_external_acquisition(materialization_request.clone())
                    .await
                    .expect("fake sidecar should materialize approved target");
                    materializations
                        .lock()
                        .await
                        .push(CapturedExternalAcquisitionMaterialization {
                            request: materialization_request,
                            response: materialized.clone(),
                        });

                    let output = AddonExternalAcquisitionActionResponse {
                        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA.to_owned(),
                        status: AddonExternalAcquisitionActionStatus::Accepted,
                        state: AddonExternalAcquisitionRunnerState::Running,
                        runner_job_ref: Some("fixture-job-materialized-1".to_owned()),
                        progress: Some(AddonExternalAcquisitionProgress {
                            percent_milli: Some(0),
                            downloaded_bytes: Some(0),
                            total_bytes: None,
                        }),
                        retryable: false,
                        retry_after_ms: None,
                        safe_message: Some("fake_sidecar_materialized".to_owned()),
                        safe_facts: std::collections::BTreeMap::from([
                            (
                                "materialization_client".to_owned(),
                                "nako_runtime".to_owned(),
                            ),
                            (
                                "materialized_link_type".to_owned(),
                                materialized.material.link_type.as_str().to_owned(),
                            ),
                            (
                                "materialized_password_present".to_owned(),
                                materialized.material.password.is_some().to_string(),
                            ),
                            (
                                "materialization_ref_present".to_owned(),
                                (!materialized.materialization_ref.trim().is_empty()).to_string(),
                            ),
                        ]),
                    };

                    Json(AddonTaskResponse {
                        protocol_version: request.protocol_version,
                        addon_id: request.addon_id,
                        task_id: request.task_id,
                        job_id: request.job_id,
                        request_id: request.request_id,
                        output: serde_json::to_value(output).unwrap(),
                    })
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (
        format!("http://{addr}"),
        requests,
        materializations,
        raw_token,
    )
}

async fn subtitle_search_addon_server() -> (String, StdArc<TokioMutex<Vec<AddonResourceRequest>>>) {
    let requests = StdArc::new(TokioMutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{addr}");
    let download_url = format!("{base_url}/download/subtitle.srt?token=secret-token");
    let router = Router::new()
        .route(
            "/subtitle",
            axum::routing::post({
                let requests = StdArc::clone(&requests);
                let download_url = download_url.clone();
                move |Json(request): Json<AddonResourceRequest>| {
                    let requests = StdArc::clone(&requests);
                    let download_url = download_url.clone();
                    async move {
                        requests.lock().await.push(request.clone());

                        Json(AddonResourceResponse {
                            protocol_version: request.protocol_version,
                            addon_id: request.addon_id,
                            resource: request.resource,
                            request_id: request.request_id,
                            payload: serde_json::to_value(AddonSubtitleSearchResponse {
                                schema: ADDON_SUBTITLE_RESPONSE_SCHEMA.to_owned(),
                                query: "Demo Movie".to_owned(),
                                total: 3,
                                subtitles: vec![
                                    AddonSubtitleCandidate {
                                        id: "subtitle-inline-1".to_owned(),
                                        title: "Secret English Subtitle".to_owned(),
                                        language: "en".to_owned(),
                                        format: AddonSubtitleFormat::Vtt,
                                        source: "fixture".to_owned(),
                                        release: Some("WEB-DL".to_owned()),
                                        score: 940,
                                        delivery: AddonSubtitleDelivery::Inline {
                                            text: "WEBVTT\n\nsecret subtitle text".to_owned(),
                                        },
                                    },
                                    AddonSubtitleCandidate {
                                        id: "subtitle-download-1".to_owned(),
                                        title: "Remote Chinese Subtitle".to_owned(),
                                        language: "zh-hant".to_owned(),
                                        format: AddonSubtitleFormat::Srt,
                                        source: "opensubtitles".to_owned(),
                                        release: None,
                                        score: 900,
                                        delivery: AddonSubtitleDelivery::DownloadUrl {
                                            url: download_url,
                                            content_type: Some("application/x-subrip".to_owned()),
                                        },
                                    },
                                    AddonSubtitleCandidate {
                                        id: "subtitle-artifact-1".to_owned(),
                                        title: "Artifact English Subtitle".to_owned(),
                                        language: "en".to_owned(),
                                        format: AddonSubtitleFormat::Vtt,
                                        source: "artifact-provider".to_owned(),
                                        release: Some("BluRay".to_owned()),
                                        score: 870,
                                        delivery: AddonSubtitleDelivery::ArtifactRef {
                                            artifact_id: "artifact-secret-id".to_owned(),
                                        },
                                    },
                                ],
                                provider_executions: vec![AddonSubtitleProviderExecution {
                                    provider_id: "fixture".to_owned(),
                                    status: AddonSubtitleProviderStatus::Ok,
                                    result_count: 3,
                                    safe_message: Some(
                                        "provider safe message with private detail".to_owned(),
                                    ),
                                }],
                            })
                            .unwrap(),
                            artifacts: Vec::new(),
                        })
                    }
                }
            }),
        )
        .route(
            "/download/subtitle.srt",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "application/x-subrip")],
                    "1\n00:00:01,000 --> 00:00:02,000\nremote secret subtitle text\n",
                )
            }),
        );
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    yield_now().await;

    (base_url, requests)
}

async fn register_subtitle_provider_and_search(
    router: &Router,
    addon_base_url: String,
    languages: Vec<String>,
) -> (AddonId, AdminAddonSubtitleSearchResponse) {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonSubtitleSearchResponse, _>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/subtitle-search"),
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages,
            limit: Some(10),
        },
    )
    .await;

    (addon_id, search)
}

fn subtitle_import_plan_request(
    source: &MediaSource,
    language: &str,
    format: AddonSubtitleFormat,
    role: AdminSubtitleSidecarRole,
    conflict_policy: AdminSubtitleImportConflictPolicy,
    backup_policy: AdminSubtitleImportBackupPolicy,
) -> AdminAddonSubtitleImportPlanRequest {
    AdminAddonSubtitleImportPlanRequest {
        media_item_id: source.item_id,
        media_source_id: source.id,
        language: language.to_owned(),
        format,
        sidecar_role: role,
        conflict_policy,
        backup_policy,
    }
}

fn subtitle_import_apply_request(
    plan_idempotency_key: String,
    source: &MediaSource,
    language: &str,
    format: AddonSubtitleFormat,
    role: AdminSubtitleSidecarRole,
    conflict_policy: AdminSubtitleImportConflictPolicy,
    backup_policy: AdminSubtitleImportBackupPolicy,
) -> AdminAddonSubtitleImportApplyRequest {
    AdminAddonSubtitleImportApplyRequest {
        plan_idempotency_key,
        media_item_id: source.item_id,
        media_source_id: source.id,
        language: language.to_owned(),
        format,
        sidecar_role: role,
        conflict_policy,
        backup_policy,
    }
}

fn task_path_manifest_with_auth(base_url: String, auth: AddonAuth) -> AddonManifest {
    let mut manifest = addon_manifest();
    manifest.id = "example.task-dispatch".to_owned();
    manifest.base_url = base_url;
    manifest.auth = auth;
    manifest.tasks = vec![
        AddonTaskDeclaration::new(
            "bulk-task",
            "Bulk Task",
            "/tasks/bulk",
            vec![AddonScope::AutomationRun],
        )
        .with_execution_bounds(Some(30_000), Some(2)),
    ];
    manifest.scopes.push(AddonScope::AutomationRun);
    manifest
}

fn event_path_manifest(base_url: String) -> AddonManifest {
    let mut manifest = addon_manifest();
    manifest.id = "example.event-delivery".to_owned();
    manifest.base_url = base_url;
    manifest.auth = AddonAuth::None;
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scanned",
        DomainEventKind::LibraryScanned.as_str(),
        "/events/library-scanned",
        vec![AddonScope::WebhookEventRead],
        serde_json::Value::Null,
    )];
    manifest.scopes.push(AddonScope::WebhookEventRead);
    manifest
}

fn resource_search_manifest(base_url: String) -> AddonManifest {
    let mut manifest = addon_manifest();
    manifest.id = "example.resource-search".to_owned();
    manifest.name = "Example Resource Search".to_owned();
    manifest.base_url = base_url;
    manifest.auth = AddonAuth::None;
    manifest.resources = vec![AddonResourceDeclaration {
        kind: AddonResource::ResourceSearch,
        path: "/resource-search".to_owned(),
        input_schema: Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned()),
        output_schema: Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned()),
        required_scopes: vec![AddonScope::AcquisitionSearchRead],
        timeout_ms: Some(5_000),
        max_attempts: Some(1),
    }];
    manifest.scopes = vec![AddonScope::AcquisitionSearchRead];
    manifest
}

async fn register_event_path_addon(router: &Router, base_url: String) -> AddonId {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: event_path_manifest(base_url),
            outbound_task_dispatch_secret_env: None,
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
    request_json::<AdminAddonRoutingPlansResponse>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;

    addon_id
}

async fn register_task_path_addon(router: &Router, base_url: String) -> AddonId {
    register_task_path_addon_with_auth(router, base_url, AddonAuth::None, None).await
}

async fn register_external_acquisition_runner_addon(router: &Router, base_url: String) -> AddonId {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: external_acquisition_runner::manifest(base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::AcquisitionActionRun],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    request_json::<AdminAddonRoutingPlansResponse>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;

    addon_id
}

async fn register_task_path_addon_with_auth(
    router: &Router,
    base_url: String,
    auth: AddonAuth,
    outbound_task_dispatch_secret_env: Option<String>,
) -> AddonId {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: task_path_manifest_with_auth(base_url, auth),
            outbound_task_dispatch_secret_env,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    request_json::<AdminAddonRoutingPlansResponse>(
        router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;

    addon_id
}

async fn wait_for_addon_task_status(
    router: &Router,
    addon_id: AddonId,
    job_id: JobId,
    expected: JobStatus,
) -> AddonTaskRunResponse {
    for _ in 0..100 {
        let response = request_json::<AddonTaskRunResponse>(
            router,
            Method::GET,
            &format!("/admin/v1/addons/{addon_id}/task-runs/{job_id}"),
        )
        .await;
        if response.run.status == expected {
            return response;
        }
        sleep(Duration::from_millis(20)).await;
    }

    panic!("addon task run {job_id} did not reach {expected:?}");
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
    nako_core::ArtworkCandidateId,
    AcceptManagedArtworkCandidateResponse,
) {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
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
    let candidate_id: nako_core::ArtworkCandidateId =
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
            outbound_task_dispatch_secret_env: None,
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
    nako_core::ArtworkCandidateId,
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
    let candidate_id: nako_core::ArtworkCandidateId =
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: Some("NAKO_ADDON_DISPATCH_SECRET".to_owned()),
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
        response
            .addon
            .summary
            .outbound_task_dispatch_secret_env
            .as_deref(),
        Some("NAKO_ADDON_DISPATCH_SECRET")
    );
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
            outbound_task_dispatch_secret_env: None,
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

    let mut unsupported_protocol = addon_manifest();
    unsupported_protocol.id = "example.metadata.unsupported-protocol".to_owned();
    unsupported_protocol.protocol_version = "0.1.0-alpha.0".to_owned();
    let unsupported = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: unsupported_protocol,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(unsupported.status(), StatusCode::BAD_REQUEST);
    let unsupported_error = body_json::<ErrorResponse>(unsupported).await;
    assert_eq!(unsupported_error.code, "invalid_input");
    assert!(
        unsupported_error
            .message
            .contains("unsupported addon protocol version")
    );

    let disabled_without_grants = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: {
                let mut manifest = addon_manifest();
                manifest.id = "example.metadata.disabled-without-grants".to_owned();
                manifest
            },
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![],
            status: Some(AddonStatus::Disabled),
        },
    )
    .await;
    assert_eq!(
        disabled_without_grants.addon.summary.status,
        AddonStatus::Disabled
    );
    assert!(
        disabled_without_grants
            .addon
            .summary
            .granted_scopes
            .is_empty()
    );

    let missing_scope = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
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
        Some("Resolved by Nako at runtime".to_owned()),
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
            image: Some("ghcr.io/nako/example-metadata-addon:0.1.0".to_owned()),
            binary: None,
            command: None,
        },
        secret_reference_bindings: vec![AddonSecretReferenceBinding {
            field_id: "metadata_api_key".to_owned(),
            secret_ref: "env:NAKO_METADATA_ADDON_TOKEN".to_owned(),
        }],
        install_notes: vec![
            "Do not include NAKO_METADATA_ADDON_TOKEN=secret-value in the guide".to_owned(),
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
        "ghcr.io/nako/example-metadata-addon:0.1.0"
    );
    assert_eq!(response.guide.required_secret_fields.len(), 1);
    assert!(response.guide.required_secret_fields[0].provided);
    assert!(response.guide.missing_required_secret_fields.is_empty());
    assert_eq!(response.guide.task_count, 1);
    assert_eq!(response.guide.event_subscription_count, 1);
    assert!(!text.contains("secret-value"));
    assert!(!text.contains("NAKO_METADATA_ADDON_TOKEN="));
    assert!(!text.contains("Bearer "));
    assert!(!text.contains("nako_at_"));
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
        Some("Resolved by Nako at runtime".to_owned()),
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

    descriptor.runtime.binary = Some("nako-metadata-addon".to_owned());
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
            image: Some("ghcr.io/nako/example-metadata-addon:0.1.0".to_owned()),
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
async fn admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work() {
    let (_temp, router, _source, store) =
        router_with_media_source_config("addon-catalog.mkv", b"media", |_| {}).await;

    let sources = request_json::<AdminAddonSourceCatalogSourcesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/sources",
    )
    .await;
    assert_eq!(sources.sources.len(), 1);
    let source = &sources.sources[0];
    assert_eq!(source.id, "nako-official");
    assert_eq!(
        source.kind,
        AdminAddonSourceCatalogSourceKind::BuiltinOfficial
    );
    assert_eq!(source.entry_count, 7);
    assert!(!source.provides_package_signing);
    assert!(!source.provides_process_supervision);
    assert!(!source.provides_provider_breadth);

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    assert_eq!(entries.source_id, "nako-official");
    assert_eq!(entries.entries.len(), 7);
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == metadata_scraper::ADDON_ID)
        .unwrap();
    assert_eq!(entry.entry_id, metadata_scraper::ADDON_ID);
    assert_eq!(entry.manifest_id, metadata_scraper::ADDON_ID);
    assert_eq!(entry.addon_name, metadata_scraper::ADDON_NAME);
    assert_eq!(entry.addon_version, metadata_scraper::ADDON_VERSION);
    assert_eq!(entry.protocol_version, ADDON_PROTOCOL_VERSION);
    assert_eq!(entry.runtime_kind, AddonRuntimeKind::HttpSidecar);
    assert_eq!(entry.resources, vec![AddonResource::Metadata]);
    assert_eq!(
        entry.scopes,
        vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
            AddonScope::AutomationRun,
            AddonScope::WebhookEventRead,
        ]
    );
    assert_eq!(
        entry.tasks,
        vec![metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID.to_owned()]
    );
    assert!(!entry.package_signing_verified);
    assert_eq!(entry.lifecycle_boundary.nako_manages_packages, false);
    assert_eq!(entry.lifecycle_boundary.nako_manages_processes, false);
    assert_eq!(entry.lifecycle_boundary.nako_manages_containers, false);

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.metadata-scraper/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, metadata_scraper::ADDON_ID);
    assert_eq!(resolved.descriptor.manifest.id, metadata_scraper::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.version,
        metadata_scraper::ADDON_VERSION
    );
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        metadata_scraper::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(
        resolved.descriptor.manifest.protocol_version,
        ADDON_PROTOCOL_VERSION
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 1);
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        metadata_scraper::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        metadata_scraper::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert_eq!(
        resolved
            .descriptor
            .manifest
            .configuration_schema
            .as_ref()
            .unwrap()
            .schema_id,
        metadata_scraper::CONFIG_SCHEMA_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].id,
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].path,
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].required_scopes,
        vec![AddonScope::AutomationRun]
    );
    assert_eq!(resolved.descriptor.manifest.event_subscriptions.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].id,
        metadata_scraper::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].event_kind,
        metadata_scraper::LIBRARY_SCANNED_EVENT_KIND
    );
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].path,
        metadata_scraper::LIBRARY_SCANNED_EVENT_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].required_scopes,
        vec![AddonScope::WebhookEventRead]
    );
    assert!(
        resolved
            .descriptor
            .manifest
            .secret_reference_fields
            .is_empty()
    );
    assert!(resolved.descriptor.secret_reference_bindings.is_empty());
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        metadata_scraper::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 1);
    assert_eq!(resolved.install_guide.event_subscription_count, 1);

    for forbidden in [
        "secret-value",
        "Bearer ",
        "nako_at_",
        "docker.sock",
        "docker start",
        "docker stop",
        "systemctl start",
        "systemctl stop",
        "operator_confirmed\":true",
    ] {
        assert!(
            !text.contains(forbidden),
            "catalog resolve leaked forbidden term: {forbidden}"
        );
    }

    assert!(
        store
            .list_addon_registrations(None)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_jobs(
                nako_core::JobListFilter::default(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_notification_bridge_event_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == notification_bridge::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, notification_bridge::ADDON_ID);
    assert_eq!(entry.addon_name, notification_bridge::ADDON_NAME);
    assert_eq!(entry.addon_version, notification_bridge::ADDON_VERSION);
    assert_eq!(entry.resources, vec![AddonResource::Webhook]);
    assert_eq!(entry.scopes, vec![AddonScope::WebhookEventRead]);
    assert!(entry.tasks.is_empty());

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.notification-bridge/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, notification_bridge::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.id,
        notification_bridge::ADDON_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        notification_bridge::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.resources[0].kind,
        AddonResource::Webhook
    );
    assert!(resolved.descriptor.manifest.tasks.is_empty());
    assert_eq!(resolved.descriptor.manifest.event_subscriptions.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].id,
        notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].event_kind,
        notification_bridge::LIBRARY_SCANNED_EVENT_KIND
    );
    assert_eq!(
        resolved.descriptor.manifest.event_subscriptions[0].path,
        notification_bridge::LIBRARY_SCANNED_EVENT_PATH
    );
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        notification_bridge::RUNTIME_IMAGE
    );
    assert!(!resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 0);
    assert_eq!(resolved.install_guide.event_subscription_count, 1);
    assert!(!text.contains("Telegram"));
    assert!(!text.contains("Discord"));
    assert!(!text.contains("nako_at_"));
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_chromecast_renderer_adapter_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == chromecast_renderer::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, chromecast_renderer::ADDON_ID);
    assert_eq!(entry.addon_name, chromecast_renderer::ADDON_NAME);
    assert_eq!(entry.addon_version, chromecast_renderer::ADDON_VERSION);
    assert_eq!(entry.resources, vec![AddonResource::RendererAdapter]);
    assert_eq!(
        entry.scopes,
        vec![
            AddonScope::RendererAdapterRead,
            AddonScope::RendererAdapterControl,
        ]
    );
    assert!(entry.tasks.is_empty());

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.chromecast-renderer/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, chromecast_renderer::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.id,
        chromecast_renderer::ADDON_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        chromecast_renderer::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.resources[0].kind,
        AddonResource::RendererAdapter
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].path,
        chromecast_renderer::RENDERER_ADAPTER_RESOURCE_PATH
    );
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        chromecast_renderer::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        chromecast_renderer::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert!(resolved.descriptor.manifest.tasks.is_empty());
    assert!(resolved.descriptor.manifest.event_subscriptions.is_empty());
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        chromecast_renderer::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 0);
    assert_eq!(resolved.install_guide.event_subscription_count, 0);
    assert!(!text.contains("renderer_ticket"));
    assert!(!text.contains("nako_rtt_"));
    assert!(!text.contains("Bearer "));
    assert!(!text.contains("docker.sock"));
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_resource_search_link_check_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == resource_search::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, resource_search::ADDON_ID);
    assert_eq!(entry.addon_name, resource_search::ADDON_NAME);
    assert_eq!(entry.addon_version, resource_search::ADDON_VERSION);
    assert_eq!(
        entry.resources,
        vec![
            AddonResource::ResourceSearch,
            AddonResource::ResourceLinkCheck,
        ]
    );
    assert_eq!(
        entry.scopes,
        vec![
            AddonScope::AcquisitionSearchRead,
            AddonScope::AcquisitionLinkCheckRead,
        ]
    );
    assert!(entry.tasks.is_empty());

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.resource-search/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, resource_search::ADDON_ID);
    assert_eq!(resolved.descriptor.manifest.id, resource_search::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        resource_search::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 2);
    assert_eq!(
        resolved.descriptor.manifest.resources[0].kind,
        AddonResource::ResourceSearch
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].path,
        resource_search::RESOURCE_SEARCH_RESOURCE_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .input_schema
            .as_deref(),
        Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .output_schema
            .as_deref(),
        Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].required_scopes,
        vec![AddonScope::AcquisitionSearchRead]
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[1].kind,
        AddonResource::ResourceLinkCheck
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[1].path,
        resource_search::RESOURCE_LINK_CHECK_RESOURCE_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[1]
            .input_schema
            .as_deref(),
        Some(ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[1]
            .output_schema
            .as_deref(),
        Some(ADDON_RESOURCE_LINK_CHECK_RESPONSE_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[1].required_scopes,
        vec![AddonScope::AcquisitionLinkCheckRead]
    );
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        resource_search::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        resource_search::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert!(resolved.descriptor.manifest.tasks.is_empty());
    assert!(resolved.descriptor.manifest.event_subscriptions.is_empty());
    assert!(
        resolved
            .descriptor
            .manifest
            .secret_reference_fields
            .is_empty()
    );
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        resource_search::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 0);
    assert_eq!(resolved.install_guide.event_subscription_count, 0);

    for forbidden in [
        "https://pan.quark.cn",
        "pan.baidu.com",
        "Bearer ",
        "nako_at_",
        "docker.sock",
        "docker start",
        "systemctl start",
    ] {
        assert!(
            !text.contains(forbidden),
            "resource-search catalog resolve leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_external_acquisition_runner_action_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == external_acquisition_runner::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, external_acquisition_runner::ADDON_ID);
    assert_eq!(entry.addon_name, external_acquisition_runner::ADDON_NAME);
    assert_eq!(
        entry.addon_version,
        external_acquisition_runner::ADDON_VERSION
    );
    assert!(entry.resources.is_empty());
    assert_eq!(entry.scopes, vec![AddonScope::AcquisitionActionRun]);
    assert_eq!(
        entry.tasks,
        vec![external_acquisition_runner::ACTION_TASK_ID.to_owned()]
    );

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.external-acquisition-runner/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(
        resolved.entry.entry_id,
        external_acquisition_runner::ADDON_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.id,
        external_acquisition_runner::ADDON_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        external_acquisition_runner::DEFAULT_CONTAINER_BASE_URL
    );
    assert!(resolved.descriptor.manifest.resources.is_empty());
    assert_eq!(resolved.descriptor.manifest.tasks.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].id,
        external_acquisition_runner::ACTION_TASK_ID
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].path,
        external_acquisition_runner::ACTION_TASK_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0]
            .input_schema
            .as_deref(),
        Some(external_acquisition_runner::ACTION_REQUEST_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0]
            .output_schema
            .as_deref(),
        Some(external_acquisition_runner::ACTION_RESPONSE_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.tasks[0].required_scopes,
        vec![AddonScope::AcquisitionActionRun]
    );
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        external_acquisition_runner::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        external_acquisition_runner::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert!(resolved.descriptor.manifest.event_subscriptions.is_empty());
    assert_eq!(
        resolved.descriptor.manifest.secret_reference_fields.len(),
        1
    );
    assert_eq!(
        resolved.descriptor.manifest.secret_reference_fields[0].id,
        external_acquisition_runner::TRANSMISSION_PASSWORD_SECRET_FIELD_ID
    );
    assert!(!resolved.descriptor.manifest.secret_reference_fields[0].required);
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        external_acquisition_runner::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.task_count, 1);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.event_subscription_count, 0);
    assert_eq!(resolved.install_guide.required_secret_fields.len(), 1);
    assert_eq!(
        resolved.install_guide.required_secret_fields[0].id,
        external_acquisition_runner::TRANSMISSION_PASSWORD_SECRET_FIELD_ID
    );
    assert!(!resolved.install_guide.required_secret_fields[0].required);
    assert!(!resolved.install_guide.required_secret_fields[0].provided);
    assert!(resolved.install_guide.provided_secret_refs.is_empty());
    assert!(
        resolved
            .install_guide
            .missing_required_secret_fields
            .is_empty()
    );

    for forbidden in ["raw_url", "magnet:", "Bearer ", "nako_at_", "docker.sock"] {
        assert!(
            !text.contains(forbidden),
            "external acquisition catalog resolve leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_subtitle_provider_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == subtitle_provider::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, subtitle_provider::ADDON_ID);
    assert_eq!(entry.addon_name, subtitle_provider::ADDON_NAME);
    assert_eq!(entry.addon_version, subtitle_provider::ADDON_VERSION);
    assert_eq!(entry.resources, vec![AddonResource::Subtitle]);
    assert_eq!(entry.scopes, vec![AddonScope::SubtitleRead]);
    assert!(entry.tasks.is_empty());

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.subtitle-provider/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, subtitle_provider::ADDON_ID);
    assert_eq!(resolved.descriptor.manifest.id, subtitle_provider::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        subtitle_provider::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.resources[0].kind,
        AddonResource::Subtitle
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].path,
        subtitle_provider::SUBTITLE_RESOURCE_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .input_schema
            .as_deref(),
        Some(subtitle_provider::SUBTITLE_REQUEST_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .output_schema
            .as_deref(),
        Some(subtitle_provider::SUBTITLE_RESPONSE_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].required_scopes,
        vec![AddonScope::SubtitleRead]
    );
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        subtitle_provider::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        subtitle_provider::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert!(resolved.descriptor.manifest.tasks.is_empty());
    assert!(resolved.descriptor.manifest.event_subscriptions.is_empty());
    assert!(
        resolved
            .descriptor
            .manifest
            .secret_reference_fields
            .is_empty()
    );
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        subtitle_provider::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 0);
    assert_eq!(resolved.install_guide.event_subscription_count, 0);

    for forbidden in [
        "subtitle_write",
        "password",
        "nako_at_",
        "Bearer ",
        "docker.sock",
    ] {
        assert!(
            !text.contains(forbidden),
            "subtitle-provider catalog resolve leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_source_catalog_resolves_dlna_renderer_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let entries = request_json::<AdminAddonSourceCatalogEntriesResponse>(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries",
    )
    .await;
    let entry = entries
        .entries
        .iter()
        .find(|entry| entry.entry_id == dlna_renderer::ADDON_ID)
        .unwrap();
    assert_eq!(entry.manifest_id, dlna_renderer::ADDON_ID);
    assert_eq!(entry.addon_name, dlna_renderer::ADDON_NAME);
    assert_eq!(entry.addon_version, dlna_renderer::ADDON_VERSION);
    assert_eq!(entry.resources, vec![AddonResource::RendererAdapter]);
    assert_eq!(
        entry.scopes,
        vec![
            AddonScope::RendererAdapterRead,
            AddonScope::RendererAdapterControl,
        ]
    );
    assert!(entry.tasks.is_empty());

    let raw = response_for(
        &router,
        Method::GET,
        "/admin/v1/addons/catalog/entries/nako.official.dlna-renderer/resolve",
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let resolved = serde_json::from_str::<AdminAddonSourceCatalogResolveResponse>(&text).unwrap();

    assert_eq!(resolved.source_id, "nako-official");
    assert_eq!(resolved.entry.entry_id, dlna_renderer::ADDON_ID);
    assert_eq!(resolved.descriptor.manifest.id, dlna_renderer::ADDON_ID);
    assert_eq!(
        resolved.descriptor.manifest.base_url,
        dlna_renderer::DEFAULT_CONTAINER_BASE_URL
    );
    assert_eq!(resolved.descriptor.manifest.resources.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.resources[0].kind,
        AddonResource::RendererAdapter
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].path,
        dlna_renderer::RENDERER_ADAPTER_RESOURCE_PATH
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .input_schema
            .as_deref(),
        Some(dlna_renderer::RENDERER_ADAPTER_REQUEST_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0]
            .output_schema
            .as_deref(),
        Some(dlna_renderer::RENDERER_ADAPTER_RESPONSE_SCHEMA)
    );
    assert_eq!(
        resolved.descriptor.manifest.resources[0].required_scopes,
        vec![
            AddonScope::RendererAdapterRead,
            AddonScope::RendererAdapterControl,
        ]
    );
    assert_eq!(resolved.descriptor.manifest.entry_points.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.entry_points[0].id,
        dlna_renderer::DIAGNOSTICS_ENTRY_POINT_ID
    );
    assert_eq!(resolved.descriptor.manifest.hosted_pages.len(), 1);
    assert_eq!(
        resolved.descriptor.manifest.hosted_pages[0].id,
        dlna_renderer::DIAGNOSTICS_HOSTED_PAGE_ID
    );
    assert!(resolved.descriptor.manifest.tasks.is_empty());
    assert!(resolved.descriptor.manifest.event_subscriptions.is_empty());
    assert!(
        resolved
            .descriptor
            .manifest
            .secret_reference_fields
            .is_empty()
    );
    assert_eq!(
        resolved.install_guide.runtime_reference.value,
        dlna_renderer::RUNTIME_IMAGE
    );
    assert!(resolved.install_guide.has_configuration_schema);
    assert_eq!(resolved.install_guide.entry_point_count, 1);
    assert_eq!(resolved.install_guide.hosted_page_count, 1);
    assert_eq!(resolved.install_guide.task_count, 0);
    assert_eq!(resolved.install_guide.event_subscription_count, 0);

    for forbidden in [
        "upnp_control_url",
        "renderer_ticket",
        "nako_rtt_",
        "Bearer ",
        "docker.sock",
    ] {
        assert!(
            !text.contains(forbidden),
            "dlna-renderer catalog resolve leaked forbidden term: {forbidden}"
        );
    }
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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
            id: Some(nako_core::AddonId::new()),
            manifest: registered.addon.manifest.clone(),
            outbound_task_dispatch_secret_env: None,
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
        schema_id: "nako.example.metadata.config.v1".to_owned(),
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
        input_schema: None,
        output_schema: None,
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
            outbound_task_dispatch_secret_env: None,
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
        "nako.example.metadata.config.v1"
    );

    let mut invalid_manifest = manifest.clone();
    invalid_manifest.tasks[0].required_scopes = vec![AddonScope::RecommendationWrite];
    let invalid = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: invalid_manifest,
            outbound_task_dispatch_secret_env: None,
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
        axum::serve(listener, nako_reference_addon::build_router())
            .await
            .unwrap();
    });
    yield_now().await;

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = nako_reference_addon::reference_manifest(addon_base_url);

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
        nako_reference_addon::REFERENCE_ADDON_ID
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
        nako_reference_addon::REFERENCE_ADDON_ID
    );
    assert_eq!(response.artifacts[0].kind, "metadata_suggestion");

    addon_server.abort();
}

#[tokio::test]
async fn admin_addon_health_check_reports_safe_reachability_without_tokens() {
    let (addon_base_url, requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "nako.health.metadata",
        true,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.health.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
    assert_eq!(response.manifest_id, "nako.health.metadata");
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
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
    assert!(!text.contains("authorization"));
}

#[tokio::test]
async fn admin_addon_health_check_classifies_unreachable_without_raw_error_leak() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let unreachable_base_url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);

    let mut manifest = nako_reference_addon::reference_manifest(unreachable_base_url);
    manifest.id = "nako.unreachable.metadata".to_owned();
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
        "nako.ready.metadata",
        true,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.ready.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
    assert_eq!(response.manifest_id, "nako.ready.metadata");
    assert_eq!(
        response.readiness.status,
        AdminAddonRuntimeReadinessStatus::Ready
    );
    assert_eq!(
        response.readiness.reason,
        AdminAddonRuntimeReadinessReason::Ready
    );
    assert!(response.readiness.checks.iter().any(|check| {
        check.name == nako_api::extension::AdminAddonRuntimeReadinessCheckName::Reachability
            && check.status == AdminAddonRuntimeReadinessStatus::Ready
    }));
    assert!(response.readiness.checks.iter().any(|check| {
        check.name == nako_api::extension::AdminAddonRuntimeReadinessCheckName::Network
            && check.status == AdminAddonRuntimeReadinessStatus::Ready
    }));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
    assert!(!text.contains("authorization"));
    assert!(!text.contains("127.0.0.1"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_preserves_sidecar_degraded_status() {
    let (addon_base_url, _) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Degraded,
        "nako.degraded.metadata",
        true,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.degraded.metadata".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
        "nako.gapped.metadata",
        true,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.gapped.metadata".to_owned();
    manifest.secret_reference_fields = vec![AddonSecretReferenceFieldDeclaration::new(
        "provider_token",
        "Provider token",
        Some("Resolved by Nako".to_owned()),
        true,
    )];

    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    let addon_id = nako_core::AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            protocol_version: manifest.protocol_version.clone(),
            base_url: manifest.base_url.clone(),
            manifest_json,
            outbound_task_dispatch_secret_env: None,
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
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("raw_network_error"));
}

#[tokio::test]
async fn admin_addon_runtime_readiness_classifies_network_policy_blockers_without_echoing_url() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest =
        nako_reference_addon::reference_manifest("http://user:secret@addon.example.test/base");
    manifest.id = "nako.blocked-network.metadata".to_owned();
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
        r#"{"protocol_version":"2020-01-01","manifest_id":"nako.protocol.metadata","status":"ok","checked_at":"2026-05-21T12:00:00.000Z","manifest":{"addon_version":"0.1.0","resource_count":1},"diagnostics":{"secret":"nako_at_should_not_echo"}}"#,
    )
    .await;
    let mut protocol_manifest = nako_reference_addon::reference_manifest(protocol_base_url);
    protocol_manifest.id = "nako.protocol.metadata".to_owned();
    let protocol_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: protocol_manifest,
            outbound_task_dispatch_secret_env: None,
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
    assert!(!protocol_text.contains("nako_at_should_not_echo"));

    let manifest_base_url = mismatched_health_addon_server("wrong-manifest", "0.1.0").await;
    let mut manifest = nako_reference_addon::reference_manifest(manifest_base_url);
    manifest.id = "nako.manifest.metadata".to_owned();
    let manifest_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
    let mut unsafe_manifest = nako_reference_addon::reference_manifest(unsafe_base_url);
    unsafe_manifest.id = "nako.unsafe-health.metadata".to_owned();
    let unsafe_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: unsafe_manifest,
            outbound_task_dispatch_secret_env: None,
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
    let mut manifest = nako_reference_addon::reference_manifest("https://addon.example.test/base");
    manifest.secret_reference_fields = vec![
        nako_addon_protocol::AddonSecretReferenceFieldDeclaration::new(
            "api_key",
            "API Key",
            Some("Resolved by Nako at runtime".to_owned()),
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
            outbound_task_dispatch_secret_env: None,
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
        nako_reference_addon::REFERENCE_ADDON_ID
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
        "nako.reference.metadata.config.v1"
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
    assert!(!text.contains("nako_at_"));
    assert!(!text.contains("launch_token"));
    assert!(!text.contains("secret_value"));
}

#[tokio::test]
async fn admin_addon_install_guide_generates_sidecar_snippets_without_lifecycle_control_or_secrets()
{
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let mut manifest = nako_reference_addon::reference_manifest("http://subtitle-lab:9100/base");
    manifest.id = "dev.nako.subtitle-lab".to_owned();
    manifest.name = "Subtitle Lab".to_owned();
    manifest.version = "0.3.0".to_owned();
    manifest.secret_reference_fields = vec![
        nako_addon_protocol::AddonSecretReferenceFieldDeclaration::new(
            "provider-api-key",
            "Provider API key",
            Some("Resolved by Nako at runtime".to_owned()),
            true,
        ),
    ];

    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;

    let path = format!("/admin/v1/addons/{addon_id}/install-guide");
    let raw = response_for(&router, Method::GET, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonInstallGuideResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.manifest_id, "dev.nako.subtitle-lab");
    assert_eq!(response.addon_name, "Subtitle Lab");
    assert_eq!(response.addon_version, "0.3.0");
    assert_eq!(response.base_url, "http://subtitle-lab:9100/base");
    assert!(!response.lifecycle_boundary.nako_manages_containers);
    assert!(!response.lifecycle_boundary.nako_manages_processes);
    assert!(!response.lifecycle_boundary.nako_manages_packages);
    assert_eq!(response.secret_references.len(), 1);
    assert_eq!(response.secret_references[0].id, "provider-api-key");
    assert_eq!(
        response.secret_references[0].env_var,
        "ADDON_SECRET_PROVIDER_API_KEY"
    );
    assert_eq!(
        response.secret_references[0].placeholder,
        "secret-reference:provider-api-key"
    );
    assert_eq!(
        response.docker_compose.filename,
        "compose.dev-nako-subtitle-lab.yml"
    );
    assert!(
        response
            .docker_compose
            .content
            .contains("dev-nako-subtitle-lab:")
    );
    assert!(
        response
            .docker_compose
            .content
            .contains("secret-reference:provider-api-key")
    );
    assert_eq!(response.systemd.filename, "dev-nako-subtitle-lab.service");
    assert!(
        response
            .systemd
            .content
            .contains("Environment=\"NAKO_ADDON_BASE_URL=http://subtitle-lab:9100/base\"")
    );
    assert!(
        response
            .systemd
            .content
            .contains("ExecStart=<addon-sidecar-command> --listen 0.0.0.0:9100")
    );
    assert_eq!(response.health_check_steps.len(), 2);
    assert!(response.health_check_steps[0].command.contains("/health"));
    assert!(
        response.health_check_steps[1]
            .command
            .contains("/health-check")
    );
    assert_eq!(response.registration_verification_steps.len(), 2);
    assert!(
        response.registration_verification_steps[0]
            .command
            .contains("/admin/v1/addons/")
    );
    assert!(
        response.registration_verification_steps[1]
            .command
            .contains("/surfaces")
    );

    for forbidden in [
        "raw_token",
        "Bearer ",
        "resolved_secret",
        "secret_value",
        "docker.sock",
        "docker stop",
        "docker start",
        "systemctl start",
        "systemctl stop",
        "source_locator",
        "storage_uri",
        "local_path",
        "C:\\",
        "/Users/",
    ] {
        assert!(
            !text.contains(forbidden),
            "install guide leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_manager_plan_combines_registry_permissions_tokens_health_and_install_guide() {
    let (addon_base_url, requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "nako.manager.metadata",
        false,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.manager.metadata".to_owned();
    manifest.name = "Manager Metadata".to_owned();
    manifest.version = "0.1.0".to_owned();
    manifest.secret_reference_fields = vec![
        nako_addon_protocol::AddonSecretReferenceFieldDeclaration::new(
            "provider-api-key",
            "Provider API key",
            Some("Resolved by Nako at runtime".to_owned()),
            true,
        ),
    ];

    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
            label: Some("manager plan runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        &router,
        Method::PUT,
        &format!("/admin/v1/addons/{addon_id}/grants"),
        &ReplaceAddonGrantsRequest {
            grants: vec![
                AddonGrantAssignment {
                    permission: AddonPermission::MetadataWrite,
                    library_id: None,
                },
                AddonGrantAssignment {
                    permission: AddonPermission::ArtworkWrite,
                    library_id: Some(library_id),
                },
            ],
        },
    )
    .await;

    let path = format!("/admin/v1/addons/{addon_id}/manager-plan");
    let raw = response_for(&router, Method::GET, &path).await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonManagerPlanResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert!(response.intent.is_none());
    assert!(!response.operator_confirmed);
    assert_eq!(response.source.summary.id, addon_id);
    assert_eq!(response.source.summary.manifest_id, "nako.manager.metadata");
    assert_eq!(response.source.summary.status, AddonStatus::Enabled);
    assert_eq!(
        response.health_check.status,
        AdminAddonHealthCheckStatus::Reachable
    );
    assert_eq!(response.tokens.tokens.len(), 1);
    assert_eq!(response.tokens.tokens[0].id, issued.token.id);
    assert_eq!(response.tokens.tokens[0].label, "manager plan runtime");
    assert_eq!(response.grants.grants.len(), 2);
    assert_eq!(
        response
            .install_guide
            .lifecycle_boundary
            .nako_manages_containers,
        false
    );
    assert_eq!(
        response
            .install_guide
            .lifecycle_boundary
            .nako_manages_processes,
        false
    );
    assert_eq!(
        response
            .install_guide
            .lifecycle_boundary
            .nako_manages_packages,
        false
    );
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    for forbidden in [
        "raw_token",
        "Bearer ",
        "resolved_secret",
        "secret_value",
        "docker.sock",
        "docker stop",
        "docker start",
        "systemctl start",
        "systemctl stop",
    ] {
        assert!(
            !text.contains(forbidden),
            "manager plan leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_manager_plan_requires_operator_confirmation_for_lifecycle_intents() {
    let (addon_base_url, _requests) = health_check_addon_server(
        StatusCode::OK,
        AddonHealthStatus::Ok,
        "nako.manager.intent",
        false,
    )
    .await;
    let mut manifest = nako_reference_addon::reference_manifest(addon_base_url);
    manifest.id = "nako.manager.intent".to_owned();
    manifest.name = "Manager Intent".to_owned();
    manifest.version = "0.1.0".to_owned();

    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/manager-plan");

    let rejected_response = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonManagerPlanRequest {
            intent: AdminAddonLifecycleIntent::Update,
            operator_confirmed: false,
        },
    )
    .await;
    assert_eq!(rejected_response.status(), StatusCode::BAD_REQUEST);
    let rejected_error = body_json::<ErrorResponse>(rejected_response).await;
    assert_eq!(rejected_error.code, "invalid_input");
    assert!(
        rejected_error
            .message
            .contains("operator confirmation is required")
    );

    for intent in [
        AdminAddonLifecycleIntent::Install,
        AdminAddonLifecycleIntent::Update,
        AdminAddonLifecycleIntent::Remove,
    ] {
        let response = request_body_json::<AdminAddonManagerPlanResponse, _>(
            &router,
            Method::POST,
            &path,
            &AdminAddonManagerPlanRequest {
                intent,
                operator_confirmed: true,
            },
        )
        .await;

        assert_eq!(response.addon_id, addon_id);
        assert_eq!(response.intent, Some(intent));
        assert!(response.operator_confirmed);
        assert_eq!(response.source.summary.manifest_id, "nako.manager.intent");
        assert_eq!(
            response.health_check.status,
            AdminAddonHealthCheckStatus::Reachable
        );
        assert!(response.tokens.tokens.is_empty());
        assert!(response.grants.grants.is_empty());
        assert_eq!(
            response
                .install_guide
                .lifecycle_boundary
                .nako_manages_containers,
            false
        );
        assert_eq!(
            response
                .install_guide
                .lifecycle_boundary
                .nako_manages_processes,
            false
        );
        assert_eq!(
            response
                .install_guide
                .lifecycle_boundary
                .nako_manages_packages,
            false
        );
    }
}

#[tokio::test]
async fn admin_addon_routing_plans_syncs_manifest_declarations_without_hidden_work() {
    let (_temp, router, _source, store) =
        router_with_media_source_config("routing-plan.mkv", b"media", |_| {}).await;

    let mut manifest = nako_reference_addon::reference_manifest("https://addon.example.test/base");
    manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
        "library-scanned",
        DomainEventKind::LibraryScanned.as_str(),
        "/events/library-scanned",
        vec![AddonScope::WebhookEventRead],
        serde_json::json!({"library_preset":"movies","token":"nako_at_should_not_echo"}),
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("library_preset"));

    assert!(
        store
            .list_jobs(
                nako_core::JobListFilter::default(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_outbox_events(
                nako_core::OutboxEventListFilter::default(),
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
        serde_json::json!({"token":"nako_at_should_not_echo"}),
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(!text.contains("nako_at_should_not_echo"));
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
        serde_json::json!({"token":"nako_at_should_not_echo"}),
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(!text.contains("nako_at_should_not_echo"));
}

#[tokio::test]
async fn addon_event_delivery_dispatches_outbox_event_to_executable_subscription() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    let addon_id = register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();

    let raw = response_for(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AddonEventDispatchResponse>(&text).unwrap();

    assert_eq!(response.event.id, event.id);
    assert_eq!(response.attempted_subscriptions, 1);
    assert_eq!(response.delivered, 1);
    assert_eq!(response.failed, 0);
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].addon_id, addon_id);
    assert_eq!(response.attempts[0].declaration_id, "library-scanned");
    assert_eq!(
        response.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Succeeded
    );
    assert_eq!(response.attempts[0].http_status, Some(202));
    assert_eq!(response.attempts[0].next_retry_at, None);
    assert!(!text.contains("nako_at_should_not_echo"));

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    let captured_request = &captured[0];
    assert_eq!(captured_request.request.addon_id, "example.event-delivery");
    assert_eq!(
        captured_request.request.protocol_version,
        ADDON_PROTOCOL_VERSION
    );
    assert_eq!(captured_request.request.subscription_id, "library-scanned");
    assert_eq!(captured_request.request.event_id, event.id.to_string());
    assert_eq!(
        captured_request.request.event_kind,
        DomainEventKind::LibraryScanned.as_str()
    );
    assert_eq!(captured_request.request.subject_kind, "library");
    assert_eq!(
        captured_request.request.subject_id,
        source.library_id.to_string()
    );
    assert_eq!(captured_request.request.attempt, 1);
    assert_eq!(
        captured_request.request.payload["source_id"],
        source.id.to_string()
    );
    assert!(
        captured_request
            .headers
            .iter()
            .any(|(name, value)| name == "x-nako-addon-operation" && value == "event-delivery")
    );
    assert!(captured_request.headers.iter().any(|(name, value)| name
        == "x-nako-addon-event-subscription"
        && value == "library-scanned"));
    assert!(
        captured_request
            .headers
            .iter()
            .all(|(name, _)| name != "authorization" && name != "x-nako-addon-secret")
    );

    let listed = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(listed.event_id, event.id);
    assert_eq!(listed.attempts, response.attempts);

    let stored_event = store.get_outbox_event(event.id).await.unwrap().unwrap();
    assert_eq!(stored_event.status, OutboxEventStatus::Pending);
    assert_eq!(stored_event.attempts, 0);
}

#[tokio::test]
async fn addon_event_scheduler_due_work_reports_redaction_safe_diagnostics() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-scheduler.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    let addon_id = register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-scheduler:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();

    let raw = response_for(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AddonEventSchedulerWorkResponse>(&text).unwrap();

    assert_eq!(response.event.id, event.id);
    assert_eq!(response.due_work_count, 1);
    assert_eq!(response.blocked_work_count, 0);
    assert_eq!(response.work.len(), 1);
    let work = &response.work[0];
    assert_eq!(work.addon_id, addon_id);
    assert_eq!(work.declaration_id, "library-scanned");
    assert_eq!(work.event_kind, DomainEventKind::LibraryScanned.as_str());
    assert_eq!(work.status, AddonEventSchedulerWorkStatus::Due);
    assert_eq!(work.safe_reason_code, None);
    assert_eq!(work.attempt_count, 0);
    assert_eq!(work.next_attempt_number, 1);
    assert_eq!(work.max_attempts, 2);
    assert_eq!(work.latest_attempt_status, None);
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("payload"));

    let captured = requests.lock().await;
    assert!(captured.is_empty());
}

#[tokio::test]
async fn addon_event_delivery_skips_already_succeeded_subscription() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-replay.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-replay:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id
            })
            .to_string(),
        })
        .await
        .unwrap();
    let path = format!("/admin/v1/events/{}/addon-events/deliver", event.id);

    let first = request_json::<AddonEventDispatchResponse>(&router, Method::POST, &path).await;
    assert_eq!(first.attempted_subscriptions, 1);
    assert_eq!(first.delivered, 1);
    assert_eq!(first.skipped_subscriptions, 0);
    assert_eq!(first.attempts.len(), 1);

    let second = request_json::<AddonEventDispatchResponse>(&router, Method::POST, &path).await;
    assert_eq!(second.attempted_subscriptions, 1);
    assert_eq!(second.delivered, 0);
    assert_eq!(second.failed, 0);
    assert_eq!(second.skipped_subscriptions, 1);
    assert!(second.attempts.is_empty());

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    drop(captured);

    let listed = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(listed.attempts.len(), 1);
    assert_eq!(
        listed.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Succeeded
    );

    let scheduler = request_json::<AddonEventSchedulerWorkResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(scheduler.due_work_count, 0);
    assert_eq!(scheduler.blocked_work_count, 0);
    assert_eq!(scheduler.work.len(), 1);
    assert_eq!(
        scheduler.work[0].status,
        AddonEventSchedulerWorkStatus::AlreadySucceeded
    );
}

#[tokio::test]
async fn addon_event_replay_forces_succeeded_subscription_with_operator_reason() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-forced-replay.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-forced-replay:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();
    let deliver_path = format!("/admin/v1/events/{}/addon-events/deliver", event.id);
    let replay_path = format!("/admin/v1/events/{}/addon-events/replay", event.id);

    let first =
        request_json::<AddonEventDispatchResponse>(&router, Method::POST, &deliver_path).await;
    assert_eq!(first.delivered, 1);
    assert_eq!(requests.lock().await.len(), 1);

    let replay_raw = response_body_json(
        &router,
        Method::POST,
        &replay_path,
        &ReplayAddonEventRequest {
            reason_code: "operator_requested".to_owned(),
        },
    )
    .await;
    assert_eq!(replay_raw.status(), StatusCode::OK);
    let replay_text = response_text(replay_raw).await;
    let replay = serde_json::from_str::<AddonEventReplayResponse>(&replay_text).unwrap();

    assert_eq!(replay.reason_code, "operator_requested");
    assert_eq!(replay.dispatch.attempted_subscriptions, 1);
    assert_eq!(replay.dispatch.delivered, 1);
    assert_eq!(replay.dispatch.failed, 0);
    assert_eq!(replay.dispatch.skipped_subscriptions, 0);
    assert_eq!(replay.dispatch.attempts.len(), 1);
    assert_eq!(replay.dispatch.attempts[0].attempt_number, 2);
    assert!(replay.dispatch.attempts[0].forced_replay);
    assert_eq!(
        replay.dispatch.attempts[0].replay_reason_code.as_deref(),
        Some("operator_requested")
    );
    assert!(!replay_text.contains("nako_at_should_not_echo"));

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[1].request.attempt, 2);
    drop(captured);

    let listed = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(listed.attempts.len(), 2);
    assert!(!listed.attempts[0].forced_replay);
    assert!(listed.attempts[1].forced_replay);
    assert_eq!(
        listed.attempts[1].replay_reason_code.as_deref(),
        Some("operator_requested")
    );
}

#[tokio::test]
async fn addon_event_filter_skips_non_matching_subscription_without_payload_leak() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-filter.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    let mut manifest = event_path_manifest(base_url);
    manifest.event_subscriptions[0].filters = serde_json::json!({
        "source_id": MediaSourceId::new().to_string()
    });
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
    request_json::<AdminAddonRoutingPlansResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-filter:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();

    let scheduler_raw = response_for(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(scheduler_raw.status(), StatusCode::OK);
    let scheduler_text = response_text(scheduler_raw).await;
    let scheduler =
        serde_json::from_str::<AddonEventSchedulerWorkResponse>(&scheduler_text).unwrap();
    assert_eq!(scheduler.due_work_count, 0);
    assert_eq!(scheduler.blocked_work_count, 1);
    assert_eq!(scheduler.work.len(), 1);
    assert_eq!(
        scheduler.work[0].status,
        AddonEventSchedulerWorkStatus::Deferred
    );
    assert_eq!(
        scheduler.work[0].safe_reason_code.as_deref(),
        Some("filter_not_matched")
    );
    assert!(!scheduler_text.contains("nako_at_should_not_echo"));

    let delivered = request_json::<AddonEventDispatchResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(delivered.attempted_subscriptions, 0);
    assert_eq!(delivered.delivered, 0);
    assert_eq!(delivered.failed, 0);
    assert_eq!(delivered.skipped_subscriptions, 1);
    assert!(delivered.attempts.is_empty());
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn addon_event_filter_allows_matching_event_facts() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-filter-match.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    let mut manifest = event_path_manifest(base_url);
    manifest.event_subscriptions[0].filters = serde_json::json!({
        "event_kind": "library.scanned",
        "subject_kind": "library",
        "library_id": source.library_id.to_string(),
        "source_id": source.id.to_string()
    });
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
    request_json::<AdminAddonRoutingPlansResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-filter-match:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id
            })
            .to_string(),
        })
        .await
        .unwrap();

    let delivered = request_json::<AddonEventDispatchResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(delivered.attempted_subscriptions, 1);
    assert_eq!(delivered.delivered, 1);
    assert_eq!(delivered.failed, 0);
    assert_eq!(delivered.skipped_subscriptions, 0);
    assert_eq!(requests.lock().await.len(), 1);
}

#[tokio::test]
async fn addon_event_scheduler_prevents_duplicate_in_flight_delivery_under_concurrency() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-concurrent.mkv", b"media", |_| {}).await;
    let gate = StdArc::new(Notify::new());
    let (base_url, requests) =
        blocking_event_path_addon_server(StatusCode::ACCEPTED, StdArc::clone(&gate)).await;
    register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-concurrent:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id
            })
            .to_string(),
        })
        .await
        .unwrap();
    let path = format!("/admin/v1/events/{}/addon-events/deliver", event.id);
    let first_router = router.clone();
    let first_path = path.clone();
    let first = tokio::spawn(async move {
        request_json::<AddonEventDispatchResponse>(&first_router, Method::POST, &first_path).await
    });
    wait_for_event_requests(&requests, 1).await;

    let in_flight = request_json::<AddonEventSchedulerWorkResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(in_flight.due_work_count, 0);
    assert_eq!(in_flight.blocked_work_count, 1);
    assert_eq!(
        in_flight.work[0].status,
        AddonEventSchedulerWorkStatus::InFlight
    );

    let second = request_json::<AddonEventDispatchResponse>(&router, Method::POST, &path).await;
    assert_eq!(second.attempted_subscriptions, 1);
    assert_eq!(second.delivered, 0);
    assert_eq!(second.failed, 0);
    assert_eq!(second.skipped_subscriptions, 1);
    assert!(second.attempts.is_empty());
    assert_eq!(requests.lock().await.len(), 1);

    gate.notify_waiters();
    let first = first.await.unwrap();
    assert_eq!(first.delivered, 1);
    assert_eq!(first.failed, 0);
    assert_eq!(first.attempts.len(), 1);
    assert_eq!(
        first.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Succeeded
    );
    assert_eq!(requests.lock().await.len(), 1);

    let listed = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(listed.attempts.len(), 1);
    assert_eq!(listed.attempts[0].attempt_number, 1);
}

#[tokio::test]
async fn addon_event_scheduler_loop_dispatches_due_outbox_event_when_enabled() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-scheduler-loop.mkv", b"media", |config| {
            config.addon_event_scheduler.enabled = true;
            config.addon_event_scheduler.interval_ms = 10;
            config.addon_event_scheduler.batch_size = 10;
            config.addon_event_scheduler.concurrency = 1;
        })
        .await;
    let (base_url, requests) = event_path_addon_server(StatusCode::ACCEPTED).await;
    register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-scheduler-loop:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id
            })
            .to_string(),
        })
        .await
        .unwrap();

    wait_for_event_requests(&requests, 1).await;
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].request.event_id, event.id.to_string());
    assert_eq!(captured[0].request.attempt, 1);
    drop(captured);

    let listed = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(listed.attempts.len(), 1);
    assert_eq!(
        listed.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Succeeded
    );

    let overview =
        request_json::<AdminOverviewResponse>(&router, Method::GET, "/admin/v1/overview").await;
    assert!(overview.startup.addon_event_scheduler_started);
}

#[tokio::test]
async fn addon_event_scheduler_acknowledges_official_notification_bridge() {
    let (_temp, router, source, store) = router_with_media_source_config(
        "official-notification-bridge-event.mkv",
        b"media",
        |config| {
            config.addon_event_scheduler.enabled = true;
            config.addon_event_scheduler.interval_ms = 10;
            config.addon_event_scheduler.batch_size = 10;
            config.addon_event_scheduler.concurrency = 1;
        },
    )
    .await;
    let (base_url, requests) = notification_bridge_server().await;
    let mut manifest = notification_bridge::default_manifest();
    manifest.base_url = base_url;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::WebhookEventRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    assert_eq!(registered.addon.manifest.id, notification_bridge::ADDON_ID);
    assert_eq!(
        registered.addon.manifest.event_subscriptions[0].id,
        notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );

    let health = request_json::<AdminAddonHealthCheckResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/health-check"),
    )
    .await;
    assert_eq!(health.status, AdminAddonHealthCheckStatus::Reachable);
    assert_eq!(health.resource_count, Some(1));

    let routing = request_json::<AdminAddonRoutingPlansResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;
    assert_eq!(routing.executable, 1);
    assert_eq!(routing.deferred, 0);
    assert_eq!(routing.plans.len(), 1);
    assert_eq!(
        routing.plans[0].declaration_id,
        notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );
    assert_eq!(routing.plans[0].target, AddonRoutingPlanTarget::EventOutbox);

    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("notification-bridge-event:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();

    wait_for_event_requests(&requests, 1).await;
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    let request = &captured[0].request;
    assert_eq!(request.addon_id, notification_bridge::ADDON_ID);
    assert_eq!(
        request.subscription_id,
        notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );
    assert_eq!(
        request.event_kind,
        notification_bridge::LIBRARY_SCANNED_EVENT_KIND
    );
    assert_eq!(request.event_id, event.id.to_string());
    assert_eq!(request.subject_id, source.library_id.to_string());
    assert_eq!(request.payload["source_id"], source.id.to_string());
    assert!(
        captured[0]
            .headers
            .iter()
            .any(|(name, value)| name == "x-nako-addon-operation" && value == "event-delivery")
    );
    assert!(
        captured[0]
            .headers
            .iter()
            .all(|(name, _)| name != "authorization" && name != "x-nako-addon-secret")
    );
    drop(captured);

    let attempts = request_json::<AddonEventDeliveryAttemptsResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-attempts", event.id),
    )
    .await;
    assert_eq!(attempts.attempts.len(), 1);
    assert_eq!(attempts.attempts[0].addon_id, addon_id);
    assert_eq!(
        attempts.attempts[0].declaration_id,
        notification_bridge::LIBRARY_SCANNED_EVENT_SUBSCRIPTION_ID
    );
    assert_eq!(
        attempts.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Succeeded
    );
    assert_eq!(attempts.attempts[0].http_status, Some(200));
    assert!(
        !serde_json::to_string(&attempts)
            .unwrap()
            .contains("nako_at_should_not_echo")
    );

    let scheduler = request_json::<AddonEventSchedulerWorkResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(scheduler.due_work_count, 0);
    assert_eq!(scheduler.work.len(), 1);
    assert_eq!(
        scheduler.work[0].status,
        AddonEventSchedulerWorkStatus::AlreadySucceeded
    );
}

#[tokio::test]
async fn addon_event_scheduler_loop_honors_configured_event_concurrency() {
    let (_temp, router, source, store) = router_with_media_source_config(
        "addon-event-scheduler-concurrency.mkv",
        b"media",
        |config| {
            config.metadata_concurrency = 8;
            config.addon_event_scheduler.enabled = true;
            config.addon_event_scheduler.interval_ms = 10;
            config.addon_event_scheduler.batch_size = 10;
            config.addon_event_scheduler.concurrency = 1;
        },
    )
    .await;
    let gate = StdArc::new(Notify::new());
    let (base_url, requests) =
        blocking_event_path_addon_server(StatusCode::ACCEPTED, StdArc::clone(&gate)).await;
    register_event_path_addon(&router, base_url).await;

    for index in 0..2 {
        store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                kind: DomainEventKind::LibraryScanned,
                subject: DomainEventSubject::Library(source.library_id),
                library_id: Some(source.library_id),
                source_id: Some(source.id),
                idempotency_key: format!("addon-event-scheduler-concurrency:{}:{index}", source.id),
                payload_json: serde_json::json!({
                    "library_id": source.library_id,
                    "source_id": source.id,
                    "index": index
                })
                .to_string(),
            })
            .await
            .unwrap();
    }

    wait_for_event_requests(&requests, 1).await;
    sleep(Duration::from_millis(80)).await;
    assert_eq!(requests.lock().await.len(), 1);

    gate.notify_waiters();
    wait_for_event_requests(&requests, 2).await;
    gate.notify_waiters();
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0].request.attempt, 1);
    assert_eq!(captured[1].request.attempt, 1);
}

#[tokio::test]
async fn addon_event_delivery_records_retryable_failure_without_echoing_payload() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-event-failure.mkv", b"media", |_| {}).await;
    let (base_url, requests) = event_path_addon_server(StatusCode::SERVICE_UNAVAILABLE).await;
    let addon_id = register_event_path_addon(&router, base_url).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(source.library_id),
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            idempotency_key: format!("addon-event-failure:{}", source.id),
            payload_json: serde_json::json!({
                "library_id": source.library_id,
                "source_id": source.id,
                "secret": "nako_at_should_not_echo"
            })
            .to_string(),
        })
        .await
        .unwrap();

    let raw = response_for(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AddonEventDispatchResponse>(&text).unwrap();

    assert_eq!(response.event.id, event.id);
    assert_eq!(response.attempted_subscriptions, 1);
    assert_eq!(response.delivered, 0);
    assert_eq!(response.failed, 1);
    assert_eq!(response.error_count, 0);
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].addon_id, addon_id);
    assert_eq!(response.attempts[0].declaration_id, "library-scanned");
    assert_eq!(
        response.attempts[0].status,
        nako_core::AddonEventDeliveryStatus::Failed
    );
    assert_eq!(response.attempts[0].http_status, Some(503));
    assert!(response.attempts[0].has_error);
    assert!(response.attempts[0].next_retry_at.is_some());
    assert!(!text.contains("retryable_http_failure"));
    assert!(!text.contains("\"retryable\":true"));
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("sidecar failed"));

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].request.attempt, 1);
    assert_eq!(
        captured[0].request.payload["secret"],
        "nako_at_should_not_echo"
    );
    drop(captured);

    let waiting = request_json::<AddonEventSchedulerWorkResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(waiting.due_work_count, 0);
    assert_eq!(waiting.work.len(), 1);
    assert_eq!(
        waiting.work[0].status,
        AddonEventSchedulerWorkStatus::WaitingRetry
    );
    assert_eq!(
        waiting.work[0].safe_reason_code.as_deref(),
        Some("retry_not_due")
    );

    let blocked_retry = request_json::<AddonEventDispatchResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(blocked_retry.attempted_subscriptions, 1);
    assert_eq!(blocked_retry.delivered, 0);
    assert_eq!(blocked_retry.failed, 0);
    assert_eq!(blocked_retry.skipped_subscriptions, 1);
    assert!(blocked_retry.attempts.is_empty());
    assert_eq!(requests.lock().await.len(), 1);

    store
        .set_addon_event_delivery_attempt_result(
            response.attempts[0].id,
            nako_core::AddonEventDeliveryStatus::Failed,
            Some(503),
            Some(r#"{"safe_error_code":"retryable_http_failure"}"#.to_owned()),
            Some("2026-05-25T00:01:00.000Z".to_owned()),
        )
        .await
        .unwrap();
    let retry_due = request_json::<AddonEventSchedulerWorkResponse>(
        &router,
        Method::GET,
        &format!("/admin/v1/events/{}/addon-event-scheduler/work", event.id),
    )
    .await;
    assert_eq!(retry_due.due_work_count, 1);
    assert_eq!(
        retry_due.work[0].status,
        AddonEventSchedulerWorkStatus::RetryDue
    );
    let retry_text = serde_json::to_string(&retry_due).unwrap();
    assert!(!retry_text.contains("nako_at_should_not_echo"));

    let retried = request_json::<AddonEventDispatchResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/events/{}/addon-events/deliver", event.id),
    )
    .await;
    assert_eq!(retried.attempted_subscriptions, 1);
    assert_eq!(retried.delivered, 0);
    assert_eq!(retried.failed, 1);
    assert_eq!(retried.skipped_subscriptions, 0);
    assert_eq!(retried.attempts.len(), 1);
    assert_eq!(retried.attempts[0].attempt_number, 2);
    assert_eq!(retried.attempts[0].next_retry_at, None);
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[1].request.attempt, 2);
}

#[tokio::test]
async fn addon_task_run_runtime_is_host_owned_and_reports_progress_result() {
    let (_temp, router, source, store) =
        router_with_media_source_config("addon-task.mkv", b"media", |_| {}).await;

    let mut manifest = addon_manifest();
    manifest.tasks = vec![
        AddonTaskDeclaration::new(
            "bulk-metadata-scrape",
            "Bulk Metadata Scrape",
            "/tasks/bulk-metadata-scrape",
            vec![AddonScope::AutomationRun],
        )
        .with_execution_bounds(Some(30_000), Some(2)),
    ];
    manifest.scopes.push(AddonScope::AutomationRun);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
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
            label: Some("worker".to_owned()),
        },
    )
    .await;
    let routing_path = format!("/admin/v1/addons/{addon_id}/routing-plans");
    request_json::<AdminAddonRoutingPlansResponse>(&router, Method::POST, &routing_path).await;
    assert!(
        store
            .list_jobs(
                nako_core::JobListFilter {
                    kind: Some(JobKind::AddonTask),
                    ..nako_core::JobListFilter::default()
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );

    let create_path = format!("/admin/v1/addons/{addon_id}/task-runs");
    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &create_path,
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-metadata-scrape".to_owned(),
            idempotency_key: "scrape:library:1".to_owned(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo",
                "mode": "missing-only"
            }),
        },
    )
    .await;

    assert!(!created.idempotent_replay);
    assert_eq!(created.run.addon_id, addon_id);
    assert_eq!(created.run.declaration_id, "bulk-metadata-scrape");
    assert_eq!(created.run.status, JobStatus::Queued);
    assert_eq!(created.run.attempt, 1);
    assert_eq!(created.run.max_attempts, Some(2));
    assert!(created.run.has_input);
    assert!(created.run.progress.is_none());
    assert!(created.run.result.is_none());
    assert_eq!(created.run.library_id, Some(source.library_id));
    assert_eq!(created.run.source_id, Some(source.id));

    let replay = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &create_path,
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-metadata-scrape".to_owned(),
            idempotency_key: "scrape:library:1".to_owned(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "secret": "nako_at_should_not_echo",
                "mode": "missing-only"
            }),
        },
    )
    .await;
    assert!(replay.idempotent_replay);
    assert_eq!(replay.run.job_id, created.run.job_id);

    let conflict = response_body_json(
        &router,
        Method::POST,
        &create_path,
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-metadata-scrape".to_owned(),
            idempotency_key: "scrape:library:1".to_owned(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({"different": true}),
        },
    )
    .await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conflict_text = String::from_utf8(
        to_bytes(conflict.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(conflict_text.contains("already used for a different request"));

    let claim = request_body_json_with_bearer::<ClaimAddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/claim",
        &issued.raw_token,
        &ClaimAddonTaskRunRequest {
            worker_id: None,
            declaration_id: Some("bulk-metadata-scrape".to_owned()),
            lease_duration_ms: 30_000,
        },
    )
    .await;
    let claimed = claim.run.expect("addon task run should be claimable");
    assert_eq!(claimed.run.job_id, created.run.job_id);
    assert_eq!(claimed.run.status, JobStatus::Running);
    assert!(claimed.cancel_requested_at.is_none());
    assert_eq!(claimed.input["schema"], "nako.addon.task_run.input.v1");
    assert_eq!(
        claimed.input["payload"]["secret"],
        "nako_at_should_not_echo"
    );

    let progress = request_body_json_with_bearer::<nako_api::extension::AddonTaskRunLease, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/progress",
        &issued.raw_token,
        &ReportAddonTaskRunProgressRequest {
            guard: claimed.guard,
            lease_duration_ms: 30_000,
            stage: "scraping".to_owned(),
            percent: Some(40),
            message: Some("Fetched provider candidates".to_owned()),
            metrics: serde_json::json!({"items": 3}),
        },
    )
    .await;
    assert_eq!(progress.run.status, JobStatus::Running);
    assert_eq!(progress.input["schema"], "nako.addon.task_run.input.v1");
    assert_eq!(
        progress.run.progress.as_ref().unwrap()["schema"],
        "nako.addon.task_run.progress.v1"
    );
    assert_eq!(progress.run.progress.as_ref().unwrap()["percent"], 40);

    let completed = request_body_json_with_bearer::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/complete",
        &issued.raw_token,
        &CompleteAddonTaskRunRequest {
            guard: progress.guard,
            output: serde_json::json!({"accepted": 2}),
        },
    )
    .await;

    assert_eq!(completed.run.status, JobStatus::Succeeded);
    assert_eq!(
        completed.run.result.as_ref().unwrap()["schema"],
        "nako.addon.task_run.result.v1"
    );
    assert_eq!(
        completed.run.result.as_ref().unwrap()["status"],
        "succeeded"
    );
    let body = serde_json::to_string(&completed).unwrap();
    assert!(!body.contains("nako_at_should_not_echo"));
    assert!(!body.contains("input_json"));
    assert!(!body.contains("summary_json"));
    assert!(!body.contains("error\":\""));
}

#[tokio::test]
async fn addon_task_run_failure_can_be_retried_until_max_attempts() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-retry.mkv", b"media", |_| {}).await;

    let mut manifest = addon_manifest();
    manifest.tasks = vec![
        AddonTaskDeclaration::new(
            "bulk-task",
            "Bulk Task",
            "/tasks/bulk",
            vec![AddonScope::AutomationRun],
        )
        .with_execution_bounds(Some(30_000), Some(2)),
    ];
    manifest.scopes.push(AddonScope::AutomationRun);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
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
            label: Some("worker".to_owned()),
        },
    )
    .await;
    request_json::<AdminAddonRoutingPlansResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "retry:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({"mode": "full"}),
        },
    )
    .await;
    let claim = request_body_json_with_bearer::<ClaimAddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/claim",
        &issued.raw_token,
        &ClaimAddonTaskRunRequest {
            worker_id: None,
            declaration_id: Some("bulk-task".to_owned()),
            lease_duration_ms: 30_000,
        },
    )
    .await
    .run
    .unwrap();
    let failed = request_body_json_with_bearer::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/fail",
        &issued.raw_token,
        &FailAddonTaskRunRequest {
            guard: claim.guard,
            safe_error_code: "rate_limited".to_owned(),
            retry_after_ms: Some(1_000),
            output: serde_json::json!({"provider": "tmdb"}),
        },
    )
    .await;
    assert_eq!(failed.run.status, JobStatus::Failed);
    assert!(failed.run.retryable);
    assert_eq!(failed.run.safe_error_code.as_deref(), Some("rate_limited"));

    let retry = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/addons/{addon_id}/task-runs/{}/retry",
            created.run.job_id
        ),
        &RetryAddonTaskRunRequest {
            idempotency_key: "retry:second".to_owned(),
        },
    )
    .await;

    assert_eq!(retry.run.status, JobStatus::Queued);
    assert_eq!(retry.run.attempt, 2);
    assert_eq!(retry.run.retry_of_job_id, Some(created.run.job_id));
    assert!(!retry.run.retryable);
}

#[tokio::test]
async fn addon_task_run_cancellation_is_requested_by_host_and_acknowledged_by_sidecar() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-cancel.mkv", b"media", |_| {}).await;

    let mut manifest = addon_manifest();
    manifest.tasks = vec![AddonTaskDeclaration::new(
        "bulk-task",
        "Bulk Task",
        "/tasks/bulk",
        vec![AddonScope::AutomationRun],
    )];
    manifest.scopes.push(AddonScope::AutomationRun);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
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
            label: Some("worker".to_owned()),
        },
    )
    .await;
    request_json::<AdminAddonRoutingPlansResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/routing-plans"),
    )
    .await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "cancel:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({"mode": "cancel"}),
        },
    )
    .await;
    let claim = request_body_json_with_bearer::<ClaimAddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/claim",
        &issued.raw_token,
        &ClaimAddonTaskRunRequest {
            worker_id: None,
            declaration_id: Some("bulk-task".to_owned()),
            lease_duration_ms: 30_000,
        },
    )
    .await
    .run
    .unwrap();

    let cancel_response = response_for(
        &router,
        Method::POST,
        &format!("/admin/v1/jobs/{}/cancel", created.run.job_id),
    )
    .await;
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_body = body_json::<AdminJobCancelRequestResponse>(cancel_response).await;
    assert!(cancel_body.requested);
    assert!(!cancel_body.terminal);
    assert_eq!(cancel_body.job.status, JobStatus::Running);

    let progress = request_body_json_with_bearer::<nako_api::extension::AddonTaskRunLease, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/progress",
        &issued.raw_token,
        &ReportAddonTaskRunProgressRequest {
            guard: claim.guard,
            lease_duration_ms: 30_000,
            stage: "stopping".to_owned(),
            percent: None,
            message: Some("Cancellation observed".to_owned()),
            metrics: serde_json::json!({}),
        },
    )
    .await;
    assert!(progress.cancel_requested_at.is_some());

    let cancelled = request_body_json_with_bearer::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        "/addon/v1/task-runs/cancel",
        &issued.raw_token,
        &CancelAddonTaskRunRequest {
            guard: progress.guard,
            output: serde_json::json!({"stopped": true}),
        },
    )
    .await;

    assert_eq!(cancelled.run.status, JobStatus::Cancelled);
    assert_eq!(
        cancelled.run.result.as_ref().unwrap()["status"],
        "cancelled"
    );
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_calls_declared_sidecar_path_and_completes() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct.mkv", b"media", |_| {}).await;
    let (base_url, requests) = task_path_addon_server(vec![StatusCode::OK]).await;
    let addon_id = register_task_path_addon(&router, base_url).await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "mode": "direct",
                "secret": "nako_at_should_not_echo"
            }),
        },
    )
    .await;
    assert_eq!(created.run.status, JobStatus::Queued);

    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    assert_eq!(
        completed.run.result.as_ref().unwrap()["status"],
        "succeeded"
    );
    assert_eq!(
        completed.run.result.as_ref().unwrap()["output"]["mode"],
        "direct"
    );
    assert_eq!(
        completed.run.progress.as_ref().unwrap()["stage"],
        "dispatched"
    );
    assert_eq!(
        completed.run.progress.as_ref().unwrap()["metrics"]["http_status"],
        200
    );

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    let request = &captured[0].request;
    assert_eq!(request.task_id, "bulk-task");
    assert_eq!(request.job_id, created.run.job_id.to_string());
    assert_eq!(request.attempt, 1);
    assert_eq!(request.library_id, Some(source.library_id.to_string()));
    assert_eq!(request.source_id, Some(source.id.to_string()));
    assert_eq!(request.payload["mode"], "direct");
    assert!(
        captured[0]
            .headers
            .iter()
            .any(|(name, value)| name == "x-nako-addon-operation" && value == "task-dispatch")
    );
    let body = serde_json::to_string(&completed).unwrap();
    assert!(!body.contains("nako_at_should_not_echo"));
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_sends_bearer_token_from_host_secret_env() {
    let secret_env = "NAKO_TEST_ADDON_TASK_BEARER_TOKEN";
    let secret = "nako_bearer_direct_secret";
    let _secret_guard = crate::app::set_test_outbound_task_dispatch_secret(secret_env, secret);
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct-bearer.mkv", b"media", |_| {}).await;
    let (base_url, requests) = task_path_addon_server(vec![StatusCode::OK]).await;
    let addon_id = register_task_path_addon_with_auth(
        &router,
        base_url,
        AddonAuth::Bearer,
        Some(secret_env.to_owned()),
    )
    .await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct-bearer:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "mode": "bearer",
                "secret": "nako_at_should_not_echo"
            }),
        },
    )
    .await;

    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    assert_eq!(completed.run.status, JobStatus::Succeeded);
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    assert!(captured[0].headers.iter().any(
        |(name, value)| name == "authorization" && value == "Bearer nako_bearer_direct_secret"
    ));
    assert!(
        captured[0]
            .headers
            .iter()
            .all(|(name, _)| name != "x-nako-addon-secret")
    );

    let body = serde_json::to_string(&completed).unwrap();
    assert!(!body.contains(secret));
    assert!(!body.contains("nako_at_should_not_echo"));
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_sends_shared_secret_from_host_secret_env() {
    let secret_env = "NAKO_TEST_ADDON_TASK_SHARED_SECRET";
    let secret = "nako_shared_direct_secret";
    let _secret_guard = crate::app::set_test_outbound_task_dispatch_secret(secret_env, secret);
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct-shared-secret.mkv", b"media", |_| {})
            .await;
    let (base_url, requests) = task_path_addon_server(vec![StatusCode::OK]).await;
    let addon_id = register_task_path_addon_with_auth(
        &router,
        base_url,
        AddonAuth::SharedSecret,
        Some(secret_env.to_owned()),
    )
    .await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct-shared-secret:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "mode": "shared-secret",
                "secret": "nako_at_should_not_echo"
            }),
        },
    )
    .await;

    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    assert_eq!(completed.run.status, JobStatus::Succeeded);
    let captured = requests.lock().await;
    assert_eq!(captured.len(), 1);
    assert!(
        captured[0]
            .headers
            .iter()
            .any(|(name, value)| name == "x-nako-addon-secret" && value == secret)
    );
    assert!(
        captured[0]
            .headers
            .iter()
            .all(|(name, _)| name != "authorization")
    );

    let body = serde_json::to_string(&completed).unwrap();
    assert!(!body.contains(secret));
    assert!(!body.contains("nako_at_should_not_echo"));
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_missing_host_secret_env_fails_safely() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct-missing-secret.mkv", b"media", |_| {})
            .await;
    let (base_url, requests) = task_path_addon_server(vec![StatusCode::OK]).await;
    let missing_env = format!(
        "NAKO_TEST_MISSING_ADDON_TASK_SECRET_{}",
        JobId::new().to_string().replace('-', "_")
    );
    let addon_id = register_task_path_addon_with_auth(
        &router,
        base_url,
        AddonAuth::Bearer,
        Some(missing_env.clone()),
    )
    .await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct-missing-secret:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "mode": "missing-secret",
                "secret": "nako_at_should_not_echo"
            }),
        },
    )
    .await;

    let failed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Failed).await;
    assert_eq!(failed.run.status, JobStatus::Failed);
    assert_eq!(
        failed.run.safe_error_code.as_deref(),
        Some("authorization_gap")
    );
    assert!(requests.lock().await.is_empty());

    let body = serde_json::to_string(&failed).unwrap();
    assert!(!body.contains(&missing_env));
    assert!(!body.contains("nako_at_should_not_echo"));
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_failure_can_be_retried_as_direct_dispatch() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct-retry.mkv", b"media", |_| {}).await;
    let (base_url, requests) =
        task_path_addon_server(vec![StatusCode::SERVICE_UNAVAILABLE, StatusCode::OK]).await;
    let addon_id = register_task_path_addon(&router, base_url).await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct-retry:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({"mode": "retry"}),
        },
    )
    .await;
    let failed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Failed).await;
    assert!(failed.run.retryable);
    assert_eq!(
        failed.run.safe_error_code.as_deref(),
        Some("retryable_http_failure")
    );

    let retry = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/addons/{addon_id}/task-runs/{}/retry",
            created.run.job_id
        ),
        &RetryAddonTaskRunRequest {
            idempotency_key: "direct-retry:second".to_owned(),
        },
    )
    .await;
    assert_eq!(retry.run.status, JobStatus::Queued);
    assert_eq!(retry.run.attempt, 2);
    assert_eq!(retry.run.retry_of_job_id, Some(created.run.job_id));

    let completed =
        wait_for_addon_task_status(&router, addon_id, retry.run.job_id, JobStatus::Succeeded).await;
    assert_eq!(
        completed.run.result.as_ref().unwrap()["output"]["mode"],
        "retry"
    );

    let captured = requests.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0].request.job_id, created.run.job_id.to_string());
    assert_eq!(captured[1].request.job_id, retry.run.job_id.to_string());
    assert_eq!(
        captured[1].request.retry_of_job_id.as_deref(),
        Some(created.run.job_id.to_string().as_str())
    );
}

#[tokio::test]
async fn addon_task_run_direct_dispatch_records_cancelled_when_host_cancel_requested_in_flight() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("addon-task-direct-cancel.mkv", b"media", |_| {}).await;
    let gate = StdArc::new(Notify::new());
    let (base_url, requests) =
        task_path_addon_server_with_gate(vec![StatusCode::OK], Some(StdArc::clone(&gate))).await;
    let addon_id = register_task_path_addon(&router, base_url).await;

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: "bulk-task".to_owned(),
            idempotency_key: "direct-cancel:first".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({"mode": "cancel"}),
        },
    )
    .await;

    for _ in 0..100 {
        if !requests.lock().await.is_empty() {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(requests.lock().await.len(), 1);

    let cancel_response = response_for(
        &router,
        Method::POST,
        &format!("/admin/v1/jobs/{}/cancel", created.run.job_id),
    )
    .await;
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_body = body_json::<AdminJobCancelRequestResponse>(cancel_response).await;
    assert!(cancel_body.requested);
    assert!(!cancel_body.terminal);
    assert_eq!(cancel_body.job.status, JobStatus::Running);

    gate.notify_one();
    let cancelled =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Cancelled)
            .await;
    assert_eq!(
        cancelled.run.result.as_ref().unwrap()["status"],
        "cancelled"
    );
    assert_eq!(
        cancelled.run.result.as_ref().unwrap()["output"]["completed_output"]["mode"],
        "cancel"
    );
}

#[tokio::test]
async fn addon_external_acquisition_action_direct_dispatches_host_owned_reference() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("external-acquisition-action.mkv", b"media", |_| {}).await;
    let (base_url, requests) =
        external_acquisition_action_addon_server(AddonExternalAcquisitionActionStatus::Accepted)
            .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
            selected_link_ref: "selected-link-ref-1".to_owned(),
        },
        runner_profile_id: "fixture".to_owned(),
        idempotency_key: "external-acquisition:enqueue:1".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:approval:1".to_owned()),
    };

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    assert_eq!(created.run.status, JobStatus::Queued);

    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    let output = &completed.run.result.as_ref().unwrap()["output"];
    assert_eq!(
        output["schema"],
        ADDON_EXTERNAL_ACQUISITION_ACTION_RESPONSE_SCHEMA
    );
    assert_eq!(output["status"], "accepted");
    assert_eq!(output["state"], "running");
    assert_eq!(output["runner_job_ref"], "fixture-job-1");
    assert_eq!(
        completed.run.progress.as_ref().unwrap()["metrics"]["external_acquisition_status"],
        "accepted"
    );

    {
        let captured = requests.lock().await;
        assert_eq!(captured.len(), 1);
        let request = &captured[0].request;
        assert_eq!(request.task_id, external_acquisition_runner::ACTION_TASK_ID);
        assert_eq!(request.job_id, created.run.job_id.to_string());
        assert_eq!(
            request.payload,
            serde_json::to_value(&action_payload).unwrap()
        );
        assert!(
            captured[0]
                .headers
                .iter()
                .any(|(name, value)| name == "x-nako-addon-operation" && value == "task-dispatch")
        );
    }
    let body = serde_json::to_string(&completed).unwrap();
    assert!(!body.contains("raw_url"));
    assert!(!body.contains("password"));

    let replay = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    assert!(replay.idempotent_replay);
    assert_eq!(replay.run.job_id, created.run.job_id);
    assert_eq!(requests.lock().await.len(), 1);
}

#[tokio::test]
async fn addon_external_acquisition_action_materializes_through_fake_sidecar_without_leaks() {
    let (_temp, router, source, store) = router_with_media_source_config(
        "external-acquisition-e2e-materialize.mkv",
        b"media",
        |_| {},
    )
    .await;
    let candidate_id = AcquisitionIntakeCandidateId::new();
    let now_ms = crate::app::current_time_ms().unwrap();
    let raw_material_uri =
        "magnet:?xt=urn:btih:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&dn=e2e-secret".to_owned();
    store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: candidate_id,
            target_library_id: source.library_id,
            source_kind: AcquisitionIntakeSourceKind::ResourceSearchSelection,
            source_key: "materialize-e2e-selected-link".to_owned(),
            source_uri: raw_material_uri.clone(),
            display_name: Some("E2E Secret Download".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Ready,
            diagnostics_json: Some(
                serde_json::json!({
                    "schema": "nako.acquisition_intake.resource_search_selection.v1",
                    "resource_search_selection": true,
                    "link": {
                        "type": "magnet",
                        "source_ref_redacted": "magnet://<redacted>",
                        "has_password": false
                    }
                })
                .to_string(),
            ),
            first_seen_at_ms: now_ms,
            last_seen_at_ms: now_ms,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    let nako_base_url = start_nako_test_http_server(router.clone()).await;
    let (base_url, action_requests, materializations, raw_token_slot) =
        external_acquisition_materializing_addon_server(nako_base_url).await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("materializing runner".to_owned()),
        },
    )
    .await;
    *raw_token_slot.lock().await = Some(issued.raw_token);

    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
            selected_link_ref: candidate_id.to_string(),
        },
        runner_profile_id: "fixture".to_owned(),
        idempotency_key: "external-acquisition:e2e-materialize:1".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:e2e-materialize:1".to_owned()),
    };
    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;

    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    let output = &completed.run.result.as_ref().unwrap()["output"];
    assert_eq!(output["status"], "accepted");
    assert_eq!(output["state"], "running");
    assert_eq!(output["runner_job_ref"], "fixture-job-materialized-1");
    assert_eq!(
        output["safe_facts"]["materialization_client"],
        "nako_runtime"
    );
    assert_eq!(output["safe_facts"]["materialized_link_type"], "magnet");
    assert_eq!(output["safe_facts"]["materialization_ref_present"], "true");

    {
        let captured = action_requests.lock().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].request.job_id, created.run.job_id.to_string());
        assert_eq!(
            captured[0].request.task_id,
            external_acquisition_runner::ACTION_TASK_ID
        );
        assert!(
            captured[0]
                .headers
                .iter()
                .any(|(name, value)| name == "x-nako-addon-operation" && value == "task-dispatch")
        );
    }
    {
        let captured = materializations.lock().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].request.job_id, created.run.job_id.to_string());
        assert_eq!(
            captured[0].request.declaration_id,
            external_acquisition_runner::ACTION_TASK_ID
        );
        assert_eq!(captured[0].request.target_ref, action_payload.target_ref);
        assert_eq!(
            captured[0].request.runner_profile_id,
            action_payload.runner_profile_id
        );
        assert_eq!(
            captured[0].request.idempotency_key,
            action_payload.idempotency_key
        );
        assert_eq!(
            captured[0].request.audit_ref,
            action_payload.audit_ref.clone().unwrap()
        );
        assert_eq!(captured[0].response.material.uri, raw_material_uri);
        assert_eq!(
            captured[0].response.material.link_type,
            AddonResourceLinkType::Magnet
        );
        assert_eq!(
            captured[0].response.safe_facts.get("source_ref_redacted"),
            Some(&"magnet://<redacted>".to_owned())
        );

        let candidate_id_text = candidate_id.to_string();
        let completed_text = serde_json::to_string(&completed).unwrap();
        for forbidden in [
            raw_material_uri.as_str(),
            "e2e-secret",
            candidate_id_text.as_str(),
            action_payload.idempotency_key.as_str(),
            captured[0].response.materialization_ref.as_str(),
        ] {
            assert!(
                !completed_text.contains(forbidden),
                "completed task response leaked forbidden term: {forbidden}"
            );
        }
    }
}

#[tokio::test]
async fn addon_external_acquisition_materialization_resolves_selected_link_for_running_action() {
    let (_temp, router, source, store) =
        router_with_media_source_config("external-acquisition-materialize.mkv", b"media", |_| {})
            .await;
    let candidate_id = AcquisitionIntakeCandidateId::new();
    let now_ms = crate::app::current_time_ms().unwrap();
    store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: candidate_id,
            target_library_id: source.library_id,
            source_kind: AcquisitionIntakeSourceKind::ResourceSearchSelection,
            source_key: "materialize-selected-link".to_owned(),
            source_uri: "magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12".to_owned(),
            display_name: Some("Secret Download".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Ready,
            diagnostics_json: Some(
                serde_json::json!({
                    "schema": "nako.acquisition_intake.resource_search_selection.v1",
                    "resource_search_selection": true,
                    "link": {
                        "type": "magnet",
                        "source_ref_redacted": "magnet://<redacted>",
                        "has_password": false
                    }
                })
                .to_string(),
            ),
            first_seen_at_ms: now_ms,
            last_seen_at_ms: now_ms,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    let gate = StdArc::new(Notify::new());
    let (base_url, requests) = external_acquisition_action_addon_server_with_gate(
        AddonExternalAcquisitionActionStatus::Accepted,
        Some(StdArc::clone(&gate)),
    )
    .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("runner".to_owned()),
        },
    )
    .await;
    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
            selected_link_ref: candidate_id.to_string(),
        },
        runner_profile_id: "transmission-local".to_owned(),
        idempotency_key: "external-acquisition:materialize:1".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:materialize:1".to_owned()),
    };
    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    for _ in 0..100 {
        if requests.lock().await.len() == 1 {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(requests.lock().await.len(), 1);

    let materialized =
        request_body_json_with_bearer::<AddonExternalAcquisitionMaterializationResponse, _>(
            &router,
            Method::POST,
            ADDON_RUNTIME_EXTERNAL_ACQUISITION_MATERIALIZE_PATH,
            &issued.raw_token,
            &AddonExternalAcquisitionMaterializationRequest {
                schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA.to_owned(),
                job_id: created.run.job_id.to_string(),
                declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
                target_ref: action_payload.target_ref.clone(),
                runner_profile_id: action_payload.runner_profile_id.clone(),
                idempotency_key: action_payload.idempotency_key.clone(),
                operation: AddonExternalAcquisitionOperation::Enqueue,
                audit_ref: "audit:materialize:1".to_owned(),
                purpose: AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue,
            },
        )
        .await;

    assert_eq!(
        materialized.schema,
        ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_RESPONSE_SCHEMA
    );
    assert_eq!(materialized.target_ref, action_payload.target_ref);
    assert_eq!(
        materialized.material.link_type,
        AddonResourceLinkType::Magnet
    );
    assert!(materialized.material.uri.starts_with("magnet:?xt="));
    assert_eq!(
        materialized.safe_facts.get("source_ref_redacted"),
        Some(&"magnet://<redacted>".to_owned())
    );
    assert!(
        !serde_json::to_string(&materialized.safe_facts)
            .unwrap()
            .contains("btih")
    );

    let rejected = addon_external_acquisition_materialization(
        &router,
        Some(&issued.raw_token),
        &AddonExternalAcquisitionMaterializationRequest {
            schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA.to_owned(),
            job_id: created.run.job_id.to_string(),
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            target_ref: AddonExternalAcquisitionTargetRef::RunnerJob {
                runner_job_ref: "fixture-job-1".to_owned(),
            },
            runner_profile_id: "transmission-local".to_owned(),
            idempotency_key: "external-acquisition:materialize:1".to_owned(),
            operation: AddonExternalAcquisitionOperation::QueryStatus,
            audit_ref: "audit:materialize:1".to_owned(),
            purpose: AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue,
        },
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let rejected_text = response_text(rejected).await;
    assert!(!rejected_text.contains("magnet:"));
    assert!(!rejected_text.contains("btih"));

    gate.notify_waiters();
    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    let completed_text = serde_json::to_string(&completed).unwrap();
    assert!(!completed_text.contains("magnet:"));
    assert!(!completed_text.contains("btih"));
}

#[tokio::test]
async fn addon_external_acquisition_materialization_resolves_intake_candidate_for_running_action() {
    let (_temp, router, source, store) = router_with_media_source_config(
        "external-acquisition-materialize-intake.mkv",
        b"media",
        |_| {},
    )
    .await;
    let candidate_id = AcquisitionIntakeCandidateId::new();
    let now_ms = crate::app::current_time_ms().unwrap();
    store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: candidate_id,
            target_library_id: source.library_id,
            source_kind: AcquisitionIntakeSourceKind::AddonProposed,
            source_key: "materialize-intake-candidate".to_owned(),
            source_uri: "https://download.example.test/private/file.torrent?token=secret"
                .to_owned(),
            display_name: Some("Private Torrent".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Ready,
            diagnostics_json: Some(
                serde_json::json!({
                    "schema": "nako.acquisition_intake.addon_proposed.v1",
                    "link": {
                        "type": "web",
                        "source_ref_redacted": "https://<redacted>"
                    }
                })
                .to_string(),
            ),
            first_seen_at_ms: now_ms,
            last_seen_at_ms: now_ms,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    let gate = StdArc::new(Notify::new());
    let (base_url, requests) = external_acquisition_action_addon_server_with_gate(
        AddonExternalAcquisitionActionStatus::Accepted,
        Some(StdArc::clone(&gate)),
    )
    .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("runner".to_owned()),
        },
    )
    .await;
    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::IntakeCandidate {
            intake_candidate_ref: candidate_id.to_string(),
        },
        runner_profile_id: "transmission-local".to_owned(),
        idempotency_key: "external-acquisition:materialize:intake".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:materialize:intake".to_owned()),
    };
    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    for _ in 0..100 {
        if requests.lock().await.len() == 1 {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(requests.lock().await.len(), 1);

    let materialized =
        request_body_json_with_bearer::<AddonExternalAcquisitionMaterializationResponse, _>(
            &router,
            Method::POST,
            ADDON_RUNTIME_EXTERNAL_ACQUISITION_MATERIALIZE_PATH,
            &issued.raw_token,
            &AddonExternalAcquisitionMaterializationRequest {
                schema: ADDON_EXTERNAL_ACQUISITION_MATERIALIZATION_REQUEST_SCHEMA.to_owned(),
                job_id: created.run.job_id.to_string(),
                declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
                target_ref: action_payload.target_ref.clone(),
                runner_profile_id: action_payload.runner_profile_id.clone(),
                idempotency_key: action_payload.idempotency_key.clone(),
                operation: AddonExternalAcquisitionOperation::Enqueue,
                audit_ref: "audit:materialize:intake".to_owned(),
                purpose: AddonExternalAcquisitionMaterializationPurpose::ExternalAcquisitionEnqueue,
            },
        )
        .await;

    assert_eq!(materialized.target_ref, action_payload.target_ref);
    assert_eq!(materialized.material.link_type, AddonResourceLinkType::Web);
    assert_eq!(
        materialized.safe_facts.get("source_ref_redacted"),
        Some(&"https://<redacted>".to_owned())
    );
    assert!(
        !serde_json::to_string(&materialized.safe_facts)
            .unwrap()
            .contains("secret")
    );

    gate.notify_waiters();
    let completed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Succeeded)
            .await;
    let completed_text = serde_json::to_string(&completed).unwrap();
    assert!(!completed_text.contains("download.example.test"));
    assert!(!completed_text.contains("token=secret"));
}

#[tokio::test]
async fn addon_external_acquisition_action_rejects_unsafe_payload_before_dispatch() {
    let (_temp, router, source, store) =
        router_with_media_source_config("external-acquisition-unsafe.mkv", b"media", |_| {}).await;
    let (base_url, requests) =
        external_acquisition_action_addon_server(AddonExternalAcquisitionActionStatus::Accepted)
            .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;

    let response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: "external-acquisition:unsafe:1".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::json!({
                "schema": ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA,
                "target_ref": {
                    "kind": "selected_link",
                    "selected_link_ref": "selected-link-ref-unsafe",
                    "raw_url": "magnet:?xt=urn:btih:secret"
                },
                "runner_profile_id": "fixture",
                "idempotency_key": "external-acquisition:unsafe:1",
                "operation": "enqueue",
                "audit_ref": "audit:unsafe:1",
                "password": "secret-code"
            }),
        },
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("invalid_input"));
    assert!(!body.contains("magnet:"));
    assert!(!body.contains("secret-code"));
    assert!(requests.lock().await.is_empty());
    assert!(
        store
            .list_jobs(
                nako_core::JobListFilter {
                    kind: Some(JobKind::AddonTask),
                    ..nako_core::JobListFilter::default()
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn addon_external_acquisition_action_requires_direct_dispatch_and_aligned_idempotency() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("external-acquisition-policy.mkv", b"media", |_| {}).await;
    let (base_url, requests) =
        external_acquisition_action_addon_server(AddonExternalAcquisitionActionStatus::Accepted)
            .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
            selected_link_ref: "selected-link-ref-policy".to_owned(),
        },
        runner_profile_id: "fixture".to_owned(),
        idempotency_key: "external-acquisition:runner-key".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:policy:1".to_owned()),
    };

    let sidecar_claim = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::SidecarClaim,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    assert_eq!(sidecar_claim.status(), StatusCode::BAD_REQUEST);

    let mismatch = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: "external-acquisition:host-key".to_owned(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;
    assert_eq!(mismatch.status(), StatusCode::BAD_REQUEST);
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn addon_external_acquisition_action_runner_rejection_fails_host_task_safely() {
    let (_temp, router, source, _store) =
        router_with_media_source_config("external-acquisition-rejected.mkv", b"media", |_| {})
            .await;
    let (base_url, requests) =
        external_acquisition_action_addon_server(AddonExternalAcquisitionActionStatus::Rejected)
            .await;
    let addon_id = register_external_acquisition_runner_addon(&router, base_url).await;
    let action_payload = AddonExternalAcquisitionActionRequest {
        schema: ADDON_EXTERNAL_ACQUISITION_ACTION_REQUEST_SCHEMA.to_owned(),
        target_ref: AddonExternalAcquisitionTargetRef::SelectedLink {
            selected_link_ref: "selected-link-ref-rejected".to_owned(),
        },
        runner_profile_id: "missing-profile".to_owned(),
        idempotency_key: "external-acquisition:rejected:1".to_owned(),
        operation: AddonExternalAcquisitionOperation::Enqueue,
        audit_ref: Some("audit:rejected:1".to_owned()),
    };

    let created = request_body_json::<AddonTaskRunResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/task-runs"),
        &CreateAddonTaskRunRequest {
            declaration_id: external_acquisition_runner::ACTION_TASK_ID.to_owned(),
            idempotency_key: action_payload.idempotency_key.clone(),
            dispatch: AddonTaskRunDispatchMode::Direct,
            library_id: Some(source.library_id),
            source_id: Some(source.id),
            payload: serde_json::to_value(&action_payload).unwrap(),
        },
    )
    .await;

    let failed =
        wait_for_addon_task_status(&router, addon_id, created.run.job_id, JobStatus::Failed).await;
    assert_eq!(
        failed.run.safe_error_code.as_deref(),
        Some("runner_profile_unavailable")
    );
    assert_eq!(
        failed.run.result.as_ref().unwrap()["output"]["status"],
        "rejected"
    );
    assert_eq!(requests.lock().await.len(), 1);
    let body = serde_json::to_string(&failed).unwrap();
    assert!(!body.contains("selected-link-ref-rejected"));
    assert!(!body.contains("missing-profile"));
}

#[tokio::test]
async fn addon_generated_artifact_handoff_enters_ailo_without_canonical_or_file_writes() {
    let (_temp, router, source, store) =
        router_with_media_source("artifact-handoff.mkv", b"media").await;
    let library_id = source.library_id;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::PATCH,
        &format!("/admin/v1/addons/{addon_id}/status"),
        &UpdateAddonStatusRequest {
            status: AddonStatus::Enabled,
        },
    )
    .await;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("generated artifact runtime".to_owned()),
        },
    )
    .await;
    let request = SubmitAddonGeneratedArtifactRequest {
        capability: AutomationCapability::MetadataCleanup,
        kind: AutomationArtifactKind::MetadataSuggestion,
        library_id: Some(library_id),
        item_id: Some(source.item_id),
        source_id: Some(source.id),
        idempotency_key: format!("addon-generated:{}", source.item_id),
        prompt: serde_json::json!({
            "source_locator": "local:///Movies/private/artifact-handoff.mkv",
            "token": "nako_at_should_not_echo"
        }),
        payload: serde_json::json!({
            "overview": "private generated overview",
            "confidence_milli": 810,
            "explanation": "private chain of thought"
        }),
    };

    let response = addon_generated_artifact(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let body = serde_json::from_str::<AddonGeneratedArtifactResponse>(&text).unwrap();

    assert!(!body.idempotent_replay);
    assert_eq!(
        body.artifact.capability,
        AutomationCapability::MetadataCleanup
    );
    assert_eq!(
        body.artifact.kind,
        AutomationArtifactKind::MetadataSuggestion
    );
    assert_eq!(body.artifact.library_id, Some(library_id));
    assert_eq!(body.artifact.item_id, Some(source.item_id));
    assert_eq!(body.artifact.source_id, Some(source.id));
    assert_eq!(body.artifact.status, AutomationArtifactStatus::Proposed);
    assert!(!body.artifact.writes_canonical_metadata);
    assert!(!body.artifact.writes_sidecar);
    assert!(!body.artifact.writes_library_files);
    assert!(!body.artifact.creates_media_source);
    assert!(!body.artifact.creates_managed_import);
    assert!(!text.contains("private generated overview"));
    assert!(!text.contains("private chain of thought"));
    assert!(!text.contains("local:///Movies/private"));
    assert!(!text.contains("nako_at_should_not_echo"));

    let proposals = request_json::<AdminGeneratedArtifactProposalListResponse>(
        &router,
        Method::GET,
        "/admin/v1/automation/generated-artifacts/proposals?limit=5",
    )
    .await;
    assert_eq!(proposals.proposals.len(), 1);
    assert_eq!(proposals.proposals[0].id, body.artifact.artifact_id);
    assert_eq!(proposals.proposals[0].payload.confidence_milli, Some(810));
    assert_eq!(
        proposals.proposals[0].readiness.status,
        nako_core::GeneratedArtifactReadinessStatus::Ready
    );
    let proposal_text = serde_json::to_string(&proposals).unwrap();
    assert!(!proposal_text.contains("private generated overview"));
    assert!(!proposal_text.contains("private chain of thought"));
    assert!(!proposal_text.contains("local:///Movies/private"));
    assert!(!proposal_text.contains("nako_at_should_not_echo"));

    let replay = addon_generated_artifact(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json::<AddonGeneratedArtifactResponse>(replay).await;
    assert!(replay.idempotent_replay);
    assert_eq!(replay.artifact.artifact_id, body.artifact.artifact_id);

    let mut conflicting_request = request.clone();
    conflicting_request.payload = serde_json::json!({
        "overview": "conflicting private generated overview",
        "confidence_milli": 811
    });
    let conflict =
        addon_generated_artifact(&router, Some(&issued.raw_token), &conflicting_request).await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conflict_body = to_bytes(conflict.into_body(), usize::MAX).await.unwrap();
    let conflict_text = String::from_utf8_lossy(&conflict_body);
    let conflict_error: ErrorResponse = serde_json::from_slice(&conflict_body).unwrap();
    assert_eq!(conflict_error.code, "conflict");
    assert!(!conflict_text.contains("conflicting private generated overview"));
    assert!(!conflict_text.contains("local:///Movies/private"));
    assert!(!conflict_text.contains("nako_at_should_not_echo"));

    let mut second_request = request.clone();
    second_request.idempotency_key = format!("addon-generated-second:{}", source.item_id);
    second_request.payload = serde_json::json!({
        "overview": "second private generated overview",
        "confidence_milli": 812
    });
    let second = addon_generated_artifact(&router, Some(&issued.raw_token), &second_request).await;
    assert_eq!(second.status(), StatusCode::OK);
    let second = body_json::<AddonGeneratedArtifactResponse>(second).await;
    assert!(!second.idempotent_replay);
    assert_ne!(second.artifact.artifact_id, body.artifact.artifact_id);

    let summary_request = SubmitAddonGeneratedArtifactRequest {
        capability: AutomationCapability::Summary,
        kind: AutomationArtifactKind::Summary,
        library_id: Some(library_id),
        item_id: Some(source.item_id),
        source_id: Some(source.id),
        idempotency_key: format!("addon-generated-summary:{}", source.item_id),
        prompt: serde_json::json!({"source_locator":"local:///Movies/private/artifact-handoff.mkv"}),
        payload: serde_json::json!({
            "summary": "private generated short summary",
            "confidence_milli": 700
        }),
    };
    let summary =
        addon_generated_artifact(&router, Some(&issued.raw_token), &summary_request).await;
    assert_eq!(summary.status(), StatusCode::OK);
    let summary = body_json::<AddonGeneratedArtifactResponse>(summary).await;
    assert_eq!(summary.artifact.capability, AutomationCapability::Summary);
    let provider = store
        .get_automation_provider(body.artifact.provider_id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        provider
            .capabilities
            .contains(&AutomationCapability::MetadataCleanup)
    );
    assert!(
        provider
            .capabilities
            .contains(&AutomationCapability::Summary)
    );

    let item = store.get_media_item(source.item_id).await.unwrap().unwrap();
    assert_eq!(item.metadata.title, "artifact-handoff.mkv");
    assert!(item.metadata.overview.is_none());
    assert_eq!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        store
            .list_managed_import_artifacts(
                nako_core::ManagedImportArtifactListFilter::all(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn addon_acquisition_candidate_handoff_enters_dwi_without_managed_import_or_media_source() {
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
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
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
            label: Some("acquisition runtime".to_owned()),
        },
    )
    .await;
    let request = SubmitAddonAcquisitionCandidateRequest {
        target_library_id: library_id,
        source_key: "reference-downloader://movie-1?token=secret".to_owned(),
        source_uri: "https://download.example/private/movie-1.mkv?token=secret".to_owned(),
        display_name: Some("movie-1.mkv".to_owned()),
        intended_locator: Some("Movies/Movie 1/movie-1.mkv".to_owned()),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
        state: Some(AcquisitionIntakeCandidateState::Ready),
        diagnostics: serde_json::json!({
            "downloader": "reference",
            "token": "nako_at_should_not_echo"
        }),
    };

    let response = addon_acquisition_candidate(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let body = serde_json::from_str::<AddonAcquisitionCandidateResponse>(&text).unwrap();

    assert!(!body.idempotent_replay);
    assert_eq!(body.candidate.target_library_id, library_id);
    assert_eq!(body.candidate.state, AcquisitionIntakeCandidateState::Ready);
    assert_eq!(body.candidate.source_kind, "addon_proposed");
    assert_eq!(body.candidate.source_scheme.as_deref(), Some("https"));
    assert_eq!(body.candidate.source_ref_redacted, "https://<redacted>");
    assert!(body.candidate.source_key_fingerprint.starts_with("sha256:"));
    assert!(body.candidate.has_display_name);
    assert!(body.candidate.has_intended_locator);
    assert!(body.candidate.has_fingerprint);
    assert!(body.candidate.has_diagnostics);
    assert_eq!(body.candidate.managed_import_artifact_id, None);
    assert!(!body.candidate.writes_library);
    assert!(!body.candidate.creates_media_source);
    assert!(!body.candidate.creates_managed_import);
    assert!(!body.candidate.promotion_apply);
    assert!(!text.contains("token=secret"));
    assert!(!text.contains("private/movie-1"));
    assert!(!text.contains("movie-1.mkv"));
    assert!(!text.contains("Movies/Movie 1"));
    assert!(!text.contains("sha256-private-fingerprint"));
    assert!(!text.contains("nako_at_should_not_echo"));

    let candidates = request_json::<AdminAcquisitionIntakeCandidateListResponse>(
        &router,
        Method::GET,
        "/admin/v1/acquisition/intake/candidates?source_kind=addon_proposed",
    )
    .await;
    assert_eq!(candidates.candidates.len(), 1);
    assert_eq!(candidates.candidates[0].id, body.candidate.id);
    assert_eq!(candidates.candidates[0].managed_import_artifact_id, None);
    let candidates_text = serde_json::to_string(&candidates).unwrap();
    assert!(!candidates_text.contains("token=secret"));
    assert!(!candidates_text.contains("private/movie-1"));
    assert!(!candidates_text.contains("movie-1.mkv"));
    assert!(!candidates_text.contains("Movies/Movie 1"));
    assert!(!candidates_text.contains("sha256-private-fingerprint"));
    assert!(!candidates_text.contains("nako_at_should_not_echo"));

    let replay = addon_acquisition_candidate(&router, Some(&issued.raw_token), &request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json::<AddonAcquisitionCandidateResponse>(replay).await;
    assert!(replay.idempotent_replay);
    assert_eq!(replay.candidate.id, body.candidate.id);
}

#[tokio::test]
async fn addon_handoff_rejects_missing_scopes_and_stale_targets_without_records() {
    let (_temp, router, source, store) =
        router_with_media_source("stale-handoff.mkv", b"media").await;
    let library_id = source.library_id;
    let mut narrow_manifest = addon_manifest();
    narrow_manifest.resources[0].required_scopes = vec![AddonScope::ItemMetadataRead];
    narrow_manifest.scopes = vec![AddonScope::ItemMetadataRead];
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: narrow_manifest.clone(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::ItemMetadataRead],
            status: Some(AddonStatus::Disabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("handoff runtime".to_owned()),
        },
    )
    .await;
    let artifact_request = SubmitAddonGeneratedArtifactRequest {
        capability: AutomationCapability::MetadataCleanup,
        kind: AutomationArtifactKind::MetadataSuggestion,
        library_id: Some(library_id),
        item_id: Some(source.item_id),
        source_id: Some(source.id),
        idempotency_key: "missing-scope".to_owned(),
        prompt: serde_json::json!({"token":"nako_at_should_not_echo"}),
        payload: serde_json::json!({"overview":"should not record"}),
    };
    let disabled =
        addon_generated_artifact(&router, Some(&issued.raw_token), &artifact_request).await;
    assert_eq!(disabled.status(), StatusCode::FORBIDDEN);
    assert_eq!(body_json::<ErrorResponse>(disabled).await.code, "forbidden");

    request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::PATCH,
        &format!("/admin/v1/addons/{addon_id}/status"),
        &UpdateAddonStatusRequest {
            status: AddonStatus::Enabled,
        },
    )
    .await;
    let missing_suggest =
        addon_generated_artifact(&router, Some(&issued.raw_token), &artifact_request).await;
    assert_eq!(missing_suggest.status(), StatusCode::FORBIDDEN);

    let registration_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: Some(addon_id),
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(registration_response.status(), StatusCode::OK);
    let registered = body_json::<AdminAddonRegistrationResponse>(registration_response).await;
    assert_eq!(registered.addon.summary.id, addon_id);
    let issued = request_body_json::<AddonTokenIssuedResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/tokens"),
        &IssueAddonTokenRequest {
            label: Some("handoff runtime enabled".to_owned()),
        },
    )
    .await;

    let stale_source = SubmitAddonGeneratedArtifactRequest {
        source_id: Some(MediaSourceId::new()),
        idempotency_key: "stale-source".to_owned(),
        ..artifact_request.clone()
    };
    let stale = addon_generated_artifact(&router, Some(&issued.raw_token), &stale_source).await;
    assert_eq!(stale.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json::<ErrorResponse>(stale).await.code,
        "invalid_input"
    );

    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "other-stale-handoff.mkv".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let other_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: other_item.id,
        locator: "local:///other-stale-handoff.mkv".to_owned(),
        file_name: "other-stale-handoff.mkv".to_owned(),
        size_bytes: Some(7),
        fingerprint: None,
    };
    store.upsert_media_item(&other_item).await.unwrap();
    store.upsert_media_source(&other_source).await.unwrap();
    let mismatched_source = SubmitAddonGeneratedArtifactRequest {
        source_id: Some(other_source.id),
        idempotency_key: "mismatched-source".to_owned(),
        ..artifact_request.clone()
    };
    let stale =
        addon_generated_artifact(&router, Some(&issued.raw_token), &mismatched_source).await;
    assert_eq!(stale.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json::<ErrorResponse>(stale).await.code,
        "invalid_input"
    );

    let outside_library_item = SubmitAddonGeneratedArtifactRequest {
        source_id: None,
        item_id: Some(other_item.id),
        idempotency_key: "outside-library-item".to_owned(),
        ..artifact_request.clone()
    };
    let outside =
        addon_generated_artifact(&router, Some(&issued.raw_token), &outside_library_item).await;
    assert_eq!(outside.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json::<ErrorResponse>(outside).await.code,
        "invalid_input"
    );

    let acquisition_request = SubmitAddonAcquisitionCandidateRequest {
        target_library_id: library_id,
        source_key: "missing-library".to_owned(),
        source_uri: "https://download.example/private/stale.mkv?token=secret".to_owned(),
        display_name: Some("stale.mkv".to_owned()),
        intended_locator: None,
        size_bytes: Some(1),
        fingerprint: Some("private-fingerprint".to_owned()),
        state: None,
        diagnostics: serde_json::json!({"token":"nako_at_should_not_echo"}),
    };
    let missing_library_candidate = SubmitAddonAcquisitionCandidateRequest {
        target_library_id: LibraryId::new(),
        ..acquisition_request
    };
    let missing_library =
        addon_acquisition_candidate(&router, Some(&issued.raw_token), &missing_library_candidate)
            .await;
    assert_eq!(missing_library.status(), StatusCode::BAD_REQUEST);

    assert!(
        store
            .list_automation_artifacts_for_item(source.item_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_acquisition_intake_candidates(
                nako_core::AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library_id),
                    state: None,
                    source_kind: Some(nako_core::AcquisitionIntakeSourceKind::AddonProposed),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn admin_addon_resource_call_diagnostic_classifies_safe_success_without_payload_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addon_base_url = format!("http://{}", listener.local_addr().unwrap());
    let addon_server = tokio::spawn(async move {
        axum::serve(listener, nako_reference_addon::build_router())
            .await
            .unwrap();
    });
    yield_now().await;

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = nako_reference_addon::reference_manifest(addon_base_url);
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
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
                "token": "nako_at_should_not_echo"
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
    assert!(!text.contains("nako_at_should_not_echo"));
    assert!(!text.contains("metadata_suggestion"));

    addon_server.abort();
}

#[tokio::test]
async fn admin_addon_resource_call_diagnostic_classifies_safe_failures() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let mut auth_gap_manifest =
        nako_reference_addon::reference_manifest("https://auth-gap.example.test/addon");
    auth_gap_manifest.id = "nako.auth-gap.metadata".to_owned();
    auth_gap_manifest.auth = AddonAuth::Bearer;
    let auth_gap = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: auth_gap_manifest,
            outbound_task_dispatch_secret_env: None,
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
            payload: serde_json::json!({"secret":"nako_at_should_not_echo"}),
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
                payload: serde_json::json!({"secret":"nako_at_should_not_echo"}),
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
    let mut retryable_manifest = nako_reference_addon::reference_manifest(retryable_base_url);
    retryable_manifest.id = "nako.retryable.metadata".to_owned();
    let retryable = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: retryable_manifest,
            outbound_task_dispatch_secret_env: None,
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
    let mut unsafe_manifest = nako_reference_addon::reference_manifest(unsafe_base_url);
    unsafe_manifest.id = "nako.unsafe.metadata".to_owned();
    let unsafe_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: unsafe_manifest,
            outbound_task_dispatch_secret_env: None,
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

    let protocol_base_url = failing_resource_addon_server(
        StatusCode::OK,
        r#"{"protocol_version":"0.1.0-alpha.0","addon_id":"nako.protocol-resource.metadata","resource":"metadata","request_id":"wrong-request","payload":{"secret":"nako_at_should_not_echo"},"artifacts":[]}"#,
    )
    .await;
    let mut protocol_manifest = nako_reference_addon::reference_manifest(protocol_base_url);
    protocol_manifest.id = "nako.protocol-resource.metadata".to_owned();
    let protocol_addon = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: protocol_manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let protocol_path = format!(
        "/admin/v1/addons/{}/diagnostics/resource-call",
        protocol_addon.addon.summary.id
    );
    let protocol_raw = response_body_json(
        &router,
        Method::POST,
        &protocol_path,
        &AdminAddonResourceCallDiagnosticRequest {
            resource: AddonResource::Metadata,
            payload: serde_json::json!({"title":"Hidden"}),
        },
    )
    .await;
    assert_eq!(protocol_raw.status(), StatusCode::OK);
    let protocol_bytes = to_bytes(protocol_raw.into_body(), usize::MAX)
        .await
        .unwrap();
    let protocol_text = String::from_utf8(protocol_bytes.to_vec()).unwrap();
    let protocol_response =
        serde_json::from_str::<AdminAddonResourceCallDiagnosticResponse>(&protocol_text).unwrap();
    assert_eq!(
        protocol_response.status,
        AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch
    );
    assert_eq!(
        protocol_response.safe_error_code.as_deref(),
        Some("protocol_mismatch")
    );
    assert_eq!(protocol_response.attempts, 1);
    assert!(!protocol_text.contains("0.1.0-alpha.0"));
    assert!(!protocol_text.contains("nako_at_should_not_echo"));
    assert!(!protocol_text.contains("Hidden"));
}

#[tokio::test]
async fn admin_addon_resource_search_diagnostic_summarizes_without_raw_payloads() {
    let (addon_base_url, captured) = resource_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: resource_search_manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::AcquisitionSearchRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/diagnostics/resource-search");
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonResourceSearchDiagnosticRequest {
            query: "  Demo Movie  ".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie".to_owned(),
            },
            limit: Some(999),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark],
            refresh: false,
            context: serde_json::json!({
                "source_locator": "local:///secret/movie.mkv",
                "token": "nako_at_should_not_echo"
            }),
        },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response =
        serde_json::from_str::<AdminAddonResourceSearchDiagnosticResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(
        response.status,
        AdminAddonResourceCallDiagnosticStatus::Succeeded
    );
    assert_eq!(response.limit, 50);
    assert_eq!(response.total, 1);
    assert_eq!(response.result_count, 1);
    assert_eq!(response.link_count, 1);
    assert_eq!(response.merged_link_count, 1);
    assert_eq!(response.http_status, Some(200));
    assert_eq!(response.safe_error_code, None);
    assert_eq!(response.provider_executions.len(), 1);
    assert_eq!(
        response.provider_executions[0].provider_id,
        "pansou_compatible"
    );
    assert!(response.provider_executions[0].has_safe_message);

    for forbidden in [
        "Demo Movie",
        "Secret Result Title",
        "https://pan.quark.cn",
        "secret-code",
        "private-note",
        "private content",
        "provider secret message",
        "local:///secret",
        "nako_at_should_not_echo",
    ] {
        assert!(
            !text.contains(forbidden),
            "resource-search diagnostic leaked forbidden term: {forbidden}"
        );
    }

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].resource, AddonResource::ResourceSearch);
    assert_eq!(
        captured[0].payload["schema"],
        ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA
    );
    assert_eq!(captured[0].payload["query"], "Demo Movie");
    assert_eq!(captured[0].payload["limit"], 50);
    assert_eq!(
        captured[0].payload["link_types"],
        serde_json::json!(["quark"])
    );
}

#[tokio::test]
async fn admin_addon_resource_search_diagnostic_requires_nonzero_limit() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: resource_search_manifest("https://resource-search.example.test".to_owned()),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::AcquisitionSearchRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let path = format!(
        "/admin/v1/addons/{}/diagnostics/resource-search",
        registered.addon.summary.id
    );
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonResourceSearchDiagnosticRequest {
            query: "Demo Movie".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie".to_owned(),
            },
            limit: Some(0),
            sources: Vec::new(),
            link_types: Vec::new(),
            refresh: false,
            context: serde_json::Value::Null,
        },
    )
    .await;

    assert_eq!(raw.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(raw).await;
    assert_eq!(error.code, "invalid_input");
}

#[tokio::test]
async fn admin_addon_resource_search_product_returns_safe_selection_cards() {
    let (addon_base_url, captured) = resource_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: resource_search_manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::AcquisitionSearchRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/resource-search");
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonResourceSearchRequest {
            query: "  Demo Movie  ".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie".to_owned(),
            },
            limit: Some(999),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark],
            refresh: false,
            context: serde_json::json!({
                "source_locator": "local:///secret/movie.mkv",
                "token": "nako_at_should_not_echo"
            }),
        },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonResourceSearchResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(
        response.status,
        AdminAddonResourceCallDiagnosticStatus::Succeeded
    );
    assert!(response.search_id.starts_with("rs_"));
    assert_eq!(response.limit, 50);
    assert_eq!(response.total, 1);
    assert_eq!(response.result_count, 1);
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].title, "Secret Result Title");
    assert_eq!(
        response.results[0].content.as_deref(),
        Some("private content")
    );
    assert_eq!(response.results[0].tags, vec!["private-tag"]);
    assert!(
        response.results[0]
            .result_ref_fingerprint
            .starts_with("sha256:")
    );
    assert_eq!(response.results[0].links.len(), 1);
    let link = &response.results[0].links[0];
    assert!(link.selection_id.starts_with("sel_"));
    assert_eq!(link.link_type, AddonResourceLinkType::Quark);
    assert_eq!(link.source, "pansou:movies");
    assert_eq!(link.source_ref_redacted, "https://<redacted>");
    assert!(link.has_password);
    assert!(link.has_note);
    assert_eq!(response.provider_executions.len(), 1);
    assert!(response.provider_executions[0].has_safe_message);

    for forbidden in [
        "https://pan.quark.cn",
        "raw-secret-link",
        "secret-code",
        "private-note",
        "provider secret message",
        "local:///secret",
        "nako_at_should_not_echo",
        "https://image.example/private.jpg",
    ] {
        assert!(
            !text.contains(forbidden),
            "resource-search product response leaked forbidden term: {forbidden}"
        );
    }

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].resource, AddonResource::ResourceSearch);
    assert_eq!(captured[0].payload["query"], "Demo Movie");
    assert_eq!(captured[0].payload["limit"], 50);
    assert_eq!(
        captured[0].payload["link_types"],
        serde_json::json!(["quark"])
    );
}

#[tokio::test]
async fn admin_addon_resource_search_product_selection_records_intake_candidate_by_opaque_id() {
    let (addon_base_url, _captured) = resource_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: resource_search_manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::AcquisitionSearchRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonResourceSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/resource-search"),
        &AdminAddonResourceSearchRequest {
            query: "Demo Movie".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie".to_owned(),
            },
            limit: Some(20),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark],
            refresh: false,
            context: serde_json::Value::Null,
        },
    )
    .await;
    let selection_id = search.results[0].links[0].selection_id.clone();
    let selection_path = format!(
        "/admin/v1/addons/{addon_id}/resource-search/{}/selections/{selection_id}/intake-candidate",
        search.search_id
    );

    let first_raw = response_body_json(
        &router,
        Method::POST,
        &selection_path,
        &AdminAddonResourceSearchSelectionRequest {
            target_library_id: library_id,
        },
    )
    .await;
    assert_eq!(first_raw.status(), StatusCode::OK);
    let first_text = response_text(first_raw).await;
    let first =
        serde_json::from_str::<AdminAddonResourceSearchSelectionResponse>(&first_text).unwrap();
    let replayed = request_body_json::<AdminAddonResourceSearchSelectionResponse, _>(
        &router,
        Method::POST,
        &selection_path,
        &AdminAddonResourceSearchSelectionRequest {
            target_library_id: library_id,
        },
    )
    .await;

    assert_eq!(first.addon_id, addon_id);
    assert_eq!(first.search_id, search.search_id);
    assert_eq!(first.selection_id, selection_id);
    assert!(!first.idempotent_replay);
    assert!(replayed.idempotent_replay);
    assert_eq!(first.candidate.id, replayed.candidate.id);
    assert_eq!(first.candidate.target_library_id, library_id);
    assert_eq!(first.candidate.source_kind, "resource_search_selection");
    assert_eq!(first.candidate.source_scheme.as_deref(), Some("https"));
    assert_eq!(first.candidate.source_ref_redacted, "https://<redacted>");
    assert!(
        first
            .candidate
            .source_key_fingerprint
            .starts_with("sha256:")
    );
    assert!(first.candidate.has_display_name);
    assert!(first.candidate.has_diagnostics);
    assert!(!first.candidate.writes_library);
    assert!(!first.candidate.creates_media_source);
    assert!(!first.candidate.creates_managed_import);
    for forbidden in [
        "https://pan.quark.cn",
        "raw-secret-link",
        "secret-code",
        "private-note",
        "Secret Result Title",
        "private content",
        "private-tag",
        "https://image.example/private.jpg",
    ] {
        assert!(
            !first_text.contains(forbidden),
            "resource-search selection response leaked forbidden term: {forbidden}"
        );
    }

    let candidates = request_json::<AdminAcquisitionIntakeCandidateListResponse>(
        &router,
        Method::GET,
        &format!(
            "/admin/v1/acquisition/intake/candidates?library_id={library_id}&source_kind=resource_search_selection"
        ),
    )
    .await;
    assert_eq!(candidates.candidates.len(), 1);
    assert_eq!(candidates.candidates[0].id, first.candidate.id);
}

#[tokio::test]
async fn admin_addon_subtitle_search_product_returns_safe_candidate_cards() {
    let (addon_base_url, captured) = subtitle_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let path = format!("/admin/v1/addons/{addon_id}/subtitle-search");
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonSubtitleSearchRequest {
            query: "  Demo Movie  ".to_owned(),
            languages: vec![
                " EN ".to_owned(),
                "en".to_owned(),
                "zh-Hant".to_owned(),
                String::new(),
            ],
            limit: Some(999),
        },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let bytes = to_bytes(raw.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let response = serde_json::from_str::<AdminAddonSubtitleSearchResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(
        response.status,
        AdminAddonResourceCallDiagnosticStatus::Succeeded
    );
    assert!(response.search_id.starts_with("sub_"));
    assert_eq!(response.limit, 50);
    assert_eq!(response.total, 3);
    assert_eq!(response.result_count, 3);
    assert_eq!(response.subtitles.len(), 3);
    assert_eq!(response.subtitles[0].title, "Secret English Subtitle");
    assert_eq!(response.subtitles[0].language, "en");
    assert_eq!(response.subtitles[0].format, AddonSubtitleFormat::Vtt);
    assert_eq!(
        response.subtitles[0].delivery_kind,
        AdminAddonSubtitleDeliveryKind::Inline
    );
    assert!(response.subtitles[0].selection_id.starts_with("sel_"));
    assert!(
        response.subtitles[0]
            .candidate_ref_fingerprint
            .starts_with("sha256:")
    );
    assert_eq!(
        response.subtitles[1].delivery_kind,
        AdminAddonSubtitleDeliveryKind::DownloadUrl
    );
    assert_eq!(
        response.subtitles[2].delivery_kind,
        AdminAddonSubtitleDeliveryKind::ArtifactRef
    );
    assert_eq!(response.provider_executions.len(), 1);
    assert!(response.provider_executions[0].has_safe_message);

    for forbidden in [
        "WEBVTT",
        "secret subtitle text",
        "https://subtitle.example",
        "secret-token",
        "artifact-secret-id",
        "provider safe message",
        "local:///secret",
        "nako_at_should_not_echo",
        "\"url\"",
        "\"text\"",
        "\"artifact_id\"",
    ] {
        assert!(
            !text.contains(forbidden),
            "subtitle search response leaked forbidden term: {forbidden}"
        );
    }

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].resource, AddonResource::Subtitle);
    assert_eq!(captured[0].payload["schema"], ADDON_SUBTITLE_REQUEST_SCHEMA);
    assert_eq!(captured[0].payload["query"], "Demo Movie");
    assert_eq!(captured[0].payload["limit"], 50);
    assert_eq!(
        captured[0].payload["languages"],
        serde_json::json!(["en", "zh-hant"])
    );
    assert!(captured[0].payload.get("context").is_none());
}

#[tokio::test]
async fn admin_addon_subtitle_search_requires_nonzero_limit() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest("https://subtitle.example.test".to_owned()),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let path = format!(
        "/admin/v1/addons/{}/subtitle-search",
        registered.addon.summary.id
    );
    let raw = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages: Vec::new(),
            limit: Some(0),
        },
    )
    .await;

    assert_eq!(raw.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(raw).await;
    assert_eq!(error.code, "invalid_input");
}

#[tokio::test]
async fn admin_addon_subtitle_search_selection_records_opaque_reference_without_import() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonSubtitleSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/subtitle-search"),
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages: vec!["zh-Hant".to_owned()],
            limit: Some(10),
        },
    )
    .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let selection_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/selected-reference",
        search.search_id
    );

    let raw_payload_rejected = response_body_json(
        &router,
        Method::POST,
        &selection_path,
        &serde_json::json!({
            "url": "https://subtitle.example/download?token=secret-token",
            "text": "WEBVTT\n\nsecret subtitle text"
        }),
    )
    .await;
    assert_eq!(
        raw_payload_rejected.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &selection_path,
        &AdminAddonSubtitleSelectionRequest::default(),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonSubtitleSelectionResponse>(&text).unwrap();

    assert_eq!(response.selected_ref.addon_id, addon_id);
    assert_eq!(response.selected_ref.search_id, search.search_id);
    assert_eq!(response.selected_ref.selection_id, selection_id);
    assert_eq!(
        response.selected_ref.delivery_kind,
        AdminAddonSubtitleDeliveryKind::DownloadUrl
    );
    assert_eq!(response.candidate.selection_id, selection_id);
    assert_eq!(response.candidate.title, "Remote Chinese Subtitle");
    assert_eq!(response.candidate.language, "zh-hant");
    assert_eq!(
        response.candidate.candidate_ref_fingerprint,
        response.selected_ref.candidate_ref_fingerprint
    );

    for forbidden in [
        "WEBVTT",
        "secret subtitle text",
        "https://subtitle.example",
        "secret-token",
        "artifact-secret-id",
        "\"url\"",
        "\"text\"",
        "\"artifact_id\"",
    ] {
        assert!(
            !text.contains(forbidden),
            "subtitle selection response leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn admin_addon_subtitle_import_plan_previews_safe_sidecar_without_writes() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonSubtitleSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/subtitle-search"),
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages: vec!["zh-Hant".to_owned()],
            limit: Some(10),
        },
    )
    .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );

    let raw_payload_rejected = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &serde_json::json!({
            "media_item_id": source.item_id,
            "media_source_id": source.id,
            "language": "zh-Hant",
            "format": "srt",
            "sidecar_role": "default",
            "conflict_policy": "create_missing",
            "backup_policy": "none",
            "target_path": "C:\\secret\\demo.zh-hant.srt",
            "url": "https://subtitle.example/download?token=secret-token",
            "text": "WEBVTT\n\nsecret subtitle text"
        }),
    )
    .await;
    assert_eq!(
        raw_payload_rejected.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &AdminAddonSubtitleImportPlanRequest {
            media_item_id: source.item_id,
            media_source_id: source.id,
            language: "zh-Hant".to_owned(),
            format: AddonSubtitleFormat::Srt,
            sidecar_role: AdminSubtitleSidecarRole::Default,
            conflict_policy: AdminSubtitleImportConflictPolicy::CreateMissing,
            backup_policy: AdminSubtitleImportBackupPolicy::None,
        },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonSubtitleImportPlanResponse>(&text).unwrap();

    assert_eq!(response.selected_ref.addon_id, addon_id);
    assert_eq!(response.selected_ref.search_id, search.search_id);
    assert_eq!(response.selected_ref.selection_id, selection_id);
    assert_eq!(
        response.selected_ref.delivery_kind,
        AdminAddonSubtitleDeliveryKind::DownloadUrl
    );
    assert_eq!(response.plan.status, AdminSubtitleImportPlanStatus::Ready);
    assert_eq!(
        response.plan.reasons,
        vec![
            AdminSubtitleImportPlanReason::MediaSourceMatchesItem,
            AdminSubtitleImportPlanReason::Ready,
        ]
    );
    assert!(response.plan.idempotency_key.starts_with("sip_"));
    assert_eq!(response.plan.target.library_id, source.library_id);
    assert_eq!(response.plan.target.media_item_id, source.item_id);
    assert_eq!(response.plan.target.media_source_id, source.id);
    assert_eq!(response.plan.target.media_file_name, "demo.mkv");
    assert!(
        response
            .plan
            .target
            .source_ref_fingerprint
            .starts_with("sha256:")
    );
    assert_eq!(response.plan.sidecar.file_name, "demo.zh-hant.srt");
    assert_eq!(response.plan.sidecar.language, "zh-hant");
    assert_eq!(response.plan.sidecar.format, AddonSubtitleFormat::Srt);
    assert_eq!(
        response.plan.sidecar.role,
        AdminSubtitleSidecarRole::Default
    );
    assert_eq!(
        response.plan.conflict_policy,
        AdminSubtitleImportConflictPolicy::CreateMissing
    );
    assert_eq!(
        response.plan.backup_policy,
        AdminSubtitleImportBackupPolicy::None
    );
    assert!(response.plan.preview_only);
    assert!(!response.plan.writes_library);

    for forbidden in [
        "WEBVTT",
        "secret subtitle text",
        "https://subtitle.example",
        "secret-token",
        "artifact-secret-id",
        "local:///",
        "C:\\",
        "target_path",
        "\"url\"",
        "\"text\"",
        "\"artifact_id\"",
        "backup_uri",
    ] {
        assert!(
            !text.contains(forbidden),
            "subtitle import plan response leaked forbidden term: {forbidden}"
        );
    }

    assert!(!temp.path().join("demo.zh-hant.srt").exists());
    let sources = store
        .list_media_sources(source.library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 1);
}

#[tokio::test]
async fn admin_addon_subtitle_import_plan_rejects_media_item_source_mismatch() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Other".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&other_item).await.unwrap();
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonSubtitleSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/subtitle-search"),
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages: vec!["zh-Hant".to_owned()],
            limit: Some(10),
        },
    )
    .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &AdminAddonSubtitleImportPlanRequest {
            media_item_id: other_item.id,
            media_source_id: source.id,
            language: "zh-Hant".to_owned(),
            format: AddonSubtitleFormat::Srt,
            sidecar_role: AdminSubtitleSidecarRole::Default,
            conflict_policy: AdminSubtitleImportConflictPolicy::CreateMissing,
            backup_policy: AdminSubtitleImportBackupPolicy::None,
        },
    )
    .await;

    assert_eq!(raw.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(raw).await;
    assert_eq!(error.code, "invalid_input");
}

#[tokio::test]
async fn admin_addon_subtitle_import_plan_blocks_candidate_language_or_format_mismatch() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (_temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: subtitle_provider::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![AddonScope::SubtitleRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonSubtitleSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/subtitle-search"),
        &AdminAddonSubtitleSearchRequest {
            query: "Demo Movie".to_owned(),
            languages: vec!["zh-Hant".to_owned()],
            limit: Some(10),
        },
    )
    .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );

    let response = request_body_json::<AdminAddonSubtitleImportPlanResponse, _>(
        &router,
        Method::POST,
        &plan_path,
        &AdminAddonSubtitleImportPlanRequest {
            media_item_id: source.item_id,
            media_source_id: source.id,
            language: "en".to_owned(),
            format: AddonSubtitleFormat::Vtt,
            sidecar_role: AdminSubtitleSidecarRole::Forced,
            conflict_policy: AdminSubtitleImportConflictPolicy::ReplaceExisting,
            backup_policy: AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest,
        },
    )
    .await;

    assert_eq!(response.plan.status, AdminSubtitleImportPlanStatus::Blocked);
    assert!(
        response
            .plan
            .reasons
            .contains(&AdminSubtitleImportPlanReason::CandidateLanguageMismatch)
    );
    assert!(
        response
            .plan
            .reasons
            .contains(&AdminSubtitleImportPlanReason::CandidateFormatMismatch)
    );
    assert_eq!(response.plan.sidecar.file_name, "demo.en.forced.vtt");
    assert_eq!(
        response.plan.conflict_policy,
        AdminSubtitleImportConflictPolicy::ReplaceExisting
    );
    assert_eq!(
        response.plan.backup_policy,
        AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest
    );
    assert!(
        !response
            .plan
            .reasons
            .contains(&AdminSubtitleImportPlanReason::Ready)
    );
    assert!(response.plan.preview_only);
    assert!(!response.plan.writes_library);
}

#[tokio::test]
async fn admin_addon_subtitle_import_apply_downloads_and_writes_safe_sidecar() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let (addon_id, search) =
        register_subtitle_provider_and_search(&router, addon_base_url, vec!["zh-Hant".to_owned()])
            .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );
    let plan = request_body_json::<AdminAddonSubtitleImportPlanResponse, _>(
        &router,
        Method::POST,
        &plan_path,
        &subtitle_import_plan_request(
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    let apply_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-apply",
        search.search_id
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            plan.plan.idempotency_key.clone(),
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonSubtitleImportApplyResponse>(&text).unwrap();

    assert_eq!(response.plan.idempotency_key, plan.plan.idempotency_key);
    assert_eq!(response.plan.status, AdminSubtitleImportPlanStatus::Ready);
    assert_eq!(
        response.apply.status,
        AdminSubtitleImportApplyStatus::Applied
    );
    assert_eq!(response.apply.idempotency_key, plan.plan.idempotency_key);
    assert_eq!(response.apply.write_mode, "create_missing");
    assert_eq!(response.apply.sidecar.file_name, "demo.zh-hant.srt");
    assert_eq!(response.apply.sidecar.language, "zh-hant");
    assert_eq!(response.apply.refreshed_fact.media_source_id, source.id);
    assert_eq!(response.apply.refreshed_fact.stream_index, 2);
    assert_eq!(response.apply.refreshed_fact.origin, "sidecar");
    assert_eq!(response.apply.refreshed_fact.language, "zh-hant");
    assert_eq!(
        response.apply.refreshed_fact.format,
        AddonSubtitleFormat::Srt
    );
    assert_eq!(
        response.apply.refreshed_fact.role,
        AdminSubtitleSidecarRole::Default
    );
    assert_eq!(response.apply.byte_len, 60);
    assert!(!response.apply.preview_only);
    assert!(response.apply.writes_library);
    assert!(!response.apply.target_existed);
    assert!(!response.apply.backup_created);
    assert!(
        response
            .apply
            .content_ref_fingerprint
            .starts_with("sha256:")
    );

    let sidecar = temp.path().join("demo.zh-hant.srt");
    assert_eq!(
        fs::read_to_string(&sidecar).unwrap(),
        "1\n00:00:01,000 --> 00:00:02,000\nremote secret subtitle text\n"
    );
    let sources = store
        .list_media_sources(source.library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 1);
    let probe = store.get_media_probe(source.id).await.unwrap().unwrap();
    assert_eq!(probe.streams.len(), 3);
    let subtitle = probe
        .streams
        .iter()
        .find(|stream| stream.kind == MediaStreamKind::Subtitle)
        .unwrap();
    assert_eq!(subtitle.index, 2);
    assert_eq!(subtitle.codec.as_deref(), Some("srt"));
    assert_eq!(subtitle.language.as_deref(), Some("zh-hant"));
    assert_eq!(
        subtitle.technical.origin.as_ref(),
        Some(&MediaStreamOrigin::Sidecar)
    );
    assert!(subtitle.technical.disposition.default);
    assert!(!subtitle.technical.disposition.forced);

    for forbidden in [
        "remote secret subtitle text",
        "secret-token",
        "127.0.0.1",
        "WEBVTT",
        "artifact-secret-id",
        "local:///",
        "C:\\",
        "\"url\"",
        "\"text\"",
        "\"artifact_id\"",
        "backup_uri",
    ] {
        assert!(
            !text.contains(forbidden),
            "subtitle import apply response leaked forbidden term: {forbidden}"
        );
    }

    let second = request_body_json::<AdminAddonSubtitleImportApplyResponse, _>(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            plan.plan.idempotency_key,
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    assert_eq!(
        second.apply.status,
        AdminSubtitleImportApplyStatus::AlreadyApplied
    );
    assert_eq!(second.apply.write_mode, "unchanged");
    assert!(second.apply.target_existed);
    assert!(!second.apply.writes_library);
    let probe = store.get_media_probe(source.id).await.unwrap().unwrap();
    assert_eq!(
        probe
            .streams
            .iter()
            .filter(|stream| stream.kind == MediaStreamKind::Subtitle
                && stream.technical.origin.as_ref() == Some(&MediaStreamOrigin::Sidecar))
            .count(),
        1
    );
}

#[tokio::test]
async fn admin_addon_subtitle_import_apply_writes_inline_vtt() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let (addon_id, search) =
        register_subtitle_provider_and_search(&router, addon_base_url, vec!["en".to_owned()]).await;
    let selection_id = search.subtitles[0].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );
    let plan = request_body_json::<AdminAddonSubtitleImportPlanResponse, _>(
        &router,
        Method::POST,
        &plan_path,
        &subtitle_import_plan_request(
            &source,
            "en",
            AddonSubtitleFormat::Vtt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    let apply_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-apply",
        search.search_id
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            plan.plan.idempotency_key,
            &source,
            "en",
            AddonSubtitleFormat::Vtt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonSubtitleImportApplyResponse>(&text).unwrap();

    assert_eq!(
        response.apply.status,
        AdminSubtitleImportApplyStatus::Applied
    );
    assert_eq!(response.apply.sidecar.file_name, "demo.en.vtt");
    assert_eq!(response.apply.refreshed_fact.stream_index, 0);
    assert_eq!(response.apply.refreshed_fact.origin, "sidecar");
    assert_eq!(
        fs::read_to_string(temp.path().join("demo.en.vtt")).unwrap(),
        "WEBVTT\n\nsecret subtitle text"
    );
    let probe = store.get_media_probe(source.id).await.unwrap().unwrap();
    let subtitle = &probe.streams[0];
    assert_eq!(subtitle.kind, MediaStreamKind::Subtitle);
    assert_eq!(subtitle.codec.as_deref(), Some("vtt"));
    assert_eq!(subtitle.language.as_deref(), Some("en"));
    assert_eq!(
        subtitle.technical.origin.as_ref(),
        Some(&MediaStreamOrigin::Sidecar)
    );
    assert!(subtitle.technical.disposition.default);
    assert!(!text.contains("WEBVTT"));
    assert!(!text.contains("secret subtitle text"));
}

#[tokio::test]
async fn admin_addon_subtitle_import_apply_rejects_stale_plan_key() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let (addon_id, search) =
        register_subtitle_provider_and_search(&router, addon_base_url, vec!["zh-Hant".to_owned()])
            .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let apply_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-apply",
        search.search_id
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            "sip_stale".to_owned(),
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;

    assert_eq!(raw.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(raw).await;
    assert_eq!(error.code, "invalid_input");
    assert!(!temp.path().join("demo.zh-hant.srt").exists());
}

#[tokio::test]
async fn admin_addon_subtitle_import_apply_preserves_existing_sidecar_policy() {
    let (addon_base_url, _captured) = subtitle_search_addon_server().await;
    let (temp, router, source, _store) = router_with_media_source("demo.mkv", b"media").await;
    let sidecar = temp.path().join("demo.zh-hant.srt");
    fs::write(&sidecar, "1\n00:00:01,000 --> 00:00:02,000\nold subtitle\n").unwrap();
    let (addon_id, search) =
        register_subtitle_provider_and_search(&router, addon_base_url, vec!["zh-Hant".to_owned()])
            .await;
    let selection_id = search.subtitles[1].selection_id.clone();
    let plan_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-plan",
        search.search_id
    );
    let create_plan = request_body_json::<AdminAddonSubtitleImportPlanResponse, _>(
        &router,
        Method::POST,
        &plan_path,
        &subtitle_import_plan_request(
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    let apply_path = format!(
        "/admin/v1/addons/{addon_id}/subtitle-search/{}/selections/{selection_id}/import-apply",
        search.search_id
    );
    let create_raw = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            create_plan.plan.idempotency_key,
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::CreateMissing,
            AdminSubtitleImportBackupPolicy::None,
        ),
    )
    .await;
    assert_eq!(create_raw.status(), StatusCode::CONFLICT);
    assert_eq!(
        fs::read_to_string(&sidecar).unwrap(),
        "1\n00:00:01,000 --> 00:00:02,000\nold subtitle\n"
    );

    let replace_plan = request_body_json::<AdminAddonSubtitleImportPlanResponse, _>(
        &router,
        Method::POST,
        &plan_path,
        &subtitle_import_plan_request(
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::ReplaceExisting,
            AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest,
        ),
    )
    .await;
    let raw = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &subtitle_import_apply_request(
            replace_plan.plan.idempotency_key,
            &source,
            "zh-Hant",
            AddonSubtitleFormat::Srt,
            AdminSubtitleSidecarRole::Default,
            AdminSubtitleImportConflictPolicy::ReplaceExisting,
            AdminSubtitleImportBackupPolicy::ExistingFileKeepLatest,
        ),
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonSubtitleImportApplyResponse>(&text).unwrap();

    assert_eq!(
        response.apply.status,
        AdminSubtitleImportApplyStatus::Applied
    );
    assert_eq!(response.apply.write_mode, "atomic_replace");
    assert!(response.apply.target_existed);
    assert!(response.apply.backup_created);
    assert!(response.apply.writes_library);
    assert_eq!(
        fs::read_to_string(&sidecar).unwrap(),
        "1\n00:00:01,000 --> 00:00:02,000\nremote secret subtitle text\n"
    );
    let backup_count = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|file_name| file_name.starts_with("demo.zh-hant.srt.nako-backup-"))
        .count();
    assert_eq!(backup_count, 1);
    assert!(!text.contains("backup_uri"));
    assert!(!text.contains("local:///"));
    assert!(!text.contains("old subtitle"));
    assert!(!text.contains("remote secret subtitle text"));
}

#[tokio::test]
async fn admin_addon_resource_link_check_product_uses_opaque_selection_without_raw_browser_payload()
{
    let (addon_base_url, captured) = resource_search_addon_server().await;
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: resource_search::manifest(addon_base_url),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::AcquisitionSearchRead,
                AddonScope::AcquisitionLinkCheckRead,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    let addon_id = registered.addon.summary.id;
    let search = request_body_json::<AdminAddonResourceSearchResponse, _>(
        &router,
        Method::POST,
        &format!("/admin/v1/addons/{addon_id}/resource-search"),
        &AdminAddonResourceSearchRequest {
            query: "Demo Movie".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie".to_owned(),
            },
            limit: Some(20),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark],
            refresh: false,
            context: serde_json::json!({
                "source_locator": "local:///secret/movie.mkv",
                "token": "nako_at_should_not_echo"
            }),
        },
    )
    .await;
    let selection_id = search.results[0].links[0].selection_id.clone();
    let link_check_path = format!(
        "/admin/v1/addons/{addon_id}/resource-search/{}/selections/{selection_id}/link-check",
        search.search_id
    );

    let raw_payload_rejected = response_body_json(
        &router,
        Method::POST,
        &link_check_path,
        &serde_json::json!({
            "refresh": true,
            "url": "https://pan.quark.cn/s/raw-secret-link",
            "password": "secret-code",
            "context": { "token": "nako_at_should_not_echo" }
        }),
    )
    .await;
    assert_eq!(
        raw_payload_rejected.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let raw = response_body_json(
        &router,
        Method::POST,
        &link_check_path,
        &AdminAddonResourceLinkCheckRequest { refresh: true },
    )
    .await;
    assert_eq!(raw.status(), StatusCode::OK);
    let text = response_text(raw).await;
    let response = serde_json::from_str::<AdminAddonResourceLinkCheckResponse>(&text).unwrap();

    assert_eq!(response.addon_id, addon_id);
    assert_eq!(response.search_id, search.search_id);
    assert_eq!(response.selection_id, selection_id);
    assert_eq!(
        response.status,
        AdminAddonResourceCallDiagnosticStatus::Succeeded
    );
    assert_eq!(response.link_type, AddonResourceLinkType::Quark);
    assert_eq!(
        response.check_status,
        Some(AddonResourceLinkCheckStatus::Reachable)
    );
    assert_eq!(response.checked_at_ms, Some(1_779_814_400_000));
    assert_eq!(response.requires_password, Some(false));
    assert_eq!(response.retryable, Some(false));
    assert!(response.has_safe_message);
    assert_eq!(
        response
            .safe_facts
            .get("http_status_class")
            .map(String::as_str),
        Some("2xx")
    );

    for forbidden in [
        "https://pan.quark.cn",
        "raw-secret-link",
        "secret-code",
        "private-note",
        "provider secret message",
        "local:///secret",
        "nako_at_should_not_echo",
        "https://image.example/private.jpg",
    ] {
        assert!(
            !text.contains(forbidden),
            "resource link-check product response leaked forbidden term: {forbidden}"
        );
    }

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0].resource, AddonResource::ResourceSearch);
    assert_eq!(captured[1].resource, AddonResource::ResourceLinkCheck);

    let link_check_request =
        serde_json::from_value::<AddonResourceLinkCheckRequest>(captured[1].payload.clone())
            .unwrap();
    assert_eq!(
        link_check_request.schema,
        ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA
    );
    assert!(link_check_request.refresh);
    assert_eq!(
        link_check_request.link.url,
        "https://pan.quark.cn/s/raw-secret-link"
    );
    assert_eq!(
        link_check_request.link.password.as_deref(),
        Some("secret-code")
    );
    assert_eq!(
        link_check_request.link.note.as_deref(),
        Some("private-note")
    );
    assert_eq!(
        link_check_request.context["schema"],
        "nako.resource_link_check.selection_context.v1"
    );
    assert_eq!(
        link_check_request.context["selection_id"],
        search.results[0].links[0].selection_id
    );
    assert_eq!(link_check_request.context["link_type"], "quark");
    assert_eq!(
        link_check_request.context["source_ref_redacted"],
        "https://<redacted>"
    );
    assert!(
        link_check_request.context["query_fingerprint"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(
        link_check_request.context["result_ref_fingerprint"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    let context_text = link_check_request.context.to_string();
    for forbidden in [
        "Demo Movie",
        "https://pan.quark.cn",
        "raw-secret-link",
        "secret-code",
        "private-note",
        "local:///secret",
        "nako_at_should_not_echo",
    ] {
        assert!(
            !context_text.contains(forbidden),
            "resource link-check addon context leaked forbidden term: {forbidden}"
        );
    }
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(issued.raw_token.starts_with("nako_at_"));
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
    assert!(rotation.raw_token.starts_with("nako_at_"));
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
            outbound_task_dispatch_secret_env: None,
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
        Some("nako_at_invalid"),
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
            outbound_task_dispatch_secret_env: None,
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
        .list_tags(nako_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(tags.len(), 2);
    assert!(
        tags.iter()
            .all(|tag| tag.source == MetadataSource::Addon(addon_id))
    );
    let genres = store
        .list_genres(nako_core::PageRequest::first_page())
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(!response_body.contains("nako-backup"));
    assert!(!response_body.contains(temp.path().to_string_lossy().as_ref()));

    let nfo = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    assert!(nfo.contains("<title>demo.mkv</title>"));
    assert!(nfo.contains(r#"<customrating system="local">five stars</customrating>"#));
    assert!(!nfo.contains("<title>Old Sidecar Title</title>"));

    let backups = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("nako-backup"))
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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
        .list_artwork_candidates_for_item(source.item_id, nako_core::PageRequest::first_page())
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
            .list_artwork_candidates_for_item(source.item_id, nako_core::PageRequest::first_page())
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
            outbound_task_dispatch_secret_env: None,
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
    let candidate_id: nako_core::ArtworkCandidateId =
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
            outbound_task_dispatch_secret_env: None,
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
    let candidate_id: nako_core::ArtworkCandidateId =
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
    assert!(temp.path().join("nako-cache").join("artwork").exists());
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

    let images = request_json::<nako_api::public_client::ImagesResponse>(
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
        request_json::<nako_api::public_client::ImagesResponse>(
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

    let images = request_json::<nako_api::public_client::ImagesResponse>(
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
    let (temp, app, source, store) =
        app_with_media_source_config("demo.mkv", b"media", |_| {}).await;
    let router = build_router(app.clone());
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
    let images: nako_api::public_client::ImagesResponse =
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
    let item_detail: nako_api::public_client::ItemDetailResponse =
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
    assert_selected_artwork_cache_control(image_response.headers());
    let original_etag = image_response
        .headers()
        .get(header::ETAG)
        .expect("selected image has an ETag")
        .to_str()
        .unwrap()
        .to_owned();
    assert!(original_etag.contains("nako-img-v1-"));
    let image_bytes = to_bytes(image_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(image_bytes.as_ref(), expected_bytes.as_slice());
    let original_weak_etag = format!("W/{original_etag}");

    let original_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_not_modified, &original_etag).await;

    let original_weak_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_weak_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_weak_not_modified, &original_etag).await;

    let original_list_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(
                    header::IF_NONE_MATCH,
                    format!("\"nako-img-v1-missing\", {original_weak_etag}"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_list_not_modified, &original_etag).await;

    let original_wildcard_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, "*")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_wildcard_not_modified, &original_etag).await;

    let original_miss = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, "\"nako-img-v1-missing\"")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(original_miss.status(), StatusCode::OK);
    assert_eq!(
        original_miss.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&original_etag).unwrap()
    );
    let original_miss_body = to_bytes(original_miss.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(original_miss_body.as_ref(), expected_bytes.as_slice());

    let original_malformed_weak_miss = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, format!("W/ {original_etag}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(original_malformed_weak_miss.status(), StatusCode::OK);
    assert_eq!(
        original_malformed_weak_miss
            .headers()
            .get(header::ETAG)
            .unwrap(),
        HeaderValue::from_str(&original_etag).unwrap()
    );
    let original_malformed_weak_miss_body =
        to_bytes(original_malformed_weak_miss.into_body(), usize::MAX)
            .await
            .unwrap();
    assert_eq!(
        original_malformed_weak_miss_body.as_ref(),
        expected_bytes.as_slice()
    );

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
    assert_selected_artwork_cache_control(head_response.headers());
    assert_eq!(
        head_response.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&original_etag).unwrap()
    );
    let head_body = to_bytes(head_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(head_body.is_empty());

    let head_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(head_not_modified, &original_etag).await;

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
    assert_selected_artwork_cache_control(variant_response.headers());
    let variant_etag = variant_response
        .headers()
        .get(header::ETAG)
        .expect("selected image variant has an ETag")
        .to_str()
        .unwrap()
        .to_owned();
    assert_ne!(variant_etag, original_etag);
    assert!(variant_etag.contains("nako-img-v1-"));
    let variant_weak_etag = format!("W/{variant_etag}");
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

    let variant_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, &variant_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(variant_not_modified, &variant_etag).await;

    let variant_weak_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, &variant_weak_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(variant_weak_not_modified, &variant_etag).await;

    let variant_wildcard_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, "*")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(variant_wildcard_not_modified, &variant_etag).await;

    let variant_original_etag_miss = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, &original_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(variant_original_etag_miss.status(), StatusCode::OK);
    assert_eq!(
        variant_original_etag_miss
            .headers()
            .get(header::ETAG)
            .unwrap(),
        HeaderValue::from_str(&variant_etag).unwrap()
    );
    let variant_original_etag_miss_body =
        to_bytes(variant_original_etag_miss.into_body(), usize::MAX)
            .await
            .unwrap();
    assert_eq!(
        variant_original_etag_miss_body.as_ref(),
        variant_bytes.as_ref()
    );

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
    assert_selected_artwork_cache_control(variant_head_response.headers());
    assert_eq!(
        variant_head_response.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&variant_etag).unwrap()
    );
    let variant_head_body = to_bytes(variant_head_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(variant_head_body.is_empty());

    let artifact_id = artifact.id.to_string();
    let artifact_shard = artifact_id.get(0..2).unwrap();
    let artifact_path = temp
        .path()
        .join("nako-cache")
        .join("artwork")
        .join(artifact_shard)
        .join(format!("{artifact_id}.png"));
    fs::remove_file(&artifact_path).unwrap();

    let original_preflight_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_preflight_not_modified, &original_etag).await;

    let original_preflight_weak_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_weak_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(original_preflight_weak_not_modified, &original_etag)
        .await;

    let variant_preflight_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, &variant_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(variant_preflight_not_modified, &variant_etag).await;

    let variant_preflight_wildcard_not_modified = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&variant_url)
                .header(header::IF_NONE_MATCH, "*")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_selected_artwork_not_modified(variant_preflight_wildcard_not_modified, &variant_etag)
        .await;

    let denied_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::None).await;
    let denied_router = public_client_router_with_principal(app, denied_principal);
    let denied = denied_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, &original_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let denied_status = denied.status();
    let denied_body = to_bytes(denied.into_body(), usize::MAX).await.unwrap();
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    let denied_text = String::from_utf8_lossy(&denied_body);
    assert!(!denied_text.contains(&remote_url));
    assert!(!denied_text.contains("token=secret"));
    assert!(!denied_text.contains(&raw_token));
    assert!(!denied_text.contains("source_uri"));
    assert!(!denied_text.contains("cache_uri"));
    assert!(!denied_text.contains("storage_uri"));
    assert!(!denied_text.contains("managed-artwork://"));
    assert!(!denied_text.contains(temp.path().to_string_lossy().as_ref()));
    assert!(!denied_text.contains("\"content_hash\""));

    let denied_head = denied_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&published.image.url)
                .header(header::IF_NONE_MATCH, "*")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied_head.status(), StatusCode::FORBIDDEN);

    let denied_invalid_variant = denied_router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("{}?width=0", published.image.url))
                .header(header::IF_NONE_MATCH, "*")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied_invalid_variant.status(), StatusCode::FORBIDDEN);

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
                .uri(format!("/images/{}", nako_core::SelectedArtworkId::new()))
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
async fn admin_managed_artwork_cleanup_requires_confirmation_and_removes_unselected_without_leaks()
{
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

    let unconfirmed_cleanup = router
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
    let unconfirmed_status = unconfirmed_cleanup.status();
    let unconfirmed_body = to_bytes(unconfirmed_cleanup.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        unconfirmed_status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&unconfirmed_body)
    );
    assert!(String::from_utf8_lossy(&unconfirmed_body).contains("confirm=true"));
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
            .is_some()
    );

    let cleanup_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/artwork/artifacts/cleanup?confirm=true")
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
        .join("nako-cache")
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
        .join("nako-cache")
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
        nako_api::admin::AdminManagedArtworkArtifactMissingRemediationRecommendation::RestoreOrRepublishSelectedArtwork
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
        .join("nako-cache")
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
            outbound_task_dispatch_secret_env: None,
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
            .list_artwork_candidates_for_item(source.item_id, nako_core::PageRequest::first_page())
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
async fn addon_side_effect_metadata_write_scalar_patch_reprojects_canonical_graph() {
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(item_genres.is_empty());
    assert!(item_tags.is_empty());
    let genres = store
        .list_genres(nako_core::PageRequest::first_page())
        .await
        .unwrap();
    let tags = store
        .list_tags(nako_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(genres, vec![tmdb_genre]);
    assert_eq!(tags, vec![tmdb_tag]);
    let hits = store
        .search(SearchQuery::from_facet_labels("Scalar-only", Vec::new(), 10, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(hits[0].item_id, source.item_id);
}

#[tokio::test]
async fn addon_side_effect_metadata_write_missing_only_preserves_existing_fields_and_fills_gaps() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut library = store
        .get_library(library_id)
        .await
        .unwrap()
        .expect("library exists");
    library.options.metadata_profile.refresh_mode = MetadataRefreshMode::MissingOnly;
    store.upsert_library(&library).await.unwrap();

    let mut item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item exists");
    item.metadata.title = "Existing Host Title".to_owned();
    item.metadata.overview = Some("Existing host overview.".to_owned());
    item.metadata.tagline = None;
    store.upsert_media_item(&item).await.unwrap();

    let addon = register_metadata_write_addon(&router, library_id).await;
    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-missing-only".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Addon Replacement Title",
            "overview": "Addon replacement overview.",
            "tagline": "Addon fills the missing tagline",
            "tags": ["addon-gap"]
        }),
    };

    let response = addon_side_effect(&router, Some(&addon.issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json::<AddonSideEffectResponse>(response).await;
    assert_eq!(
        body.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    let report = body
        .side_effect
        .apply_report
        .expect("metadata application report should be returned");
    assert_eq!(report["source"], format!("addon:{}", addon.addon_id));
    assert_eq!(report["refresh_mode"], "missing_only");
    assert_eq!(report["changed"], true);
    assert!(report.get("payload").is_none());

    let item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item was updated");
    assert_eq!(item.metadata.title, "Existing Host Title");
    assert_eq!(
        item.metadata.overview.as_deref(),
        Some("Existing host overview.")
    );
    assert_eq!(
        item.metadata.tagline.as_deref(),
        Some("Addon fills the missing tagline")
    );
    assert_eq!(item.metadata.tags, vec!["addon-gap"]);
}

#[tokio::test]
async fn addon_side_effect_metadata_write_respects_user_locked_fields() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item exists");
    item.metadata.title = "User Locked Title".to_owned();
    item.metadata.overview = None;
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: source.item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();

    let addon = register_metadata_write_addon(&router, library_id).await;
    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-user-lock".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Addon Title Should Not Win",
            "overview": "Addon can still fill an unlocked field."
        }),
    };

    let response = addon_side_effect(&router, Some(&addon.issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item was updated");
    assert_eq!(item.metadata.title, "User Locked Title");
    assert_eq!(
        item.metadata.overview.as_deref(),
        Some("Addon can still fill an unlocked field.")
    );
}

#[tokio::test]
async fn addon_side_effect_metadata_write_same_source_lock_allows_refresh() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let mut item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item exists");
    item.metadata.title = "Addon Previous Title".to_owned();
    store.upsert_media_item(&item).await.unwrap();

    let addon = register_metadata_write_addon(&router, library_id).await;
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: source.item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::Addon(addon.addon_id),
        })
        .await
        .unwrap();

    let request = SubmitAddonSideEffectRequest {
        permission: AddonPermission::MetadataWrite,
        library_id,
        target: AddonSideEffectTargetRequest {
            kind: AddonSideEffectTargetKind::MediaSource,
            id: source.id.to_string(),
        },
        idempotency_key: "metadata-same-source-lock".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Addon Refreshed Title"
        }),
    };

    let response = addon_side_effect(&router, Some(&addon.issued.raw_token), &request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let item = store
        .get_media_item(source.item_id)
        .await
        .unwrap()
        .expect("media item was updated");
    assert_eq!(item.metadata.title, "Addon Refreshed Title");
}

#[tokio::test]
async fn addon_side_effect_metadata_write_label_patch_reprojects_full_catalog_graph() {
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
            outbound_task_dispatch_secret_env: None,
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
    assert!(item_genres.is_empty());
    let genres = store
        .list_genres(nako_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(genres, vec![tmdb_genre]);
    let tags = store
        .list_tags(nako_core::PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "addon-tag");
    assert_eq!(tags[0].source, MetadataSource::Addon(addon_id));
    let hits = store
        .search(SearchQuery::from_facet_labels("addon-tag", Vec::new(), 10, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(hits[0].item_id, source.item_id);
}

#[tokio::test]
async fn addon_side_effect_metadata_write_graph_patch_hydrates_catalog_graph() {
    let (_temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    let library_id = source.library_id;
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
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
        idempotency_key: "metadata-graph-patch".to_owned(),
        provenance: serde_json::json!({"origin": "reference-addon"}),
        payload: serde_json::json!({
            "title": "Native AV Title",
            "credits": [
                {
                    "name": "Alice Actor",
                    "role": "actor",
                    "order": 1,
                    "external_ids": [{"provider": "javdb_actor", "value": "alice"}]
                },
                {"name": "Diana Director", "role": "director"}
            ],
            "studios": [
                {
                    "name": "Studio One",
                    "external_ids": [{"provider": "javbus_studio", "value": "studio-one"}]
                }
            ],
            "collections": [
                {
                    "name": "Series One",
                    "sort_order": 7,
                    "external_ids": [{"provider": "javbus_series", "value": "series-one"}]
                }
            ],
            "images": [
                {
                    "kind": "poster",
                    "uri": "https://images.example.test/poster.jpg",
                    "provider": "javdb",
                    "width": 800,
                    "height": 1200
                },
                {
                    "kind": "backdrop",
                    "uri": "https://images.example.test/backdrop.jpg",
                    "provider": "javbus"
                }
            ],
            "ratings": [{"source": "javdb", "value": "4.8"}],
            "external_ids": [{"provider": "javdb", "value": "abc-001"}]
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
    assert_eq!(item.metadata.title, "Native AV Title");
    assert_eq!(item.metadata.credits.len(), 2);
    assert_eq!(item.metadata.studios[0].name, "Studio One");
    assert_eq!(item.metadata.collections[0].name, "Series One");
    assert_eq!(item.metadata.images.len(), 2);
    assert_eq!(item.metadata.external_ids[0].value, "abc-001");

    let people = store.list_people(PageRequest::first_page()).await.unwrap();
    assert_eq!(people.len(), 2);
    assert!(people.iter().any(|person| person.name == "Alice Actor"));
    assert!(people.iter().any(|person| person.name == "Diana Director"));
    let credits = store.list_item_credits(source.item_id).await.unwrap();
    assert_eq!(credits.len(), 2);
    assert!(
        credits
            .iter()
            .any(|credit| credit.role == CreditRole::Actor)
    );
    assert!(
        credits
            .iter()
            .any(|credit| credit.role == CreditRole::Director)
    );

    let collections = store
        .list_collections(PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].name, "Series One");
    assert_eq!(
        store
            .list_item_collections(source.item_id)
            .await
            .unwrap()
            .len(),
        1
    );

    let studios = store.list_studios(PageRequest::first_page()).await.unwrap();
    assert_eq!(studios.len(), 1);
    assert_eq!(studios[0].name, "Studio One");
    assert_eq!(
        store.list_item_studios(source.item_id).await.unwrap().len(),
        1
    );

    let images = store.list_item_images(source.item_id).await.unwrap();
    assert_eq!(images.len(), 2);
    assert!(images.iter().any(|image| image.kind == ImageKind::Poster));
    assert!(images.iter().any(|image| image.kind == ImageKind::Backdrop));

    let hits = store
        .search(SearchQuery::from_facet_labels("Alice Actor", Vec::new(), 10, 0).unwrap())
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
        .upsert_library(&nako_core::Library {
            id: ungranted_library_id,
            name: "Other Movies".to_owned(),
            roots: vec!["local:///Other".to_owned()],
            options: nako_core::LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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
            outbound_task_dispatch_secret_env: None,
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

async fn addon_external_acquisition_materialization(
    router: &Router,
    raw_token: Option<&str>,
    request: &AddonExternalAcquisitionMaterializationRequest,
) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(ADDON_RUNTIME_EXTERNAL_ACQUISITION_MATERIALIZE_PATH)
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

#[derive(Clone, Debug)]
struct MetadataWriteAddonHarness {
    addon_id: AddonId,
    issued: AddonTokenIssuedResponse,
}

async fn register_metadata_write_addon(
    router: &Router,
    library_id: LibraryId,
) -> MetadataWriteAddonHarness {
    let registered = request_body_json::<AdminAddonRegistrationResponse, _>(
        router,
        Method::POST,
        "/admin/v1/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            outbound_task_dispatch_secret_env: None,
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
            label: Some("metadata runtime".to_owned()),
        },
    )
    .await;
    request_body_json::<AddonGrantsResponse, _>(
        router,
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

    MetadataWriteAddonHarness { addon_id, issued }
}

async fn addon_generated_artifact(
    router: &Router,
    raw_token: Option<&str>,
    request: &SubmitAddonGeneratedArtifactRequest,
) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/addon/v1/generated-artifacts")
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

async fn addon_acquisition_candidate(
    router: &Router,
    raw_token: Option<&str>,
    request: &SubmitAddonAcquisitionCandidateRequest,
) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/addon/v1/acquisition/intake/candidates")
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
