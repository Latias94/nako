use axum::{
    Json,
    extract::{Request, connect_info::ConnectInfo},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use nako_api::public_client::{ClientErrorCode, ErrorResponse};

use crate::config::NetworkAccessConfig;

#[derive(Clone, Debug)]
pub(super) struct NetworkBoundaryState {
    allowed_origins: Vec<String>,
    trusted_proxy_sources: Vec<String>,
    trust_forwarded_headers: bool,
}

impl NetworkBoundaryState {
    #[must_use]
    pub(super) fn from_config(config: &NetworkAccessConfig) -> Self {
        Self {
            allowed_origins: config.allowed_origins.clone(),
            trusted_proxy_sources: config.trusted_proxy_sources.clone(),
            trust_forwarded_headers: config.trusted_proxy_headers
                && !config.trusted_proxy_sources.is_empty(),
        }
    }
}

pub(super) async fn enforce_network_boundary(request: Request, next: Next) -> Response {
    let Some(state) = request.extensions().get::<NetworkBoundaryState>().cloned() else {
        return next.run(request).await;
    };

    let request_origin = request_origin(&request).map(ToOwned::to_owned);
    if let Some(origin) = request_origin.as_deref()
        && !state.allows_origin(origin)
    {
        return forbidden_origin_response();
    }

    let external_origin = state.external_origin(&request);
    let mut response = next.run(request).await;
    annotate_allowed_origin(&mut response, &state, request_origin.as_deref());
    annotate_external_origin_header(&mut response, external_origin.as_deref());
    response
}

pub(super) async fn annotate_external_origin(request: Request, next: Next) -> Response {
    let Some(state) = request.extensions().get::<NetworkBoundaryState>().cloned() else {
        return next.run(request).await;
    };

    let request_origin = request_origin(&request).map(ToOwned::to_owned);
    let external_origin = state.external_origin(&request);
    if is_cors_preflight(&request) {
        let mut response = if request_origin
            .as_deref()
            .is_some_and(|origin| state.allows_origin(origin))
        {
            StatusCode::NO_CONTENT.into_response()
        } else {
            forbidden_origin_response()
        };

        annotate_allowed_origin(&mut response, &state, request_origin.as_deref());
        annotate_preflight_headers(&mut response);
        annotate_external_origin_header(&mut response, external_origin.as_deref());
        return response;
    }

    let mut response = next.run(request).await;
    annotate_allowed_origin(&mut response, &state, request_origin.as_deref());
    annotate_external_origin_header(&mut response, external_origin.as_deref());
    response
}

impl NetworkBoundaryState {
    fn allows_origin(&self, origin: &str) -> bool {
        self.allowed_origins
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(origin))
    }

    fn external_origin(&self, request: &Request) -> Option<String> {
        if !self.trust_forwarded_headers || !self.trusts_proxy_source(request) {
            return None;
        }

        let host = sanitized_forwarded_header(request, "x-forwarded-host")?;
        let proto = sanitized_forwarded_header(request, "x-forwarded-proto")?;
        if proto != "https" && proto != "http" {
            return None;
        }

        Some(format!("{proto}://{host}"))
    }

    fn trusts_proxy_source(&self, request: &Request) -> bool {
        let Some(ConnectInfo(remote_addr)) = request
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
        else {
            return false;
        };
        self.trusted_proxy_sources
            .iter()
            .any(|source| proxy_source_matches(source, remote_addr.ip()))
    }
}

fn request_origin(request: &Request) -> Option<&str> {
    request.headers().get(header::ORIGIN)?.to_str().ok()
}

fn is_cors_preflight(request: &Request) -> bool {
    request.method() == Method::OPTIONS
        && request.headers().contains_key(header::ORIGIN)
        && request
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
}

fn annotate_allowed_origin(
    response: &mut Response,
    state: &NetworkBoundaryState,
    request_origin: Option<&str>,
) {
    if let Some(origin) = request_origin
        && state.allows_origin(origin)
        && let Ok(value) = HeaderValue::from_str(origin)
    {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        response.headers_mut().insert(
            header::VARY,
            HeaderValue::from_static(header::ORIGIN.as_str()),
        );
    }
}

fn annotate_preflight_headers(response: &mut Response) {
    if !response.status().is_success() {
        return;
    }

    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET,POST,PUT,PATCH,DELETE,OPTIONS"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("authorization,content-type,range,x-request-id"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("600"),
    );
}

fn annotate_external_origin_header(response: &mut Response, external_origin: Option<&str>) {
    if let Some(origin) = external_origin
        && let Ok(value) = HeaderValue::from_str(origin)
    {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-nako-external-origin"), value);
    }
}

fn sanitized_forwarded_header(request: &Request, name: &'static str) -> Option<String> {
    let value = request.headers().get(name)?.to_str().ok()?.trim();
    if value.is_empty()
        || value.contains(['/', '\\', '?', '#', '@', ',', ';'])
        || value.chars().any(char::is_whitespace)
    {
        return None;
    }

    Some(value.to_ascii_lowercase())
}

fn proxy_source_matches(source: &str, remote_ip: std::net::IpAddr) -> bool {
    let source = source.trim();
    if source.is_empty() {
        return false;
    }
    if remote_ip.is_loopback()
        && (source.eq_ignore_ascii_case("localhost") || source.eq_ignore_ascii_case("loopback"))
    {
        return true;
    }
    if let Ok(source_ip) = source.parse::<std::net::IpAddr>() {
        return source_ip == remote_ip;
    }

    let Some((network, prefix)) = source.split_once('/') else {
        return false;
    };
    let Ok(network) = network.parse::<std::net::IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.parse::<u8>() else {
        return false;
    };

    cidr_contains(network, prefix, remote_ip)
}

fn cidr_contains(network: std::net::IpAddr, prefix: u8, remote_ip: std::net::IpAddr) -> bool {
    match (network, remote_ip) {
        (std::net::IpAddr::V4(network), std::net::IpAddr::V4(remote_ip)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            u32::from(network) & mask == u32::from(remote_ip) & mask
        }
        (std::net::IpAddr::V6(network), std::net::IpAddr::V6(remote_ip)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            u128::from(network) & mask == u128::from(remote_ip) & mask
        }
        _ => false,
    }
}

fn forbidden_origin_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse::new(
            ClientErrorCode::Forbidden,
            "origin is not allowed by network policy",
        )),
    )
        .into_response()
}
