mod confirmation;
mod mapping;
mod provider_attempt;
mod providers;
mod registry;
mod runtime;
mod strategy;
mod types;

pub use confirmation::{
    HierarchyConfirmationItem, HierarchyConfirmationRequest, HierarchyConfirmationService,
    HierarchyConfirmationSummary, HierarchyProviderSubject,
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
    MetadataProviderAttempt, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshService, MetadataRefreshSummary, MetadataStrategyExecutor,
};
pub use taru_core::MetadataMergePolicy;
pub use types::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataLookup, MetadataProvider,
};

#[cfg(test)]
mod tests;
