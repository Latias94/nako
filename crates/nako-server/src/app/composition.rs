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
    renderer_transport_ticket::RendererTransportTicketService,
    runtime::RuntimeSupervisor,
    startup::{ServerStartupReport, ServerStartupWorkflow},
    storage::{StorageBackendRegistry, StorageDiagnosticsAppService},
    user_playback::UserPlaybackAppService,
    webhooks::WebhookAppService,
};

#[derive(Debug)]
pub(super) struct NakoAppComposition {
    pub(super) config: NakoServerConfig,
    pub(super) store: NakoDatabase,
    pub(super) runtime: RuntimeSupervisor,
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

        Ok(Self {
            config,
            store,
            runtime,
            services,
            startup_report: ServerStartupReport {
                artwork_ingest_worker_started,
                addon_event_scheduler_started,
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
    pub(super) renderer: RendererAppService,
    pub(super) user_playback: UserPlaybackAppService,
}

impl NakoAppServices {
    fn build(
        config: &NakoServerConfig,
        store: NakoDatabase,
        runtime: NakoRuntimeResources,
    ) -> Result<Self> {
        let jobs = JobAppService::new(store.clone());
        let acquisition_intake = AcquisitionIntakeAppService::new_with_storage(
            store.clone(),
            runtime.storage_backends.clone(),
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
        let automation = AutomationAppService::new(store.clone());
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
        let renderer = RendererAppService::new(store.clone());
        let casting = CastingAppService::new(renderer.clone(), playback.clone());
        let user_playback = UserPlaybackAppService::new(store);

        Ok(Self {
            acquisition_intake,
            jobs,
            library_scan,
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
            renderer,
            user_playback,
        })
    }
}

#[derive(Debug)]
struct NakoRuntimeResources {
    supervisor: RuntimeSupervisor,
    storage_backends: StorageBackendRegistry,
    scan_permits: Arc<Semaphore>,
    metadata_permits: Arc<Semaphore>,
    webhook_permits: Arc<Semaphore>,
    metadata_providers: MetadataProviderRegistry,
}

impl NakoRuntimeResources {
    fn build(config: &NakoServerConfig, store: NakoDatabase) -> Result<Self> {
        Ok(Self {
            supervisor: RuntimeSupervisor::new(),
            storage_backends: StorageBackendRegistry::new(config, store),
            scan_permits: Arc::new(Semaphore::new(config.scan_concurrency.max(1))),
            metadata_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
            webhook_permits: Arc::new(Semaphore::new(config.webhook_concurrency.max(1))),
            metadata_providers: metadata_runtime::build_metadata_provider_registry(config)?,
        })
    }
}
