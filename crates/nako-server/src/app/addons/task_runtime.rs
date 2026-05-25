use nako_addon_client::{
    AddonClientError, AddonTaskCallFailure, AddonTaskCallRequest, ReqwestAddonTransport,
    call_addon_task_with_outcome,
};
use nako_addon_protocol::{AddonAuth, AddonScope, AddonTaskDeclaration, validate_manifest};
use nako_api::extension::{
    AddonTaskRunDispatchMode, AddonTaskRunLease, AddonTaskRunResponse, AddonTaskRunSummary,
    AddonTaskRunsResponse, CancelAddonTaskRunRequest, ClaimAddonTaskRunRequest,
    ClaimAddonTaskRunResponse, CompleteAddonTaskRunRequest, CreateAddonTaskRunRequest,
    FailAddonTaskRunRequest, ReportAddonTaskRunProgressRequest, RetryAddonTaskRunRequest,
    addon_task_run_progress_json, addon_task_run_result_json,
};
use nako_core::{
    ADDON_TASK_RUN_INPUT_SCHEMA, AddonId, AddonManifestFingerprint, AddonRegistrationRecord,
    AddonRepository, AddonRoutingDeclarationKind, AddonRoutingPlanStatus, AddonRoutingPlanTarget,
    AddonStatus, AddonTaskRunClaimRequest, AddonTaskRunLeaseGuard, AddonTaskRunListFilter,
    AddonTaskRunRepository, AddonTaskRunRequestFingerprint, CancelAddonTaskRun,
    CompleteAddonTaskRun, FailAddonTaskRun, JobId, JobKind, JobStatus, JobWorkerId,
    LeasedAddonTaskRun, NakoError, NewAddonTaskRun, NewJob, PageRequest,
    ReportAddonTaskRunProgress, Result,
};

use super::{
    AddonAppService, declaration_scopes_granted, ensure_addon_accepts_runtime_authority,
    resolve_outbound_task_dispatch_secret, stored_granted_scopes,
};

