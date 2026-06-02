use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationCapability, AutomationJobInput,
    AutomationJobSummary, AutomationProviderConfigRecord, AutomationProviderId,
    AutomationProviderStatus, AutomationRepository, Job, JobId, JobKind, JobPriority,
    JobRepository, NakoError, NewAutomationArtifact, NewJob, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProviderDescriptor {
    pub id: AutomationProviderId,
    pub name: String,
    pub capabilities: Vec<AutomationCapability>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AutomationRequest {
    pub job_id: JobId,
    pub provider: AutomationProviderConfigRecord,
    pub capability: AutomationCapability,
    pub input: Value,
    pub secret: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AutomationOutcome {
    pub job_id: JobId,
    pub artifact_kind: AutomationArtifactKind,
    pub output: Value,
    pub accepted_into_canonical_metadata: bool,
}

#[async_trait]
pub trait AutomationProvider: Send + Sync {
    fn descriptor(&self) -> AutomationProviderDescriptor;

    async fn run(&self, request: AutomationRequest) -> Result<AutomationOutcome>;
}

#[derive(Clone, Debug, Default)]
pub struct AutomationCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl AutomationCancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        while !self.is_cancelled() {
            time::sleep(Duration::from_millis(25)).await;
        }
    }
}

#[derive(Clone, Debug)]
pub struct AutomationJobService<P> {
    provider: Arc<P>,
}

impl<P> AutomationJobService<P>
where
    P: AutomationProvider,
{
    #[must_use]
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
        }
    }

    pub async fn enqueue_job<R>(&self, repository: &R, input: AutomationJobInput) -> Result<Job>
    where
        R: AutomationRepository + JobRepository,
    {
        let provider = automation_provider_or_not_found(repository, input.provider_id).await?;
        validate_provider_for_capability(&provider, input.capability)?;
        let input_json = serde_json::to_string(&input).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize automation job input: {err}"),
        })?;

        repository
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::Automation,
                resource_class: "automation.external_api".to_owned(),
                priority: JobPriority::Normal,
                library_id: input.library_id,
                source_id: input.source_id,
                input_json: Some(input_json),
            })
            .await
    }

    pub async fn run_job_once<R>(
        &self,
        repository: &R,
        job_id: JobId,
        secret: Option<String>,
        cancellation: AutomationCancellationToken,
    ) -> Result<Job>
    where
        R: AutomationRepository + JobRepository,
    {
        let queued = repository
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })?;
        if queued.kind != JobKind::Automation {
            return Err(NakoError::InvalidInput {
                message: format!("job {job_id} is not an automation job"),
            });
        }
        let input = parse_job_input(&queued)?;
        let provider = automation_provider_or_not_found(repository, input.provider_id).await?;
        validate_provider_for_capability(&provider, input.capability)?;
        let started = repository.start_job(job_id).await?;
        let run = self.run_provider(&started, input, provider, secret, cancellation);

        match run.await {
            Ok((summary, artifact)) => {
                let summary_json =
                    serde_json::to_string(&summary).map_err(|err| NakoError::InvalidInput {
                        message: format!("failed to serialize automation job summary: {err}"),
                    })?;
                repository.create_automation_artifact(artifact).await?;
                repository.succeed_job(job_id, Some(summary_json)).await
            }
            Err(err) => {
                let failed = repository.fail_job(job_id, safe_error(&err)).await?;
                Err(NakoError::Provider {
                    provider: "automation".to_owned(),
                    message: failed.error.unwrap_or_else(|| err.to_string()),
                })
            }
        }
    }

    async fn run_provider(
        &self,
        job: &Job,
        input: AutomationJobInput,
        provider: AutomationProviderConfigRecord,
        secret: Option<String>,
        cancellation: AutomationCancellationToken,
    ) -> Result<(AutomationJobSummary, NewAutomationArtifact)> {
        let prompt =
            serde_json::from_str(&input.prompt_json).map_err(|err| NakoError::InvalidInput {
                message: format!("failed to parse automation prompt JSON: {err}"),
            })?;
        let request = AutomationRequest {
            job_id: job.id,
            provider: provider.clone(),
            capability: input.capability,
            input: prompt,
            secret,
        };
        let timeout = Duration::from_millis(provider.timeout_ms);
        let mut last_error = None;
        let mut outcome = None;
        let mut attempt_count = 0;

        for attempt in 1..=provider.max_attempts {
            attempt_count = attempt;
            let request = request.clone();
            let provider_run = self.provider.run(request);
            let result = tokio::select! {
                () = cancellation.cancelled() => {
                    Err(NakoError::Provider {
                        provider: provider.name.clone(),
                        message: "automation job was cancelled".to_owned(),
                    })
                }
                result = time::timeout(timeout, provider_run) => {
                    result.map_err(|_| NakoError::Provider {
                        provider: provider.name.clone(),
                        message: format!("automation provider timed out after {} ms", provider.timeout_ms),
                    })?
                }
            };

            match result {
                Ok(value) => {
                    outcome = Some(value);
                    break;
                }
                Err(err) if !cancellation.is_cancelled() && attempt < provider.max_attempts => {
                    last_error = Some(err);
                }
                Err(err) => return Err(err),
            }
        }

        let outcome = outcome.ok_or_else(|| {
            last_error.unwrap_or_else(|| NakoError::Provider {
                provider: provider.name.clone(),
                message: "automation provider did not return an outcome".to_owned(),
            })
        })?;
        if outcome.accepted_into_canonical_metadata {
            return Err(NakoError::InvalidInput {
                message: "automation outcomes cannot mutate canonical metadata during M5.3"
                    .to_owned(),
            });
        }

        let output_json =
            serde_json::to_string(&outcome.output).map_err(|err| NakoError::InvalidInput {
                message: format!("failed to serialize automation provider output: {err}"),
            })?;
        let artifact_id = AutomationArtifactId::new();
        let artifact = NewAutomationArtifact {
            id: artifact_id,
            job_id: job.id,
            provider_id: provider.id,
            capability: input.capability,
            kind: outcome.artifact_kind,
            library_id: input.library_id,
            item_id: input.item_id,
            source_id: input.source_id,
            artifact_json: output_json.clone(),
        };
        let summary = AutomationJobSummary {
            provider_id: provider.id,
            capability: input.capability,
            accepted_into_canonical_metadata: false,
            artifact_ids: vec![artifact_id],
            output_json,
            attempt_count,
        };

        Ok((summary, artifact))
    }
}

