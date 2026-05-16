use axum::Router;

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
        .with_state(app)
}

#[cfg(test)]
mod tests;
