mod helpers;
mod registration;
use std::sync::Arc;

use nako_db::NakoDatabase;
use tokio::sync::{Mutex, Semaphore};

use super::{runtime::RuntimeSupervisor, storage::StorageBackendRegistry};

mod artwork_write;
mod catalog;
mod diagnostics;
mod event_runtime;
mod external_acquisition;
mod intake;
mod library_file_write;
mod metadata_write;
mod principal;
mod resource_flow;
mod resource_search;
mod routing;
mod runtime;
mod scan_metadata;
mod side_effect_apply;
mod subtitles;
mod surfaces;
mod target;
mod task_runtime;

#[cfg(test)]
pub(crate) use helpers::set_test_outbound_task_dispatch_secret;
pub(super) use helpers::{
    addon_surface_url, fingerprint_key, optional_non_empty, redact_uri, sha256_hex,
};
use registration::AddonRegistrationStore;
pub(crate) use scan_metadata::{
    ScanAddonBulkMetadataScrapeRequest, ScanAddonBulkMetadataScrapeSummary,
};

#[derive(Clone, Debug)]
pub(crate) struct AddonAppService {
    store: NakoDatabase,
    registration_store: Arc<dyn AddonRegistrationStore>,
    resource_search_sessions: Arc<Mutex<resource_search::ResourceSearchSessionStore>>,
    subtitle_search_sessions: Arc<Mutex<subtitles::SubtitleSearchSessionStore>>,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
}

impl AddonAppService {
    pub(super) fn new(
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            registration_store: Arc::new(store.clone()),
            resource_search_sessions: Arc::new(Mutex::new(
                resource_search::ResourceSearchSessionStore::default(),
            )),
            subtitle_search_sessions: Arc::new(Mutex::new(
                subtitles::SubtitleSearchSessionStore::default(),
            )),
            store,
            permits,
            storage_backends,
            runtime,
        }
    }
}
