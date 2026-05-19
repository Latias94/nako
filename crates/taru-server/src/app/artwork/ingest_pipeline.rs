use std::{fmt::Write as _, sync::Arc, time::Duration};

use futures_util::StreamExt;
use image::GenericImageView;
use serde::Serialize;
use sha2::{Digest, Sha256};
use taru_core::{
    ArtworkCandidateId, ArtworkCandidateSourceKind, ManagedArtworkArtifactId,
    ManagedArtworkIngestClaimRecord, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    NewManagedArtworkArtifact, Result, TaruError,
};
use tokio::sync::Semaphore;

use crate::config::ArtworkConfig;

use super::artifact_store::{LocalManagedArtworkArtifactStore, StoredManagedArtworkArtifact};

#[derive(Clone, Debug)]
pub(super) struct ManagedArtworkIngestPipeline {
    fetcher: ManagedArtworkFetcher,
    validator: ManagedArtworkImageValidator,
}

impl ManagedArtworkIngestPipeline {
    pub(super) fn new(config: ArtworkConfig) -> Result<Self> {
        Ok(Self {
            fetcher: ManagedArtworkFetcher::new(config.clone())?,
            validator: ManagedArtworkImageValidator::new(config),
        })
    }

    pub(super) async fn prepare_artifact(
        &self,
        claim: &ManagedArtworkIngestClaimRecord,
        artifact_store: &LocalManagedArtworkArtifactStore,
    ) -> std::result::Result<PreparedManagedArtworkArtifact, ManagedArtworkFailure> {
        if claim.candidate.source_kind != ArtworkCandidateSourceKind::RemoteUrl {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::UnsupportedSource,
            ));
        }

        let fetched = self.fetcher.fetch(&claim.candidate.source_uri).await?;
        let validated = self.validator.validate(&fetched)?;
        let artifact_id = ManagedArtworkArtifactId::new();
        let stored = artifact_store
            .write(artifact_id, validated.extension, &fetched.bytes)
            .await
            .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::StorageFailed))?;

        PreparedManagedArtworkArtifact::from_validated(claim, artifact_id, stored, validated)
    }

    pub(super) fn failure_summary_json(
        failure: ManagedArtworkFailure,
        claim: &ManagedArtworkIngestClaimRecord,
    ) -> (String, Option<String>) {
        let summary = ManagedArtworkIngestFailureSummary {
            ingest_id: claim.ingest.id,
            candidate_id: claim.candidate.id,
            status: ManagedArtworkIngestStatus::Failed.as_str(),
            failure_code: failure.code.as_str(),
        };
        (
            failure.code.as_str().to_owned(),
            serde_json::to_string(&summary).ok(),
        )
    }
}

#[derive(Clone, Debug)]
pub(super) struct PreparedManagedArtworkArtifact {
    pub(super) stored: StoredManagedArtworkArtifact,
    pub(super) artifact: NewManagedArtworkArtifact,
    pub(super) summary_json: String,
}

impl PreparedManagedArtworkArtifact {
    fn from_validated(
        claim: &ManagedArtworkIngestClaimRecord,
        artifact_id: ManagedArtworkArtifactId,
        stored: StoredManagedArtworkArtifact,
        validated: ValidatedManagedArtwork,
    ) -> std::result::Result<Self, ManagedArtworkFailure> {
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
            .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::StorageFailed))?;
        let artifact = NewManagedArtworkArtifact {
            id: artifact_id,
            ingest_id: claim.ingest.id,
            library_id: claim.ingest.library_id,
            item_id: claim.ingest.item_id,
            kind: claim.ingest.kind.clone(),
            storage_uri: stored.storage_uri.clone(),
            content_hash: Some(validated.content_hash),
            width: Some(validated.width),
            height: Some(validated.height),
            byte_len: Some(validated.byte_len),
            media_type: Some(validated.media_type),
        };

        Ok(Self {
            stored,
            artifact,
            summary_json,
        })
    }
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
        let url = reqwest::Url::parse(source_uri).map_err(|_| {
            ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedSource)
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::UnsupportedSource,
            ));
        }

        let _permit = self.permits.acquire().await.map_err(|_| {
            ManagedArtworkFailure::new(ManagedArtworkFailureCode::ResourceBudgetClosed)
        })?;
        let mut last_failure = ManagedArtworkFailure::new(ManagedArtworkFailureCode::FetchFailed);
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
                ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchTimeout)
            } else {
                ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchFailed)
            }
        })?;

        if !response.status().is_success() {
            return Err(ManagedArtworkFailure::retryable(
                ManagedArtworkFailureCode::FetchHttpStatus,
            ));
        }

        if response
            .content_length()
            .is_some_and(|len| len > self.config.fetch_max_bytes)
        {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::TooLarge,
            ));
        }

        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(normalize_media_type)
            .ok_or_else(|| {
                ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedMediaType)
            })?;

        let mut bytes = Vec::new();
        let mut total_len = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| {
                if err.is_timeout() {
                    ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchTimeout)
                } else {
                    ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchFailed)
                }
            })?;
            let chunk_len = u64::try_from(chunk.len())
                .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?;
            total_len = total_len
                .checked_add(chunk_len)
                .ok_or_else(|| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?;
            if total_len > self.config.fetch_max_bytes {
                return Err(ManagedArtworkFailure::new(
                    ManagedArtworkFailureCode::TooLarge,
                ));
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
        let (format, extension) = super::image_format_for_media_type(&fetched.media_type)
            .ok_or_else(|| {
                ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedMediaType)
            })?;
        let image = image::load_from_memory_with_format(&fetched.bytes, format)
            .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::InvalidImage))?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::InvalidImage,
            ));
        }
        if width > self.max_width || height > self.max_height {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::DimensionLimitExceeded,
            ));
        }

        Ok(ValidatedManagedArtwork {
            media_type: fetched.media_type.clone(),
            extension,
            width,
            height,
            byte_len: u64::try_from(fetched.bytes.len())
                .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?,
            content_hash: sha256_hex(&fetched.bytes),
        })
    }
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

#[derive(Serialize)]
struct ManagedArtworkIngestFailureSummary {
    ingest_id: ManagedArtworkIngestId,
    candidate_id: ArtworkCandidateId,
    status: &'static str,
    failure_code: &'static str,
}

#[derive(Clone, Debug)]
pub(super) struct ManagedArtworkFailure {
    code: ManagedArtworkFailureCode,
    retryable: bool,
}

impl ManagedArtworkFailure {
    const fn new(code: ManagedArtworkFailureCode) -> Self {
        Self {
            code,
            retryable: false,
        }
    }

    pub(super) const fn storage_failed() -> Self {
        Self::new(ManagedArtworkFailureCode::StorageFailed)
    }

    const fn retryable(code: ManagedArtworkFailureCode) -> Self {
        Self {
            code,
            retryable: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ManagedArtworkFailureCode {
    UnsupportedSource,
    UnsupportedMediaType,
    TooLarge,
    InvalidImage,
    DimensionLimitExceeded,
    FetchTimeout,
    FetchFailed,
    FetchHttpStatus,
    StorageFailed,
    ResourceBudgetClosed,
}

impl ManagedArtworkFailureCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSource => "unsupported_source",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::TooLarge => "too_large",
            Self::InvalidImage => "invalid_image",
            Self::DimensionLimitExceeded => "dimension_limit_exceeded",
            Self::FetchTimeout => "fetch_timeout",
            Self::FetchFailed => "fetch_failed",
            Self::FetchHttpStatus => "fetch_http_status",
            Self::StorageFailed => "storage_failed",
            Self::ResourceBudgetClosed => "resource_budget_closed",
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

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}
