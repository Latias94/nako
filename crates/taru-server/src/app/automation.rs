use async_trait::async_trait;
use taru_api::extension::{
    AutomationArtifactsResponse, AutomationProviderResponse, AutomationProvidersResponse,
    EnqueueAutomationJobRequest, UpsertAutomationProviderRequest,
};
use taru_automation::AutomationJobService;
use taru_core::{
    AutomationCapability, AutomationProviderId, AutomationRepository, Job, JobId, JobRepository,
    MediaItemId, MediaRepository, NewAutomationProviderConfig, PageRequest, Result, TaruError,
};
use taru_db::TaruDatabase;

#[derive(Clone, Debug)]
struct UnavailableAutomationProvider;

#[async_trait]
impl taru_automation::AutomationProvider for UnavailableAutomationProvider {
    fn descriptor(&self) -> taru_automation::AutomationProviderDescriptor {
        taru_automation::AutomationProviderDescriptor {
            id: AutomationProviderId::new(),
            name: "unavailable".to_owned(),
            capabilities: vec![
                AutomationCapability::Recommendation,
                AutomationCapability::MetadataCleanup,
                AutomationCapability::Summary,
                AutomationCapability::TitleMatch,
            ],
        }
    }

    async fn run(
        &self,
        _request: taru_automation::AutomationRequest,
    ) -> Result<taru_automation::AutomationOutcome> {
        Err(TaruError::Provider {
            provider: "automation".to_owned(),
            message: "no concrete automation provider runner is configured".to_owned(),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AutomationAppService {
    store: TaruDatabase,
}

impl AutomationAppService {
    pub(crate) fn new(store: TaruDatabase) -> Self {
        Self { store }
    }

    fn normalize_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<NewAutomationProviderConfig> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "automation provider name cannot be empty".to_owned(),
            });
        }

        let base_url = request.base_url.trim().to_owned();
        if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
            return Err(TaruError::InvalidInput {
                message: "automation provider base_url must use http or https".to_owned(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        let capabilities = request
            .capabilities
            .into_iter()
            .filter(|capability| seen.insert(*capability))
            .collect::<Vec<_>>();
        if capabilities.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "automation provider must declare at least one capability".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(30_000);
        if !(100..=120_000).contains(&timeout_ms) {
            return Err(TaruError::InvalidInput {
                message: "automation provider timeout_ms must be between 100 and 120000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(2);
        if !(1..=5).contains(&max_attempts) {
            return Err(TaruError::InvalidInput {
                message: "automation provider max_attempts must be between 1 and 5".to_owned(),
            });
        }

        let secret_env = request.secret_env.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(NewAutomationProviderConfig {
            id: request.id.unwrap_or_else(AutomationProviderId::new),
            name,
            base_url,
            secret_env,
            capabilities,
            timeout_ms,
            max_attempts,
            status: request.status,
        })
    }

    pub async fn upsert_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<AutomationProviderResponse> {
        let provider = self.normalize_automation_provider(request)?;
        let provider = self.store.upsert_automation_provider(provider).await?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn get_automation_provider(
        &self,
        provider_id: AutomationProviderId,
    ) -> Result<AutomationProviderResponse> {
        let provider = self
            .store
            .get_automation_provider(provider_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "automation_provider",
                id: provider_id.to_string(),
            })?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn list_enabled_automation_providers(&self) -> Result<AutomationProvidersResponse> {
        let providers = self.store.list_enabled_automation_providers().await?;

        Ok(AutomationProvidersResponse { providers })
    }

    pub async fn enqueue_automation_job(
        &self,
        request: EnqueueAutomationJobRequest,
    ) -> Result<Job> {
        let input = request
            .into_job_input()
            .map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize automation prompt: {err}"),
            })?;
        let service = AutomationJobService::new(UnavailableAutomationProvider);

        service.enqueue_job(&self.store, input).await
    }

    pub async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<AutomationArtifactsResponse> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })?;
        let artifacts = self.store.list_automation_artifacts_for_job(job_id).await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }

    pub async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<AutomationArtifactsResponse> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let artifacts = self
            .store
            .list_automation_artifacts_for_item(item_id, page)
            .await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }
}