impl AddonAppService {
    pub async fn create_addon_task_run(
        &self,
        addon_id: AddonId,
        request: CreateAddonTaskRunRequest,
    ) -> Result<AddonTaskRunResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "create addon task run")?;
        if addon.status != nako_core::AddonStatus::Enabled {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is not enabled"),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;
        let task = manifest_task_declaration(&manifest.tasks, &request.declaration_id)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        ensure_task_scopes_granted(task, &granted_scopes, addon_id)?;
        self.ensure_executable_task_routing_plan(addon_id, &request.declaration_id)
            .await?;

        let idempotency_key = normalized_idempotency_key(&request.idempotency_key)?;
        let request = CreateAddonTaskRunRequest {
            idempotency_key: idempotency_key.clone(),
            ..request
        };
        let job_id = JobId::new();
        let manifest_fingerprint = AddonManifestFingerprint::new(&addon.manifest_json);
        let addon_for_dispatch = addon.clone();
        let input_json = addon_task_run_input_json(
            addon_id,
            &addon.manifest_id,
            &addon.version,
            manifest_fingerprint.as_str(),
            task,
            &request,
            1,
            None,
        )?;
        let request_fingerprint = AddonTaskRunRequestFingerprint::new(
            &addon.manifest_id,
            &addon.version,
            &manifest_fingerprint,
            &task.id,
            &task.path,
            &input_json,
        );
        let created = self
            .store
            .create_addon_task_run(
                NewJob {
                    id: job_id,
                    kind: JobKind::AddonTask,
                    resource_class: addon_task_resource_class(&task.id),
                    library_id: request.library_id,
                    source_id: request.source_id,
                    input_json: Some(input_json.clone()),
                },
                NewAddonTaskRun {
                    job_id,
                    addon_id,
                    manifest_id: addon.manifest_id.clone(),
                    manifest_version: addon.version.clone(),
                    manifest_fingerprint,
                    declaration_id: task.id.clone(),
                    declaration_name: task.name.clone(),
                    declaration_path: task.path.clone(),
                    idempotency_key,
                    request_fingerprint,
                    attempt: 1,
                    max_attempts: task.max_attempts,
                    retry_of_job_id: None,
                    input_json,
                },
            )
            .await?;
        if request.dispatch == AddonTaskRunDispatchMode::Direct && !created.idempotent_replay {
            self.spawn_direct_addon_task_dispatch(
                addon_for_dispatch,
                task.id.clone(),
                job_id,
                created.run.job.resource_class.clone(),
            );
        }

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(created.run),
            idempotent_replay: created.idempotent_replay,
        })
    }

    pub async fn get_addon_task_run(
        &self,
        addon_id: AddonId,
        job_id: JobId,
    ) -> Result<AddonTaskRunResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let run = self
            .store
            .get_addon_task_run(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_task_run",
                id: job_id.to_string(),
            })?;
        ensure_run_belongs_to_addon(&run, addon_id)?;

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(run),
            idempotent_replay: false,
        })
    }

    pub async fn list_addon_task_runs(
        &self,
        addon_id: AddonId,
        declaration_id: Option<String>,
        page: PageRequest,
    ) -> Result<AddonTaskRunsResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let runs = self
            .store
            .list_addon_task_runs(
                AddonTaskRunListFilter {
                    addon_id: Some(addon_id),
                    declaration_id,
                    ..AddonTaskRunListFilter::default()
                },
                page,
            )
            .await?
            .into_iter()
            .map(AddonTaskRunSummary::from_record)
            .collect();

        Ok(AddonTaskRunsResponse { runs })
    }

    pub async fn retry_addon_task_run(
        &self,
        addon_id: AddonId,
        job_id: JobId,
        request: RetryAddonTaskRunRequest,
    ) -> Result<AddonTaskRunResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "retry addon task run")?;
        let previous = self
            .store
            .get_addon_task_run(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_task_run",
                id: job_id.to_string(),
            })?;
        ensure_run_belongs_to_addon(&previous, addon_id)?;
        if previous.job.status != JobStatus::Failed {
            return Err(NakoError::Conflict {
                message: "only failed addon task runs can be retried".to_owned(),
            });
        }
        if previous
            .max_attempts
            .is_some_and(|max_attempts| previous.attempt >= max_attempts)
        {
            return Err(NakoError::Conflict {
                message: "addon task run has exhausted max_attempts".to_owned(),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        let task = manifest_task_declaration(&manifest.tasks, &previous.declaration_id)?;
        let new_job_id = JobId::new();
        let manifest_fingerprint = AddonManifestFingerprint::new(&addon.manifest_json);
        let addon_for_dispatch = addon.clone();
        let idempotency_key = normalized_idempotency_key(&request.idempotency_key)?;
        let retry_request = CreateAddonTaskRunRequest {
            declaration_id: previous.declaration_id.clone(),
            idempotency_key: idempotency_key.clone(),
            dispatch: retry_dispatch_from_previous_input(&previous.input_json)?,
            library_id: previous.job.library_id,
            source_id: previous.job.source_id,
            payload: retry_payload_from_previous_input(&previous.input_json)?,
        };
        let attempt = previous.attempt.saturating_add(1);
        let input_json = addon_task_run_input_json(
            addon_id,
            &addon.manifest_id,
            &addon.version,
            manifest_fingerprint.as_str(),
            task,
            &retry_request,
            attempt,
            Some(job_id),
        )?;
        let request_fingerprint = AddonTaskRunRequestFingerprint::new(
            &addon.manifest_id,
            &addon.version,
            &manifest_fingerprint,
            &task.id,
            &task.path,
            &input_json,
        );
        let created = self
            .store
            .create_addon_task_run(
                NewJob {
                    id: new_job_id,
                    kind: JobKind::AddonTask,
                    resource_class: addon_task_resource_class(&task.id),
                    library_id: retry_request.library_id,
                    source_id: retry_request.source_id,
                    input_json: Some(input_json.clone()),
                },
                NewAddonTaskRun {
                    job_id: new_job_id,
                    addon_id,
                    manifest_id: addon.manifest_id.clone(),
                    manifest_version: addon.version.clone(),
                    manifest_fingerprint,
                    declaration_id: task.id.clone(),
                    declaration_name: task.name.clone(),
                    declaration_path: task.path.clone(),
                    idempotency_key,
                    request_fingerprint,
                    attempt,
                    max_attempts: task.max_attempts,
                    retry_of_job_id: Some(job_id),
                    input_json,
                },
            )
            .await?;
        if retry_request.dispatch == AddonTaskRunDispatchMode::Direct && !created.idempotent_replay
        {
            self.spawn_direct_addon_task_dispatch(
                addon_for_dispatch,
                task.id.clone(),
                new_job_id,
                created.run.job.resource_class.clone(),
            );
        }

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(created.run),
            idempotent_replay: created.idempotent_replay,
        })
    }

    pub async fn claim_addon_task_run(
        &self,
        raw_token: &str,
        request: ClaimAddonTaskRunRequest,
    ) -> Result<ClaimAddonTaskRunResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        let claimed = self
            .store
            .claim_next_addon_task_run(AddonTaskRunClaimRequest {
                addon_id: principal.addon.id,
                worker_id: request.worker_id.unwrap_or_else(JobWorkerId::new),
                lease_duration_ms: request.lease_duration_ms,
                declaration_id: request.declaration_id,
                job_id: None,
            })
            .await?;

        Ok(ClaimAddonTaskRunResponse {
            run: claimed.map(addon_task_run_lease_from_leased).transpose()?,
        })
    }

    pub async fn report_addon_task_run_progress(
        &self,
        raw_token: &str,
        request: ReportAddonTaskRunProgressRequest,
    ) -> Result<AddonTaskRunLease> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        self.ensure_addon_owns_task_run(principal.addon.id, request.guard.job_id)
            .await?;
        let progress = addon_task_run_progress_json(
            request.stage,
            request.percent,
            request.message,
            request.metrics,
        )
        .to_string();
        let reported = self
            .store
            .report_addon_task_run_progress(ReportAddonTaskRunProgress {
                guard: request.guard,
                lease_duration_ms: request.lease_duration_ms,
                progress_json: progress,
            })
            .await?;

        addon_task_run_lease_from_leased(reported)
    }

    pub async fn complete_addon_task_run(
        &self,
        raw_token: &str,
        request: CompleteAddonTaskRunRequest,
    ) -> Result<AddonTaskRunResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        self.ensure_addon_owns_task_run(principal.addon.id, request.guard.job_id)
            .await?;
        let result = addon_task_run_result_json("succeeded", request.output, None, None);
        let completed = self
            .store
            .complete_addon_task_run(CompleteAddonTaskRun {
                guard: request.guard,
                result_json: result.to_string(),
            })
            .await?;

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(completed),
            idempotent_replay: false,
        })
    }

    pub async fn fail_addon_task_run(
        &self,
        raw_token: &str,
        request: FailAddonTaskRunRequest,
    ) -> Result<AddonTaskRunResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        self.ensure_addon_owns_task_run(principal.addon.id, request.guard.job_id)
            .await?;
        let safe_error_code = normalized_safe_error_code(&request.safe_error_code)?;
        let result = addon_task_run_result_json(
            "failed",
            request.output,
            Some(&safe_error_code),
            request.retry_after_ms,
        );
        let failed = self
            .store
            .fail_addon_task_run(FailAddonTaskRun {
                guard: request.guard,
                safe_error_code,
                result_json: Some(result.to_string()),
            })
            .await?;

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(failed),
            idempotent_replay: false,
        })
    }

    pub async fn cancel_addon_task_run(
        &self,
        raw_token: &str,
        request: CancelAddonTaskRunRequest,
    ) -> Result<AddonTaskRunResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        self.ensure_addon_owns_task_run(principal.addon.id, request.guard.job_id)
            .await?;
        let result = addon_task_run_result_json("cancelled", request.output, None, None);
        let cancelled = self
            .store
            .cancel_addon_task_run(CancelAddonTaskRun {
                guard: request.guard,
                result_json: Some(result.to_string()),
            })
            .await?;

        Ok(AddonTaskRunResponse {
            run: AddonTaskRunSummary::from_record(cancelled),
            idempotent_replay: false,
        })
    }

    pub(super) async fn ensure_executable_task_routing_plan(
        &self,
        addon_id: AddonId,
        declaration_id: &str,
    ) -> Result<()> {
        let plans = self.store.list_addon_routing_plans(addon_id).await?;
        let Some(plan) = plans.iter().find(|plan| {
            plan.declaration_kind == AddonRoutingDeclarationKind::Task
                && plan.declaration_id == declaration_id
        }) else {
            return Err(NakoError::Conflict {
                message: format!(
                    "addon task declaration {declaration_id} has no synchronized routing plan"
                ),
            });
        };

        if plan.status == AddonRoutingPlanStatus::Executable
            && plan.target == AddonRoutingPlanTarget::AddonTaskJob
            && plan.job_kind == Some(JobKind::AddonTask)
        {
            return Ok(());
        }

        Err(NakoError::Conflict {
            message: plan.safe_reason_code.as_ref().map_or_else(
                || format!("addon task declaration {declaration_id} is not executable"),
                |reason| format!("addon task declaration {declaration_id} is deferred: {reason}"),
            ),
        })
    }

    async fn ensure_addon_owns_task_run(&self, addon_id: AddonId, job_id: JobId) -> Result<()> {
        let run = self
            .store
            .get_addon_task_run(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_task_run",
                id: job_id.to_string(),
            })?;
        ensure_run_belongs_to_addon(&run, addon_id)
    }

    fn spawn_direct_addon_task_dispatch(
        &self,
        addon: AddonRegistrationRecord,
        declaration_id: String,
        job_id: JobId,
        resource_class: String,
    ) {
        let service = self.clone();
        self.runtime.spawn_job(
            "addon_task_direct_dispatch",
            resource_class,
            job_id,
            move |_context| async move {
                service
                    .dispatch_addon_task_run_direct(addon, declaration_id, job_id)
                    .await
            },
        );
    }

    async fn dispatch_addon_task_run_direct(
        &self,
        addon: AddonRegistrationRecord,
        declaration_id: String,
        job_id: JobId,
    ) -> Result<nako_core::Job> {
        let Some(claimed) = self
            .store
            .claim_next_addon_task_run(AddonTaskRunClaimRequest {
                addon_id: addon.id,
                worker_id: JobWorkerId::new(),
                lease_duration_ms: 30_000,
                declaration_id: Some(declaration_id.clone()),
                job_id: Some(job_id),
            })
            .await?
        else {
            return self
                .store
                .get_addon_task_run(job_id)
                .await?
                .map(|run| run.job)
                .ok_or_else(|| NakoError::NotFound {
                    entity: "addon_task_run",
                    id: job_id.to_string(),
                });
        };

        if claimed.run.job.id != job_id {
            return Err(NakoError::Conflict {
                message: format!(
                    "direct addon task dispatch claimed unexpected job {} while dispatching {job_id}",
                    claimed.run.job.id
                ),
            });
        }

        let guard = AddonTaskRunLeaseGuard::from(claimed.lease.guard());
        let dispatch = self
            .call_addon_task_run_path(&addon, &claimed.run, &claimed.lease)
            .await;
        match dispatch {
            Ok(dispatch) if dispatch.cancel_requested_at.is_some() => {
                let result = addon_task_run_result_json(
                    "cancelled",
                    serde_json::json!({
                        "completed_output": dispatch.output,
                    }),
                    None,
                    None,
                );
                Ok(self
                    .store
                    .cancel_addon_task_run(CancelAddonTaskRun {
                        guard: dispatch.guard,
                        result_json: Some(result.to_string()),
                    })
                    .await?
                    .job)
            }
            Ok(dispatch) => {
                let result = addon_task_run_result_json("succeeded", dispatch.output, None, None);
                Ok(self
                    .store
                    .complete_addon_task_run(CompleteAddonTaskRun {
                        guard: dispatch.guard,
                        result_json: result.to_string(),
                    })
                    .await?
                    .job)
            }
            Err(failure) => {
                let result = addon_task_run_result_json(
                    "failed",
                    failure.output,
                    Some(&failure.safe_error_code),
                    None,
                );
                Ok(self
                    .store
                    .fail_addon_task_run(FailAddonTaskRun {
                        guard,
                        safe_error_code: failure.safe_error_code,
                        result_json: Some(result.to_string()),
                    })
                    .await?
                    .job)
            }
        }
    }

    async fn call_addon_task_run_path(
        &self,
        addon: &AddonRegistrationRecord,
        run: &nako_core::AddonTaskRunRecord,
        lease: &nako_core::JobLeaseRecord,
    ) -> std::result::Result<AddonTaskDirectDispatchOutput, AddonTaskDirectDispatchFailure> {
        if addon.status != AddonStatus::Enabled {
            return Err(AddonTaskDirectDispatchFailure::host_contract(
                "addon_disabled",
            ));
        }
        let manifest = self
            .stored_manifest(addon)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("invalid_manifest"))?;
        validate_manifest(&manifest)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("invalid_manifest"))?;
        let task = manifest_task_declaration(&manifest.tasks, &run.declaration_id)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("task_not_declared"))?;
        if task.path != run.declaration_path {
            return Err(AddonTaskDirectDispatchFailure::host_contract(
                "task_path_changed",
            ));
        }
        let input = addon_task_run_input(&run.input_json)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("invalid_run_input"))?;
        let granted_scopes = stored_granted_scopes(addon)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("invalid_grants"))?;
        ensure_task_scopes_granted(task, &granted_scopes, addon.id)
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("missing_grant"))?;
        let outbound_secret = match manifest.auth {
            AddonAuth::None => None,
            AddonAuth::Bearer | AddonAuth::SharedSecret => {
                resolve_outbound_task_dispatch_secret(addon).map_err(|_| {
                    AddonTaskDirectDispatchFailure::host_contract("authorization_gap")
                })?
            }
        };
        let outcome = call_addon_task_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            AddonTaskCallRequest {
                task_id: run.declaration_id.clone(),
                job_id: run.job.id.to_string(),
                request_id: format!("addon-task-{}", run.job.id),
                attempt: run.attempt,
                retry_of_job_id: run.retry_of_job_id.map(|id| id.to_string()),
                library_id: run.job.library_id.map(|id| id.to_string()),
                source_id: run.job.source_id.map(|id| id.to_string()),
                payload: input
                    .get("payload")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            },
            outbound_secret
                .as_ref()
                .map(nako_core::SecretString::expose_secret),
        )
        .await
        .map_err(AddonTaskDirectDispatchFailure::from_client_failure)?;

        let progress = addon_task_run_progress_json(
            "dispatched".to_owned(),
            Some(100),
            Some("Addon task path completed".to_owned()),
            serde_json::json!({
                "http_status": outcome.http_status,
                "attempts": outcome.attempts,
                "lease_expires_at": lease.lease_expires_at,
            }),
        )
        .to_string();
        let reported = self
            .store
            .report_addon_task_run_progress(ReportAddonTaskRunProgress {
                guard: AddonTaskRunLeaseGuard::from(lease.guard()),
                lease_duration_ms: 30_000,
                progress_json: progress,
            })
            .await
            .map_err(|_| AddonTaskDirectDispatchFailure::host_contract("stale_run_lease"))?;

        Ok(AddonTaskDirectDispatchOutput {
            output: outcome.response.output,
            guard: AddonTaskRunLeaseGuard::from(reported.lease.guard()),
            cancel_requested_at: reported.lease.cancel_requested_at,
        })
    }
}

