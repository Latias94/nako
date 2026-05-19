use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures_util::StreamExt;
use image::GenericImageView;
use serde::Serialize;
use sha2::{Digest, Sha256};
use taru_api::{AcceptManagedArtworkCandidateResponse, ProcessManagedArtworkIngestResponse};
use taru_core::{
    ArtworkCandidateId, ArtworkCandidateRepository, ArtworkCandidateSourceKind,
    ArtworkCandidateStatus, JobId, JobKind, LibraryItemRepository, ManagedArtworkArtifactId,
    ManagedArtworkIngestClaimRecord, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    ManagedArtworkRepository, MediaRepository, NewJob, NewManagedArtworkArtifact,
    NewManagedArtworkIngest, Result, TaruError,
};
use taru_db::SqliteStore;
use tokio::{fs, io::AsyncWriteExt, sync::Semaphore};

use crate::config::ArtworkConfig;

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkAppService {
    store: SqliteStore,
    fetcher: ManagedArtworkFetcher,
    validator: ManagedArtworkImageValidator,
    artifact_store: LocalManagedArtworkArtifactStore,
}

impl ManagedArtworkAppService {
    pub(crate) fn new(config: ArtworkConfig, store: SqliteStore) -> Result<Self> {
        Ok(Self {
            store,
            fetcher: ManagedArtworkFetcher::new(config.clone())?,
            validator: ManagedArtworkImageValidator::new(config.clone()),
            artifact_store: LocalManagedArtworkArtifactStore::new(config.artifact_root),
        })
    }

    pub(crate) async fn accept_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<AcceptManagedArtworkCandidateResponse> {
        let candidate = self
            .store
            .get_artwork_candidate(candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: candidate_id.to_string(),
            })?;

        if candidate.status == ArtworkCandidateStatus::Rejected {
            return Err(TaruError::InvalidInput {
                message: "rejected artwork candidates cannot be accepted".to_owned(),
            });
        }

