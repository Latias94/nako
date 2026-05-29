use sqlx::SqlitePool;

pub(crate) mod acquisition_intake;
pub(crate) mod addon_events;
pub(crate) mod addon_tasks;
pub(crate) mod addons;
pub(crate) mod admin_settings;
pub(crate) mod artwork;
pub(crate) mod automation;
pub(crate) mod catalog;
pub(crate) mod catalog_governance;
pub(crate) mod codec;
pub(crate) mod event_outbox;
pub(crate) mod identity;
pub(crate) mod ingestion;
pub(crate) mod jobs;
pub(crate) mod library;
pub(crate) mod library_item;
pub(crate) mod local_inference;
pub(crate) mod managed_import;
pub(crate) mod media;
pub(crate) mod metadata;
mod migrations;
pub(crate) mod nfo_sidecar_apply;
pub(crate) mod playback;
pub(crate) mod provider_mapping;
pub(crate) mod renderer;
mod runtime;
pub(crate) mod scan;
pub(crate) mod search;
pub(crate) mod source_duplicate;
pub(crate) mod staging;
pub(crate) mod user_playback;
pub(crate) mod user_playlist;
pub(crate) mod vfs_cache;
pub(crate) mod webhooks;

pub use runtime::SqliteRuntimeOptions;

#[derive(Clone, Debug)]
pub(crate) struct SqliteStore {
    pub(crate) pool: SqlitePool,
}
