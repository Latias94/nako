use serde::{Deserialize, Serialize};
use taru_catalog::hydrate_item_catalog;
use taru_core::{
    CanonicalMetadata, CatalogRepository, Credit, CreditRole, ExternalId, ExternalProvider,
    ImageKind, ImageRef, JobId, LocalMetadataPolicy, MediaItem, MediaItemId, MediaKind,
    MediaRepository, MediaSource, MediaSourceId, MetadataField, MetadataFieldLock,
    MetadataRepository, MetadataSource, PageRequest, Result, TaruError,
};
use taru_search::SearchIndex;
use taru_vfs::{StorageBackend, StorageUri};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoDocument {
    pub metadata: CanonicalMetadata,
    pub external_ids: Vec<ExternalId>,
}

pub trait NfoCodec: Send + Sync {
    fn parse(&self, xml: &str) -> Result<NfoDocument>;

    fn render(&self, document: &NfoDocument) -> Result<String>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoSidecar {
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub source_locator: String,
    pub nfo_uri: StorageUri,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoJobInput {
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportRequest {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportRequest {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportSummary {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub scanned_sources: u64,
    pub discovered_nfo: u64,
    pub imported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportSummary {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub scanned_sources: u64,
    pub exported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoFailure {
    pub source_id: MediaSourceId,
    pub locator: String,
    pub message: String,
}

#[derive(Debug)]
pub struct NfoService<B, R, C> {
    backend: B,
    repository: R,
    codec: C,
}

impl<B, R, C> NfoService<B, R, C> {
    pub fn new(backend: B, repository: R, codec: C) -> Self {
        Self {
            backend,
            repository,
            codec,
        }
    }
}

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: CatalogRepository + MediaRepository + MetadataRepository + SearchIndex,
    C: NfoCodec,
{
    pub async fn discover_sidecars(
        &self,
        library_id: taru_core::LibraryId,
    ) -> Result<Vec<NfoSidecar>> {
        let sources = self.list_all_sources(library_id).await?;
        let mut sidecars = Vec::new();

        for source in sources {
            let nfo_uri = nfo_uri_for_source(&source)?;
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => {
                    sidecars.push(NfoSidecar {
                        source_id: source.id,
                        item_id: source.item_id,
                        source_locator: source.locator,
                        nfo_uri,
                    });
                }
                Err(TaruError::NotFound { .. }) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(sidecars)
    }

    pub async fn import_library(&self, request: NfoImportRequest) -> Result<NfoImportSummary> {
        ensure_import_policy(request.policy)?;

        let sources = self.list_all_sources(request.library_id).await?;
        let mut summary = NfoImportSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            scanned_sources: sources.len() as u64,
            discovered_nfo: 0,
            imported_items: 0,
            skipped_items: 0,
            failed_items: 0,
            failures: Vec::new(),
        };

        for source in sources {
            match self
                .import_source(source, request.policy, request.force)
                .await
            {
                NfoImportOutcome::Imported => {
                    summary.discovered_nfo += 1;
                    summary.imported_items += 1;
                }
                NfoImportOutcome::Skipped { discovered } => {
                    if discovered {
                        summary.discovered_nfo += 1;
                    }
                    summary.skipped_items += 1;
                }
                NfoImportOutcome::Failed(failure) => {
                    summary.failed_items += 1;
                    summary.failures.push(failure);
                }
            }
        }

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(summary)
    }

    pub async fn export_library(&self, request: NfoExportRequest) -> Result<NfoExportSummary> {
        ensure_export_policy(request.policy)?;

        let sources = self.list_all_sources(request.library_id).await?;
        let mut summary = NfoExportSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            scanned_sources: sources.len() as u64,
            exported_items: 0,
            skipped_items: 0,
            failed_items: 0,
            failures: Vec::new(),
        };

        for source in sources {
            match self.export_source(source, request.force).await {
                NfoExportOutcome::Exported => summary.exported_items += 1,
                NfoExportOutcome::Skipped => summary.skipped_items += 1,
                NfoExportOutcome::Failed(failure) => {
                    summary.failed_items += 1;
                    summary.failures.push(failure);
                }
            }
        }

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(summary)
    }

    async fn list_all_sources(&self, library_id: taru_core::LibraryId) -> Result<Vec<MediaSource>> {
        let mut offset = 0;
        let mut sources = Vec::new();

        loop {
            let page = self
                .repository
                .list_media_sources(
                    library_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = page.len();
            sources.extend(page);

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(sources)
    }

    async fn import_source(
        &self,
        source: MediaSource,
        policy: LocalMetadataPolicy,
        force: bool,
    ) -> NfoImportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return import_failure(&source, err),
        };
        let xml = match self.backend.read_to_string(&nfo_uri).await {
            Ok(xml) => xml,
            Err(TaruError::NotFound { .. }) => {
                return NfoImportOutcome::Skipped { discovered: false };
            }
            Err(err) => return import_failure(&source, err),
        };
        let document = match self.codec.parse(&xml) {
            Ok(document) => document,
            Err(err) => return import_failure(&source, err),
        };
        let existing = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return import_failure(
                    &source,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return import_failure(&source, err),
        };

        if !force && policy == LocalMetadataPolicy::RemoteFirst && !is_missing_metadata(&existing) {
            return NfoImportOutcome::Skipped { discovered: true };
        }

        let locks = match self.repository.list_field_locks(existing.id).await {
            Ok(locks) => locks,
            Err(err) => return import_failure(&source, err),
        };
        let merged = merge_nfo_metadata(&existing.metadata, &document.metadata, policy, &locks);
        let changed = merged != existing.metadata;
        if !changed && !force {
            if let Err(err) =
                hydrate_item_catalog(&self.repository, existing.id, MetadataSource::Nfo).await
            {
                return import_failure(&source, err);
            }
            return NfoImportOutcome::Skipped { discovered: true };
        }

        let updated = MediaItem {
            metadata: merged,
            ..existing
        };
        if let Err(err) = self.repository.upsert_media_item(&updated).await {
            return import_failure(&source, err);
        }

        if locks_should_be_written(policy) {
            for field in populated_fields(&document.metadata) {
                if let Err(err) = self
                    .repository
                    .upsert_field_lock(&MetadataFieldLock {
                        item_id: updated.id,
                        field,
                        locked: true,
                        source: MetadataSource::Nfo,
                    })
                    .await
                {
                    return import_failure(&source, err);
                }
            }
        }

        if let Err(err) =
            hydrate_item_catalog(&self.repository, updated.id, MetadataSource::Nfo).await
        {
            return import_failure(&source, err);
        }

        NfoImportOutcome::Imported
    }

    async fn export_source(&self, source: MediaSource, force: bool) -> NfoExportOutcome {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => return export_failure(&source, err),
        };
        if !force {
            match self.backend.stat(&nfo_uri).await {
                Ok(_) => return NfoExportOutcome::Skipped,
                Err(TaruError::NotFound { .. }) => {}
                Err(err) => return export_failure(&source, err),
            }
        }

        let item = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return export_failure(
                    &source,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => return export_failure(&source, err),
        };

        if item.kind != MediaKind::Movie {
            return NfoExportOutcome::Skipped;
        }

        let xml = match self.codec.render(&NfoDocument {
            metadata: item.metadata,
            external_ids: Vec::new(),
        }) {
            Ok(xml) => xml,
            Err(err) => return export_failure(&source, err),
        };

        match self.backend.write_string(&nfo_uri, &xml).await {
            Ok(()) => NfoExportOutcome::Exported,
            Err(err) => export_failure(&source, err),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MovieNfoCodec;

impl NfoCodec for MovieNfoCodec {
    fn parse(&self, xml: &str) -> Result<NfoDocument> {
        let metadata = CanonicalMetadata {
            title: required_tag(xml, "title")?,
            original_title: optional_tag(xml, "originaltitle"),
            sort_title: optional_tag(xml, "sorttitle"),
            overview: optional_tag(xml, "plot"),
            release_date: optional_tag(xml, "releasedate").or_else(|| optional_tag(xml, "year")),
            runtime_minutes: optional_tag(xml, "runtime").and_then(|value| value.parse().ok()),
            tagline: optional_tag(xml, "tagline"),
            genres: tags(xml, "genre"),
            tags: tags(xml, "tag"),
            images: images_from_nfo(xml),
            credits: credits_from_nfo(xml),
            ..CanonicalMetadata::default()
        };

        Ok(NfoDocument {
            metadata,
            external_ids: Vec::new(),
        })
    }

    fn render(&self, document: &NfoDocument) -> Result<String> {
        let metadata = &document.metadata;
        let mut output = String::from("<movie>\n");

        push_tag(&mut output, "title", Some(&metadata.title));
        push_tag(
            &mut output,
            "originaltitle",
            metadata.original_title.as_deref(),
        );
        push_tag(&mut output, "sorttitle", metadata.sort_title.as_deref());
        push_tag(&mut output, "plot", metadata.overview.as_deref());
        push_tag(&mut output, "releasedate", metadata.release_date.as_deref());
        if let Some(runtime) = metadata.runtime_minutes {
            push_tag(&mut output, "runtime", Some(&runtime.to_string()));
        }
        push_tag(&mut output, "tagline", metadata.tagline.as_deref());
        for genre in &metadata.genres {
            push_tag(&mut output, "genre", Some(genre));
        }
        for tag in &metadata.tags {
            push_tag(&mut output, "tag", Some(tag));
        }
        for credit in &metadata.credits {
            match &credit.role {
                CreditRole::Actor => {
                    output.push_str("  <actor>\n");
                    push_tag(&mut output, "name", Some(&credit.name));
                    push_tag(&mut output, "role", credit.character.as_deref());
                    if let Some(order) = credit.order {
                        push_tag(&mut output, "order", Some(&order.to_string()));
                    }
                    output.push_str("  </actor>\n");
                }
                CreditRole::Director => push_tag(&mut output, "director", Some(&credit.name)),
                CreditRole::Writer => push_tag(&mut output, "writer", Some(&credit.name)),
                _ => {}
            }
        }
        for image in &metadata.images {
            match &image.kind {
                ImageKind::Poster => push_tag(&mut output, "poster", Some(&image.uri)),
                ImageKind::Backdrop => push_tag(&mut output, "fanart", Some(&image.uri)),
                ImageKind::Thumbnail => push_tag(&mut output, "thumb", Some(&image.uri)),
                _ => {}
            }
        }

        output.push_str("</movie>\n");
        Ok(output)
    }
}

enum NfoImportOutcome {
    Imported,
    Skipped { discovered: bool },
    Failed(NfoFailure),
}

enum NfoExportOutcome {
    Exported,
    Skipped,
    Failed(NfoFailure),
}

fn import_failure(source: &MediaSource, err: impl ToString) -> NfoImportOutcome {
    NfoImportOutcome::Failed(NfoFailure {
        source_id: source.id,
        locator: source.locator.clone(),
        message: err.to_string(),
    })
}

fn export_failure(source: &MediaSource, err: impl ToString) -> NfoExportOutcome {
    NfoExportOutcome::Failed(NfoFailure {
        source_id: source.id,
        locator: source.locator.clone(),
        message: err.to_string(),
    })
}

fn ensure_import_policy(policy: LocalMetadataPolicy) -> Result<()> {
    match policy {
        LocalMetadataPolicy::Disabled | LocalMetadataPolicy::WriteSidecar => {
            Err(TaruError::Unsupported(
                "NFO import requires read-only, local-first, or remote-first local metadata policy",
            ))
        }
        LocalMetadataPolicy::ReadOnly
        | LocalMetadataPolicy::LocalFirst
        | LocalMetadataPolicy::RemoteFirst => Ok(()),
    }
}

fn ensure_export_policy(policy: LocalMetadataPolicy) -> Result<()> {
    if policy == LocalMetadataPolicy::WriteSidecar {
        Ok(())
    } else {
        Err(TaruError::Unsupported(
            "NFO export requires write-sidecar local metadata policy",
        ))
    }
}

fn nfo_uri_for_source(source: &MediaSource) -> Result<StorageUri> {
    let uri = StorageUri::parse(&source.locator)?;
    let path = uri.path_part();
    let Some((stem, _extension)) = path.rsplit_once('.') else {
        return Err(TaruError::InvalidInput {
            message: format!(
                "media source has no extension for NFO sidecar: {}",
                source.locator
            ),
        });
    };

    StorageUri::parse(format!("{}://{stem}.nfo", uri.scheme()))
}

fn merge_nfo_metadata(
    existing: &CanonicalMetadata,
    incoming: &CanonicalMetadata,
    policy: LocalMetadataPolicy,
    locks: &[MetadataFieldLock],
) -> CanonicalMetadata {
    match policy {
        LocalMetadataPolicy::ReadOnly
        | LocalMetadataPolicy::LocalFirst
        | LocalMetadataPolicy::WriteSidecar => merge_with_mode(existing, incoming, locks, false),
        LocalMetadataPolicy::RemoteFirst => merge_with_mode(existing, incoming, locks, true),
        LocalMetadataPolicy::Disabled => existing.clone(),
    }
}

fn merge_with_mode(
    existing: &CanonicalMetadata,
    incoming: &CanonicalMetadata,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> CanonicalMetadata {
    let mut merged = existing.clone();

    if should_replace_text(MetadataField::Title, &merged.title, locks, missing_only) {
        merged.title = incoming.title.clone();
    }
    if should_replace_option(
        MetadataField::OriginalTitle,
        &merged.original_title,
        locks,
        missing_only,
    ) {
        merged.original_title = incoming.original_title.clone();
    }
    if should_replace_option(
        MetadataField::SortTitle,
        &merged.sort_title,
        locks,
        missing_only,
    ) {
        merged.sort_title = incoming.sort_title.clone();
    }
    if should_replace_option(
        MetadataField::Overview,
        &merged.overview,
        locks,
        missing_only,
    ) {
        merged.overview = incoming.overview.clone();
    }
    if should_replace_option(
        MetadataField::ReleaseDate,
        &merged.release_date,
        locks,
        missing_only,
    ) {
        merged.release_date = incoming.release_date.clone();
    }
    if should_replace_option(
        MetadataField::RuntimeMinutes,
        &merged.runtime_minutes,
        locks,
        missing_only,
    ) {
        merged.runtime_minutes = incoming.runtime_minutes;
    }
    if should_replace_option(MetadataField::Tagline, &merged.tagline, locks, missing_only) {
        merged.tagline = incoming.tagline.clone();
    }
    if should_replace_list(MetadataField::Genres, &merged.genres, locks, missing_only) {
        merged.genres = incoming.genres.clone();
    }
    if should_replace_list(MetadataField::Tags, &merged.tags, locks, missing_only) {
        merged.tags = incoming.tags.clone();
    }
    if should_replace_list(MetadataField::Ratings, &merged.ratings, locks, missing_only) {
        merged.ratings = incoming.ratings.clone();
    }
    if should_replace_list(MetadataField::Images, &merged.images, locks, missing_only) {
        merged.images = incoming.images.clone();
    }
    if should_replace_list(MetadataField::Credits, &merged.credits, locks, missing_only) {
        merged.credits = incoming.credits.clone();
    }
    if should_replace_list(
        MetadataField::Collections,
        &merged.collections,
        locks,
        missing_only,
    ) {
        merged.collections = incoming.collections.clone();
    }
    if should_replace_list(MetadataField::Studios, &merged.studios, locks, missing_only) {
        merged.studios = incoming.studios.clone();
    }
    if should_replace_list(
        MetadataField::ExternalIds,
        &merged.external_ids,
        locks,
        missing_only,
    ) {
        merged.external_ids = incoming.external_ids.clone();
    }

    merged
}

fn should_replace_text(
    field: MetadataField,
    existing: &str,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_empty())
}

fn should_replace_option<T>(
    field: MetadataField,
    existing: &Option<T>,
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_none())
}

fn should_replace_list<T>(
    field: MetadataField,
    existing: &[T],
    locks: &[MetadataFieldLock],
    missing_only: bool,
) -> bool {
    !is_protected_by_non_nfo_lock(field, locks) && (!missing_only || existing.is_empty())
}

fn is_protected_by_non_nfo_lock(field: MetadataField, locks: &[MetadataFieldLock]) -> bool {
    locks.iter().any(|lock| {
        lock.locked && lock.field == field && !matches!(lock.source, MetadataSource::Nfo)
    })
}

fn is_missing_metadata(item: &MediaItem) -> bool {
    let metadata = &item.metadata;
    metadata.title.trim().is_empty()
        || metadata.overview.is_none()
        || metadata.release_date.is_none()
        || metadata.runtime_minutes.is_none()
        || metadata.genres.is_empty()
        || metadata.tags.is_empty()
}

fn locks_should_be_written(policy: LocalMetadataPolicy) -> bool {
    matches!(
        policy,
        LocalMetadataPolicy::ReadOnly | LocalMetadataPolicy::LocalFirst
    )
}

fn populated_fields(metadata: &CanonicalMetadata) -> Vec<MetadataField> {
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

fn required_tag(xml: &str, name: &str) -> Result<String> {
    optional_tag(xml, name).ok_or_else(|| TaruError::InvalidInput {
        message: format!("NFO is missing required <{name}> tag"),
    })
}

fn optional_tag(xml: &str, name: &str) -> Option<String> {
    tags(xml, name).into_iter().next()
}

fn tags(xml: &str, name: &str) -> Vec<String> {
    element_blocks(xml, name)
        .into_iter()
        .map(|value| unescape_xml(value.trim()))
        .collect()
}

fn element_blocks<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let open_prefix = format!("<{name}");
    let close = format!("</{name}>");
    let mut values = Vec::new();
    let mut remaining = xml;

    while let Some(open_start) = remaining.find(&open_prefix) {
        let after_prefix = &remaining[open_start + open_prefix.len()..];
        let Some(next) = after_prefix.chars().next() else {
            break;
        };
        if next != '>' && !next.is_whitespace() {
            remaining = after_prefix;
            continue;
        }
        let Some(open_end) = after_prefix.find('>') else {
            break;
        };
        let after_open = &after_prefix[open_end + 1..];
        let Some((value, after_close)) = after_open.split_once(&close) else {
            break;
        };
        values.push(value);
        remaining = after_close;
    }

    values
}

fn credits_from_nfo(xml: &str) -> Vec<Credit> {
    let mut credits = Vec::new();

    for block in element_blocks(xml, "actor") {
        let Some(name) = optional_tag(block, "name") else {
            continue;
        };
        credits.push(Credit {
            name,
            role: CreditRole::Actor,
            character: optional_tag(block, "role"),
            order: optional_tag(block, "order").and_then(|value| value.parse().ok()),
            external_ids: Vec::new(),
        });
    }

    for director in tags(xml, "director") {
        credits.push(Credit {
            name: director,
            role: CreditRole::Director,
            character: None,
            order: None,
            external_ids: Vec::new(),
        });
    }

    for writer in tags(xml, "writer") {
        credits.push(Credit {
            name: writer,
            role: CreditRole::Writer,
            character: None,
            order: None,
            external_ids: Vec::new(),
        });
    }

    credits
}

fn images_from_nfo(xml: &str) -> Vec<ImageRef> {
    let mut images = Vec::new();

    for uri in tags(xml, "poster") {
        push_nfo_image(&mut images, ImageKind::Poster, uri);
    }
    for uri in tags(xml, "thumb") {
        push_nfo_image(&mut images, ImageKind::Thumbnail, uri);
    }
    for block in element_blocks(xml, "fanart") {
        let thumbs = tags(block, "thumb");
        if thumbs.is_empty() {
            push_nfo_image(&mut images, ImageKind::Backdrop, unescape_xml(block.trim()));
        } else {
            for uri in thumbs {
                push_nfo_image(&mut images, ImageKind::Backdrop, uri);
            }
        }
    }

    images
}

fn push_nfo_image(images: &mut Vec<ImageRef>, kind: ImageKind, uri: String) {
    let uri = uri.trim();

    if uri.is_empty()
        || images
            .iter()
            .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri: uri.to_owned(),
        provider: ExternalProvider::Local,
        width: None,
        height: None,
        language: None,
    });
}

fn push_tag(output: &mut String, name: &str, value: Option<&str>) {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return;
    };

    output.push_str("  <");
    output.push_str(name);
    output.push('>');
    output.push_str(&escape_xml(value));
    output.push_str("</");
    output.push_str(name);
    output.push_str(">\n");
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use taru_core::{
        Library, LibraryId, LibraryOptions, LibraryPreset, MediaRepository, PageRequest,
        TransactionManager,
        repository::{CatalogRepository, LibraryRepository, MetadataRepository},
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchIndex, SearchQuery};
    use taru_vfs::LocalFsBackend;

    use super::*;

    #[test]
    fn movie_nfo_round_trips_core_fields() {
        let document = NfoDocument {
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                original_title: Some("The Matrix".to_owned()),
                sort_title: Some("Matrix, The".to_owned()),
                overview: Some("A hacker discovers reality.".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                runtime_minutes: Some(136),
                tagline: Some("Welcome to the Real World".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                tags: vec!["cyberpunk".to_owned()],
                credits: vec![Credit {
                    name: "Keanu Reeves".to_owned(),
                    role: CreditRole::Actor,
                    character: Some("Neo".to_owned()),
                    order: Some(0),
                    external_ids: Vec::new(),
                }],
                ..CanonicalMetadata::default()
            },
            external_ids: Vec::new(),
        };
        let codec = MovieNfoCodec;

        let xml = codec.render(&document).unwrap();
        let parsed = codec.parse(&xml).unwrap();

        assert_eq!(parsed.metadata.title, "The Matrix");
        assert_eq!(parsed.metadata.sort_title, Some("Matrix, The".to_owned()));
        assert_eq!(parsed.metadata.runtime_minutes, Some(136));
        assert_eq!(
            parsed.metadata.genres,
            vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
        assert_eq!(parsed.metadata.tags, vec!["cyberpunk".to_owned()]);
        assert_eq!(parsed.metadata.credits[0].name, "Keanu Reeves");
    }

    #[tokio::test]
    async fn nfo_service_discovers_and_imports_movie_sidecar_with_locks() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies").join("Demo")).unwrap();
        fs::write(
            temp.path().join("Movies").join("Demo").join("demo.mkv"),
            b"media",
        )
        .unwrap();
        fs::write(
            temp.path().join("Movies").join("Demo").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
  <releasedate>1999-03-31</releasedate>
  <runtime>136</runtime>
  <genre>Action</genre>
  <tag>cyberpunk</tag>
  <actor>
    <name>Demo Actor</name>
    <role>Lead</role>
    <order>0</order>
  </actor>
  <director>Demo Director</director>
  <poster>local:///Movies/Demo/poster.jpg</poster>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/Demo/demo.mkv").await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let sidecars = service.discover_sidecars(library_id).await.unwrap();
        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        let people = store.list_people(PageRequest::first_page()).await.unwrap();
        let tags = store.list_tags(PageRequest::first_page()).await.unwrap();
        let images = store.list_item_images(item.id).await.unwrap();
        let hits = store
            .search(SearchQuery {
                query: "Demo Actor".to_owned(),
                facets: vec!["tag:cyberpunk".to_owned()],
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(sidecars.len(), 1);
        assert_eq!(
            sidecars[0].nfo_uri.as_str(),
            "local:///Movies/Demo/demo.nfo"
        );
        assert_eq!(summary.scanned_sources, 1);
        assert_eq!(summary.discovered_nfo, 1);
        assert_eq!(summary.imported_items, 1);
        assert_eq!(loaded.metadata.title, "NFO Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert_eq!(loaded.metadata.tags, vec!["cyberpunk"]);
        assert!(people.iter().any(|person| person.name == "Demo Actor"));
        assert_eq!(tags[0].name, "cyberpunk");
        assert_eq!(images[0].source_uri, "local:///Movies/Demo/poster.jpg");
        assert_eq!(hits[0].item_id, item.id);
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
        }));
    }

    #[tokio::test]
    async fn nfo_service_remote_first_import_only_fills_missing_fields_without_locks() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
  <genre>Action</genre>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Remote Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::RemoteFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();

        assert_eq!(summary.imported_items, 1);
        assert_eq!(loaded.metadata.title, "Remote Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.is_empty());
    }

    #[tokio::test]
    async fn nfo_service_preserves_user_locked_fields_during_import() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/demo.mkv").await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            })
            .await
            .unwrap();
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        assert_eq!(loaded.metadata.title, "File Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
    }

    #[tokio::test]
    async fn nfo_service_exports_movie_sidecar_when_policy_allows_writing() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Exported Title".to_owned(),
                overview: Some("Exported overview".to_owned()),
                genres: vec!["Action".to_owned()],
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: false,
            })
            .await
            .unwrap();

        let xml = fs::read_to_string(temp.path().join("Movies").join("demo.nfo")).unwrap();
        assert_eq!(summary.exported_items, 1);
        assert!(xml.contains("<title>Exported Title</title>"));
        assert!(xml.contains("<genre>Action</genre>"));
    }

    #[tokio::test]
    async fn nfo_service_rejects_export_without_write_sidecar_policy() {
        let temp = tempfile::tempdir().unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let service = NfoService::new(
            LocalFsBackend::new(temp.path()).unwrap(),
            store,
            MovieNfoCodec,
        );

        let err = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap_err();

        assert_eq!(
            err,
            TaruError::Unsupported("NFO export requires write-sidecar local metadata policy")
        );
    }

    async fn seed_item(store: &SqliteStore, library_id: LibraryId, locator: &str) -> MediaItem {
        seed_item_with_metadata(
            store,
            library_id,
            locator,
            CanonicalMetadata {
                title: "File Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await
    }

    async fn seed_item_with_metadata(
        store: &SqliteStore,
        library_id: LibraryId,
        locator: &str,
        metadata: CanonicalMetadata,
    ) -> MediaItem {
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata,
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: locator.to_owned(),
            file_name: locator.rsplit('/').next().unwrap_or(locator).to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        };
        let library = Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        item
    }
}