        self.store
            .get_media_item(candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: candidate.item_id.to_string(),
            })?;
        self.store
            .get_library_item_state(candidate.library_id, candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library_item_state",
                id: format!("{}:{}", candidate.library_id, candidate.item_id),
            })?;

        let job_id = JobId::new();
        let input = ManagedArtworkIngestJobInput {
            candidate_id,
            library_id: candidate.library_id,
            item_id: candidate.item_id,
            image_kind: image_kind_label(&candidate.kind).to_owned(),
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize managed artwork ingest job input: {err}"),
        })?;
        let acceptance = self
            .store
            .accept_managed_artwork_candidate_ingest(
                candidate_id,
                NewManagedArtworkIngest {
                    id: ManagedArtworkIngestId::new(),
                    candidate_id,
                    job_id,
                    library_id: candidate.library_id,
                    item_id: candidate.item_id,
                    kind: candidate.kind,
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    library_id: Some(candidate.library_id),
                    source_id: None,
                    input_json: Some(input_json),
                },
            )
            .await?;

        Ok(AcceptManagedArtworkCandidateResponse::from_acceptance(
            acceptance,
        ))
    }

    pub(crate) async fn process_next(&self) -> Result<ProcessManagedArtworkIngestResponse> {
        let Some(claim) = self
            .store
            .claim_next_queued_managed_artwork_ingest()
            .await?
        else {
            return Ok(ProcessManagedArtworkIngestResponse::empty());
        };

        match self.process_claim(claim.clone()).await {
            Ok(processing) => Ok(ProcessManagedArtworkIngestResponse::from_processing(
                processing,
            )),
            Err(failure) => {
                let processing = self
                    .store
                    .fail_managed_artwork_ingest(
                        claim.ingest.id,
                        failure.code.to_owned(),
                        failure.code.to_owned(),
                    )
                    .await?;
                Ok(ProcessManagedArtworkIngestResponse::from_processing(
                    processing,
                ))
            }
        }
    }

    async fn process_claim(
        &self,
        claim: ManagedArtworkIngestClaimRecord,
    ) -> std::result::Result<taru_core::ManagedArtworkIngestProcessingRecord, ManagedArtworkFailure>
    {
        if claim.candidate.source_kind != ArtworkCandidateSourceKind::RemoteUrl {
            return Err(ManagedArtworkFailure::new("unsupported_source"));
        }

        let fetched = self.fetcher.fetch(&claim.candidate.source_uri).await?;
        let validated = self.validator.validate(&fetched)?;
        let artifact_id = ManagedArtworkArtifactId::new();
        let stored = self
            .artifact_store
            .write(artifact_id, validated.extension, &fetched.bytes)
            .await?;

        let summary = ManagedArtworkIngestJobSummary {
            ingest_id: claim.ingest.id,
            candidate_id: claim.candidate.id,
            artifact_id,
            status: ManagedArtworkIngestStatus::Stored.as_str(),
            media_type: validated.media_type.clone(),
            byte_len: validated.byte_len,
            width: validated.width,
            height: validated.height,
            content_hash: validated.content_hash.clone(),
        };
        let summary_json = serde_json::to_string(&summary)
            .map_err(|_| ManagedArtworkFailure::new("storage_failed"))?;
        let result = self
            .store
            .commit_managed_artwork_artifact(
                claim.ingest.id,
                NewManagedArtworkArtifact {
                    id: artifact_id,
                    ingest_id: claim.ingest.id,
                    library_id: claim.ingest.library_id,
                    item_id: claim.ingest.item_id,
                    kind: claim.ingest.kind,
                    storage_uri: stored.storage_uri.clone(),
                    content_hash: Some(validated.content_hash),
                    width: Some(validated.width),
                    height: Some(validated.height),
                    byte_len: Some(validated.byte_len),
                    media_type: Some(validated.media_type),
                },
                Some(summary_json),
            )
            .await;

        match result {
            Ok(processing) => Ok(processing),
            Err(_) => {
                self.artifact_store.delete_best_effort(&stored).await;
                Err(ManagedArtworkFailure::new("storage_failed"))
            }
        }
    }
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobInput {
    candidate_id: ArtworkCandidateId,
    library_id: taru_core::LibraryId,
    item_id: taru_core::MediaItemId,
    image_kind: String,
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobSummary {
    ingest_id: ManagedArtworkIngestId,
    candidate_id: ArtworkCandidateId,
    artifact_id: ManagedArtworkArtifactId,
    status: &'static str,
    media_type: String,
    byte_len: u64,
    width: u32,
    height: u32,
    content_hash: String,
}

#[derive(Clone, Debug)]
struct ManagedArtworkFetcher {
    client: reqwest::Client,
    config: ArtworkConfig,
    permits: Arc<Semaphore>,
}

#[derive(Clone, Debug)]
struct FetchedManagedArtwork {
    bytes: Vec<u8>,
    media_type: String,
}

impl ManagedArtworkFetcher {
    fn new(config: ArtworkConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(config.fetch_user_agent.clone())
            .timeout(Duration::from_millis(config.fetch_timeout_ms));

        if let Some(proxy) = config
            .fetch_proxy
            .as_ref()
            .filter(|proxy| !proxy.is_blank())
        {
            builder = builder.proxy(reqwest::Proxy::all(proxy.expose_secret()).map_err(|err| {
                TaruError::InvalidInput {
                    message: format!("invalid artwork fetch proxy configuration: {err}"),
                }
            })?);
        }

        let client = builder.build().map_err(|err| TaruError::InvalidInput {
            message: format!("failed to build artwork fetch HTTP client: {err}"),
        })?;
        let permits = Arc::new(Semaphore::new(config.fetch_concurrency.max(1)));

        Ok(Self {
            client,
            config,
            permits,
        })
    }

    async fn fetch(
        &self,
        source_uri: &str,
    ) -> std::result::Result<FetchedManagedArtwork, ManagedArtworkFailure> {
        let url = reqwest::Url::parse(source_uri)
            .map_err(|_| ManagedArtworkFailure::new("unsupported_source"))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ManagedArtworkFailure::new("unsupported_source"));
        }

        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| ManagedArtworkFailure::new("resource_budget_closed"))?;
        let mut last_failure = ManagedArtworkFailure::new("fetch_failed");
        let attempts = self.config.fetch_max_attempts.max(1);

        for _ in 0..attempts {
            match self.fetch_once(url.clone()).await {
                Ok(fetched) => return Ok(fetched),
                Err(failure) if failure.retryable => last_failure = failure,
                Err(failure) => return Err(failure),
            }
        }

        Err(last_failure)
    }

    async fn fetch_once(
        &self,
        url: reqwest::Url,
    ) -> std::result::Result<FetchedManagedArtwork, ManagedArtworkFailure> {
        let response = self.client.get(url).send().await.map_err(|err| {
            if err.is_timeout() {
                ManagedArtworkFailure::retryable("fetch_timeout")
            } else {
                ManagedArtworkFailure::retryable("fetch_failed")
            }
        })?;

        if !response.status().is_success() {
            return Err(ManagedArtworkFailure::retryable("fetch_http_status"));
        }

        if response
            .content_length()
            .is_some_and(|len| len > self.config.fetch_max_bytes)
        {
            return Err(ManagedArtworkFailure::new("too_large"));
        }

        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(normalize_media_type)
            .ok_or_else(|| ManagedArtworkFailure::new("unsupported_media_type"))?;

        let mut bytes = Vec::new();
        let mut total_len = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| {
                if err.is_timeout() {
                    ManagedArtworkFailure::retryable("fetch_timeout")
                } else {
                    ManagedArtworkFailure::retryable("fetch_failed")
                }
            })?;
            let chunk_len =
                u64::try_from(chunk.len()).map_err(|_| ManagedArtworkFailure::new("too_large"))?;
            total_len = total_len
                .checked_add(chunk_len)
                .ok_or_else(|| ManagedArtworkFailure::new("too_large"))?;
            if total_len > self.config.fetch_max_bytes {
                return Err(ManagedArtworkFailure::new("too_large"));
            }
            bytes.extend_from_slice(&chunk);
        }

        Ok(FetchedManagedArtwork { bytes, media_type })
    }
}