fn parse_job_input(job: &Job) -> Result<AutomationJobInput> {
    let Some(input_json) = &job.input_json else {
        return Err(NakoError::InvalidInput {
            message: format!("automation job {} is missing input JSON", job.id),
        });
    };

    serde_json::from_str(input_json).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to parse automation job input: {err}"),
    })
}

async fn automation_provider_or_not_found<R>(
    repository: &R,
    id: AutomationProviderId,
) -> Result<AutomationProviderConfigRecord>
where
    R: AutomationRepository,
{
    repository
        .get_automation_provider(id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "automation_provider",
            id: id.to_string(),
        })
}

fn validate_provider_for_capability(
    provider: &AutomationProviderConfigRecord,
    capability: AutomationCapability,
) -> Result<()> {
    if provider.status != AutomationProviderStatus::Enabled {
        return Err(NakoError::InvalidInput {
            message: format!("automation provider {} is disabled", provider.id),
        });
    }
    if !provider.capabilities.contains(&capability) {
        return Err(NakoError::InvalidInput {
            message: format!(
                "automation provider {} does not support {}",
                provider.id,
                capability.as_str()
            ),
        });
    }

    Ok(())
}

fn safe_error(error: &NakoError) -> String {
    match error {
        NakoError::Provider { provider, message } => {
            format!("automation provider {provider} failed: {message}")
        }
        NakoError::Storage { .. } => "automation job storage operation failed".to_owned(),
        NakoError::Database { .. } => "automation job database operation failed".to_owned(),
        NakoError::InvalidInput { .. }
        | NakoError::NotFound { .. }
        | NakoError::Conflict { .. }
        | NakoError::Unauthorized { .. }
        | NakoError::Forbidden { .. }
        | NakoError::Unsupported(_) => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use nako_core::{
        AutomationArtifactStatus, AutomationProviderStatus, DatabaseLifecycle, Library, LibraryId,
        LibraryOptions, LibraryPreset, LibraryRepository, NewAutomationProviderConfig,
    };
    use nako_db::NakoDatabase;

    use super::*;

    struct FakeProvider {
        accepted: bool,
        fail_first_attempts: usize,
        requests: Mutex<Vec<AutomationRequest>>,
    }

    #[async_trait]
    impl AutomationProvider for FakeProvider {
        fn descriptor(&self) -> AutomationProviderDescriptor {
            AutomationProviderDescriptor {
                id: AutomationProviderId::new(),
                name: "fake".to_owned(),
                capabilities: vec![AutomationCapability::Summary],
            }
        }

        async fn run(&self, request: AutomationRequest) -> Result<AutomationOutcome> {
            let mut requests = self.requests.lock().unwrap();
            requests.push(request.clone());
            if requests.len() <= self.fail_first_attempts {
                return Err(NakoError::Provider {
                    provider: "fake".to_owned(),
                    message: "transient failure".to_owned(),
                });
            }
            drop(requests);

            Ok(AutomationOutcome {
                job_id: request.job_id,
                artifact_kind: AutomationArtifactKind::Summary,
                output: serde_json::json!({"summary":"Generated summary"}),
                accepted_into_canonical_metadata: self.accepted,
            })
        }
    }

    #[tokio::test]
    async fn automation_job_runner_persists_proposed_artifact_and_summary() {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();
        let provider = store
            .upsert_automation_provider(NewAutomationProviderConfig {
                id: AutomationProviderId::new(),
                name: "fake".to_owned(),
                base_url: "https://example.test/automation".to_owned(),
                secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
                capabilities: vec![AutomationCapability::Summary],
                timeout_ms: 5_000,
                max_attempts: 2,
                status: AutomationProviderStatus::Enabled,
            })
            .await
            .unwrap();
        let service = AutomationJobService::new(FakeProvider {
            accepted: false,
            fail_first_attempts: 0,
            requests: Mutex::new(Vec::new()),
        });
        let job = service
            .enqueue_job(
                &store,
                AutomationJobInput {
                    provider_id: provider.id,
                    capability: AutomationCapability::Summary,
                    library_id: Some(library.id),
                    item_id: None,
                    source_id: None,
                    prompt_json: r#"{"title":"The Matrix"}"#.to_owned(),
                    idempotency_key: "summary:matrix".to_owned(),
                },
            )
            .await
            .unwrap();

        let completed = service
            .run_job_once(
                &store,
                job.id,
                Some("secret".to_owned()),
                AutomationCancellationToken::new(),
            )
            .await
            .unwrap();

        assert_eq!(completed.kind, JobKind::Automation);
        assert!(completed.summary_json.is_some());
        assert!(!completed.input_json.unwrap().contains("secret"));
        let artifacts = store
            .list_automation_artifacts_for_job(job.id)
            .await
            .unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].status, AutomationArtifactStatus::Proposed);
        assert!(artifacts[0].accepted_at.is_none());
        let proposals = store
            .list_generated_artifact_proposals(nako_core::PageRequest::first_page())
            .await
            .unwrap();
        assert_eq!(proposals.len(), 1);
        assert_eq!(
            proposals[0].readiness.status,
            nako_core::GeneratedArtifactReadinessStatus::Ready
        );
        assert!(proposals[0].readiness.actionable);
        assert_eq!(
            proposals[0].target.kind,
            nako_core::GeneratedArtifactTargetKind::Library
        );
        assert_eq!(proposals[0].target.library_id, Some(library.id));
        assert_eq!(proposals[0].provenance.attempt_count, Some(1));
        assert!(
            proposals[0]
                .provenance
                .prompt_fingerprint
                .as_deref()
                .is_some_and(|fingerprint| fingerprint.starts_with("sha256:"))
        );
        let proposal_body = serde_json::to_string(&proposals[0]).unwrap();
        assert!(!proposal_body.contains("The Matrix"));
        assert!(!proposal_body.contains("Generated summary"));
        assert!(!proposal_body.contains("secret"));
    }

    #[tokio::test]
    async fn automation_job_runner_rejects_canonical_mutation() {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let provider = store
            .upsert_automation_provider(NewAutomationProviderConfig {
                id: AutomationProviderId::new(),
                name: "fake".to_owned(),
                base_url: "https://example.test/automation".to_owned(),
                secret_env: None,
                capabilities: vec![AutomationCapability::Summary],
                timeout_ms: 5_000,
                max_attempts: 1,
                status: AutomationProviderStatus::Enabled,
            })
            .await
            .unwrap();
        let service = AutomationJobService::new(FakeProvider {
            accepted: true,
            fail_first_attempts: 0,
            requests: Mutex::new(Vec::new()),
        });
        let job = service
            .enqueue_job(
                &store,
                AutomationJobInput {
                    provider_id: provider.id,
                    capability: AutomationCapability::Summary,
                    library_id: None,
                    item_id: None,
                    source_id: None,
                    prompt_json: "{}".to_owned(),
                    idempotency_key: "summary:blocked".to_owned(),
                },
            )
            .await
            .unwrap();

        let err = service
            .run_job_once(&store, job.id, None, AutomationCancellationToken::new())
            .await
            .unwrap_err();

        assert!(err.to_string().contains("canonical metadata"));
        let failed = store.get_job(job.id).await.unwrap().unwrap();
        assert!(failed.error.unwrap().contains("canonical metadata"));
    }

    #[tokio::test]
    async fn automation_job_runner_retries_provider_failures() {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let provider = store
            .upsert_automation_provider(NewAutomationProviderConfig {
                id: AutomationProviderId::new(),
                name: "fake".to_owned(),
                base_url: "https://example.test/automation".to_owned(),
                secret_env: None,
                capabilities: vec![AutomationCapability::Summary],
                timeout_ms: 5_000,
                max_attempts: 2,
                status: AutomationProviderStatus::Enabled,
            })
            .await
            .unwrap();
        let service = AutomationJobService::new(FakeProvider {
            accepted: false,
            fail_first_attempts: 1,
            requests: Mutex::new(Vec::new()),
        });
        let job = service
            .enqueue_job(
                &store,
                AutomationJobInput {
                    provider_id: provider.id,
                    capability: AutomationCapability::Summary,
                    library_id: None,
                    item_id: None,
                    source_id: None,
                    prompt_json: "{}".to_owned(),
                    idempotency_key: "summary:retry".to_owned(),
                },
            )
            .await
            .unwrap();

        let completed = service
            .run_job_once(&store, job.id, None, AutomationCancellationToken::new())
            .await
            .unwrap();

        let summary = serde_json::from_str::<AutomationJobSummary>(
            completed.summary_json.as_deref().unwrap(),
        )
        .unwrap();
        assert_eq!(summary.attempt_count, 2);
        assert_eq!(
            store
                .list_automation_artifacts_for_job(job.id)
                .await
                .unwrap()
                .len(),
            1
        );
    }
}
