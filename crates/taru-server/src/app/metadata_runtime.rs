use std::{collections::HashSet, env};

use taru_api::metadata_diagnostics::{
    MetadataProviderDiagnostic, MetadataProviderDiagnosticStatus,
    MetadataProviderRuntimeDiagnostic, MetadataProviderRuntimeStateScope,
};
use taru_core::{ExternalProvider, Result, SecretString, TaruError};
use taru_metadata::{
    BangumiMetadataProvider, BangumiProviderConfig, DoubanMetadataProvider, DoubanProviderConfig,
    MetadataHttpRuntimeConfig, MetadataHttpRuntimeStatus, MetadataProviderRegistrationStatus,
    MetadataProviderRegistry, TmdbMetadataProvider, TmdbProviderConfig,
};

use crate::config::{
    MetadataProviderConfig, MetadataProviderHeaderConfig, MetadataProviderRuntimeConfig,
    TaruServerConfig,
};

pub(super) fn provider_resource_name(provider: &ExternalProvider) -> &str {
    match provider {
        ExternalProvider::Tmdb => "tmdb",
        ExternalProvider::Douban => "douban",
        ExternalProvider::Bangumi => "bangumi",
        ExternalProvider::Imdb => "imdb",
        ExternalProvider::Local => "local",
        ExternalProvider::Other(_) => "other",
    }
}

pub(super) fn build_metadata_provider_registry(
    config: &TaruServerConfig,
) -> Result<MetadataProviderRegistry> {
    let mut registry = MetadataProviderRegistry::new();
    let mut seen = HashSet::new();

    for provider in &config.metadata.providers {
        if !seen.insert(provider.provider.clone()) {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "duplicate metadata provider config: {}",
                    provider_resource_name(&provider.provider)
                ),
            });
        }

        match build_configured_metadata_provider(provider, &config.metadata.runtime) {
            Ok(BuiltMetadataProvider::Tmdb(provider)) => {
                registry.register(provider);
            }
            Ok(BuiltMetadataProvider::Bangumi(provider)) => {
                registry.register(provider);
            }
            Ok(BuiltMetadataProvider::Douban(provider)) => {
                registry.register(provider);
            }
            Err(MetadataProviderBuildError::Disabled(provider, message)) => {
                registry.register_disabled(provider, message);
            }
            Err(MetadataProviderBuildError::Unavailable(provider, message)) => {
                registry.register_unavailable(provider, message);
            }
        }
    }

    Ok(registry)
}

pub(super) fn metadata_provider_diagnostics(
    config: &TaruServerConfig,
    registry: &MetadataProviderRegistry,
) -> Vec<MetadataProviderDiagnostic> {
    config
        .metadata
        .providers
        .iter()
        .map(|settings| {
            configured_provider_diagnostic(settings, &config.metadata.runtime, registry)
        })
        .collect()
}

fn build_configured_metadata_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<BuiltMetadataProvider, MetadataProviderBuildError> {
    if !settings.enabled {
        return Err(MetadataProviderBuildError::Disabled(
            settings.provider.clone(),
            format!(
                "{} metadata provider is disabled in config",
                provider_resource_name(&settings.provider).to_uppercase()
            ),
        ));
    }

    match settings.provider {
        ExternalProvider::Tmdb => {
            build_tmdb_provider(settings, inherited_runtime).map(BuiltMetadataProvider::Tmdb)
        }
        ExternalProvider::Bangumi => {
            build_bangumi_provider(settings, inherited_runtime).map(BuiltMetadataProvider::Bangumi)
        }
        ExternalProvider::Douban => {
            build_douban_provider(settings, inherited_runtime).map(BuiltMetadataProvider::Douban)
        }
        _ => Err(MetadataProviderBuildError::Unavailable(
            settings.provider.clone(),
            format!(
                "{} metadata provider is not implemented",
                provider_resource_name(&settings.provider)
            ),
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MetadataProviderBuildError {
    Disabled(ExternalProvider, String),
    Unavailable(ExternalProvider, String),
}

enum BuiltMetadataProvider {
    Tmdb(TmdbMetadataProvider),
    Bangumi(BangumiMetadataProvider),
    Douban(DoubanMetadataProvider),
}

fn configured_provider_diagnostic(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
    registry: &MetadataProviderRegistry,
) -> MetadataProviderDiagnostic {
    let provider = settings.provider.clone();
    let runtime = provider_runtime_diagnostic(
        settings.runtime.as_ref().unwrap_or(inherited_runtime),
        registry
            .describe(&provider)
            .and_then(|diagnostic| diagnostic.runtime_status),
    );

    registry_provider_diagnostic(registry, provider, runtime)
}

fn registry_provider_diagnostic(
    registry: &MetadataProviderRegistry,
    provider: ExternalProvider,
    runtime: MetadataProviderRuntimeDiagnostic,
) -> MetadataProviderDiagnostic {
    let Some(diagnostic) = registry.describe(&provider) else {
        return MetadataProviderDiagnostic {
            provider,
            status: MetadataProviderDiagnosticStatus::Unavailable,
            provider_name: None,
            reason: Some("metadata provider is not registered".to_owned()),
            runtime,
        };
    };

    MetadataProviderDiagnostic {
        provider: diagnostic.provider,
        status: match diagnostic.status {
            MetadataProviderRegistrationStatus::Available => {
                MetadataProviderDiagnosticStatus::Available
            }
            MetadataProviderRegistrationStatus::Disabled => {
                MetadataProviderDiagnosticStatus::Disabled
            }
            MetadataProviderRegistrationStatus::Unavailable => {
                MetadataProviderDiagnosticStatus::Unavailable
            }
        },
        provider_name: diagnostic.provider_name,
        reason: diagnostic.reason,
        runtime,
    }
}

fn provider_runtime_diagnostic(
    config: &MetadataProviderRuntimeConfig,
    status: Option<MetadataHttpRuntimeStatus>,
) -> MetadataProviderRuntimeDiagnostic {
    MetadataProviderRuntimeDiagnostic {
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
        min_interval_ms: config.min_interval_ms,
        concurrency: config.concurrency,
        user_agent: config.user_agent.clone(),
        proxy_configured: config.proxy.as_ref().is_some_and(|proxy| !proxy.is_blank()),
        circuit_breaker_failures: config.circuit_breaker_failures,
        circuit_breaker_backoff_ms: config.circuit_breaker_backoff_ms,
        circuit_open: status.as_ref().is_some_and(|status| status.circuit_open),
        circuit_open_until_ms: status
            .as_ref()
            .and_then(|status| status.circuit_open_until_ms),
        consecutive_failures: status
            .as_ref()
            .map_or(0, |status| status.consecutive_failures),
        last_error: status.as_ref().and_then(|status| status.last_error.clone()),
        last_rate_limit_wait_ms: status.map_or(0, |status| status.last_rate_limit_wait_ms),
        state_scope: MetadataProviderRuntimeStateScope::ProcessLocal,
    }
}

fn build_tmdb_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<TmdbMetadataProvider, MetadataProviderBuildError> {
    let token_env = settings
        .token_env
        .as_deref()
        .unwrap_or("TMDB_READ_ACCESS_TOKEN");
    let token = resolve_required_secret(ExternalProvider::Tmdb, token_env, "access token")?;
    let mut config = TmdbProviderConfig::new(token);
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }
    if let Some(image_base_url) = settings.image_base_url.as_ref() {
        config.image_base_url = image_base_url.clone();
    }
    if let Some(language) = settings.language.as_ref() {
        config.language = language.clone();
    }
    config.include_adult = settings.include_adult;
    config.runtime = runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime));

    TmdbMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Tmdb, err.to_string())
    })
}

