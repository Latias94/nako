mod merge;
mod providers;
mod registry;
mod runtime;
mod strategy;
mod types;

pub use merge::MetadataMergePolicy;
pub use providers::{
    BangumiMetadataProvider, BangumiProviderConfig, DoubanMetadataProvider, DoubanProviderConfig,
    TmdbMetadataProvider, TmdbProviderConfig,
};
pub use registry::MetadataProviderRegistry;
pub use runtime::{MetadataHttpJsonResponse, MetadataHttpRuntime, MetadataHttpRuntimeConfig};
pub use strategy::{
    MetadataProviderAttempt, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshService, MetadataRefreshSummary, MetadataStrategyExecutor,
};
pub use types::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataLookup, MetadataProvider,
};

#[cfg(test)]
mod tests;
