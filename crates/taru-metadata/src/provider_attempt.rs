use taru_core::{
    ExternalProvider, JobId, MediaItem, MediaItemId, MetadataMatchKind, MetadataProviderAttemptId,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, NewMetadataProviderAttempt,
    ProviderRawResponse, Result, TaruError,
};

use crate::{
    MetadataFetchRequest, MetadataLookup, MetadataMergePolicy, MetadataProvider,
    providers::{now_utc_string, release_year},
};

use super::strategy::{
    MetadataAttemptPort, MetadataProviderAttempt, MetadataRefreshCommit, MetadataRefreshRequest,
    MetadataRefreshSnapshot, MetadataRefreshSummary,
};

pub(super) struct ProviderAttemptRuntimeOutcome {
    pub(super) attempt: MetadataProviderAttempt,
    pub(super) result:
        std::result::Result<MetadataProviderRefreshSuccess, MetadataProviderRefreshError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MetadataProviderRefreshSuccess {
    pub(super) commit: MetadataRefreshCommit,
    pub(super) provider: ExternalProvider,
    provider_key: String,
    matched_by: MetadataMatchKind,
    refresh_mode: taru_core::MetadataRefreshMode,
    updated: bool,
}

impl MetadataProviderRefreshSuccess {
    pub(super) fn into_summary(
        self,
        job_id: JobId,
        attempted_providers: Vec<MetadataProviderAttempt>,
    ) -> MetadataRefreshSummary {
        MetadataRefreshSummary {
            job_id,
            item_id: self.commit.item.id,
            provider: self.provider.clone(),
            selected_provider: self.provider,
            provider_key: self.provider_key,
            matched_by: self.matched_by,
            refresh_mode: self.refresh_mode,
            updated: self.updated,
            attempted_providers,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum MetadataProviderRefreshError {
    NoMatch(String),
    ProviderFailed(String),
    Fatal(TaruError),
}

impl MetadataProviderRefreshError {
    pub(super) fn into_error(self) -> TaruError {
        match self {
            Self::NoMatch(message) => TaruError::NotFound {
                entity: "metadata_candidate",
                id: message,
            },
            Self::ProviderFailed(message) => TaruError::Provider {
                provider: "metadata_provider".to_owned(),
                message,
            },
            Self::Fatal(err) => err,
        }
    }
}

pub(super) async fn run_available_provider_attempt<R, P>(
    repository: &R,
    provider: &P,
    request: &MetadataRefreshRequest,
    snapshot: &MetadataRefreshSnapshot,
) -> Result<ProviderAttemptRuntimeOutcome>
where
    R: MetadataAttemptPort,
    P: MetadataProvider + ?Sized,
{
    let started_at = now_utc_string()?;
    let result = refresh_existing_with_provider(provider, request, snapshot).await;
    let finished_at = now_utc_string()?;
    let attempt = attempt_from_result(provider.provider(), &result);
    record_metadata_attempt(
        repository,
        request.job_id,
        request.item_id,
        &attempt,
        started_at,
        finished_at,
    )
    .await?;

    Ok(ProviderAttemptRuntimeOutcome { attempt, result })
}

pub(super) async fn record_skipped_provider_attempt<R>(
    repository: &R,
    job_id: JobId,
    item_id: MediaItemId,
    provider: ExternalProvider,
    status: MetadataProviderAttemptStatus,
    message: String,
) -> Result<MetadataProviderAttempt>
where
    R: MetadataAttemptPort,
{
    let now = now_utc_string()?;
    let attempt = skipped_attempt(provider, status, message);
    record_metadata_attempt(repository, job_id, item_id, &attempt, now.clone(), now).await?;
    Ok(attempt)
}

async fn record_metadata_attempt<R>(
    repository: &R,
    job_id: JobId,
    item_id: MediaItemId,
    attempt: &MetadataProviderAttempt,
    started_at: String,
    finished_at: String,
) -> Result<()>
where
    R: MetadataAttemptPort,
{
    repository
        .record_metadata_attempt(NewMetadataProviderAttempt {
            id: MetadataProviderAttemptId::new(),
            job_id,
            item_id,
            provider: attempt.provider.clone(),
            status: attempt.status,
            provider_key: attempt.provider_key.clone(),
            matched_by: attempt.matched_by,
            started_at,
            finished_at,
            error_class: attempt.error_class,
            message: attempt.message.clone(),
        })
        .await
}

fn attempt_from_result(
    provider: ExternalProvider,
    result: &std::result::Result<MetadataProviderRefreshSuccess, MetadataProviderRefreshError>,
) -> MetadataProviderAttempt {
    match result {
        Ok(success) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::Succeeded,
            message: None,
            provider_key: Some(success.provider_key.clone()),
            matched_by: Some(success.matched_by),
            error_class: None,
        },
        Err(MetadataProviderRefreshError::NoMatch(message)) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::NoMatch,
            message: Some(message.clone()),
            provider_key: None,
            matched_by: None,
            error_class: Some(MetadataProviderErrorClass::NoMatch),
        },
        Err(MetadataProviderRefreshError::ProviderFailed(message)) => {
            let error_class = classify_provider_failure_message(message);

            MetadataProviderAttempt {
                provider,
                status: attempt_status_for_error_class(error_class),
                message: Some(message.clone()),
                provider_key: None,
                matched_by: None,
                error_class: Some(error_class),
            }
        }
        Err(MetadataProviderRefreshError::Fatal(err)) => {
            let error_class = classify_provider_error_class(err);

            MetadataProviderAttempt {
                provider,
                status: attempt_status_for_error_class(error_class),
                message: Some(err.to_string()),
                provider_key: None,
                matched_by: None,
                error_class: Some(error_class),
            }
        }
    }
}

fn attempt_status_for_error_class(
    error_class: MetadataProviderErrorClass,
) -> MetadataProviderAttemptStatus {
    match error_class {
        MetadataProviderErrorClass::RateLimited => MetadataProviderAttemptStatus::RateLimited,
        _ => MetadataProviderAttemptStatus::Failed,
    }
}

fn classify_provider_failure_message(message: &str) -> MetadataProviderErrorClass {
    classify_provider_error_class(&TaruError::Provider {
        provider: "metadata_provider".to_owned(),
        message: message.to_owned(),
    })
}

fn skipped_attempt(
    provider: ExternalProvider,
    status: MetadataProviderAttemptStatus,
    message: String,
) -> MetadataProviderAttempt {
    let error_class = match status {
        MetadataProviderAttemptStatus::SkippedDisabled
        | MetadataProviderAttemptStatus::SkippedUnavailable => {
            Some(MetadataProviderErrorClass::Unavailable)
        }
        MetadataProviderAttemptStatus::NotImplemented => {
            Some(MetadataProviderErrorClass::Unsupported)
        }
        MetadataProviderAttemptStatus::NoMatch => Some(MetadataProviderErrorClass::NoMatch),
        MetadataProviderAttemptStatus::RateLimited => Some(MetadataProviderErrorClass::RateLimited),
        MetadataProviderAttemptStatus::Failed => Some(MetadataProviderErrorClass::Unknown),
        MetadataProviderAttemptStatus::Succeeded => None,
    };

    MetadataProviderAttempt {
        provider,
        status,
        message: Some(message),
        provider_key: None,
        matched_by: None,
        error_class,
    }
}

async fn refresh_existing_with_provider<P>(
    provider: &P,
    request: &MetadataRefreshRequest,
    snapshot: &MetadataRefreshSnapshot,
) -> std::result::Result<MetadataProviderRefreshSuccess, MetadataProviderRefreshError>
where
    P: MetadataProvider + ?Sized,
{
    let existing = &snapshot.item;
    let (provider_key, matched_by) = resolve_provider_key(provider, request, existing).await?;
    let fetched = provider
        .fetch(MetadataFetchRequest {
            kind: existing.kind,
            provider_key: provider_key.clone(),
            language: request.profile.language.clone(),
        })
        .await
        .map_err(classify_provider_error)?;

    let provider_id = provider.provider();
    if fetched.provider != provider_id {
        return Err(MetadataProviderRefreshError::ProviderFailed(format!(
            "provider {} returned metadata for {}",
            provider_label(&provider_id),
            provider_label(&fetched.provider)
        )));
    }

    let policy = MetadataMergePolicy::from_locks_and_mode(
        &snapshot.field_locks,
        request.profile.refresh_mode,
    );
    let merged_metadata = policy.merge(&existing.metadata, &fetched.metadata);
    let updated = merged_metadata != existing.metadata;
    let updated_item = MediaItem {
        metadata: merged_metadata,
        ..existing.clone()
    };

    Ok(MetadataProviderRefreshSuccess {
        commit: MetadataRefreshCommit {
            item: updated_item,
            raw_response: ProviderRawResponse {
                item_id: existing.id,
                provider: fetched.provider.clone(),
                provider_key: fetched.provider_key.clone(),
                fetched_at: now_utc_string().map_err(MetadataProviderRefreshError::Fatal)?,
                body_json: fetched.raw_json.clone(),
            },
        },
        provider: fetched.provider,
        provider_key: fetched.provider_key,
        matched_by,
        refresh_mode: request.profile.refresh_mode,
        updated,
    })
}

async fn resolve_provider_key<P>(
    provider: &P,
    request: &MetadataRefreshRequest,
    item: &MediaItem,
) -> std::result::Result<(String, MetadataMatchKind), MetadataProviderRefreshError>
where
    P: MetadataProvider + ?Sized,
{
    let provider_id = provider.provider();
    if let Some(external_id) = item
        .metadata
        .external_ids
        .iter()
        .find(|external_id| external_id.provider == provider_id)
    {
        return Ok((external_id.value.clone(), MetadataMatchKind::ExternalId));
    }

    let lookup = MetadataLookup {
        kind: Some(item.kind),
        title: item.metadata.title.clone(),
        year: release_year(item.metadata.release_date.as_deref()),
        language: request.profile.language.clone(),
        external_ids: item.metadata.external_ids.clone(),
    };
    let candidates = provider
        .search(lookup)
        .await
        .map_err(classify_provider_error)?;
    let candidate = candidates
        .into_iter()
        .filter(|candidate| candidate.provider == provider_id)
        .max_by(|left, right| left.score.total_cmp(&right.score))
        .ok_or_else(|| {
            MetadataProviderRefreshError::NoMatch(format!(
                "{} returned no metadata candidate for item {}",
                provider_label(&provider_id),
                item.id
            ))
        })?;

    Ok((candidate.provider_key, MetadataMatchKind::Search))
}

fn classify_provider_error(error: TaruError) -> MetadataProviderRefreshError {
    match error {
        TaruError::NotFound { .. } => MetadataProviderRefreshError::NoMatch(error.to_string()),
        TaruError::Unsupported(_)
        | TaruError::InvalidInput { .. }
        | TaruError::Conflict { .. }
        | TaruError::Unauthorized { .. }
        | TaruError::Forbidden { .. }
        | TaruError::Provider { .. } => {
            MetadataProviderRefreshError::ProviderFailed(error.to_string())
        }
        TaruError::Storage { .. } | TaruError::Database { .. } => {
            MetadataProviderRefreshError::Fatal(error)
        }
    }
}

fn classify_provider_error_class(error: &TaruError) -> MetadataProviderErrorClass {
    match error {
        TaruError::NotFound { .. } => MetadataProviderErrorClass::NoMatch,
        TaruError::Unsupported(_) => MetadataProviderErrorClass::Unsupported,
        TaruError::InvalidInput { .. } | TaruError::Conflict { .. } => {
            MetadataProviderErrorClass::Unknown
        }
        TaruError::Unauthorized { .. } | TaruError::Forbidden { .. } => {
            MetadataProviderErrorClass::Auth
        }
        TaruError::Provider { message, .. } => {
            let lower = message.to_ascii_lowercase();
            if lower.contains("timeout") || lower.contains("timed out") {
                MetadataProviderErrorClass::Timeout
            } else if lower.contains("429") || lower.contains("rate") {
                MetadataProviderErrorClass::RateLimited
            } else if lower.contains("401") || lower.contains("403") || lower.contains("auth") {
                MetadataProviderErrorClass::Auth
            } else if lower.contains("http") {
                MetadataProviderErrorClass::HttpStatus
            } else if lower.contains("parse") || lower.contains("json") {
                MetadataProviderErrorClass::Parse
            } else {
                MetadataProviderErrorClass::Network
            }
        }
        TaruError::Storage { .. } | TaruError::Database { .. } => {
            MetadataProviderErrorClass::Unknown
        }
    }
}

pub(super) fn summarize_attempts(attempts: &[MetadataProviderAttempt]) -> String {
    if attempts.is_empty() {
        return "no providers were attempted".to_owned();
    }

    attempts
        .iter()
        .map(|attempt| {
            let detail = attempt
                .message
                .as_deref()
                .filter(|message| !message.trim().is_empty())
                .unwrap_or("no detail");
            format!(
                "{}={} ({detail})",
                provider_label(&attempt.provider),
                attempt.status.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn provider_label(provider: &ExternalProvider) -> String {
    match provider {
        ExternalProvider::Tmdb => "tmdb".to_owned(),
        ExternalProvider::Douban => "douban".to_owned(),
        ExternalProvider::Bangumi => "bangumi".to_owned(),
        ExternalProvider::Imdb => "imdb".to_owned(),
        ExternalProvider::Local => "local".to_owned(),
        ExternalProvider::Other(value) => format!("other:{value}"),
    }
}