fn build_bangumi_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<BangumiMetadataProvider, MetadataProviderBuildError> {
    let access_token = settings
        .token_env
        .as_deref()
        .map(|env_name| {
            resolve_required_secret(ExternalProvider::Bangumi, env_name, "access token")
        })
        .transpose()?;
    let mut config = BangumiProviderConfig {
        access_token,
        include_nsfw: settings.include_adult,
        runtime: runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime)),
        ..BangumiProviderConfig::default()
    };
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }
    if let Some(image_base_url) = settings.image_base_url.as_ref() {
        config.image_base_url = image_base_url.clone();
    }

    BangumiMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Bangumi, err.to_string())
    })
}

fn build_douban_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<DoubanMetadataProvider, MetadataProviderBuildError> {
    let api_key = settings
        .api_key_env
        .as_deref()
        .map(|env_name| resolve_required_secret(ExternalProvider::Douban, env_name, "API key"))
        .transpose()?;
    let mut config = DoubanProviderConfig {
        api_key,
        image_base_url: settings.image_base_url.clone(),
        runtime: runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime)),
        headers: resolve_headers(ExternalProvider::Douban, &settings.headers)?,
        ..DoubanProviderConfig::default()
    };
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }

    DoubanMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Douban, err.to_string())
    })
}

fn runtime_config(config: &MetadataProviderRuntimeConfig) -> MetadataHttpRuntimeConfig {
    MetadataHttpRuntimeConfig {
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
        min_interval_ms: config.min_interval_ms,
        concurrency: config.concurrency,
        user_agent: config.user_agent.clone(),
        proxy: config.proxy.clone(),
        circuit_breaker_failures: config.circuit_breaker_failures,
        circuit_breaker_backoff_ms: config.circuit_breaker_backoff_ms,
    }
}

fn resolve_required_secret(
    provider: ExternalProvider,
    env_name: &str,
    label: &str,
) -> std::result::Result<SecretString, MetadataProviderBuildError> {
    let value = env::var(env_name).map_err(|err| {
        MetadataProviderBuildError::Unavailable(
            provider.clone(),
            format!(
                "failed to read {} {label} from environment variable {env_name}: {err}",
                provider_resource_name(&provider).to_uppercase()
            ),
        )
    })?;

    if value.trim().is_empty() {
        return Err(MetadataProviderBuildError::Unavailable(
            provider.clone(),
            format!(
                "{} {label} environment variable {env_name} is empty",
                provider_resource_name(&provider).to_uppercase()
            ),
        ));
    }

    Ok(SecretString::new(value))
}

fn resolve_headers(
    provider: ExternalProvider,
    headers: &[MetadataProviderHeaderConfig],
) -> std::result::Result<Vec<(String, SecretString)>, MetadataProviderBuildError> {
    headers
        .iter()
        .map(|header| {
            let value = match (&header.value, &header.value_env) {
                (Some(value), None) => Ok(value.clone()),
                (None, Some(env_name)) => resolve_required_secret(
                    provider.clone(),
                    env_name,
                    &format!("header {}", header.name),
                ),
                (Some(_), Some(_)) => Err(MetadataProviderBuildError::Unavailable(
                    provider.clone(),
                    format!(
                        "{} metadata provider header {} cannot set both value and value_env",
                        provider_resource_name(&provider).to_uppercase(),
                        header.name
                    ),
                )),
                (None, None) => Err(MetadataProviderBuildError::Unavailable(
                    provider.clone(),
                    format!(
                        "{} metadata provider header {} must set value or value_env",
                        provider_resource_name(&provider).to_uppercase(),
                        header.name
                    ),
                )),
            }?;

            Ok((header.name.clone(), value))
        })
        .collect()
}
