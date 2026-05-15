use axum::{
    Json,
    extract::{Path, State},
};
use taru_api::JobResponse;
use taru_core::JobId;
use tracing::instrument;

use crate::app::TaruApp;

use super::error::ApiResult;

#[instrument(skip(app))]
pub(super) async fn get_job(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<Json<JobResponse>> {
    Ok(Json(JobResponse::from_job(app.get_job(job_id).await?)))
}