fn manifest_task_declaration<'a>(
    tasks: &'a [AddonTaskDeclaration],
    declaration_id: &str,
) -> Result<&'a AddonTaskDeclaration> {
    tasks
        .iter()
        .find(|task| task.id == declaration_id)
        .ok_or_else(|| NakoError::NotFound {
            entity: "addon_task_declaration",
            id: declaration_id.to_owned(),
        })
}

fn ensure_task_scopes_granted(
    task: &AddonTaskDeclaration,
    granted_scopes: &[AddonScope],
    addon_id: AddonId,
) -> Result<()> {
    if declaration_scopes_granted(&task.required_scopes, granted_scopes) {
        return Ok(());
    }

    Err(NakoError::Forbidden {
        message: format!(
            "addon {addon_id} is missing grants required for task {}",
            task.id
        ),
    })
}

fn addon_task_run_input_json(
    addon_id: AddonId,
    manifest_id: &str,
    manifest_version: &str,
    manifest_fingerprint: &str,
    task: &AddonTaskDeclaration,
    request: &CreateAddonTaskRunRequest,
    attempt: u32,
    retry_of_job_id: Option<JobId>,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "schema": ADDON_TASK_RUN_INPUT_SCHEMA,
        "addon_id": addon_id,
        "manifest_id": manifest_id,
        "manifest_version": manifest_version,
        "manifest_fingerprint": manifest_fingerprint,
        "dispatch": request.dispatch,
        "declaration": {
            "id": task.id,
            "name": task.name,
            "path": task.path,
            "timeout_ms": task.timeout_ms,
            "max_attempts": task.max_attempts,
        },
        "idempotency_key": request.idempotency_key,
        "attempt": attempt,
        "retry_of_job_id": retry_of_job_id,
        "library_id": request.library_id,
        "source_id": request.source_id,
        "payload": request.payload,
    }))
    .map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon task run input: {err}"),
    })
}

