use std::sync::Arc;

use nako_core::{
    AdminSettingsEffect, AdminSettingsRepository, AdminSettingsSource, DatabaseLifecycle,
    NakoError, Result,
};
use nako_db::NakoDatabase;
use nako_metadata::MetadataProviderRegistry;
use tokio::sync::Semaphore;

use crate::config::NakoServerConfig;

use super::apply_playback_runtime_settings;

use super::{
    SourceDuplicateReconciliationAppService, SourceFingerprintHashAppService,
    acquisition_intake::AcquisitionIntakeAppService,
    addons::AddonAppService,
    artwork::ManagedArtworkAppService,
    automation::AutomationAppService,
    casting::CastingAppService,
    catalog::CatalogAppService,
    jobs::{JobAppService, LibraryScanAppService},
    library::LibraryAppService,
    managed_import::ManagedImportAppService,
    management_context::ManagementContextAppService,
    metadata::MetadataAppService,
    metadata_runtime,
    nfo::NfoAppService,
    playback::{PlaybackAppService, PlaybackRuntimeStore},
    playback_ticket::BrowserPlaybackTicketService,
    renderer::RendererAppService,
    renderer_adapter::RendererAdapterBridgeService,
    renderer_transport_ticket::RendererTransportTicketService,
    runtime::{
        RUNTIME_RESOURCE_CLASS_ADDON_TASK, RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST,
        RUNTIME_RESOURCE_CLASS_DISK_SCAN, RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
        RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK, RuntimeResourceClassRegistry, RuntimeSupervisor,
    },
    startup::{ServerStartupReport, ServerStartupWorkflow},
    storage::{StorageBackendRegistry, StorageDiagnosticsAppService},
    user_playback::UserPlaybackAppService,
    user_playlist::UserPlaylistAppService,
    watch_folder_runtime::WatchFolderRuntimeAppService,
    watch_folder_suppression::WatchFolderSuppressionAppService,
    webhooks::WebhookAppService,
};

#[derive(Debug)]
pub(super) struct NakoAppComposition {
    pub(super) config: NakoServerConfig,
    pub(super) store: NakoDatabase,
    pub(super) runtime: RuntimeSupervisor,
    pub(super) runtime_resource_classes: RuntimeResourceClassRegistry,
    pub(super) services: NakoAppServices,
    pub(super) startup_report: ServerStartupReport,
}

impl NakoAppComposition {
    pub(super) async fn build(config: NakoServerConfig, store: NakoDatabase) -> Result<Self> {
        store.migrate().await?;
        let config = effective_startup_config(config, &store).await?;
        validate_configured_backend_runtime_scope(&config, &store)?;
        let runtime_resources = NakoRuntimeResources::build(&config, store.clone())?;
        let runtime = runtime_resources.supervisor.clone();
        let runtime_resource_classes = runtime_resources.resource_classes.clone();
        let services = NakoAppServices::build(&config, store.clone(), runtime_resources)?;

        let startup_report = ServerStartupWorkflow::new(&config, &store, services.metadata.clone())
            .run()
            .await?;
        let artwork_ingest_worker_started = config
            .artwork
            .ingest_worker_enabled
            .then(|| services.artwork.start_ingest_worker(&runtime))
            .unwrap_or(false);
        let addon_event_scheduler_started = services
            .addons
            .start_addon_event_scheduler(config.addon_event_scheduler);
        let watch_folder_runtime_coverage = services
            .watch_folder_runtime
            .start_enabled_watchers(&runtime)
            .await?;
        let watch_folder_runtimes_started = watch_folder_runtime_coverage.started_libraries();

        Ok(Self {
            config,
            store,
            runtime,
            runtime_resource_classes,
            services,
            startup_report: ServerStartupReport {
                artwork_ingest_worker_started,
                addon_event_scheduler_started,
                watch_folder_runtimes_started,
                watch_folder_runtime_coverage,
                ..startup_report
            },
        })
    }

    pub(super) fn shutdown_runtime(&self) {
        self.runtime.shutdown();
    }
}

