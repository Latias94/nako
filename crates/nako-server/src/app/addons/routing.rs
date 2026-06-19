use nako_addon_protocol::{AddonManifest, AddonScope, validate_manifest};
use nako_api::extension::AdminAddonRoutingPlansResponse;
use nako_core::{
    AddonId, AddonManifestFingerprint, AddonRepository, AddonRoutingDeclarationKind,
    AddonRoutingPlanId, AddonRoutingPlanStatus, AddonRoutingPlanTarget, AddonStatus,
    DomainEventKind, JobKind, NakoError, NewAddonRoutingPlan, Result,
};

use super::{
    AddonAppService, helpers::ensure_addon_accepts_runtime_authority,
    helpers::stored_granted_scopes,
};
impl AddonAppService {
    pub async fn sync_addon_routing_plans(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRoutingPlansResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "sync addon routing plans")?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        let granted_scopes = stored_granted_scopes(&addon)?;
        let manifest_fingerprint = AddonManifestFingerprint::new(&addon.manifest_json);
        let plans = build_addon_routing_plans(
            addon_id,
            addon.status,
            &addon.manifest_id,
            &manifest.version,
            &manifest_fingerprint,
            &manifest,
            &granted_scopes,
        )?;
        let records = self
            .store
            .replace_addon_routing_plans(addon_id, plans)
            .await?;

        Ok(AdminAddonRoutingPlansResponse::from_records(
            addon_id,
            addon.manifest_id,
            manifest.version,
            manifest_fingerprint.to_string(),
            records,
        ))
    }
}

fn build_addon_routing_plans(
    addon_id: AddonId,
    addon_status: AddonStatus,
    manifest_id: &str,
    manifest_version: &str,
    manifest_fingerprint: &AddonManifestFingerprint,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
) -> Result<Vec<NewAddonRoutingPlan>> {
    let mut plans = Vec::new();
    let addon_enabled = addon_status == AddonStatus::Enabled;

    for task in &manifest.tasks {
        let granted = declaration_scopes_granted(&task.required_scopes, granted_scopes);
        let reason = if !addon_enabled {
            Some("addon_disabled")
        } else if !granted {
            Some("missing_grant")
        } else {
            None
        };
        let executable = reason.is_none();
        let status = if executable {
            AddonRoutingPlanStatus::Executable
        } else {
            AddonRoutingPlanStatus::Deferred
        };
        let target = if executable {
            AddonRoutingPlanTarget::AddonTaskJob
        } else {
            AddonRoutingPlanTarget::None
        };
        let job_kind = executable.then_some(JobKind::AddonTask);
        plans.push(NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: manifest_id.to_owned(),
            manifest_version: manifest_version.to_owned(),
            manifest_fingerprint: manifest_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::Task,
            declaration_id: task.id.clone(),
            status,
            target,
            safe_reason_code: reason.map(ToOwned::to_owned),
            job_kind,
            event_kind: None,
            plan_json: routing_plan_json(RoutingPlanJson {
                declaration_kind: AddonRoutingDeclarationKind::Task,
                declaration_id: &task.id,
                target,
                status,
                safe_reason_code: reason,
                job_kind: job_kind.map(JobKind::as_str),
                event_kind: None,
                required_scope_count: task.required_scopes.len(),
                filter_configured: false,
                timeout_ms: task.timeout_ms,
                max_attempts: task.max_attempts,
            })?,
        });
    }

    for subscription in &manifest.event_subscriptions {
        let granted = declaration_scopes_granted(&subscription.required_scopes, granted_scopes);
        let parsed_event_kind = DomainEventKind::parse(&subscription.event_kind).ok();
        let reason = if !addon_enabled {
            Some("addon_disabled")
        } else if !granted {
            Some("missing_grant")
        } else if parsed_event_kind.is_none() {
            Some("unsupported_event_kind")
        } else {
            None
        };
        let status = if reason.is_none() {
            AddonRoutingPlanStatus::Executable
        } else {
            AddonRoutingPlanStatus::Deferred
        };
        let target = if reason.is_none() {
            AddonRoutingPlanTarget::EventOutbox
        } else {
            AddonRoutingPlanTarget::None
        };
        let event_kind = parsed_event_kind.map(|kind| kind.as_str().to_owned());
        plans.push(NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: manifest_id.to_owned(),
            manifest_version: manifest_version.to_owned(),
            manifest_fingerprint: manifest_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
            declaration_id: subscription.id.clone(),
            status,
            target,
            safe_reason_code: reason.map(ToOwned::to_owned),
            job_kind: None,
            event_kind: event_kind.clone(),
            plan_json: routing_plan_json(RoutingPlanJson {
                declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                declaration_id: &subscription.id,
                target,
                status,
                safe_reason_code: reason,
                job_kind: None,
                event_kind: event_kind.as_deref(),
                required_scope_count: subscription.required_scopes.len(),
                filter_configured: !subscription.filters.is_null(),
                timeout_ms: None,
                max_attempts: None,
            })?,
        });
    }

    Ok(plans)
}

pub(super) fn declaration_scopes_granted(required: &[AddonScope], granted: &[AddonScope]) -> bool {
    required.iter().all(|scope| granted.contains(scope))
}

struct RoutingPlanJson<'a> {
    declaration_kind: AddonRoutingDeclarationKind,
    declaration_id: &'a str,
    target: AddonRoutingPlanTarget,
    status: AddonRoutingPlanStatus,
    safe_reason_code: Option<&'static str>,
    job_kind: Option<&'static str>,
    event_kind: Option<&'a str>,
    required_scope_count: usize,
    filter_configured: bool,
    timeout_ms: Option<u64>,
    max_attempts: Option<u32>,
}

fn routing_plan_json(plan: RoutingPlanJson<'_>) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "schema": "nako.addon.routing_plan.v1",
        "declaration_kind": plan.declaration_kind,
        "declaration_id": plan.declaration_id,
        "target": plan.target,
        "status": plan.status,
        "safe_reason_code": plan.safe_reason_code,
        "job_kind": plan.job_kind,
        "event_kind": plan.event_kind,
        "required_scope_count": plan.required_scope_count,
        "filter_configured": plan.filter_configured,
        "timeout_ms": plan.timeout_ms,
        "max_attempts": plan.max_attempts,
    }))
    .map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon routing plan: {err}"),
    })
}
