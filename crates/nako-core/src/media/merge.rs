use crate::{
    CanonicalMetadata, LocalMetadataPolicy, MetadataField, MetadataFieldLock, MetadataRefreshMode,
    MetadataSource,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataMergePolicy {
    protected_fields: Vec<MetadataField>,
    mode: MetadataMergeMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataMergeMode {
    FullRefresh,
    MissingOnly,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataLockScope<'a> {
    All,
    OtherSources(&'a MetadataSource),
}

impl MetadataMergePolicy {
    #[must_use]
    pub fn from_locks(locks: &[MetadataFieldLock]) -> Self {
        Self::from_locks_and_mode(locks, MetadataRefreshMode::FullRefresh)
    }

    #[must_use]
    pub fn from_locks_and_mode(locks: &[MetadataFieldLock], mode: MetadataRefreshMode) -> Self {
        Self::from_parts(locks, MetadataLockScope::All, MetadataMergeMode::from(mode))
    }

    #[must_use]
    pub fn for_source(
        locks: &[MetadataFieldLock],
        source: &MetadataSource,
        mode: MetadataMergeMode,
    ) -> Self {
        Self::from_parts(locks, MetadataLockScope::OtherSources(source), mode)
    }

    #[must_use]
    pub fn for_source_refresh_mode(
        locks: &[MetadataFieldLock],
        source: &MetadataSource,
        mode: MetadataRefreshMode,
    ) -> Self {
        Self::for_source(locks, source, MetadataMergeMode::from(mode))
    }

    #[must_use]
    pub fn for_nfo_import(policy: LocalMetadataPolicy, locks: &[MetadataFieldLock]) -> Self {
        let mode = match policy {
            LocalMetadataPolicy::Disabled => MetadataMergeMode::Disabled,
            LocalMetadataPolicy::RemoteFirst => MetadataMergeMode::MissingOnly,
            LocalMetadataPolicy::ReadOnly
            | LocalMetadataPolicy::LocalFirst
            | LocalMetadataPolicy::WriteSidecar => MetadataMergeMode::FullRefresh,
        };

        Self::for_source(locks, &MetadataSource::Nfo, mode)
    }

    #[must_use]
    pub fn from_parts(
        locks: &[MetadataFieldLock],
        lock_scope: MetadataLockScope<'_>,
        mode: MetadataMergeMode,
    ) -> Self {
        let mut protected_fields = Vec::new();
        for lock in locks {
            if !lock.locked || !lock_scope.protects(&lock.source) {
                continue;
            }
            if !protected_fields.contains(&lock.field) {
                protected_fields.push(lock.field);
            }
        }

        Self {
            protected_fields,
            mode,
        }
    }

    #[must_use]
    pub fn merge(
        &self,
        existing: &CanonicalMetadata,
        incoming: &CanonicalMetadata,
    ) -> CanonicalMetadata {
        if self.mode == MetadataMergeMode::Disabled {
            return existing.clone();
        }

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

    fn is_protected(&self, field: MetadataField) -> bool {
        self.protected_fields.contains(&field)
    }

    fn should_replace_text(&self, field: MetadataField, existing: &str) -> bool {
        !self.is_protected(field)
            && (self.mode != MetadataMergeMode::MissingOnly || existing.is_empty())
    }

    fn should_replace_option<T>(&self, field: MetadataField, existing: &Option<T>) -> bool {
        !self.is_protected(field)
            && (self.mode != MetadataMergeMode::MissingOnly || existing.is_none())
    }

    fn should_replace_list<T>(&self, field: MetadataField, existing: &[T]) -> bool {
        !self.is_protected(field)
            && (self.mode != MetadataMergeMode::MissingOnly || existing.is_empty())
    }
}

impl From<MetadataRefreshMode> for MetadataMergeMode {
    fn from(value: MetadataRefreshMode) -> Self {
        match value {
            MetadataRefreshMode::None | MetadataRefreshMode::ValidationOnly => Self::Disabled,
            MetadataRefreshMode::Default | MetadataRefreshMode::FullRefresh => Self::FullRefresh,
            MetadataRefreshMode::MissingOnly => Self::MissingOnly,
        }
    }
}

impl MetadataLockScope<'_> {
    fn protects(self, source: &MetadataSource) -> bool {
        match self {
            Self::All => true,
            Self::OtherSources(current_source) => source != current_source,
        }
    }
}

#[must_use]
pub fn populated_metadata_fields(metadata: &CanonicalMetadata) -> Vec<MetadataField> {
    let mut fields = Vec::new();

    if !metadata.title.trim().is_empty() {
        fields.push(MetadataField::Title);
    }
    if metadata.original_title.is_some() {
        fields.push(MetadataField::OriginalTitle);
    }
    if metadata.sort_title.is_some() {
        fields.push(MetadataField::SortTitle);
    }
    if metadata.overview.is_some() {
        fields.push(MetadataField::Overview);
    }
    if metadata.release_date.is_some() {
        fields.push(MetadataField::ReleaseDate);
    }
    if metadata.runtime_minutes.is_some() {
        fields.push(MetadataField::RuntimeMinutes);
    }
    if metadata.tagline.is_some() {
        fields.push(MetadataField::Tagline);
    }
    if !metadata.genres.is_empty() {
        fields.push(MetadataField::Genres);
    }
    if !metadata.tags.is_empty() {
        fields.push(MetadataField::Tags);
    }
    if !metadata.ratings.is_empty() {
        fields.push(MetadataField::Ratings);
    }
    if !metadata.images.is_empty() {
        fields.push(MetadataField::Images);
    }
    if !metadata.credits.is_empty() {
        fields.push(MetadataField::Credits);
    }
    if !metadata.collections.is_empty() {
        fields.push(MetadataField::Collections);
    }
    if !metadata.studios.is_empty() {
        fields.push(MetadataField::Studios);
    }
    if !metadata.external_ids.is_empty() {
        fields.push(MetadataField::ExternalIds);
    }

    fields
}
