use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use nako_api::admin::JobResponse;
use nako_core::JobId;
use tracing::instrument;

use crate::app::NakoApp;

use super::error::ApiResult;

pub(super) fn routes() -> Router<NakoApp> {
    Router::new().route("/jobs/{job_id}", get(get_job))
}

#[instrument(skip(app))]
pub(super) async fn get_job(
    State(app): State<NakoApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<Json<JobResponse>> {
    Ok(Json(JobResponse::from_job(
        app.jobs().get_job(job_id).await?,
    )))
}
