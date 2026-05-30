use std::collections::HashSet;

use nako_addon_protocol::{
    AddonMetadataCollection, AddonMetadataContentRating, AddonMetadataCredit,
    AddonMetadataExternalId, AddonMetadataImage, AddonMetadataPatch, AddonMetadataStudio,
};
use nako_core::{
    AddonMetadataWriteCatalogCommit, AddonMetadataWritePersistenceCommit, AddonSideEffectRecord,
    CanonicalMetadata, CollectionRef, ContentRating, Credit, CreditRole, ExternalId,
    ExternalProvider, ImageKind, ImageRef, MetadataSource, NakoError, Result, StudioRef,
};
use nako_db::NakoDatabase;

use super::{
    side_effect_apply::AddonSideEffectApplyCommand, target::resolve_side_effect_media_item,
};
use crate::app::metadata_application::{
    MetadataApplication, MetadataApplicationCommand, MetadataApplicationLockScope,
    MetadataApplicationMode, MetadataApplicationProvenance,
};

#[derive(Clone, Debug)]
pub(super) struct AddonMetadataWriteAdapter {
    application: MetadataApplication,
    store: NakoDatabase,
}

impl AddonMetadataWriteAdapter {
    pub(super) fn new(store: NakoDatabase) -> Self {
        Self {
            application: MetadataApplication::new(store.clone()),
            store,
        }
    }

    pub(super) async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectApplyCommand> {
        let existing = resolve_side_effect_media_item(&self.store, side_effect).await?;
        let patch = parse_addon_metadata_patch(&side_effect.payload_json)?;
        let incoming = patch.apply_to(existing.metadata.clone())?;
        let source = MetadataSource::Addon(side_effect.addon_id);
        let applied = self
            .application
            .apply(MetadataApplicationCommand {
                item: existing,
                source,
                incoming,
                mode: MetadataApplicationMode::LibraryProfile {
                    library_id: side_effect.library_id,
                },
                lock_scope: MetadataApplicationLockScope::ProtectOtherSourceLocks,
                provenance: MetadataApplicationProvenance::AddonSideEffect {
                    addon_id: side_effect.addon_id,
                    library_id: side_effect.library_id,
                },
            })
            .await?;
        let catalog = AddonMetadataWriteCatalogCommit {
            graph: Some(applied.projection.graph),
            search: applied.projection.search,
        };

        Ok(AddonSideEffectApplyCommand::MetadataWrite(
            AddonMetadataWritePersistenceCommit {
                side_effect_id: side_effect.id,
                item: applied.item,
                catalog,
                applied_source: applied.applied_source,
                apply_report_json: applied.apply_report_json,
            },
        ))
    }
}

trait AddonMetadataPatchExt {
    fn apply_to(self, metadata: CanonicalMetadata) -> Result<CanonicalMetadata>;
    fn validate(&self) -> Result<()>;
}

