mod candidate_review;
mod confirmation;
mod mapping;
mod matching;
mod provider_attempt;
mod providers;
mod registry;
mod runtime;
mod strategy;
mod types;

pub use candidate_review::{
    MetadataCandidateReviewApplicationPlanRequest, MetadataCandidateReviewApplicationPlanSummary,
    MetadataCandidateReviewApplicationRequest, MetadataCandidateReviewApplicationService,
    MetadataCandidateReviewApplicationSummary, MetadataCandidateReviewDecision,
    MetadataCandidateReviewDecisionRequest, MetadataCandidateReviewDecisionService,
    MetadataCandidateReviewDecisionSummary,
    MetadataCandidateReviewRelatedHierarchyApplicationRequest,
    MetadataCandidateReviewRelatedHierarchyApplicationSummary,
    build_candidate_review_application_plan, build_candidate_review_plan,
};
pub use confirmation::{
    HierarchyConfirmationItem, HierarchyConfirmationRequest, HierarchyConfirmationService,
    HierarchyConfirmationSummary, HierarchyProviderSubject,
};
pub use matching::{
    MetadataCandidateConflictReview, MetadataCandidateConflictReviewStatus, MetadataCandidateMatch,
    MetadataCandidateMatchDecision, MetadataCandidateMatchReason, MetadataCandidateMatchingPolicy,
    build_candidate_conflict_review,
};
pub use nako_core::{
    MetadataCandidateGraph, MetadataCandidateRecord, MetadataCandidateRelationship,
    MetadataCandidateRelationshipKind, MetadataCandidateReviewApplicationAction,
    MetadataCandidateReviewApplicationPlan, MetadataCandidateReviewApplicationReason,
    MetadataCandidateReviewNode, MetadataCandidateReviewPlan, MetadataCandidateReviewRelationship,
    MetadataCandidateSource, MetadataCandidateSubject, MetadataMergePolicy,
};
pub use providers::{
    BangumiMetadataProvider, BangumiProviderConfig, DoubanMetadataProvider, DoubanProviderConfig,
    TmdbMetadataProvider, TmdbProviderConfig,
};
pub use registry::{
    MetadataProviderRegistrationDiagnostic, MetadataProviderRegistrationStatus,
    MetadataProviderRegistry,
};
pub use runtime::{
    MetadataHttpJsonResponse, MetadataHttpRuntime, MetadataHttpRuntimeConfig,
    MetadataHttpRuntimeStatus,
};
pub use strategy::{
    MetadataAttemptPort, MetadataProviderAttempt, MetadataRefreshCommit, MetadataRefreshJobInput,
    MetadataRefreshPort, MetadataRefreshRequest, MetadataRefreshService, MetadataRefreshSnapshot,
    MetadataRefreshSummary, MetadataStrategyExecutor,
};
pub use types::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataLookup, MetadataProvider,
    MetadataProviderCapabilities, MetadataProviderCredentialRequirement,
};

#[cfg(test)]
mod tests;
