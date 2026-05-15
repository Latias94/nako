use std::{env, sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::{
    Client, Method, StatusCode, Url,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use roxmltree::Document;
use taru_core::{Result, TaruError};

use crate::{
    ByteRange, ObjectKind, ObjectMetadata, StorageBackend, StorageCapabilities, StorageUri,
    VirtualFile,
};

#[derive(Clone, Debug)]
pub struct WebDavBackendConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub password_env: Option<String>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
}

impl WebDavBackendConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            username: None,
            password_env: None,
            timeout_ms: 30_000,
            max_attempts: 2,
        }
    }
}

pub trait WebDavSecretResolver: Send + Sync {
    fn resolve(&self, env_name: &str) -> Result<String>;
}

#[derive(Clone, Debug, Default)]
pub struct EnvWebDavSecretResolver;

impl WebDavSecretResolver for EnvWebDavSecretResolver {
    fn resolve(&self, env_name: &str) -> Result<String> {
        env::var(env_name).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to resolve WebDAV secret env {env_name}: {err}"),
        })
    }
}

#[derive(Clone)]
pub struct WebDavBackend {
    base_url: Url,
    username: Option<String>,
    password: Option<String>,
    timeout: Duration,
    max_attempts: u32,
    client: Client,
}

impl WebDavBackend {
    pub fn new(config: WebDavBackendConfig) -> Result<Self> {
        Self::new_with_client_and_resolver(config, Client::new(), Arc::new(EnvWebDavSecretResolver))
    }

    pub fn new_with_client_and_resolver(
        config: WebDavBackendConfig,
        client: Client,
        resolver: Arc<dyn WebDavSecretResolver>,
    ) -> Result<Self> {
        let base_url = Url::parse(&config.base_url).map_err(|err| TaruError::InvalidInput {
            message: format!("invalid WebDAV base_url: {err}"),
        })?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err(TaruError::InvalidInput {
                message: "WebDAV base_url must use http or https".to_owned(),
            });
        }
        if base_url.username() != "" || base_url.password().is_some() {
            return Err(TaruError::InvalidInput {
                message: "WebDAV base_url must not contain credentials".to_owned(),
            });
        }
        if !(100..=120_000).contains(&config.timeout_ms) {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "WebDAV timeout_ms is outside allowed range: {}",
                    config.timeout_ms
                ),
            });
        }
        if !(1..=10).contains(&config.max_attempts) {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "WebDAV max_attempts is outside allowed range: {}",
                    config.max_attempts
                ),
            });
        }

        let password = match config.password_env.as_deref() {
            Some(env_name) => Some(resolver.resolve(env_name)?),
            None => None,
        };

        Ok(Self {
            base_url,
            username: config.username,
            password,
            timeout: Duration::from_millis(config.timeout_ms),
            max_attempts: config.max_attempts,
            client,
        })
    }

    async fn propfind(&self, uri: &StorageUri, depth: &'static str) -> Result<Vec<WebDavProp>> {
        self.ensure_webdav_scheme(uri)?;
        let url = self.url_for(uri)?;
        let method = Method::from_bytes(b"PROPFIND").map_err(|err| TaruError::Storage {
            uri: uri.to_string(),
            message: format!("failed to build WebDAV PROPFIND method: {err}"),
        })?;
        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:resourcetype/>
    <D:getcontentlength/>
    <D:getlastmodified/>
    <D:getetag/>
  </D:prop>
</D:propfind>"#;
        let response = self
            .send_with_retry(|| {
                let mut request = self
                    .client
                    .request(method.clone(), url.clone())
                    .timeout(self.timeout)
                    .headers(xml_headers(depth))
                    .body(body.to_owned());
                request = self.apply_auth(request);
                request
            })
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(TaruError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            });
        }
        if !response.status().is_success() && response.status().as_u16() != 207 {
            return Err(TaruError::Storage {
                uri: uri.to_string(),
                message: format!("WebDAV PROPFIND returned {}", response.status()),
            });
        }

        let text = response.text().await.map_err(|err| TaruError::Storage {
            uri: uri.to_string(),
            message: format!("failed to read WebDAV PROPFIND response: {err}"),
        })?;
        parse_multistatus(uri, self.base_url.path(), &text)
    }

    async fn head_or_get_metadata(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        let props = self.propfind(uri, "0").await?;
        props
            .into_iter()
            .find(|prop| same_storage_object(&prop.uri, uri))
            .map(WebDavProp::into_metadata)
            .ok_or_else(|| TaruError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            })
    }

    fn ensure_webdav_scheme(&self, uri: &StorageUri) -> Result<()> {
        if uri.scheme() != self.scheme() {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "WebDAV backend only accepts '{}' uris, got '{}'",
                    self.scheme(),
                    uri.scheme()
                ),
            });
        }

        Ok(())
    }

    fn url_for(&self, uri: &StorageUri) -> Result<Url> {
        self.ensure_webdav_scheme(uri)?;
        if uri.as_str().contains('@') {
            return Err(TaruError::InvalidInput {
                message: "WebDAV storage uri must not contain credentials".to_owned(),
            });
        }

        let mut url = self.base_url.clone();
        let mut path = self.base_url.path().trim_end_matches('/').to_owned();
        let relative = webdav_relative_path(uri)?;
        if !relative.is_empty() {
            path.push('/');
            path.push_str(&relative);
        }
        if uri.path_part().ends_with('/') && !path.ends_with('/') {
            path.push('/');
        }
        url.set_path(&path);
        url.set_query(None);
        url.set_fragment(None);
        Ok(url)
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match (self.username.as_deref(), self.password.as_deref()) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        }
    }

    async fn send_with_retry<F>(&self, mut build: F) -> Result<reqwest::Response>
    where
        F: FnMut() -> reqwest::RequestBuilder,
    {
        let mut last_error = None;
        for attempt in 1..=self.max_attempts {
            match build().send().await {
                Ok(response) if !is_retryable_status(response.status()) => return Ok(response),
                Ok(response) => {
                    if attempt == self.max_attempts {
                        return Ok(response);
                    }
                    last_error = Some(format!("WebDAV returned {}", response.status()));
                }
                Err(err) => {
                    if attempt == self.max_attempts || !err.is_timeout() && !err.is_connect() {
                        return Err(TaruError::Storage {
                            uri: self.base_url.to_string(),
                            message: format!("WebDAV request failed: {err}"),
                        });
                    }
                    last_error = Some(err.to_string());
                }
            }
        }

        Err(TaruError::Storage {
            uri: self.base_url.to_string(),
            message: last_error.unwrap_or_else(|| "WebDAV request failed".to_owned()),
        })
    }
}