fn addon_task_run_lease_from_leased(run: LeasedAddonTaskRun) -> Result<AddonTaskRunLease> {
    let input = addon_task_run_input(&run.run.input_json)?;

    Ok(AddonTaskRunLease {
        guard: AddonTaskRunLeaseGuard::from(run.lease.guard()),
        lease_expires_at: run.lease.lease_expires_at,
        cancel_requested_at: run.lease.cancel_requested_at,
        input,
        run: AddonTaskRunSummary::from_record(run.run),
    })
}

fn retry_payload_from_previous_input(input_json: &str) -> Result<serde_json::Value> {
    let input = addon_task_run_input(input_json)?;

    Ok(input
        .get("payload")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

fn retry_dispatch_from_previous_input(input_json: &str) -> Result<AddonTaskRunDispatchMode> {
    let input = addon_task_run_input(input_json)?;
    let Some(value) = input.get("dispatch").and_then(serde_json::Value::as_str) else {
        return Ok(AddonTaskRunDispatchMode::SidecarClaim);
    };

    match value {
        "sidecar_claim" => Ok(AddonTaskRunDispatchMode::SidecarClaim),
        "direct" => Ok(AddonTaskRunDispatchMode::Direct),
        _ => Err(NakoError::InvalidInput {
            message: format!("unknown addon task run dispatch mode: {value}"),
        }),
    }
}

fn addon_task_run_input(input_json: &str) -> Result<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(input_json).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to parse addon task run input: {err}"),
    })
}

