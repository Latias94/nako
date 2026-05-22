use nako_addon_protocol::AddonScope;
use nako_api::extension::{
    AddonAcquisitionCandidateResponse, AddonAcquisitionCandidateSummary,
    AddonGeneratedArtifactResponse, AddonGeneratedArtifactSummary, AddonSideEffectResponse,
    SubmitAddonAcquisitionCandidateRequest, SubmitAddonGeneratedArtifactRequest,
    SubmitAddonSideEffectRequest,
};
use nako_core::{
    AcquisitionIntakeCandidateState, AcquisitionIntakeRepository, AcquisitionIntakeSourceKind,
    AddonPrincipal, AutomationArtifactId, AutomationJobInput, AutomationProviderId,
    AutomationProviderStatus, AutomationRepository, JobId, JobKind, JobRepository, LibraryId,
    LibraryItemRepository, LibraryRepository, MediaRepository, NakoError, NewAutomationArtifact,
    NewAutomationProviderConfig, NewJob, Result,
};

use super::{AddonAppService, runtime::AddonSideEffectRuntime};

impl AddonAppService {
    pub async fn submit_addon_side_effect(
        &self,
        raw_token: &str,
        request: SubmitAddonSideEffectRequest,
    ) -> Result<AddonSideEffectResponse> {
        AddonSideEffectRuntime::new(
            self.store.clone(),
            self.permits.clone(),
            self.storage_backends.clone(),
        )
        .submit(raw_token, request)
        .await
    }

    pub async fn submit_addon_generated_artifact(
        &self,
        raw_token: &str,
        request: SubmitAddonGeneratedArtifactRequest,
    ) -> Result<AddonGeneratedArtifactResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        ensure_addon_scope(&principal, AddonScope::ItemMetadataSuggest)?;
        let request = AddonGeneratedArtifactRuntimeRequest::normalize(request)?;
        self.validate_generated_artifact_target(&request).await?;
        let provider_id = addon_automation_provider_id(&principal.addon.manifest_id);

        if let Some(existing) = self
            .find_generated_artifact_replay(provider_id, &request)
            .await?
        {
            return Ok(AddonGeneratedArtifactResponse {
                artifact: AddonGeneratedArtifactSummary::from_record(existing),
                idempotent_replay: true,
            });
        }

