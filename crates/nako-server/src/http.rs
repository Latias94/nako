use axum::http::{HeaderName, HeaderValue};
use axum::{Extension, Router, middleware, response::Response};
use nako_api::public_client::{API_VERSION, API_VERSION_HEADER};

use crate::app::NakoApp;

mod access;
mod account;
mod addons;
mod admin;
mod auth;
mod automation;
mod catalog;
mod error;
mod jobs;
mod library;
mod management_context;
mod metadata;
mod network;
mod playback;
mod query;
mod renderer;
mod system;
mod trace_context;
mod user_playback;
mod user_playlist;
mod webhooks;

pub fn build_router(app: NakoApp) -> Router {
    let auth = auth::InboundAuthState::from_config(&app.config().auth);
    build_router_with_auth(app, auth)
}

fn build_router_with_auth(app: NakoApp, auth: auth::InboundAuthState) -> Router {
    let network = network::NetworkBoundaryState::from_config(&app.config().network);
    let unauthenticated_sensitive_routes = Router::new()
        .merge(account::public_routes())
        .layer(middleware::from_fn(network::enforce_network_boundary));
    let protected_routes = Router::new()
        .merge(system::routes())
        .merge(account::routes())
        .merge(admin::routes())
        .merge(library::routes())
        .merge(catalog::routes())
        .merge(management_context::routes())
        .merge(metadata::routes())
        .merge(playback::routes())
        .merge(renderer::routes())
        .merge(user_playlist::routes())
        .merge(user_playback::routes())
        .merge(webhooks::routes())
        .merge(automation::routes())
        .merge(addons::routes())
        .merge(jobs::routes())
        .layer(middleware::from_fn(network::enforce_network_boundary))
        .layer(middleware::from_fn(auth::require_auth))
        .layer(Extension(app.clone()))
        .layer(Extension(auth));

    Router::new()
        .merge(system::public_routes())
        .merge(unauthenticated_sensitive_routes)
        .merge(protected_routes)
        .merge(addons::runtime_routes())
        .layer(middleware::map_response(add_api_version_header))
        .layer(middleware::from_fn(network::annotate_external_origin))
        .layer(middleware::from_fn(
            trace_context::attach_http_trace_context,
        ))
        .layer(Extension(network))
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