#[async_trait]
impl StorageBackend for WebDavBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.head_or_get_metadata(uri).await
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        let root = normalize_collection_uri(uri)?;
        let props = self.propfind(&root, "1").await?;
        let root_value = root.as_str();
        let mut entries = props
            .into_iter()
            .filter(|prop| prop.uri.as_str() != root_value)
            .map(WebDavProp::into_metadata)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.uri.as_str().cmp(right.uri.as_str()));
        Ok(entries)
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        let metadata = self.stat(uri).await?;
        if metadata.kind != ObjectKind::File {
            return Err(TaruError::InvalidInput {
                message: format!("cannot open non-file WebDAV uri: {uri}"),
            });
        }
        if let Some(range) = range {
            if let Some(len) = metadata.len {
                validate_range(uri, range, len)?;
            }
        }

        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        let url = self.url_for(uri)?;
        let response = self
            .send_with_retry(|| {
                let request = self.client.get(url.clone()).timeout(self.timeout);
                self.apply_auth(request)
            })
            .await?;

        if !response.status().is_success() {
            return Err(TaruError::Storage {
                uri: uri.to_string(),
                message: format!("WebDAV GET returned {}", response.status()),
            });
        }

        response.text().await.map_err(|err| TaruError::Storage {
            uri: uri.to_string(),
            message: format!("failed to read WebDAV text response: {err}"),
        })
    }

    async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
        let _ = uri;
        Err(TaruError::Unsupported(
            "WebDAV backend is read-only in M6.1",
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WebDavProp {
    uri: StorageUri,
    kind: ObjectKind,
    len: Option<u64>,
    modified_at: Option<String>,
    etag: Option<String>,
}

impl WebDavProp {
    fn into_metadata(self) -> ObjectMetadata {
        let fingerprint = self
            .etag
            .as_ref()
            .map(|etag| format!("webdav:etag={etag}"))
            .or_else(|| {
                self.len
                    .zip(self.modified_at.as_ref())
                    .map(|(len, modified_at)| format!("webdav:size={len}:modified={modified_at}"))
            });

        ObjectMetadata {
            uri: self.uri,
            kind: self.kind,
            len: self.len,
            modified_at: self.modified_at,
            etag: self.etag,
            fingerprint,
            capabilities: webdav_capabilities(self.kind),
        }
    }
}

fn parse_multistatus(
    request_uri: &StorageUri,
    base_path: &str,
    xml: &str,
) -> Result<Vec<WebDavProp>> {
    let document = Document::parse(xml).map_err(|err| TaruError::Storage {
        uri: request_uri.to_string(),
        message: format!("failed to parse WebDAV multistatus XML: {err}"),
    })?;
    let mut props = Vec::new();
    for response in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "response")
    {
        let href = child_text(response, "href").ok_or_else(|| TaruError::Storage {
            uri: request_uri.to_string(),
            message: "WebDAV response missing href".to_owned(),
        })?;
        let uri = storage_uri_from_href(&href, base_path)?;
        let is_collection = response.descendants().any(|node| {
            node.is_element()
                && node.tag_name().name() == "collection"
                && node.ancestors().any(|ancestor| {
                    ancestor.is_element() && ancestor.tag_name().name() == "resourcetype"
                })
        });
        let kind = if is_collection {
            ObjectKind::Directory
        } else {
            ObjectKind::File
        };
        let len = child_text(response, "getcontentlength").and_then(|value| value.parse().ok());
        let modified_at = child_text(response, "getlastmodified");
        let etag = child_text(response, "getetag").map(clean_etag);

        props.push(WebDavProp {
            uri,
            kind,
            len,
            modified_at,
            etag,
        });
    }

    Ok(props)
}

fn child_text(node: roxmltree::Node<'_, '_>, name: &str) -> Option<String> {
    node.descendants()
        .find(|child| child.is_element() && child.tag_name().name() == name)
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn storage_uri_from_href(href: &str, base_path: &str) -> Result<StorageUri> {
    let path = if let Ok(url) = Url::parse(href) {
        url.path().to_owned()
    } else {
        href.to_owned()
    };
    let base_path = base_path.trim_end_matches('/');
    let relative = if base_path.is_empty() || base_path == "/" {
        path.as_str()
    } else {
        path.strip_prefix(base_path).unwrap_or(path.as_str())
    };

    StorageUri::from_parts("webdav", relative.trim_start_matches('/'))
}

fn same_storage_object(left: &StorageUri, right: &StorageUri) -> bool {
    left.scheme() == right.scheme()
        && left.path_part().trim_end_matches('/') == right.path_part().trim_end_matches('/')
}

fn webdav_relative_path(uri: &StorageUri) -> Result<String> {
    if uri.as_str().contains('@') {
        return Err(TaruError::InvalidInput {
            message: "WebDAV storage uri must not contain credentials".to_owned(),
        });
    }
    let raw = uri.path_part().trim_start_matches(['/', '\\']);
    let normalized = raw.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                return Err(TaruError::InvalidInput {
                    message: format!("WebDAV uri path is not allowed to escape root: {uri}"),
                });
            }
            value => parts.push(value),
        }
    }

    Ok(parts.join("/"))
}

