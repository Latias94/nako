use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

pub(super) const X_REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

const GENERATED_REQUEST_ID_PREFIX: &str = "req_";
const MAX_INBOUND_REQUEST_ID_LEN: usize = 96;
const MIN_INBOUND_REQUEST_ID_LEN: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HttpTraceContext {
    request_id: String,
}

impl HttpTraceContext {
    #[must_use]
    pub(super) fn request_id(&self) -> &str {
        &self.request_id
    }
}

pub(super) async fn attach_http_trace_context(mut request: Request, next: Next) -> Response {
    let context = HttpTraceContext {
        request_id: request_id_from_header(&request).unwrap_or_else(generate_request_id),
    };
    let request_id = context.request_id().to_owned();
    request.extensions_mut().insert(context);

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(X_REQUEST_ID_HEADER, value);
    }
    response
}

fn request_id_from_header(request: &Request) -> Option<String> {
    let value = request.headers().get(&X_REQUEST_ID_HEADER)?.to_str().ok()?;
    normalize_inbound_request_id(value)
}

fn normalize_inbound_request_id(value: &str) -> Option<String> {
    if value.len() < MIN_INBOUND_REQUEST_ID_LEN || value.len() > MAX_INBOUND_REQUEST_ID_LEN {
        return None;
    }
    if !value.bytes().all(is_safe_request_id_byte) {
        return None;
    }

    Some(value.to_ascii_lowercase())
}

fn is_safe_request_id_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_' | b'.')
}

fn generate_request_id() -> String {
    format!(
        "{GENERATED_REQUEST_ID_PREFIX}{}",
        uuid::Uuid::new_v4().simple()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Extension, Router, body::Body, routing::get};
    use tower::ServiceExt;

    async fn trace_context_request_id(Extension(context): Extension<HttpTraceContext>) -> String {
        context.request_id().to_owned()
    }

    #[test]
    fn normalizes_safe_request_id() {
        assert_eq!(
            normalize_inbound_request_id("REQ-ABC_123.trace"),
            Some("req-abc_123.trace".to_owned())
        );
    }

    #[test]
    fn rejects_unsafe_request_id_values() {
        assert_eq!(normalize_inbound_request_id(""), None);
        assert_eq!(normalize_inbound_request_id("short"), None);
        assert_eq!(normalize_inbound_request_id(" REQ-ABC_123"), None);
        assert_eq!(normalize_inbound_request_id("https://secret.example"), None);
        assert_eq!(normalize_inbound_request_id("token,other"), None);
        assert_eq!(normalize_inbound_request_id("local\\path"), None);
        assert_eq!(normalize_inbound_request_id(&"a".repeat(97)), None);
    }

    #[test]
    fn generated_request_id_is_safe() {
        let request_id = generate_request_id();
        assert!(request_id.starts_with(GENERATED_REQUEST_ID_PREFIX));
        assert!(normalize_inbound_request_id(&request_id).is_some());
    }

    #[tokio::test]
    async fn middleware_inserts_typed_trace_context_and_response_header() {
        let response = Router::new()
            .route("/", get(trace_context_request_id))
            .layer(axum::middleware::from_fn(attach_http_trace_context))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .header(&X_REQUEST_ID_HEADER, "REQ-ABC_123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.headers()[X_REQUEST_ID_HEADER], "req-abc_123");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"req-abc_123");
    }
}