impl AddonMetadataPatchExt for AddonMetadataPatch {
    fn apply_to(self, mut metadata: CanonicalMetadata) -> Result<CanonicalMetadata> {
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
        if let Some(ratings) = self.ratings {
            metadata.ratings = normalize_ratings(ratings);
        }
        if let Some(images) = self.images {
            metadata.images = normalize_images(images)?;
        }
        if let Some(credits) = self.credits {
            metadata.credits = normalize_credits(credits)?;
        }
        if let Some(collections) = self.collections {
            metadata.collections = normalize_collections(collections)?;
        }
        if let Some(studios) = self.studios {
            metadata.studios = normalize_studios(studios)?;
        }
        if let Some(external_ids) = self.external_ids {
            metadata.external_ids = normalize_external_ids(external_ids)?;
        }

        Ok(metadata)
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
        self.validate_ratings_field(self.ratings.as_ref())?;
        self.validate_images_field(self.images.as_ref())?;
        self.validate_credits_field(self.credits.as_ref())?;
        self.validate_collections_field(self.collections.as_ref())?;
        self.validate_studios_field(self.studios.as_ref())?;
        self.validate_external_ids_field("external_ids", self.external_ids.as_ref())?;

        if self.runtime_minutes == Some(0) {
            return Err(NakoError::InvalidInput {
                message: "addon metadata_write payload runtime_minutes must be greater than zero"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

trait AddonMetadataPatchValidationExt {
    fn has_any_field(&self) -> bool;
    fn validate_text_field(&self, field: &str, value: Option<&String>) -> Result<()>;
    fn validate_list_field(&self, field: &str, value: Option<&Vec<String>>) -> Result<()>;
    fn validate_ratings_field(&self, value: Option<&Vec<AddonMetadataContentRating>>)
    -> Result<()>;
    fn validate_images_field(&self, value: Option<&Vec<AddonMetadataImage>>) -> Result<()>;
    fn validate_credits_field(&self, value: Option<&Vec<AddonMetadataCredit>>) -> Result<()>;
    fn validate_collections_field(
        &self,
        value: Option<&Vec<AddonMetadataCollection>>,
    ) -> Result<()>;
    fn validate_studios_field(&self, value: Option<&Vec<AddonMetadataStudio>>) -> Result<()>;
    fn validate_external_ids_field(
        &self,
        field: &str,
        value: Option<&Vec<AddonMetadataExternalId>>,
    ) -> Result<()>;
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
            || self.ratings.is_some()
            || self.images.is_some()
            || self.credits.is_some()
            || self.collections.is_some()
            || self.studios.is_some()
            || self.external_ids.is_some()
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

    fn validate_ratings_field(
        &self,
        value: Option<&Vec<AddonMetadataContentRating>>,
    ) -> Result<()> {
        let Some(ratings) = value else {
            return Ok(());
        };

        for rating in ratings {
            validate_non_empty("ratings.source", &rating.source)?;
            validate_non_empty("ratings.value", &rating.value)?;
        }

        Ok(())
    }

    fn validate_images_field(&self, value: Option<&Vec<AddonMetadataImage>>) -> Result<()> {
        let Some(images) = value else {
            return Ok(());
        };

        for image in images {
            validate_non_empty("images.kind", &image.kind)?;
            validate_non_empty("images.uri", &image.uri)?;
            validate_non_empty("images.provider", &image.provider)?;
            validate_positive_optional("images.width", image.width)?;
            validate_positive_optional("images.height", image.height)?;
            if let Some(language) = image.language.as_ref() {
                validate_non_empty("images.language", language)?;
            }
        }

        Ok(())
    }

    fn validate_credits_field(&self, value: Option<&Vec<AddonMetadataCredit>>) -> Result<()> {
        let Some(credits) = value else {
            return Ok(());
        };

        for credit in credits {
            validate_non_empty("credits.name", &credit.name)?;
            validate_non_empty("credits.role", &credit.role)?;
            if let Some(character) = credit.character.as_ref() {
                validate_non_empty("credits.character", character)?;
            }
            validate_external_ids("credits.external_ids", &credit.external_ids)?;
        }

        Ok(())
    }

    fn validate_collections_field(
        &self,
        value: Option<&Vec<AddonMetadataCollection>>,
    ) -> Result<()> {
        let Some(collections) = value else {
            return Ok(());
        };

        for collection in collections {
            validate_non_empty("collections.name", &collection.name)?;
            if let Some(overview) = collection.overview.as_ref() {
                validate_non_empty("collections.overview", overview)?;
            }
            validate_external_ids("collections.external_ids", &collection.external_ids)?;
        }

        Ok(())
    }

    fn validate_studios_field(&self, value: Option<&Vec<AddonMetadataStudio>>) -> Result<()> {
        let Some(studios) = value else {
            return Ok(());
        };

        for studio in studios {
            validate_non_empty("studios.name", &studio.name)?;
            validate_external_ids("studios.external_ids", &studio.external_ids)?;
        }

        Ok(())
    }

    fn validate_external_ids_field(
        &self,
        field: &str,
        value: Option<&Vec<AddonMetadataExternalId>>,
    ) -> Result<()> {
        let Some(external_ids) = value else {
            return Ok(());
        };

        validate_external_ids(field, external_ids)
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

fn normalize_ratings(values: Vec<AddonMetadataContentRating>) -> Vec<ContentRating> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(|rating| {
            Some(ContentRating {
                source: non_empty_trimmed(rating.source)?,
                value: non_empty_trimmed(rating.value)?,
            })
        })
        .filter(|rating| {
            seen.insert(format!(
                "{}\u{1f}{}",
                rating.source.to_lowercase(),
                rating.value.to_lowercase()
            ))
        })
        .collect()
}

fn normalize_images(values: Vec<AddonMetadataImage>) -> Result<Vec<ImageRef>> {
    let mut seen = HashSet::new();
    let mut images = Vec::new();
    for value in values {
        let Some(uri) = non_empty_trimmed(value.uri) else {
            continue;
        };
        let Some(kind) = non_empty_trimmed(value.kind) else {
            continue;
        };
        let Some(provider) = non_empty_trimmed(value.provider) else {
            continue;
        };
        let image = ImageRef {
            kind: normalize_image_kind(kind),
            uri,
            provider: normalize_external_provider(provider),
            width: value.width,
            height: value.height,
            language: value.language.and_then(normalize_optional_text),
        };
        if seen.insert(format!(
            "{}\u{1f}{}",
            image_kind_key(&image.kind),
            image.uri.to_lowercase()
        )) {
            images.push(image);
        }
    }
    Ok(images)
}

fn normalize_credits(values: Vec<AddonMetadataCredit>) -> Result<Vec<Credit>> {
    let mut seen = HashSet::new();
    let mut credits = Vec::new();
    for value in values {
        let Some(name) = non_empty_trimmed(value.name) else {
            continue;
        };
        let Some(role) = non_empty_trimmed(value.role) else {
            continue;
        };
        let credit = Credit {
            name,
            role: normalize_credit_role(role),
            character: value.character.and_then(normalize_optional_text),
            order: value.order,
            external_ids: normalize_external_ids(value.external_ids)?,
        };
        if seen.insert(format!(
            "{}\u{1f}{}\u{1f}{:?}",
            credit.name.to_lowercase(),
            credit.character.clone().unwrap_or_default().to_lowercase(),
            credit.role
        )) {
            credits.push(credit);
        }
    }
    Ok(credits)
}

fn normalize_collections(values: Vec<AddonMetadataCollection>) -> Result<Vec<CollectionRef>> {
    let mut seen = HashSet::new();
    let mut collections = Vec::new();
    for value in values {
        let Some(name) = non_empty_trimmed(value.name) else {
            continue;
        };
        let collection = CollectionRef {
            name,
            overview: value.overview.and_then(normalize_optional_text),
            sort_order: value.sort_order,
            external_ids: normalize_external_ids(value.external_ids)?,
        };
        if seen.insert(collection.name.to_lowercase()) {
            collections.push(collection);
        }
    }
    Ok(collections)
}

fn normalize_studios(values: Vec<AddonMetadataStudio>) -> Result<Vec<StudioRef>> {
    let mut seen = HashSet::new();
    let mut studios = Vec::new();
    for value in values {
        let Some(name) = non_empty_trimmed(value.name) else {
            continue;
        };
        let studio = StudioRef {
            name,
            external_ids: normalize_external_ids(value.external_ids)?,
        };
        if seen.insert(studio.name.to_lowercase()) {
            studios.push(studio);
        }
    }
    Ok(studios)
}

fn normalize_external_ids(values: Vec<AddonMetadataExternalId>) -> Result<Vec<ExternalId>> {
    let mut seen = HashSet::new();
    let mut external_ids = Vec::new();
    for value in values {
        let Some(provider) = non_empty_trimmed(value.provider) else {
            continue;
        };
        let Some(id_value) = non_empty_trimmed(value.value) else {
            continue;
        };
        let external_id = ExternalId {
            provider: normalize_external_provider(provider),
            value: id_value,
        };
        if seen.insert(format!(
            "{}\u{1f}{}",
            external_provider_key(&external_id.provider),
            external_id.value.to_lowercase()
        )) {
            external_ids.push(external_id);
        }
    }
    Ok(external_ids)
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("addon metadata_write payload {field} must not be empty"),
        });
    }

    Ok(())
}

fn validate_positive_optional(field: &str, value: Option<u32>) -> Result<()> {
    if value == Some(0) {
        return Err(NakoError::InvalidInput {
            message: format!("addon metadata_write payload {field} must be greater than zero"),
        });
    }

    Ok(())
}

fn validate_external_ids(field: &str, external_ids: &[AddonMetadataExternalId]) -> Result<()> {
    for external_id in external_ids {
        validate_non_empty(&format!("{field}.provider"), &external_id.provider)?;
        validate_non_empty(&format!("{field}.value"), &external_id.value)?;
    }

    Ok(())
}

fn normalize_external_provider(value: String) -> ExternalProvider {
    match value.to_ascii_lowercase().as_str() {
        "tmdb" => ExternalProvider::Tmdb,
        "douban" => ExternalProvider::Douban,
        "bangumi" => ExternalProvider::Bangumi,
        "imdb" => ExternalProvider::Imdb,
        "local" => ExternalProvider::Local,
        _ => ExternalProvider::Other(value),
    }
}

fn normalize_image_kind(value: String) -> ImageKind {
    match value.to_ascii_lowercase().as_str() {
        "poster" => ImageKind::Poster,
        "backdrop" => ImageKind::Backdrop,
        "logo" => ImageKind::Logo,
        "thumbnail" => ImageKind::Thumbnail,
        "banner" => ImageKind::Banner,
        _ => ImageKind::Other(value),
    }
}

fn normalize_credit_role(value: String) -> CreditRole {
    match value.to_ascii_lowercase().as_str() {
        "actor" => CreditRole::Actor,
        "director" => CreditRole::Director,
        "writer" => CreditRole::Writer,
        "producer" => CreditRole::Producer,
        "creator" => CreditRole::Creator,
        _ => CreditRole::Other(value),
    }
}

fn external_provider_key(provider: &ExternalProvider) -> String {
    match provider {
        ExternalProvider::Tmdb => "tmdb".to_owned(),
        ExternalProvider::Douban => "douban".to_owned(),
        ExternalProvider::Bangumi => "bangumi".to_owned(),
        ExternalProvider::Imdb => "imdb".to_owned(),
        ExternalProvider::Local => "local".to_owned(),
        ExternalProvider::Other(value) => format!("other:{}", value.to_lowercase()),
    }
}

fn image_kind_key(kind: &ImageKind) -> String {
    match kind {
        ImageKind::Poster => "poster".to_owned(),
        ImageKind::Backdrop => "backdrop".to_owned(),
        ImageKind::Logo => "logo".to_owned(),
        ImageKind::Thumbnail => "thumbnail".to_owned(),
        ImageKind::Banner => "banner".to_owned(),
        ImageKind::Other(value) => format!("other:{}", value.to_lowercase()),
    }
}
