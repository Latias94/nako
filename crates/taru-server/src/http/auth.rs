use axum::{
    Json,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use taru_api::{ClientErrorCode, ErrorResponse};
use taru_core::{SecretString, UserPrincipalId};

use crate::config::AuthConfig;

#[derive(Clone, Debug)]
pub(super) struct InboundAuthState {
    enabled: bool,
    token: Option<SecretString>,
}

impl InboundAuthState {
    #[must_use]
    pub(super) fn from_config(config: &AuthConfig) -> Self {
        let token = config
            .token_env
            .as_deref()
            .and_then(|name| std::env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .map(SecretString::new);

        Self {
            enabled: config.enabled,
            token,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(super) fn bearer_token(token: impl Into<String>) -> Self {
        Self {
            enabled: true,
            token: Some(SecretString::new(token)),
        }
    }
}

pub(super) async fn require_auth(request: Request, next: Next) -> Response {
    let mut request = request;
    let Some(auth) = request.extensions().get::<InboundAuthState>().cloned() else {
        return unauthorized_response();
    };

    if !auth.enabled {
        request
            .extensions_mut()
            .insert(UserPrincipalId::local_admin());
        return next.run(request).await;
    }

    let authorized = bearer_token(request.headers()).is_some_and(|token| {
        auth.token.as_ref().is_some_and(|expected| {
            constant_time_eq(token.as_bytes(), expected.expose_secret().as_bytes())
        })
    });

    if authorized {
        request
            .extensions_mut()
            .insert(UserPrincipalId::local_admin());
        next.run(request).await
    } else {
        unauthorized_response()
    }
}

pub(super) fn request_bearer_token(headers: &HeaderMap) -> Option<&str> {
    bearer_token(headers)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        None
    } else {
        Some(token)
    }
}

fn unauthorized_response() -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse::new(
            ClientErrorCode::Unauthorized,
            "authentication required",
        )),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

fn constant_time_eq(actual: &[u8], expected: &[u8]) -> bool {
    let max_len = actual.len().max(expected.len());
    let mut diff = actual.len() ^ expected.len();

    for index in 0..max_len {
        let actual_byte = actual.get(index).copied().unwrap_or_default();
        let expected_byte = expected.get(index).copied().unwrap_or_default();
        diff |= usize::from(actual_byte ^ expected_byte);
    }

    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use tower::ServiceExt;

    #[test]
    fn constant_time_eq_checks_length_and_bytes() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"other"));
        assert!(!constant_time_eq(b"token", b"token-extra"));
    }

    #[tokio::test]
    async fn require_auth_inserts_local_admin_principal_for_valid_token() {
        let router = Router::new()
            .route(
                "/principal",
                get(|request: Request| async move {
                    request
                        .extensions()
                        .get::<UserPrincipalId>()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                }),
            )
            .layer(axum::middleware::from_fn(require_auth))
            .layer(axum::Extension(InboundAuthState::bearer_token("secret")));
        let request = axum::http::Request::builder()
            .uri("/principal")
            .header(header::AUTHORIZATION, "Bearer secret")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(body.as_ref(), b"local-admin");
    }

    #[tokio::test]
    async fn require_auth_inserts_local_admin_principal_when_auth_disabled() {
        let router = Router::new()
            .route(
                "/principal",
                get(|request: Request| async move {
                    request
                        .extensions()
                        .get::<UserPrincipalId>()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                }),
            )
            .layer(axum::middleware::from_fn(require_auth))
            .layer(axum::Extension(InboundAuthState {
                enabled: false,
                token: None,
            }));
        let request = axum::http::Request::builder()
            .uri("/principal")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(body.as_ref(), b"local-admin");
    }
}