async fn effective_startup_config(
    mut config: NakoServerConfig,
    store: &NakoDatabase,
) -> Result<NakoServerConfig> {
    if let Some(record) = store.get_admin_metadata_raw_cache_settings().await?
        && record.source == AdminSettingsSource::Admin
        && matches!(
            record.effect,
            AdminSettingsEffect::RequiresRestart | AdminSettingsEffect::Active
        )
    {
        config.metadata.raw_cache_retention_ms = record.settings.retention_ms;
        config.metadata.maintenance.raw_cache_cleanup_on_startup =
            record.settings.cleanup_on_startup;
    }
    if let Some(record) = store
        .get_admin_settings_document(nako_core::AdminSettingsDocumentKey::PlaybackRuntime)
        .await?
        && record.source == AdminSettingsSource::Admin
        && matches!(
            record.effect,
            AdminSettingsEffect::RequiresRestart | AdminSettingsEffect::Active
        )
    {
        let settings =
            serde_json::from_str(&record.payload_json).map_err(|err| NakoError::Database {
                message: format!("invalid persisted playback runtime settings: {err}"),
            })?;
        apply_playback_runtime_settings(&mut config, &settings);
    }

    Ok(config)
}

fn validate_configured_backend_runtime_scope(
    config: &NakoServerConfig,
    store: &NakoDatabase,
) -> Result<()> {
    let capabilities = store.capabilities();
    if config.artwork.ingest_worker_enabled && !capabilities.managed_artwork {
        return Err(NakoError::Unsupported(
            "Configured database backend cannot enable Managed Artwork ingest worker without Managed Artwork repository support",
        ));
    }

    Ok(())
}

impl Drop for NakoAppComposition {
    fn drop(&mut self) {
        self.shutdown_runtime();
    }
}

#[derive(Debug)]
pub(super) struct NakoAppServices {
    pub(super) acquisition_intake: AcquisitionIntakeAppService,
    pub(super) jobs: JobAppService,
    pub(super) library_scan: LibraryScanAppService,
    pub(super) source_hash: SourceFingerprintHashAppService,
    pub(super) source_duplicate_reconciliation: SourceDuplicateReconciliationAppService,
    pub(super) artwork: ManagedArtworkAppService,
    pub(super) addons: AddonAppService,
    pub(super) automation: AutomationAppService,
    pub(super) webhooks: WebhookAppService,
    pub(super) catalog: CatalogAppService,
    pub(super) casting: CastingAppService,
    pub(super) library: LibraryAppService,
    pub(super) management_context: ManagementContextAppService,
    pub(super) storage: StorageDiagnosticsAppService,
    pub(super) metadata: MetadataAppService,
    pub(super) managed_import: ManagedImportAppService,
    pub(super) nfo: NfoAppService,
    pub(super) playback: PlaybackAppService,
    pub(super) playback_tickets: BrowserPlaybackTicketService,
    pub(super) renderer_transport_tickets: RendererTransportTicketService,
    pub(super) renderer_adapters: RendererAdapterBridgeService,
    pub(super) renderer: RendererAppService,
    pub(super) user_playlist: UserPlaylistAppService,
    pub(super) user_playback: UserPlaybackAppService,
    pub(super) watch_folder_runtime: WatchFolderRuntimeAppService,
    pub(super) watch_folder_suppression: WatchFolderSuppressionAppService,
}

