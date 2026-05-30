use nako_catalog::plan_item_catalog_projection;
use nako_core::{
    AddonId, AutomationArtifactId, AutomationProviderId, CanonicalMetadata,
    CatalogItemProjectionCommit, ExternalProvider, LibraryId, LibraryRepository, MediaItem,
    MetadataMergePolicy, MetadataRefreshMode, MetadataRepository, MetadataSource, NakoError,
    Result,
};
use nako_db::NakoDatabase;
use serde::Serialize;

#[derive(Clone, Debug)]
pub(crate) struct MetadataApplication {
    store: NakoDatabase,
}

impl MetadataApplication {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(crate) async fn apply(
        &self,
        command: MetadataApplicationCommand,
    ) -> Result<MetadataApplicationResult> {
        let refresh_mode = self.resolve_refresh_mode(command.mode).await?;
        let source = command.source;
        let previous_metadata = command.item.metadata.clone();
        let locks = self.store.list_field_locks(command.item.id).await?;
        let policy = match command.lock_scope {
            MetadataApplicationLockScope::ProtectAllLocks => {
                MetadataMergePolicy::from_locks_and_mode(&locks, refresh_mode)
            }
            MetadataApplicationLockScope::ProtectOtherSourceLocks => {
                MetadataMergePolicy::for_source_refresh_mode(&locks, &source, refresh_mode)
            }
        };
        let merged = policy.merge(&previous_metadata, &command.incoming);
        let item = MediaItem {
            metadata: merged,
            ..command.item
        };
        let projection =
            plan_item_catalog_projection(&self.store, item.clone(), source.clone()).await?;
        let applied_source = metadata_source_label(&source);
        let changed = previous_metadata != item.metadata;
        let apply_report_json = Some(metadata_application_report_json(
            &applied_source,
            refresh_mode,
            changed,
            command.provenance,
        )?);

        Ok(MetadataApplicationResult {
            item,
            projection,
            applied_source,
            apply_report_json,
            changed,
        })
    }

    async fn resolve_refresh_mode(
        &self,
        mode: MetadataApplicationMode,
    ) -> Result<MetadataRefreshMode> {
        match mode {
            MetadataApplicationMode::LibraryProfile { library_id } => {
                let library = self.store.get_library(library_id).await?.ok_or_else(|| {
                    NakoError::NotFound {
                        entity: "library",
                        id: library_id.to_string(),
                    }
                })?;
                Ok(library.options.metadata_profile.refresh_mode)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MetadataApplicationCommand {
    pub(crate) item: MediaItem,
    pub(crate) source: MetadataSource,
    pub(crate) incoming: CanonicalMetadata,
    pub(crate) mode: MetadataApplicationMode,
    pub(crate) lock_scope: MetadataApplicationLockScope,
    pub(crate) provenance: MetadataApplicationProvenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MetadataApplicationMode {
    LibraryProfile { library_id: LibraryId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MetadataApplicationLockScope {
    ProtectAllLocks,
    ProtectOtherSourceLocks,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum MetadataApplicationProvenance {
    AddonSideEffect {
        addon_id: AddonId,
        library_id: LibraryId,
    },
    GeneratedArtifact {
        artifact_id: AutomationArtifactId,
        provider_id: AutomationProviderId,
        library_id: LibraryId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataApplicationResult {
    pub(crate) item: MediaItem,
    pub(crate) projection: CatalogItemProjectionCommit,
    pub(crate) applied_source: String,
    pub(crate) apply_report_json: Option<String>,
    pub(crate) changed: bool,
}

#[derive(Serialize)]
struct MetadataApplicationReport {
    source: String,
    refresh_mode: MetadataRefreshMode,
    changed: bool,
    provenance: MetadataApplicationProvenance,
}

fn metadata_application_report_json(
    source: &str,
    refresh_mode: MetadataRefreshMode,
    changed: bool,
    provenance: MetadataApplicationProvenance,
) -> Result<String> {
    serde_json::to_string(&MetadataApplicationReport {
        source: source.to_owned(),
        refresh_mode,
        changed,
        provenance,
    })
    .map_err(|err| NakoError::InvalidInput {
        message: format!("metadata application report serialization failed: {err}"),
    })
}

fn metadata_source_label(source: &MetadataSource) -> String {
    match source {
        MetadataSource::Local => "local".to_owned(),
        MetadataSource::Nfo => "nfo".to_owned(),
        MetadataSource::Provider(provider) => {
            format!("provider:{}", external_provider_label(provider))
        }
        MetadataSource::User => "user".to_owned(),
        MetadataSource::Addon(addon_id) => format!("addon:{addon_id}"),
    }
}

fn external_provider_label(provider: &ExternalProvider) -> String {
    match provider {
        ExternalProvider::Tmdb => "tmdb".to_owned(),
        ExternalProvider::Douban => "douban".to_owned(),
        ExternalProvider::Bangumi => "bangumi".to_owned(),
        ExternalProvider::Imdb => "imdb".to_owned(),
        ExternalProvider::Local => "local".to_owned(),
        ExternalProvider::Other(value) => format!("other:{value}"),
    }
}