fn normalize_collection_uri(uri: &StorageUri) -> Result<StorageUri> {
    if uri.path_part().ends_with('/') {
        return Ok(uri.clone());
    }
    StorageUri::parse(format!("{}/", uri.as_str()))
}

fn xml_headers(depth: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Depth", HeaderValue::from_static(depth));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    headers
}

fn clean_etag(value: String) -> String {
    value.trim_matches('"').to_owned()
}

fn webdav_capabilities(kind: ObjectKind) -> StorageCapabilities {
    let base = StorageCapabilities::SEEKABLE
        | StorageCapabilities::RANGE_READABLE
        | StorageCapabilities::EXPENSIVE_LISTING
        | StorageCapabilities::RATE_LIMITED
        | StorageCapabilities::REMOTE_LATENCY;

    match kind {
        ObjectKind::File | ObjectKind::Directory => base,
        ObjectKind::Symlink | ObjectKind::Other => StorageCapabilities::REMOTE_LATENCY,
    }
}

fn is_retryable_status(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn validate_range(uri: &StorageUri, range: ByteRange, len: u64) -> Result<()> {
    if range.offset > len {
        return Err(TaruError::InvalidInput {
            message: format!(
                "range offset {} exceeds file length {len}: {uri}",
                range.offset
            ),
        });
    }

    if let Some(length) = range.length {
        let Some(end) = range.offset.checked_add(length) else {
            return Err(TaruError::InvalidInput {
                message: format!("range overflows file length: {uri}"),
            });
        };

        if end > len {
            return Err(TaruError::InvalidInput {
                message: format!("range end {end} exceeds file length {len}: {uri}"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::State,
        http::{HeaderMap as AxumHeaderMap, StatusCode as AxumStatusCode, header},
        response::{IntoResponse, Response},
        routing::any,
    };
    use tokio::net::TcpListener;

    use super::*;

    #[tokio::test]
    async fn webdav_backend_stats_lists_and_opens_ranges_without_local_paths() {
        let server = MockWebDavServer::start().await;
        let backend = WebDavBackend::new_with_client_and_resolver(
            WebDavBackendConfig {
                base_url: server.base_url(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 2,
            },
            Client::new(),
            Arc::new(TestSecretResolver::default()),
        )
        .unwrap();

        let movie = StorageUri::from_parts("webdav", "Movies/Demo.mkv").unwrap();
        let stat = backend.stat(&movie).await.unwrap();
        assert_eq!(stat.kind, ObjectKind::File);
        assert_eq!(stat.len, Some(4));
        assert_eq!(stat.etag, Some("etag-demo".to_owned()));
        assert!(
            stat.capabilities
                .contains(StorageCapabilities::RANGE_READABLE)
        );
        assert!(
            stat.capabilities
                .contains(StorageCapabilities::REMOTE_LATENCY)
        );

        let file = backend
            .open_range(
                &movie,
                Some(ByteRange {
                    offset: 1,
                    length: Some(2),
                }),
            )
            .await
            .unwrap();
        assert_eq!(file.uri, movie);
        assert_eq!(file.local_path_hint, None);

        let entries = backend
            .list(&StorageUri::from_parts("webdav", "Movies").unwrap())
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].uri.as_str(), "webdav:///Movies/Demo.mkv");
        assert_eq!(entries[0].kind, ObjectKind::File);
    }

    #[tokio::test]
    async fn webdav_backend_uses_secret_reference_without_leaking_credentials_to_locator() {
        let server = MockWebDavServer::start().await;
        let resolver = Arc::new(TestSecretResolver::new([(
            "TARU_WEBDAV_PASSWORD".to_owned(),
            "secret-password".to_owned(),
        )]));
        let backend = WebDavBackend::new_with_client_and_resolver(
            WebDavBackendConfig {
                base_url: server.base_url(),
                username: Some("media".to_owned()),
                password_env: Some("TARU_WEBDAV_PASSWORD".to_owned()),
                timeout_ms: 5_000,
                max_attempts: 1,
            },
            Client::new(),
            resolver,
        )
        .unwrap();

        let uri = StorageUri::from_parts("webdav", "Movies/Demo.mkv").unwrap();
        let stat = backend.stat(&uri).await.unwrap();

        assert_eq!(stat.uri.as_str(), "webdav:///Movies/Demo.mkv");
        assert!(!stat.uri.as_str().contains("secret-password"));
        assert!(!stat.uri.as_str().contains("media@"));
        let auth = server.last_authorization().unwrap();
        assert!(auth.starts_with("Basic "));
    }

    #[tokio::test]
    async fn webdav_backend_rejects_credentials_in_urls_and_uris() {
        let err = match WebDavBackend::new(WebDavBackendConfig {
            base_url: "https://user:password@example.test/dav".to_owned(),
            username: None,
            password_env: None,
            timeout_ms: 5_000,
            max_attempts: 1,
        }) {
            Ok(_) => panic!("expected WebDAV backend to reject credentials in base_url"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("must not contain credentials"));

        let server = MockWebDavServer::start().await;
        let backend = WebDavBackend::new(WebDavBackendConfig::new(server.base_url())).unwrap();
        let uri = StorageUri::parse("webdav://user:password@example.test/Movies/Demo.mkv").unwrap();
        let err = backend.stat(&uri).await.unwrap_err();
        assert!(err.to_string().contains("must not contain credentials"));
    }

    #[tokio::test]
    async fn webdav_backend_is_read_only() {
        let server = MockWebDavServer::start().await;
        let backend = WebDavBackend::new(WebDavBackendConfig::new(server.base_url())).unwrap();
        let uri = StorageUri::from_parts("webdav", "Movies/Demo.nfo").unwrap();

        let err = backend.write_string(&uri, "bad").await.unwrap_err();

        assert!(err.to_string().contains("read-only"));
    }

    #[derive(Default)]
    struct TestSecretResolver {
        values: HashMap<String, String>,
    }

    impl TestSecretResolver {
        fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
            Self {
                values: values.into_iter().collect(),
            }
        }
    }

    impl WebDavSecretResolver for TestSecretResolver {
        fn resolve(&self, env_name: &str) -> Result<String> {
            self.values
                .get(env_name)
                .cloned()
                .ok_or_else(|| TaruError::InvalidInput {
                    message: format!("missing test secret: {env_name}"),
                })
        }
    }

    #[derive(Clone, Default)]
    struct MockWebDavState {
        last_authorization: Arc<Mutex<Option<String>>>,
    }

    struct MockWebDavServer {
        addr: std::net::SocketAddr,
        state: MockWebDavState,
    }

    impl MockWebDavServer {
        async fn start() -> Self {
            let state = MockWebDavState::default();
            let router = Router::new()
                .route("/{*path}", any(webdav_handler))
                .with_state(state.clone());
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });

            Self { addr, state }
        }

        fn base_url(&self) -> String {
            format!("http://{}/dav", self.addr)
        }

        fn last_authorization(&self) -> Option<String> {
            self.state.last_authorization.lock().unwrap().clone()
        }
    }

    async fn webdav_handler(
        State(state): State<MockWebDavState>,
        method: axum::http::Method,
        uri: axum::http::Uri,
        headers: AxumHeaderMap,
        body: Body,
    ) -> Response {
        if let Some(value) = headers.get(header::AUTHORIZATION) {
            *state.last_authorization.lock().unwrap() = Some(value.to_str().unwrap().to_owned());
        }

        let path = uri.path();
        if method.as_str() == "PROPFIND" {
            if path.ends_with("/Movies/") {
                return (
                    AxumStatusCode::MULTI_STATUS,
                    [(header::CONTENT_TYPE, "application/xml")],
                    multistatus(&[
                        WebDavFixture {
                            href: "/dav/Movies/",
                            collection: true,
                            len: None,
                            etag: None,
                        },
                        WebDavFixture {
                            href: "/dav/Movies/Demo.mkv",
                            collection: false,
                            len: Some(4),
                            etag: Some("etag-demo"),
                        },
                    ]),
                )
                    .into_response();
            }

            if path.ends_with("/Movies/Demo.mkv") {
                return (
                    AxumStatusCode::MULTI_STATUS,
                    [(header::CONTENT_TYPE, "application/xml")],
                    multistatus(&[WebDavFixture {
                        href: "/dav/Movies/Demo.mkv",
                        collection: false,
                        len: Some(4),
                        etag: Some("etag-demo"),
                    }]),
                )
                    .into_response();
            }

            return AxumStatusCode::NOT_FOUND.into_response();
        }

        if method == axum::http::Method::GET && path.ends_with("/Movies/Demo.nfo") {
            return "nfo".into_response();
        }

        let _ = to_bytes(body, usize::MAX).await.unwrap();
        AxumStatusCode::METHOD_NOT_ALLOWED.into_response()
    }

    struct WebDavFixture {
        href: &'static str,
        collection: bool,
        len: Option<u64>,
        etag: Option<&'static str>,
    }

    fn multistatus(fixtures: &[WebDavFixture]) -> String {
        let responses = fixtures
            .iter()
            .map(|fixture| {
                let resourcetype = if fixture.collection {
                    "<D:resourcetype><D:collection/></D:resourcetype>"
                } else {
                    "<D:resourcetype/>"
                };
                let length = fixture
                    .len
                    .map(|value| format!("<D:getcontentlength>{value}</D:getcontentlength>"))
                    .unwrap_or_default();
                let etag = fixture
                    .etag
                    .map(|value| format!("<D:getetag>\"{value}\"</D:getetag>"))
                    .unwrap_or_default();
                format!(
                    r#"<D:response>
  <D:href>{}</D:href>
  <D:propstat>
    <D:prop>
      {}
      {}
      {}
      <D:getlastmodified>Fri, 15 May 2026 00:00:00 GMT</D:getlastmodified>
    </D:prop>
    <D:status>HTTP/1.1 200 OK</D:status>
  </D:propstat>
</D:response>"#,
                    fixture.href, resourcetype, length, etag
                )
            })
            .collect::<String>();
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">{responses}</D:multistatus>"#
        )
    }
}