impl NakoAppServices {
    fn build(
        config: &NakoServerConfig,
        store: NakoDatabase,
        runtime: NakoRuntimeResources,
    ) -> Result<Self> {
        let jobs = JobAppService::new(store.clone());
        let source_hash =
            SourceFingerprintHashAppService::new(store.clone(), runtime.storage_backends.clone());
        let source_duplicate_reconciliation =
            SourceDuplicateReconciliationAppService::new(store.clone());
        let watch_folder_suppression = WatchFolderSuppressionAppService::new();
        let acquisition_intake = AcquisitionIntakeAppService::new_with_storage_and_suppression(
            store.clone(),
            runtime.storage_backends.clone(),
            watch_folder_suppression.clone(),
        );
        let artwork = ManagedArtworkAppService::new(config.artwork.clone(), store.clone())?;
        let addons = AddonAppService::new(
            store.clone(),
            runtime.metadata_permits.clone(),
            runtime.storage_backends.clone(),
            runtime.supervisor.clone(),
        );
        let library_scan = LibraryScanAppService::new(
            config.clone(),
            store.clone(),
            runtime.scan_permits,
            runtime.storage_backends.clone(),
            runtime.supervisor.clone(),
            addons.clone(),
        );
        let automation = AutomationAppService::new(store.clone(), runtime.metadata_permits.clone());
        let webhooks = WebhookAppService::new(store.clone(), runtime.webhook_permits);
        let catalog = CatalogAppService::new(store.clone());
        let library = LibraryAppService::new(store.clone());
        let management_context = ManagementContextAppService::new(store.clone());
        let storage = StorageDiagnosticsAppService::new(runtime.storage_backends.clone());
        let metadata = MetadataAppService::new(
            config.clone(),
            store.clone(),
            runtime.metadata_permits.clone(),
            runtime.metadata_providers,
            runtime.supervisor.clone(),
        );
        let managed_import = ManagedImportAppService::new_with_storage(
            store.clone(),
            runtime.storage_backends.clone(),
        );
        let nfo = NfoAppService::new(
            store.clone(),
            runtime.metadata_permits,
            runtime.storage_backends.clone(),
            runtime.supervisor.clone(),
        );
        let runtime_store: Arc<dyn PlaybackRuntimeStore> = Arc::new(store.clone());
        let staging_store: Arc<dyn nako_core::StagingManifestRepository> = Arc::new(store.clone());
        let playback = PlaybackAppService::new(
            config.clone(),
            runtime_store,
            staging_store,
            runtime.storage_backends,
            runtime.supervisor,
        )?;
        let playback_tickets = BrowserPlaybackTicketService::new();
        let renderer_transport_tickets = RendererTransportTicketService::new();
        let renderer_adapters = RendererAdapterBridgeService::new();
        let renderer = RendererAppService::new(store.clone());
        let casting = CastingAppService::new(renderer.clone(), playback.clone());
        let user_playlist = UserPlaylistAppService::new(store.clone());
        let user_playback = UserPlaybackAppService::new(store.clone());
        let watch_folder_runtime = WatchFolderRuntimeAppService::new(
            store,
            acquisition_intake.clone(),
            library_scan.clone(),
        );

        Ok(Self {
            acquisition_intake,
            jobs,
            library_scan,
            source_hash,
            source_duplicate_reconciliation,
            artwork,
            addons,
            automation,
            webhooks,
            catalog,
            casting,
            library,
            management_context,
            storage,
            metadata,
            managed_import,
            nfo,
            playback,
            playback_tickets,
            renderer_transport_tickets,
            renderer_adapters,
            renderer,
            user_playlist,
            user_playback,
            watch_folder_runtime,
            watch_folder_suppression,
        })
    }
}

#[derive(Debug)]
struct NakoRuntimeResources {
    supervisor: RuntimeSupervisor,
    resource_classes: RuntimeResourceClassRegistry,
    storage_backends: StorageBackendRegistry,
    scan_permits: Arc<Semaphore>,
    metadata_permits: Arc<Semaphore>,
    webhook_permits: Arc<Semaphore>,
    metadata_providers: MetadataProviderRegistry,
}

impl NakoRuntimeResources {
    fn build(config: &NakoServerConfig, store: NakoDatabase) -> Result<Self> {
        let resource_classes = RuntimeResourceClassRegistry::new([
            (
                RUNTIME_RESOURCE_CLASS_DISK_SCAN,
                config.scan_concurrency.max(1),
            ),
            (
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
                config.metadata_concurrency.max(1),
            ),
            (
                RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK,
                config.webhook_concurrency.max(1),
            ),
            (
                RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST,
                config.artwork.fetch_concurrency.max(1),
            ),
            (
                RUNTIME_RESOURCE_CLASS_ADDON_TASK,
                config.addon_event_scheduler.concurrency.max(1),
            ),
        ])?;
        let scan_permits = resource_classes.semaphore(RUNTIME_RESOURCE_CLASS_DISK_SCAN)?;
        let metadata_permits =
            resource_classes.semaphore(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)?;
        let webhook_permits = resource_classes.semaphore(RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK)?;

        Ok(Self {
            supervisor: RuntimeSupervisor::new(),
            resource_classes,
            storage_backends: StorageBackendRegistry::new(config, store),
            scan_permits,
            metadata_permits,
            webhook_permits,
            metadata_providers: metadata_runtime::build_metadata_provider_registry(config)?,
        })
    }
}
