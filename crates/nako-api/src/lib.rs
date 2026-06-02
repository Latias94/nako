#![recursion_limit = "512"]

pub mod admin;
pub mod admin_contract;
pub mod extension;
pub mod metadata_diagnostics;
pub mod openapi;
pub mod public_client;
pub mod sdk;

#[cfg(test)]
pub(crate) const PROVIDER_GOVERNANCE_PUBLIC_FORBIDDEN_TERMS: &[&str] = &[
    "catalog/governance",
    "catalog_governance",
    "cataloggovernance",
    "provider-governance",
    "provider_governance",
    "providergovernance",
    "provider-mappings",
    "provider_mapping",
    "providermapping",
    "provider-mappings/{mapping_id}/review",
    "provider-mappings/{mapping_id}/review-plan",
    "provider-mapping-review",
    "provider_mapping_review",
    "providermappingreview",
    "metadata/candidate-reviews",
    "metadata/items/{item_id}/candidate-reviews",
    "metadata_candidate_review",
    "candidate-reviews",
    "candidate_review",
    "candidatereview",
    "batch-application-plan",
    "batch_application_plan",
    "batchapplicationplan",
    "batch-apply",
    "batch_apply",
    "batchapply",
    "metadatacandidatereview",
    "idempotency_key",
    "idempotencykey",
    "idempotency key",
    "source_fingerprint",
    "sourcefingerprint",
    "source fingerprint",
    "raw_provider",
    "rawprovider",
    "raw_provider_response",
    "rawproviderresponse",
    "raw provider",
    "provider_payload",
    "providerpayload",
    "provider payload",
    "provider_response",
    "providerresponse",
    "provider response",
];
