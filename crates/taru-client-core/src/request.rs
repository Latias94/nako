use crate::encoding::{path_with_query, url_on};
use crate::redaction::sanitize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpHeader {
    pub name: String,
    pub value: String,
}

impl CoreHttpHeader {
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreQueryParam {
    pub name: String,
    pub value: String,
}

impl CoreQueryParam {
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpRequestSpec {
    pub request_id: String,
    pub base_url: String,
    pub method: String,
    pub path: String,
    pub query: Vec<CoreQueryParam>,
    pub headers: Vec<CoreHttpHeader>,
    pub access_token: Option<String>,
    pub body_utf8: Option<String>,
}

impl CoreHttpRequestSpec {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        base_url: impl Into<String>,
        method: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            base_url: base_url.into(),
            method: method.into(),
            path: path.into(),
            query: Vec::new(),
            headers: Vec::new(),
            access_token: None,
            body_utf8: None,
        }
    }

    #[must_use]
    pub fn query(mut self, query: Vec<CoreQueryParam>) -> Self {
        self.query = query;
        self
    }

    #[must_use]
    pub fn headers(mut self, headers: Vec<CoreHttpHeader>) -> Self {
        self.headers = headers;
        self
    }

    #[must_use]
    pub fn access_token(mut self, access_token: Option<String>) -> Self {
        self.access_token = access_token;
        self
    }

    #[must_use]
    pub fn body_utf8(mut self, body_utf8: Option<String>) -> Self {
        self.body_utf8 = body_utf8;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpRequest {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: Option<String>,
    pub safe_preview: CoreSafeRequestPreview,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreSafeRequestPreview {
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
}

#[must_use]
pub fn build_core_request(spec: &CoreHttpRequestSpec) -> CoreHttpRequest {
    let mut headers = spec.headers.clone();
    let access_token = spec.access_token.as_deref().map(str::trim);
    if let Some(token) = access_token.filter(|token| !token.is_empty()) {
        if !headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("authorization"))
        {
            headers.insert(
                0,
                CoreHttpHeader::new("Authorization", format!("Bearer {token}")),
            );
        }
    }

    let secrets = access_token.into_iter().collect::<Vec<_>>();
    request(
        &spec.request_id,
        &spec.method,
        &url_on(&spec.base_url, &path_with_query(&spec.path, &spec.query)),
        headers,
        spec.body_utf8.clone(),
        &secrets,
    )
}

pub(crate) fn request(
    request_id: &str,
    method: &str,
    url: &str,
    headers: Vec<CoreHttpHeader>,
    body_utf8: Option<String>,
    secrets: &[&str],
) -> CoreHttpRequest {
    let safe_preview = CoreSafeRequestPreview {
        method: method.to_owned(),
        url: sanitize(url, secrets),
        headers: headers
            .iter()
            .map(|header| safe_header(header, secrets))
            .collect(),
    };
    CoreHttpRequest {
        request_id: request_id.to_owned(),
        method: method.to_owned(),
        url: url.to_owned(),
        headers,
        body_utf8,
        safe_preview,
    }
}

fn safe_header(header: &CoreHttpHeader, secrets: &[&str]) -> CoreHttpHeader {
    if header.name.eq_ignore_ascii_case("authorization") {
        return CoreHttpHeader::new(&header.name, "Bearer <redacted>");
    }
    CoreHttpHeader::new(&header.name, sanitize(&header.value, secrets))
}
