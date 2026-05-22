use std::path::PathBuf;

use futures_util::{StreamExt, stream};
use nako_core::{
    IngestionFailurePhase, IngestionFailureRepository, MediaProbeRepository, MediaRepository,
    MediaSource, NakoError, NewIngestionFailure, PageRequest, Result,
};
use nako_media_probe::{MediaProbe, MediaProbeRequest};
use nako_vfs::{ByteRange, StorageBackend, StorageUri};

use super::{
    failure::{ingestion_failure_class, ingestion_failure_is_retryable, ingestion_failure_time_ms},
    summary::{LibraryProbeFailure, LibraryProbeRequest, LibraryProbeSummary},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryProbeOptions {
    pub max_concurrent_probes: usize,
    pub staging_root: Option<PathBuf>,
}

impl Default for LibraryProbeOptions {
    fn default() -> Self {
        Self {
            max_concurrent_probes: 2,
            staging_root: None,
        }
    }
}

#[derive(Debug)]
pub struct LibraryProbeService<B, P, R> {
    backend: B,
    probe: P,
    repository: R,
    options: LibraryProbeOptions,
}

impl<B, P, R> LibraryProbeService<B, P, R> {
    pub fn new(backend: B, probe: P, repository: R) -> Self {
        Self {
            backend,
            probe,
            repository,
            options: LibraryProbeOptions::default(),
        }
    }

    pub fn with_options(backend: B, probe: P, repository: R, options: LibraryProbeOptions) -> Self {
        Self {
            backend,
            probe,
            repository,
            options,
        }
    }

    #[must_use]
    pub fn options(&self) -> &LibraryProbeOptions {
        &self.options
    }
}

impl<B, P, R> LibraryProbeService<B, P, R>
where
    B: StorageBackend,
    P: MediaProbe,
    R: IngestionFailureRepository + MediaRepository + MediaProbeRepository,
{
    pub async fn probe_library(&self, request: LibraryProbeRequest) -> Result<LibraryProbeSummary> {
        let sources = self.list_all_media_sources(request.library_id).await?;
        let total_sources = sources.len() as u64;
        let max_concurrent = self.options.max_concurrent_probes.max(1);
        let outcomes = stream::iter(sources)
            .map(|source| async move { self.probe_source(source, request.force).await })
            .buffer_unordered(max_concurrent)
            .collect::<Vec<_>>()
            .await;

        let mut summary = LibraryProbeSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            total_sources,
            probed_sources: 0,
            skipped_sources: 0,
            failed_sources: 0,
            failures: Vec::new(),
        };

        for outcome in outcomes {
            match outcome {
                ProbeSourceOutcome::Probed => summary.probed_sources += 1,
                ProbeSourceOutcome::Skipped => summary.skipped_sources += 1,
                ProbeSourceOutcome::Failed(failure) => {
                    self.persist_probe_failure(&request, &failure).await?;
                    summary.failed_sources += 1;
                    summary.failures.push(failure);
                }
            }
        }

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));

        Ok(summary)
    }

    async fn list_all_media_sources(
        &self,
        library_id: nako_core::LibraryId,
    ) -> Result<Vec<MediaSource>> {
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

    async fn persist_probe_failure(
        &self,
        request: &LibraryProbeRequest,
        failure: &LibraryProbeFailure,
    ) -> Result<()> {
        self.repository
            .record_ingestion_failure(NewIngestionFailure {
                library_id: request.library_id,
                job_id: Some(request.job_id),
                scan_id: None,
                source_id: failure.source_id,
                phase: IngestionFailurePhase::Probe,
                target_uri: failure.locator.clone(),
                target_kind: "source".to_owned(),
                failure_class: failure.failure_class,
                message: failure.message.clone(),
                retryable: failure.retryable,
                failed_at_ms: ingestion_failure_time_ms(),
            })
            .await?;

        Ok(())
    }

    async fn probe_source(&self, source: MediaSource, force: bool) -> ProbeSourceOutcome {
        if !force {
            match self.repository.get_media_probe(source.id).await {
                Ok(Some(_existing)) => {
                    return match self.resolve_probe_failure(&source).await {
                        Ok(()) => ProbeSourceOutcome::Skipped,
                        Err(err) => probe_failure(&source, err),
                    };
                }
                Ok(None) => {}
                Err(err) => return probe_failure(&source, err),
            }
        }

        let uri = match StorageUri::parse(&source.locator) {
            Ok(uri) => uri,
            Err(err) => return probe_failure(&source, err),
        };
        let virtual_file = match self
            .backend
            .open_range(
                &uri,
                Some(ByteRange {
                    offset: 0,
                    length: None,
                }),
            )
            .await
        {
            Ok(virtual_file) => virtual_file,
            Err(err) => return probe_failure(&source, err),
        };
        let local_path_hint = match virtual_file.local_path_hint {
            Some(path) => Some(path),
            None => match &self.options.staging_root {
                Some(root) => match self
                    .backend
                    .stage(nako_vfs::StageRequest::new(uri.clone(), root.clone()))
                    .await
                {
                    Ok(staged) => Some(staged.path),
                    Err(err) => return probe_failure(&source, err),
                },
                None => None,
            },
        };
        let probe_result = match self
            .probe
            .probe(MediaProbeRequest {
                source: uri,
                local_path_hint,
            })
            .await
        {
            Ok(result) => result,
            Err(err) => return probe_failure(&source, err),
        };

        match self
            .repository
            .upsert_media_probe(source.id, &probe_result)
            .await
        {
            Ok(()) => match self.resolve_probe_failure(&source).await {
                Ok(()) => ProbeSourceOutcome::Probed,
                Err(err) => probe_failure(&source, err),
            },
            Err(err) => probe_failure(&source, err),
        }
    }

    async fn resolve_probe_failure(&self, source: &MediaSource) -> Result<()> {
        self.repository
            .resolve_ingestion_failure(
                source.library_id,
                IngestionFailurePhase::Probe,
                &source.locator,
                ingestion_failure_time_ms(),
            )
            .await?;

        Ok(())
    }
}

enum ProbeSourceOutcome {
    Probed,
    Skipped,
    Failed(LibraryProbeFailure),
}

fn probe_failure(source: &MediaSource, err: NakoError) -> ProbeSourceOutcome {
    ProbeSourceOutcome::Failed(LibraryProbeFailure {
        source_id: Some(source.id),
        locator: source.locator.clone(),
        failure_class: ingestion_failure_class(&err),
        message: err.to_string(),
        retryable: ingestion_failure_is_retryable(&err),
    })
}