        self.store
            .upsert_automation_provider(NewAutomationProviderConfig {
                id: provider_id,
                name: format!("Addon: {}", principal.addon.name),
                base_url: principal.addon.base_url.clone(),
                secret_env: None,
                capabilities: self
                    .addon_automation_provider_capabilities(provider_id, request.capability)
                    .await?,
                timeout_ms: 10_000,
                max_attempts: 1,
                status: AutomationProviderStatus::Enabled,
            })
            .await?;
        let input = AutomationJobInput {
            provider_id,
            capability: request.capability,
            library_id: request.library_id,
            item_id: request.item_id,
            source_id: request.source_id,
            prompt_json: request.prompt_json,
            idempotency_key: request.idempotency_key,
        };
        let job = self
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::AddonTask,
                resource_class: "addon.generated_artifact_handoff".to_owned(),
                library_id: request.library_id,
                source_id: request.source_id,
                input_json: Some(serde_json::to_string(&input).map_err(|err| {
                    NakoError::InvalidInput {
                        message: format!(
                            "failed to serialize addon generated artifact input: {err}"
                        ),
                    }
                })?),
            })
            .await?;
        let artifact = self
            .store
            .create_automation_artifact(NewAutomationArtifact {
                id: AutomationArtifactId::new(),
                job_id: job.id,
                provider_id,
                capability: request.capability,
                kind: request.kind,
                library_id: request.library_id,
                item_id: request.item_id,
                source_id: request.source_id,
                artifact_json: request.payload_json,
            })
            .await?;

        Ok(AddonGeneratedArtifactResponse {
            artifact: AddonGeneratedArtifactSummary::from_record(artifact),
            idempotent_replay: false,
        })
    }

    pub async fn submit_addon_acquisition_candidate(
        &self,
        raw_token: &str,
        request: SubmitAddonAcquisitionCandidateRequest,
    ) -> Result<AddonAcquisitionCandidateResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        ensure_addon_scope(&principal, AddonScope::AutomationRun)?;
        ensure_library_target_exists(&self.store, request.target_library_id).await?;
        let source_key =
            normalize_non_empty("addon acquisition candidate source_key", request.source_key)?;
        let existing = self
            .store
            .find_acquisition_intake_candidate_by_source_key(
                request.target_library_id,
                &AcquisitionIntakeSourceKind::AddonProposed,
                &source_key,
            )
            .await?;
        let diagnostic =
            crate::app::acquisition_intake::AcquisitionIntakeAppService::new_with_storage(
                self.store.clone(),
                self.storage_backends.clone(),
            )
            .record_candidate(
                crate::app::acquisition_intake::RecordAcquisitionIntakeCandidateRequest {
                    id: None,
                    target_library_id: request.target_library_id,
                    source_kind: AcquisitionIntakeSourceKind::AddonProposed,
                    source_key,
                    source_uri: request.source_uri,
                    display_name: request.display_name,
                    intended_locator: request.intended_locator,
                    size_bytes: request.size_bytes,
                    fingerprint: request.fingerprint,
                    managed_import_artifact_id: None,
                    state: Some(
                        request
                            .state
                            .unwrap_or(AcquisitionIntakeCandidateState::Ready),
                    ),
                    diagnostics_json: Some(serialize_json_field(
                        "addon acquisition candidate diagnostics",
                        &request.diagnostics,
                    )?),
                },
            )
            .await?;

        Ok(AddonAcquisitionCandidateResponse {
            candidate: AddonAcquisitionCandidateSummary {
                id: diagnostic.id,
                target_library_id: diagnostic.target_library_id,
                state: diagnostic.state,
                source_kind: diagnostic.source_kind,
                source_scheme: diagnostic.source_scheme,
                source_ref_redacted: diagnostic.source_uri_redacted,
                source_key_fingerprint: diagnostic.source_key_fingerprint,
                has_display_name: diagnostic.has_display_name,
                has_intended_locator: diagnostic.has_intended_locator,
                size_bytes: diagnostic.size_bytes,
                has_fingerprint: diagnostic.has_fingerprint,
                has_diagnostics: diagnostic.has_diagnostics,
                managed_import_artifact_id: diagnostic.managed_import_artifact_id,
                writes_library: false,
                creates_media_source: false,
                creates_managed_import: false,
                promotion_apply: false,
            },
            idempotent_replay: existing.is_some(),
        })
    }

    async fn validate_generated_artifact_target(
        &self,
        request: &AddonGeneratedArtifactRuntimeRequest,
    ) -> Result<()> {
        let Some(library_id) = request.library_id else {
            return Err(NakoError::InvalidInput {
                message: "addon generated artifact requires a library target".to_owned(),
            });
        };
        ensure_library_target_exists(&self.store, library_id).await?;
        let item_id = request.item_id.ok_or_else(|| NakoError::InvalidInput {
            message: "addon generated artifact requires a media item target".to_owned(),
        })?;
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::InvalidInput {
                message: "addon generated artifact media item target is missing".to_owned(),
            })?;
        self.store
            .get_library_item_state(library_id, item_id)
            .await?
            .ok_or_else(|| NakoError::InvalidInput {
                message: "addon generated artifact media item target is outside library".to_owned(),
            })?;
        if let Some(source_id) = request.source_id {
            let source = self
                .store
                .get_media_source(source_id)
                .await?
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "addon generated artifact media source target is missing".to_owned(),
                })?;
            if source.library_id != library_id || source.item_id != item_id {
                return Err(NakoError::InvalidInput {
                    message: "addon generated artifact target is stale".to_owned(),
                });
            }
        }

        Ok(())
    }

    async fn find_generated_artifact_replay(
        &self,
        provider_id: AutomationProviderId,
        request: &AddonGeneratedArtifactRuntimeRequest,
    ) -> Result<Option<nako_core::AutomationArtifactRecord>> {
        let item_id = request.item_id.ok_or_else(|| NakoError::InvalidInput {
            message: "addon generated artifact requires a media item target".to_owned(),
        })?;
        for artifact in self
            .store
            .list_automation_artifacts_for_item(
                item_id,
                nako_core::PageRequest::new(nako_core::PageRequest::MAX_LIMIT, 0),
            )
            .await?
        {
            if artifact.provider_id != provider_id {
                continue;
            }
            let Some(job) = self.store.get_job(artifact.job_id).await? else {
                continue;
            };
            let Some(input_json) = job.input_json.as_deref() else {
                continue;
            };
            let Ok(input) = serde_json::from_str::<AutomationJobInput>(input_json) else {
                continue;
            };
            if input.idempotency_key != request.idempotency_key {
                continue;
            }
            if input.provider_id == provider_id
                && input.capability == request.capability
                && input.library_id == request.library_id
                && input.item_id == request.item_id
                && input.source_id == request.source_id
                && input.prompt_json == request.prompt_json
                && artifact.kind == request.kind
                && artifact.artifact_json == request.payload_json
            {
                return Ok(Some(artifact));
            }

            return Err(NakoError::Conflict {
                message: "addon generated artifact idempotency key was already used for a different request"
                    .to_owned(),
            });
        }

        Ok(None)
    }

    async fn addon_automation_provider_capabilities(
        &self,
        provider_id: AutomationProviderId,
        capability: nako_core::AutomationCapability,
    ) -> Result<Vec<nako_core::AutomationCapability>> {
        let mut capabilities = self
            .store
            .get_automation_provider(provider_id)
            .await?
            .map(|provider| provider.capabilities)
            .unwrap_or_default();
        if !capabilities.contains(&capability) {
            capabilities.push(capability);
        }
        Ok(capabilities)
    }
}

