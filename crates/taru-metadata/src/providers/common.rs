use std::time::Duration;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use taru_core::{ExternalProvider, ImageKind, ImageRef, Result, TaruError};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::TMDB_PROVIDER_NAME;
pub(crate) fn provider_request_error(provider: &str, error: reqwest::Error) -> TaruError {
    TaruError::Provider {
        provider: provider.to_owned(),
        message: error.to_string(),
    }
}

pub(crate) fn tmdb_parse_error(operation: &str, error: impl ToString) -> TaruError {
    provider_parse_error(TMDB_PROVIDER_NAME, operation, error)
}

pub(crate) fn provider_parse_error(
    provider: &str,
    operation: &str,
    error: impl ToString,
) -> TaruError {
    TaruError::Provider {
        provider: provider.to_owned(),
        message: format!(
            "failed to parse {provider} {operation} response: {}",
            error.to_string()
        ),
    }
}

pub(crate) fn bearer_headers(token: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|err| {
        TaruError::InvalidInput {
            message: format!("invalid bearer token for metadata provider header: {err}"),
        }
    })?;
    headers.insert(AUTHORIZATION, value);
    Ok(headers)
}

pub(crate) fn api_key_query(name: &str, value: &Option<String>) -> Vec<(String, String)> {
    value
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![(name.to_owned(), value.clone())])
        .unwrap_or_default()
}

pub(crate) fn header_map_from_pairs(pairs: &[(String, String)]) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|err| TaruError::InvalidInput {
                message: format!("invalid metadata provider header name {name}: {err}"),
            })?;
        let value = HeaderValue::from_str(value).map_err(|err| TaruError::InvalidInput {
            message: format!("invalid metadata provider header value for {name}: {err}"),
        })?;
        headers.insert(name, value);
    }
    Ok(headers)
}

pub(crate) fn retry_delay(attempt: u32) -> Duration {
    Duration::from_millis(100 * u64::from(attempt))
}

pub(crate) fn truncate_message(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    value.chars().take(max_chars).collect::<String>()
}

pub(crate) fn push_provider_image_uri(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    uri: Option<&str>,
    image_base_url: &str,
    provider: ExternalProvider,
    width: Option<u32>,
    height: Option<u32>,
    language: Option<String>,
) {
    let Some(uri) = uri.filter(|uri| !uri.trim().is_empty()) else {
        return;
    };
    let uri =
        if uri.starts_with("http://") || uri.starts_with("https://") || image_base_url.is_empty() {
            uri.to_owned()
        } else {
            format!("{}{}", image_base_url.trim_end_matches('/'), uri)
        };

    if images
        .iter()
        .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri,
        provider,
        width,
        height,
        language,
    });
}

pub(crate) fn release_year(value: Option<&str>) -> Option<u16> {
    let year = value?.get(0..4)?;

    if year.chars().all(|character| character.is_ascii_digit()) {
        year.parse().ok()
    } else {
        None
    }
}

pub(crate) fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
}

pub(crate) fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn now_utc_string() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("failed to format metadata refresh timestamp: {err}"),
        })
}
