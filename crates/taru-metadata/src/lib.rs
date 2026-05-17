mod confirmation;
mod mapping;
mod merge;
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
pub use merge::MetadataMergePolicy;
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
pub use types::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataLookup, MetadataProvider,
};

#[cfg(test)]
mod tests;
