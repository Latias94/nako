use axum::{
    Router,
    routing::{get, post},
};

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

use addons::{get_addon, list_addons, register_addon};
use automation::{
    enqueue_automation_job, get_automation_provider, list_automation_job_artifacts,
    list_automation_providers, list_item_automation_artifacts, upsert_automation_provider,
};
use catalog::{
    get_item, get_person, get_source_probe, list_genre_items, list_genres, list_item_credits,
    list_item_images, list_items, list_people, list_person_items, list_tag_items, list_tags,
    search_items,
};
use jobs::get_job;
use library::{export_nfo, import_nfo, list_libraries, list_library_sources, scan_library};
use metadata::{
    list_item_metadata_attempts, list_item_metadata_raw_responses, list_metadata_providers,
    refresh_item_metadata,
};
use playback::{
    get_playback_session, get_source_playback_decision, head_stream_source, hls_playlist_source,
    hls_segment, remux_stream_source, stream_source,
};
use system::health;
use webhooks::{
    deliver_webhooks_for_event, get_webhook_endpoint, list_webhook_delivery_attempts,
    list_webhook_endpoints, upsert_webhook_endpoint,
};

pub fn build_router(app: TaruApp) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/libraries", get(list_libraries))
        .route("/libraries/{library_id}/scan", post(scan_library))
        .route("/libraries/{library_id}/nfo/import", post(import_nfo))
        .route("/libraries/{library_id}/nfo/export", post(export_nfo))
        .route("/libraries/{library_id}/sources", get(list_library_sources))
        .route("/items", get(list_items))
        .route("/items/{item_id}", get(get_item))
        .route("/items/{item_id}/credits", get(list_item_credits))
        .route("/items/{item_id}/images", get(list_item_images))
        .route("/people", get(list_people))
        .route("/people/{person_id}", get(get_person))
        .route("/people/{person_id}/items", get(list_person_items))
        .route("/tags", get(list_tags))
        .route("/tags/{tag_id}/items", get(list_tag_items))
        .route("/genres", get(list_genres))
        .route("/genres/{genre_id}/items", get(list_genre_items))
        .route("/search", get(search_items))
        .route(
            "/items/{item_id}/metadata/refresh",
            post(refresh_item_metadata),
        )
        .route(
            "/items/{item_id}/metadata/attempts",
            get(list_item_metadata_attempts),
        )
        .route(
            "/items/{item_id}/metadata/raw",
            get(list_item_metadata_raw_responses),
        )
        .route("/metadata/providers", get(list_metadata_providers))
        .route("/sources/{source_id}/probe", get(get_source_probe))
        .route(
            "/sources/{source_id}/playback/decision",
            get(get_source_playback_decision),
        )
        .route(
            "/sources/{source_id}/stream",
            get(stream_source).head(head_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/remux",
            get(remux_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/hls/playlist.m3u8",
            get(hls_playlist_source),
        )
        .route("/playback/sessions/{session_id}", get(get_playback_session))
        .route(
            "/playback/sessions/{session_id}/hls/segments/{segment_name}",
            get(hls_segment),
        )
        .route(
            "/webhooks/endpoints",
            get(list_webhook_endpoints).post(upsert_webhook_endpoint),
        )
        .route(
            "/webhooks/endpoints/{endpoint_id}",
            get(get_webhook_endpoint),
        )
        .route(
            "/events/{event_id}/webhook-attempts",
            get(list_webhook_delivery_attempts),
        )
        .route(
            "/events/{event_id}/webhooks/deliver",
            post(deliver_webhooks_for_event),
        )
        .route(
            "/automation/providers",
            get(list_automation_providers).post(upsert_automation_provider),
        )
        .route(
            "/automation/providers/{provider_id}",
            get(get_automation_provider),
        )
        .route("/addons", get(list_addons).post(register_addon))
        .route("/addons/{addon_id}", get(get_addon))
        .route("/automation/jobs", post(enqueue_automation_job))
        .route(
            "/automation/jobs/{job_id}/artifacts",
            get(list_automation_job_artifacts),
        )
        .route(
            "/items/{item_id}/automation/artifacts",
            get(list_item_automation_artifacts),
        )
        .route("/jobs/{job_id}", get(get_job))
        .with_state(app)
}

#[cfg(test)]
mod tests;
