use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use taru_api::{EnqueueAutomationJobRequest, JobResponse, UpsertAutomationProviderRequest};
use taru_core::{AutomationProviderId, JobId, MediaItemId};
use tracing::instrument;

use crate::app::TaruApp;

use super::{error::ApiResult, query::PageQuery};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route(
            "/automation/providers",
            get(list_automation_providers).post(upsert_automation_provider),
        )
        .route(
            "/automation/providers/{provider_id}",
            get(get_automation_provider),
        )
        .route("/automation/jobs", post(enqueue_automation_job))
        .route(
            "/automation/jobs/{job_id}/artifacts",
            get(list_automation_job_artifacts),
        )
        .route(
            "/items/{item_id}/automation/artifacts",
            get(list_item_automation_artifacts),
        )
}

#[instrument(skip(app))]
pub(super) async fn upsert_automation_provider(
    State(app): State<TaruApp>,
    Json(request): Json<UpsertAutomationProviderRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.automation().upsert_automation_provider(request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_automation_providers(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.automation().list_enabled_automation_providers().await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_automation_provider(
    State(app): State<TaruApp>,
    Path(provider_id): Path<AutomationProviderId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.automation()
            .get_automation_provider(provider_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn enqueue_automation_job(
    State(app): State<TaruApp>,
    Json(request): Json<EnqueueAutomationJobRequest>,
) -> ApiResult<impl IntoResponse> {
    let job = app.automation().enqueue_automation_job(request).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_automation_job_artifacts(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.automation()
            .list_automation_artifacts_for_job(job_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_item_automation_artifacts(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.automation()
            .list_automation_artifacts_for_item(item_id, page.try_into()?)
            .await?,
    ))
}
