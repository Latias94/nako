use std::collections::HashSet;

use taru_core::{CanonicalMetadata, MetadataField, MetadataFieldLock, MetadataRefreshMode};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataMergePolicy {
    locked_fields: HashSet<MetadataField>,
    mode: MetadataRefreshMode,
}

impl MetadataMergePolicy {
    #[must_use]
    pub fn from_locks(locks: &[MetadataFieldLock]) -> Self {
        Self::from_locks_and_mode(locks, MetadataRefreshMode::FullRefresh)
    }

    #[must_use]
    pub fn from_locks_and_mode(locks: &[MetadataFieldLock], mode: MetadataRefreshMode) -> Self {
        Self {
            locked_fields: locks
                .iter()
                .filter(|lock| lock.locked)
                .map(|lock| lock.field)
                .collect(),
            mode,
        }
    }

    #[must_use]
    pub fn merge(
        &self,
        existing: &CanonicalMetadata,
        incoming: &CanonicalMetadata,
    ) -> CanonicalMetadata {
        let mut merged = existing.clone();

        if self.should_replace_text(MetadataField::Title, &existing.title) {
            merged.title = incoming.title.clone();
        }
        if self.should_replace_option(MetadataField::OriginalTitle, &existing.original_title) {
            merged.original_title = incoming.original_title.clone();
        }
        if self.should_replace_option(MetadataField::SortTitle, &existing.sort_title) {
            merged.sort_title = incoming.sort_title.clone();
        }
        if self.should_replace_option(MetadataField::Overview, &existing.overview) {
            merged.overview = incoming.overview.clone();
        }
        if self.should_replace_option(MetadataField::ReleaseDate, &existing.release_date) {
            merged.release_date = incoming.release_date.clone();
        }
        if self.should_replace_option(MetadataField::RuntimeMinutes, &existing.runtime_minutes) {
            merged.runtime_minutes = incoming.runtime_minutes;
        }
        if self.should_replace_option(MetadataField::Tagline, &existing.tagline) {
            merged.tagline = incoming.tagline.clone();
        }
        if self.should_replace_list(MetadataField::Genres, &existing.genres) {
            merged.genres = incoming.genres.clone();
        }
        if self.should_replace_list(MetadataField::Tags, &existing.tags) {
            merged.tags = incoming.tags.clone();
        }
        if self.should_replace_list(MetadataField::Ratings, &existing.ratings) {
            merged.ratings = incoming.ratings.clone();
        }
        if self.should_replace_list(MetadataField::Images, &existing.images) {
            merged.images = incoming.images.clone();
        }
        if self.should_replace_list(MetadataField::Credits, &existing.credits) {
            merged.credits = incoming.credits.clone();
        }
        if self.should_replace_list(MetadataField::Collections, &existing.collections) {
            merged.collections = incoming.collections.clone();
        }
        if self.should_replace_list(MetadataField::Studios, &existing.studios) {
            merged.studios = incoming.studios.clone();
        }
        if self.should_replace_list(MetadataField::ExternalIds, &existing.external_ids) {
            merged.external_ids = incoming.external_ids.clone();
        }

        merged
    }

    fn is_locked(&self, field: MetadataField) -> bool {
        self.locked_fields.contains(&field)
    }

    fn should_replace_text(&self, field: MetadataField, existing: &str) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_empty())
    }

    fn should_replace_option<T>(&self, field: MetadataField, existing: &Option<T>) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_none())
    }

    fn should_replace_list<T>(&self, field: MetadataField, existing: &[T]) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_empty())
    }
}