fn normalized_idempotency_key(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "addon task run idempotency_key must not be empty".to_owned(),
        });
    }

    Ok(value.to_owned())
}

fn normalized_safe_error_code(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "safe_error_code must not be empty".to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_lowercase() || character == '_' || character == '-')
    {
        return Err(NakoError::InvalidInput {
            message: "safe_error_code must use lowercase safe characters".to_owned(),
        });
    }

    Ok(value.to_owned())
}

fn addon_task_resource_class(declaration_id: &str) -> String {
    let mut normalized = String::from("addon.task.");
    let mut last_was_dot = false;
    for character in declaration_id.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            last_was_dot = false;
        } else if !last_was_dot {
            normalized.push('.');
            last_was_dot = true;
        }
    }
    while normalized.ends_with('.') {
        normalized.pop();
    }
    normalized
}

fn ensure_run_belongs_to_addon(
    run: &nako_core::AddonTaskRunRecord,
    addon_id: AddonId,
) -> Result<()> {
    if run.addon_id == addon_id {
        return Ok(());
    }

    Err(NakoError::NotFound {
        entity: "addon_task_run",
        id: run.job.id.to_string(),
    })
}

struct AddonTaskDirectDispatchOutput {
    output: serde_json::Value,
    guard: AddonTaskRunLeaseGuard,
    cancel_requested_at: Option<String>,
}

