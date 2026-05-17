use axum::http::{HeaderName, HeaderValue};
use axum::{Router, middleware, response::Response};
use taru_api::{API_VERSION, API_VERSION_HEADER};

use crate::app::TaruApp;

mod addons;
mod automation;
mod catalog;
mod error;
mod jobs;
mod library;
mod metadata;
mod playback;
mod query;
mod system;
mod webhooks;

pub fn build_router(app: TaruApp) -> Router {
    Router::new()
        .merge(system::routes())
        .merge(library::routes())
        .merge(catalog::routes())
        .merge(metadata::routes())
        .merge(playback::routes())
        .merge(webhooks::routes())
        .merge(automation::routes())
        .merge(addons::routes())
        .merge(jobs::routes())
        .layer(middleware::map_response(add_api_version_header))
        .with_state(app)
}

async fn add_api_version_header(mut response: Response) -> Response {
    response.headers_mut().insert(
        HeaderName::from_static(API_VERSION_HEADER),
        HeaderValue::from_static(API_VERSION),
    );
    response
}

#[cfg(test)]
mod tests;