struct AddonGeneratedArtifactRuntimeRequest {
    capability: nako_core::AutomationCapability,
    kind: nako_core::AutomationArtifactKind,
    library_id: Option<LibraryId>,
    item_id: Option<nako_core::MediaItemId>,
    source_id: Option<nako_core::MediaSourceId>,
    idempotency_key: String,
    prompt_json: String,
    payload_json: String,
}

impl AddonGeneratedArtifactRuntimeRequest {
    fn normalize(request: SubmitAddonGeneratedArtifactRequest) -> Result<Self> {
        Ok(Self {
            capability: request.capability,
            kind: request.kind,
            library_id: request.library_id,
            item_id: request.item_id,
            source_id: request.source_id,
            idempotency_key: normalize_non_empty(
                "addon generated artifact idempotency_key",
                request.idempotency_key,
            )?,
            prompt_json: serialize_json_field("addon generated artifact prompt", &request.prompt)?,
            payload_json: serialize_json_field(
                "addon generated artifact payload",
                &request.payload,
            )?,
        })
    }
}

async fn ensure_library_target_exists(
    store: &nako_db::NakoDatabase,
    library_id: LibraryId,
) -> Result<()> {
    store
        .get_library(library_id)
        .await?
        .ok_or_else(|| NakoError::InvalidInput {
            message: "addon handoff target library is missing".to_owned(),
        })?;
    Ok(())
}

fn ensure_addon_scope(principal: &AddonPrincipal, scope: AddonScope) -> Result<()> {
    if principal
        .addon
        .granted_scopes
        .iter()
        .any(|granted| granted == scope.as_str())
    {
        return Ok(());
    }

    Err(NakoError::Forbidden {
        message: format!(
            "addon {} is not granted {}",
            principal.addon.id,
            scope.as_str()
        ),
    })
}

fn normalize_non_empty(label: &str, value: String) -> Result<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("{label} cannot be empty"),
        });
    }
    Ok(value)
}

fn serialize_json_field(label: &str, value: &serde_json::Value) -> Result<String> {
    serde_json::to_string(value).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize {label}: {err}"),
    })
}

fn addon_automation_provider_id(manifest_id: &str) -> AutomationProviderId {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(format!("nako.addon.automation-provider.v1:{manifest_id}"));
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    AutomationProviderId::from_uuid(uuid::Uuid::from_bytes(bytes))
}