struct AddonTaskDirectDispatchFailure {
    safe_error_code: String,
    output: serde_json::Value,
}

impl AddonTaskDirectDispatchFailure {
    fn host_contract(safe_error_code: &'static str) -> Self {
        Self {
            safe_error_code: safe_error_code.to_owned(),
            output: serde_json::json!({
                "safe_error_code": safe_error_code,
                "source": "host",
            }),
        }
    }

    fn from_client_failure(failure: AddonTaskCallFailure) -> Self {
        let safe_error_code = safe_addon_task_client_error_code(&failure.error);
        Self {
            safe_error_code: safe_error_code.to_owned(),
            output: serde_json::json!({
                "safe_error_code": safe_error_code,
                "attempts": failure.attempts,
                "error_kind": failure.error.kind(),
                "http_status": failure.error.http_status(),
                "retryable": failure.error.was_retryable_http_status(),
            }),
        }
    }
}

fn safe_addon_task_client_error_code(error: &AddonClientError) -> &'static str {
    match error {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => "authorization_gap",
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::TaskNotDeclared {
            ..
        }) => "task_not_declared",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScopeForDeclaration { .. },
        ) => "missing_grant",
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => "unsafe_response",
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => "retryable_http_failure",
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => "http_failure",
        AddonClientError::Http { .. } => "sidecar_unreachable",
    }
}
