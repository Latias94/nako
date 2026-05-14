use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use taru_api::{API_VERSION, ErrorResponse, HealthResponse, JobResponse, SourceProbeResponse};
use taru_core::{JobId, LibraryId, MediaItemId, MediaSourceId, PageRequest, TaruError};
use tracing::{error, instrument, warn};

use crate::app::TaruApp;

pub fn build_router(app: TaruApp) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/libraries", get(list_libraries))
        .route("/libraries/{library_id}/scan", post(scan_library))
        .route("/libraries/{library_id}/nfo/import", post(import_nfo))
        .route("/libraries/{library_id}/nfo/export", post(export_nfo))
        .route("/libraries/{library_id}/sources", get(list_library_sources))
        .route("/items", get(list_items))
        .route(
            "/items/{item_id}/metadata/refresh",
            post(refresh_item_metadata),
        )
        .route("/sources/{source_id}/probe", get(get_source_probe))
        .route("/jobs/{job_id}", get(get_job))
        .with_state(app)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}

#[instrument(skip(app))]
async fn list_libraries(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_libraries(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn scan_library(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_library_scan(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn import_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn export_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn list_library_sources(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_library_sources(library_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
async fn list_items(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_items(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn refresh_item_metadata(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_metadata_refresh(item_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn get_source_probe(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    Ok(Json(app.get_source_probe(source_id).await?))
}

#[instrument(skip(app))]
async fn get_job(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<Json<JobResponse>> {
    Ok(Json(JobResponse::from_job(app.get_job(job_id).await?)))
}

type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct PageQuery {
    limit: Option<u32>,
    offset: Option<u64>,
}

impl TryFrom<PageQuery> for PageRequest {
    type Error = TaruError;

    fn try_from(value: PageQuery) -> Result<Self, Self::Error> {
        let limit = value.limit.unwrap_or(PageRequest::DEFAULT_LIMIT);

        if limit > PageRequest::MAX_LIMIT {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "limit must be less than or equal to {}",
                    PageRequest::MAX_LIMIT
                ),
            });
        }

        Ok(PageRequest {
            limit,
            offset: value.offset.unwrap_or_default(),
        }
        .clamped())
    }
}

#[derive(Debug)]
struct ApiError(TaruError);

impl From<TaruError> for ApiError {
    fn from(value: TaruError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_for_error(&self.0);
        let body = ErrorResponse {
            code: code_for_error(&self.0).to_owned(),
            message: public_message(&self.0),
        };

        if status.is_server_error() {
            error!(error = %self.0, status = %status, "request failed");
        } else {
            warn!(error = %self.0, status = %status, "request rejected");
        }

        (status, Json(body)).into_response()
    }
}

fn status_for_error(error: &TaruError) -> StatusCode {
    match error {
        TaruError::InvalidInput { .. } | TaruError::Unsupported(_) => StatusCode::BAD_REQUEST,
        TaruError::NotFound { .. } => StatusCode::NOT_FOUND,
        TaruError::Provider { .. } | TaruError::Storage { .. } => StatusCode::BAD_GATEWAY,
        TaruError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn code_for_error(error: &TaruError) -> &'static str {
    match error {
        TaruError::InvalidInput { .. } => "invalid_input",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}

fn public_message(error: &TaruError) -> String {
    match error {
        TaruError::Database { .. } => "database operation failed".to_owned(),
        TaruError::Provider { provider, .. } => {
            format!("external provider operation failed: {provider}")
        }
        TaruError::Storage { .. } => "storage operation failed".to_owned(),
        TaruError::InvalidInput { .. } | TaruError::NotFound { .. } | TaruError::Unsupported(_) => {
            error.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use serde::de::DeserializeOwned;
    use taru_api::{HealthResponse, JobResponse, LibraryListResponse};
    use taru_core::{
        CanonicalMetadata, JobId, JobKind, JobStatus, LibraryId, MediaItem, MediaKind,
        MediaRepository, MediaSourceId,
    };
    use taru_db::SqliteStore;
    use tower::ServiceExt;

    use super::*;
    use crate::config::{LocalLibraryConfig, MetadataConfig, TaruServerConfig};

    #[tokio::test]
    async fn health_and_libraries_routes_work() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;

        let health = request_json::<HealthResponse>(&router, Method::GET, "/health").await;
        let libraries =
            request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;

        assert_eq!(health.status, "ok");
        assert_eq!(libraries.libraries.len(), 1);
        assert_eq!(libraries.libraries[0].id, library_id);
    }

    #[tokio::test]
    async fn scan_route_queues_background_job() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let path = format!("/libraries/{library_id}/scan");

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let job = body_json::<JobResponse>(response).await;
        assert_eq!(job.kind, taru_core::JobKind::LibraryScan);
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("library_id"))
                .and_then(serde_json::Value::as_str),
            Some(library_id.to_string().as_str())
        );

        let loaded_path = format!("/jobs/{}", job.id);
        let loaded_job = request_json::<JobResponse>(&router, Method::GET, &loaded_path).await;
        assert_eq!(loaded_job.id, job.id);
    }

    #[tokio::test]
    async fn nfo_routes_queue_background_jobs() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;

        let import_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/libraries/{library_id}/nfo/import"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let export_response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/libraries/{library_id}/nfo/export"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(import_response.status(), StatusCode::ACCEPTED);
        assert_eq!(export_response.status(), StatusCode::ACCEPTED);
        let import_job = body_json::<JobResponse>(import_response).await;
        let export_job = body_json::<JobResponse>(export_response).await;

        assert_eq!(import_job.kind, JobKind::NfoImport);
        assert_eq!(import_job.resource_class, "metadata.nfo.import");
        assert_eq!(import_job.library_id, Some(library_id));
        assert_eq!(
            import_job
                .input
                .as_ref()
                .and_then(|input| input.get("policy"))
                .and_then(serde_json::Value::as_str),
            Some("local_first")
        );
        assert_eq!(export_job.kind, JobKind::NfoExport);
        assert_eq!(export_job.resource_class, "metadata.nfo.export");
        assert_eq!(export_job.library_id, Some(library_id));
    }

    #[tokio::test]
    async fn missing_job_returns_404() {
        let temp = tempfile::tempdir().unwrap();
        let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
        let missing = JobId::new();
        let path = format!("/jobs/{missing}");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn metadata_refresh_route_queues_background_job() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let router = build_router(app);
        let path = format!("/items/{}/metadata/refresh", item.id);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let job = body_json::<JobResponse>(response).await;
        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("item_id"))
                .and_then(serde_json::Value::as_str),
            Some(item.id.to_string().as_str())
        );
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("provider"))
                .and_then(serde_json::Value::as_str),
            Some("tmdb")
        );
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("refresh_mode"))
                .and_then(serde_json::Value::as_str),
            Some("default")
        );
    }

    #[tokio::test]
    async fn empty_sources_and_items_routes_work() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let sources_path = format!("/libraries/{library_id}/sources");

        let sources =
            request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path)
                .await;
        let items = request_json::<taru_api::ItemsResponse>(&router, Method::GET, "/items").await;

        assert_eq!(sources.library.id, library_id);
        assert_eq!(sources.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
        assert_eq!(sources.page.offset, 0);
        assert!(sources.sources.is_empty());
        assert_eq!(items.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
        assert!(items.items.is_empty());
    }

    #[tokio::test]
    async fn missing_source_probe_returns_404() {
        let temp = tempfile::tempdir().unwrap();
        let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
        let missing = MediaSourceId::new();
        let path = format!("/sources/{missing}/probe");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn paginated_routes_echo_page_info_and_reject_large_limits() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let sources_path = format!("/libraries/{library_id}/sources?limit=10&offset=20");

        let sources =
            request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path)
                .await;
        assert_eq!(sources.page.limit, 10);
        assert_eq!(sources.page.offset, 20);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/items?limit=501")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn test_router(root: PathBuf, library_id: LibraryId) -> Router {
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root,
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store).await.unwrap();
        build_router(app)
    }

    async fn request_json<T>(router: &Router, method: Method, uri: &str) -> T
    where
        T: DeserializeOwned,
    {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        body_json(response).await
    }

    async fn body_json<T>(response: axum::response::Response) -> T
    where
        T: DeserializeOwned,
    {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
}
