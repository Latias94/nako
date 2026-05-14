use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use taru_core::{
    CanonicalMetadata, ExternalId, MediaKind, MetadataField, MetadataFieldLock, Result,
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataLookup {
    pub kind: Option<MediaKind>,
    pub title: String,
    pub year: Option<u16>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidate {
    pub score: f32,
    pub metadata: CanonicalMetadata,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataMergePolicy {
    locked_fields: HashSet<MetadataField>,
}

impl MetadataMergePolicy {
    #[must_use]
    pub fn from_locks(locks: &[MetadataFieldLock]) -> Self {
        Self {
            locked_fields: locks
                .iter()
                .filter(|lock| lock.locked)
                .map(|lock| lock.field)
                .collect(),
        }
    }

    #[must_use]
    pub fn merge(
        &self,
        existing: &CanonicalMetadata,
        incoming: &CanonicalMetadata,
    ) -> CanonicalMetadata {
        let mut merged = existing.clone();

        if !self.is_locked(MetadataField::Title) {
            merged.title = incoming.title.clone();
        }
        if !self.is_locked(MetadataField::OriginalTitle) {
            merged.original_title = incoming.original_title.clone();
        }
        if !self.is_locked(MetadataField::SortTitle) {
            merged.sort_title = incoming.sort_title.clone();
        }
        if !self.is_locked(MetadataField::Overview) {
            merged.overview = incoming.overview.clone();
        }
        if !self.is_locked(MetadataField::ReleaseDate) {
            merged.release_date = incoming.release_date.clone();
        }
        if !self.is_locked(MetadataField::RuntimeMinutes) {
            merged.runtime_minutes = incoming.runtime_minutes;
        }
        if !self.is_locked(MetadataField::Tagline) {
            merged.tagline = incoming.tagline.clone();
        }
        if !self.is_locked(MetadataField::Genres) {
            merged.genres = incoming.genres.clone();
        }
        if !self.is_locked(MetadataField::Ratings) {
            merged.ratings = incoming.ratings.clone();
        }
        if !self.is_locked(MetadataField::Images) {
            merged.images = incoming.images.clone();
        }
        if !self.is_locked(MetadataField::Credits) {
            merged.credits = incoming.credits.clone();
        }
        if !self.is_locked(MetadataField::ExternalIds) {
            merged.external_ids = incoming.external_ids.clone();
        }

        merged
    }

    fn is_locked(&self, field: MetadataField) -> bool {
        self.locked_fields.contains(&field)
    }
}

#[cfg(test)]
mod tests {
    use taru_core::{MediaItemId, MetadataSource};

    use super::*;

    #[test]
    fn merge_preserves_locked_fields() {
        let item_id = MediaItemId::new();
        let policy = MetadataMergePolicy::from_locks(&[
            MetadataFieldLock {
                item_id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            },
            MetadataFieldLock {
                item_id,
                field: MetadataField::Genres,
                locked: true,
                source: MetadataSource::Nfo,
            },
        ]);
        let existing = CanonicalMetadata {
            title: "Local Title".to_owned(),
            overview: Some("old".to_owned()),
            genres: vec!["Local".to_owned()],
            ..CanonicalMetadata::default()
        };
        let incoming = CanonicalMetadata {
            title: "Provider Title".to_owned(),
            overview: Some("new".to_owned()),
            genres: vec!["Action".to_owned()],
            tagline: Some("Wake up.".to_owned()),
            ..CanonicalMetadata::default()
        };

        let merged = policy.merge(&existing, &incoming);

        assert_eq!(merged.title, "Local Title");
        assert_eq!(merged.overview, Some("new".to_owned()));
        assert_eq!(merged.genres, vec!["Local"]);
        assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
    }
}
