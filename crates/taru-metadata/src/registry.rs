use std::{collections::HashMap, fmt, sync::Arc};

use taru_core::ExternalProvider;

use crate::{MetadataHttpRuntimeStatus, MetadataProvider};
#[derive(Clone, Default)]
pub struct MetadataProviderRegistry {
    providers: HashMap<ExternalProvider, RegisteredMetadataProvider>,
}

impl MetadataProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P) -> &mut Self
    where
        P: MetadataProvider + 'static,
    {
        let provider_id = provider.provider();
        self.providers.insert(
            provider_id,
            RegisteredMetadataProvider::Available(Arc::new(provider)),
        );
        self
    }

    #[must_use]
    pub fn with_provider<P>(mut self, provider: P) -> Self
    where
        P: MetadataProvider + 'static,
    {
        self.register(provider);
        self
    }

    pub fn register_arc(
        &mut self,
        provider_id: ExternalProvider,
        provider: Arc<dyn MetadataProvider>,
    ) -> &mut Self {
        self.providers
            .insert(provider_id, RegisteredMetadataProvider::Available(provider));
        self
    }

    pub fn register_disabled(
        &mut self,
        provider: ExternalProvider,
        reason: impl Into<String>,
    ) -> &mut Self {
        self.providers.insert(
            provider,
            RegisteredMetadataProvider::Disabled {
                reason: reason.into(),
            },
        );
        self
    }

    pub fn register_unavailable(
        &mut self,
        provider: ExternalProvider,
        reason: impl Into<String>,
    ) -> &mut Self {
        self.providers.insert(
            provider,
            RegisteredMetadataProvider::Unavailable {
                reason: reason.into(),
            },
        );
        self
    }

    pub(crate) fn get(&self, provider: &ExternalProvider) -> Option<&RegisteredMetadataProvider> {
        self.providers.get(provider)
    }

    #[must_use]
    pub fn describe(
        &self,
        provider: &ExternalProvider,
    ) -> Option<MetadataProviderRegistrationDiagnostic> {
        self.providers
            .get(provider)
            .map(|registered| match registered {
                RegisteredMetadataProvider::Available(provider_impl) => {
                    MetadataProviderRegistrationDiagnostic {
                        provider: provider.clone(),
                        status: MetadataProviderRegistrationStatus::Available,
                        provider_name: Some(provider_impl.provider_name().to_owned()),
                        reason: None,
                        runtime_status: provider_impl.runtime_status(),
                    }
                }
                RegisteredMetadataProvider::Disabled { reason } => {
                    MetadataProviderRegistrationDiagnostic {
                        provider: provider.clone(),
                        status: MetadataProviderRegistrationStatus::Disabled,
                        provider_name: None,
                        reason: Some(reason.clone()),
                        runtime_status: None,
                    }
                }
                RegisteredMetadataProvider::Unavailable { reason } => {
                    MetadataProviderRegistrationDiagnostic {
                        provider: provider.clone(),
                        status: MetadataProviderRegistrationStatus::Unavailable,
                        provider_name: None,
                        reason: Some(reason.clone()),
                        runtime_status: None,
                    }
                }
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataProviderRegistrationStatus {
    Available,
    Disabled,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataProviderRegistrationDiagnostic {
    pub provider: ExternalProvider,
    pub status: MetadataProviderRegistrationStatus,
    pub provider_name: Option<String>,
    pub reason: Option<String>,
    pub runtime_status: Option<MetadataHttpRuntimeStatus>,
}

impl fmt::Debug for MetadataProviderRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MetadataProviderRegistry")
            .field("providers", &self.providers)
            .finish()
    }
}

#[derive(Clone)]
pub(crate) enum RegisteredMetadataProvider {
    Available(Arc<dyn MetadataProvider>),
    Disabled { reason: String },
    Unavailable { reason: String },
}

impl fmt::Debug for RegisteredMetadataProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available(provider) => formatter
                .debug_struct("Available")
                .field("provider", &provider.provider())
                .field("provider_name", &provider.provider_name())
                .finish(),
            Self::Disabled { reason } => formatter
                .debug_struct("Disabled")
                .field("reason", reason)
                .finish(),
            Self::Unavailable { reason } => formatter
                .debug_struct("Unavailable")
                .field("reason", reason)
                .finish(),
        }
    }
}
