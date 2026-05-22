use std::collections::HashSet;

use nako_addon_protocol::AddonMetadataPatch;
use nako_catalog::{
    CatalogLabelHydrationSelection, plan_item_catalog_label_projection, plan_item_search_projection,
};
use nako_core::{
    AddonId, AddonMetadataWriteCatalogCommit, AddonMetadataWritePersistenceCommit,
    AddonSideEffectRecord, CanonicalMetadata, MediaItem, MetadataMergePolicy, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NakoError, Result,
};
use nako_db::NakoDatabase;

use super::{
    side_effect_apply::AddonSideEffectApplyCommand, target::resolve_side_effect_media_item,
};

#[derive(Clone, Debug)]
pub(super) struct AddonMetadataWriteAdapter {
    store: NakoDatabase,
}

impl AddonMetadataWriteAdapter {
    pub(super) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(super) async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectApplyCommand> {
        let existing = resolve_side_effect_media_item(&self.store, side_effect).await?;
        let patch = parse_addon_metadata_patch(&side_effect.payload_json)?;
        let label_selection = patch.catalog_label_selection();
        let incoming = patch.apply_to(existing.metadata.clone());
        let source = MetadataSource::Addon(side_effect.addon_id);
        let locks = self.store.list_field_locks(existing.id).await?;
        let policy = MetadataMergePolicy::for_source_refresh_mode(
            &locks,
            &source,
            MetadataRefreshMode::FullRefresh,
        );
        let merged = policy.merge(&existing.metadata, &incoming);
        let updated = MediaItem {
            metadata: merged,
            ..existing
        };
        let catalog = if label_selection.any() {
            let projection = plan_item_catalog_label_projection(
                &self.store,
                updated.clone(),
                source,
                label_selection,
            )
            .await?;
            AddonMetadataWriteCatalogCommit {
                graph: Some(projection.graph),
                search: projection.search,
            }
        } else {
            AddonMetadataWriteCatalogCommit {
                graph: None,
                search: plan_item_search_projection(&self.store, updated.clone()).await?,
            }
        };
        let applied_source = addon_metadata_source_label(side_effect.addon_id);

        Ok(AddonSideEffectApplyCommand::MetadataWrite(
            AddonMetadataWritePersistenceCommit {
                side_effect_id: side_effect.id,
                item: updated,
                catalog,
                applied_source,
                apply_report_json: None,
            },
        ))
    }
}

trait AddonMetadataPatchExt {
    fn apply_to(self, metadata: CanonicalMetadata) -> CanonicalMetadata;
    fn validate(&self) -> Result<()>;
    fn catalog_label_selection(&self) -> CatalogLabelHydrationSelection;
}

impl AddonMetadataPatchExt for AddonMetadataPatch {
    fn apply_to(self, mut metadata: CanonicalMetadata) -> CanonicalMetadata {
        if let Some(title) = self.title.and_then(non_empty_trimmed) {
            metadata.title = title;
        }
        if let Some(value) = self.original_title.map(normalize_optional_text) {
            metadata.original_title = value;
        }
        if let Some(value) = self.sort_title.map(normalize_optional_text) {
            metadata.sort_title = value;
        }
        if let Some(value) = self.overview.map(normalize_optional_text) {
            metadata.overview = value;
        }
        if let Some(value) = self.release_date.map(normalize_optional_text) {
            metadata.release_date = value;
        }
        if let Some(runtime_minutes) = self.runtime_minutes {
            metadata.runtime_minutes = (runtime_minutes > 0).then_some(runtime_minutes);
        }
        if let Some(value) = self.tagline.map(normalize_optional_text) {
            metadata.tagline = value;
        }
        if let Some(genres) = self.genres {
            metadata.genres = normalize_label_list(genres);
        }
        if let Some(tags) = self.tags {
            metadata.tags = normalize_label_list(tags);
        }

        metadata
    }

    fn validate(&self) -> Result<()> {
        if !self.has_any_field() {
            return Err(NakoError::InvalidInput {
                message:
                    "addon metadata_write payload must include at least one supported metadata field"
                        .to_owned(),
            });
        }

        self.validate_text_field("title", self.title.as_ref())?;
        self.validate_text_field("original_title", self.original_title.as_ref())?;
        self.validate_text_field("sort_title", self.sort_title.as_ref())?;
        self.validate_text_field("overview", self.overview.as_ref())?;
        self.validate_text_field("release_date", self.release_date.as_ref())?;
        self.validate_text_field("tagline", self.tagline.as_ref())?;
        self.validate_list_field("genres", self.genres.as_ref())?;
        self.validate_list_field("tags", self.tags.as_ref())?;

        if self.runtime_minutes == Some(0) {
            return Err(NakoError::InvalidInput {
                message: "addon metadata_write payload runtime_minutes must be greater than zero"
                    .to_owned(),
            });
        }

        Ok(())
    }

    fn catalog_label_selection(&self) -> CatalogLabelHydrationSelection {
        CatalogLabelHydrationSelection {
            genres: self.genres.is_some(),
            tags: self.tags.is_some(),
        }
    }
}

trait AddonMetadataPatchValidationExt {
    fn has_any_field(&self) -> bool;
    fn validate_text_field(&self, field: &str, value: Option<&String>) -> Result<()>;
    fn validate_list_field(&self, field: &str, value: Option<&Vec<String>>) -> Result<()>;
}

impl AddonMetadataPatchValidationExt for AddonMetadataPatch {
    fn has_any_field(&self) -> bool {
        self.title.is_some()
            || self.original_title.is_some()
            || self.sort_title.is_some()
            || self.overview.is_some()
            || self.release_date.is_some()
            || self.runtime_minutes.is_some()
            || self.tagline.is_some()
            || self.genres.is_some()
            || self.tags.is_some()
    }

    fn validate_text_field(&self, field: &str, value: Option<&String>) -> Result<()> {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(NakoError::InvalidInput {
                message: format!("addon metadata_write payload {field} must not be empty"),
            });
        }

        Ok(())
    }

    fn validate_list_field(&self, field: &str, value: Option<&Vec<String>>) -> Result<()> {
        if value.is_some_and(|values| values.iter().any(|value| value.trim().is_empty())) {
            return Err(NakoError::InvalidInput {
                message: format!("addon metadata_write payload {field} entries must not be empty"),
            });
        }

        Ok(())
    }
}

fn parse_addon_metadata_patch(payload_json: &str) -> Result<AddonMetadataPatch> {
    let patch = serde_json::from_str::<AddonMetadataPatch>(payload_json).map_err(|err| {
        NakoError::InvalidInput {
            message: format!("invalid addon metadata_write payload: {err}"),
        }
    })?;

    patch.validate()?;
    Ok(patch)
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalize_optional_text(value: String) -> Option<String> {
    non_empty_trimmed(value)
}

fn normalize_label_list(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(non_empty_trimmed)
        .filter(|value| seen.insert(value.to_lowercase()))
        .collect()
}

fn addon_metadata_source_label(addon_id: AddonId) -> String {
    format!("addon:{addon_id}")
}
