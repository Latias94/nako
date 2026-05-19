use axum::http::{HeaderName, HeaderValue};
use axum::{Extension, Router, middleware, response::Response};
use taru_api::{API_VERSION, API_VERSION_HEADER};

use crate::app::TaruApp;

mod addons;
mod admin;
mod auth;
mod automation;
mod catalog;
mod error;
mod jobs;
mod library;
mod metadata;
mod playback;
mod query;
mod system;
mod user_playback;
mod webhooks;

pub fn build_router(app: TaruApp) -> Router {
    let auth = auth::InboundAuthState::from_config(&app.config().auth);
    build_router_with_auth(app, auth)
}

fn build_router_with_auth(app: TaruApp, auth: auth::InboundAuthState) -> Router {
    let protected_routes = Router::new()
        .merge(system::routes())
        .merge(admin::routes())
        .merge(library::routes())
        .merge(catalog::routes())
        .merge(metadata::routes())
        .merge(playback::routes())
        .merge(user_playback::routes())
        .merge(webhooks::routes())
        .merge(automation::routes())
        .merge(addons::routes())
        .merge(jobs::routes())
        .layer(middleware::from_fn(auth::require_auth))
        .layer(Extension(auth));

    Router::new()
        .merge(system::public_routes())
        .merge(protected_routes)
        .merge(addons::runtime_routes())
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
