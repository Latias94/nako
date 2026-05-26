use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use nako_core::{AuthenticatedPrincipal, NakoError};
use serde::Deserialize;
use tracing::instrument;

use crate::app::{ManagementContextRequest, NakoApp};

use super::error::ApiResult;

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ManagementContextLinksQuery {
    pub(super) library_id: Option<String>,
    pub(super) item_id: Option<String>,
    pub(super) source_id: Option<String>,
    pub(super) playback_session_id: Option<String>,
}

pub(super) fn routes() -> Router<NakoApp> {
    Router::new().route(
        "/management/context-links",
        get(get_management_context_links),
    )
}

#[instrument(skip(app))]
pub(super) async fn get_management_context_links(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(query): Query<ManagementContextLinksQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.management_context()
            .context_links(&principal, query.into_request()?)
            .await?,
    ))
}

impl ManagementContextLinksQuery {
    fn into_request(self) -> Result<ManagementContextRequest, NakoError> {
        Ok(ManagementContextRequest {
            library_id: parse_optional_id(self.library_id, "library_id")?,
            item_id: parse_optional_id(self.item_id, "item_id")?,
            source_id: parse_optional_id(self.source_id, "source_id")?,
            playback_session_id: parse_optional_id(
                self.playback_session_id,
                "playback_session_id",
            )?,
        })
    }
}

fn parse_optional_id<T>(value: Option<String>, field: &str) -> Result<Option<T>, NakoError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .map(|value| {
            value.parse::<T>().map_err(|err| NakoError::InvalidInput {
                message: format!("invalid {field} filter: {err}"),
            })
        })
        .transpose()
}