#[derive(Clone, Debug)]
struct ManagedArtworkImageValidator {
    max_width: u32,
    max_height: u32,
}

#[derive(Clone, Debug)]
struct ValidatedManagedArtwork {
    media_type: String,
    extension: &'static str,
    width: u32,
    height: u32,
    byte_len: u64,
    content_hash: String,
}

impl ManagedArtworkImageValidator {
    fn new(config: ArtworkConfig) -> Self {
        Self {
            max_width: config.max_width,
            max_height: config.max_height,
        }
    }

    fn validate(
        &self,
        fetched: &FetchedManagedArtwork,
    ) -> std::result::Result<ValidatedManagedArtwork, ManagedArtworkFailure> {
        let (format, extension) = image_format_for_media_type(&fetched.media_type)
            .ok_or_else(|| ManagedArtworkFailure::new("unsupported_media_type"))?;
        let image = image::load_from_memory_with_format(&fetched.bytes, format)
            .map_err(|_| ManagedArtworkFailure::new("invalid_image"))?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(ManagedArtworkFailure::new("invalid_image"));
        }
        if width > self.max_width || height > self.max_height {
            return Err(ManagedArtworkFailure::new("dimension_limit_exceeded"));
        }

        Ok(ValidatedManagedArtwork {
            media_type: fetched.media_type.clone(),
            extension,
            width,
            height,
            byte_len: u64::try_from(fetched.bytes.len())
                .map_err(|_| ManagedArtworkFailure::new("too_large"))?,
            content_hash: sha256_hex(&fetched.bytes),
        })
    }
}

#[derive(Clone, Debug)]
struct LocalManagedArtworkArtifactStore {
    root: PathBuf,
}

#[derive(Clone, Debug)]
struct StoredManagedArtworkArtifact {
    storage_uri: String,
    path: PathBuf,
}

impl LocalManagedArtworkArtifactStore {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    async fn write(
        &self,
        artifact_id: ManagedArtworkArtifactId,
        extension: &str,
        bytes: &[u8],
    ) -> std::result::Result<StoredManagedArtworkArtifact, ManagedArtworkFailure> {
        let artifact_id_text = artifact_id.to_string();
        let shard = artifact_id_text
            .get(0..2)
            .ok_or_else(|| ManagedArtworkFailure::new("storage_failed"))?;
        let directory = self.root.join(shard);
        let final_path = directory.join(format!("{artifact_id_text}.{extension}"));
        let temp_path = directory.join(format!("{artifact_id_text}.tmp"));

        let result = async {
            fs::create_dir_all(&directory).await?;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await?;
            file.write_all(bytes).await?;
            file.sync_all().await?;
            drop(file);
            fs::rename(&temp_path, &final_path).await
        }
        .await;

        if result.is_err() {
            let _ = fs::remove_file(&temp_path).await;
            return Err(ManagedArtworkFailure::new("storage_failed"));
        }

        Ok(StoredManagedArtworkArtifact {
            storage_uri: format!("managed-artwork://artifact/{artifact_id_text}"),
            path: final_path,
        })
    }

    async fn delete_best_effort(&self, stored: &StoredManagedArtworkArtifact) {
        if path_has_prefix(&stored.path, &self.root) {
            let _ = fs::remove_file(&stored.path).await;
        }
    }
}

#[derive(Clone, Debug)]
struct ManagedArtworkFailure {
    code: &'static str,
    retryable: bool,
}

impl ManagedArtworkFailure {
    const fn new(code: &'static str) -> Self {
        Self {
            code,
            retryable: false,
        }
    }

    const fn retryable(code: &'static str) -> Self {
        Self {
            code,
            retryable: true,
        }
    }
}

fn normalize_media_type(value: &str) -> Option<String> {
    let media_type = value.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.is_empty() {
        None
    } else {
        Some(media_type)
    }
}

fn image_format_for_media_type(media_type: &str) -> Option<(image::ImageFormat, &'static str)> {
    match media_type {
        "image/jpeg" => Some((image::ImageFormat::Jpeg, "jpg")),
        "image/png" => Some((image::ImageFormat::Png, "png")),
        "image/webp" => Some((image::ImageFormat::WebP, "webp")),
        _ => None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn path_has_prefix(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

fn image_kind_label(kind: &taru_core::ImageKind) -> &'static str {
    match kind {
        taru_core::ImageKind::Poster => "poster",
        taru_core::ImageKind::Backdrop => "backdrop",
        taru_core::ImageKind::Logo => "logo",
        taru_core::ImageKind::Thumbnail => "thumbnail",
        taru_core::ImageKind::Banner => "banner",
        taru_core::ImageKind::Other(_) => "other",
    }
}
