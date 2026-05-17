use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{
    Method, StatusCode, Url,
    header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue},
};
use serde::de::DeserializeOwned;
pub use taru_client_protocol::{
    API_VERSION_HEADER, CLIENT_PROTOCOL_VERSION as API_VERSION, ErrorResponse, GenreItemsResponse,
    GenreListResponse, HealthResponse, ImagesResponse, ItemCreditsResponse, ItemDetailResponse,
    ItemsResponse, LibraryListResponse, LibraryResponse, LibrarySourcesResponse, PageInfo,
    PeopleResponse, PersonItemsResponse, PersonResponse, PlaybackDecisionResponse, SearchResponse,
    SourceProbeResponse, TagItemsResponse, TagsResponse, TranscodeSessionResponse,
};
use thiserror::Error;

pub const PUBLIC_CLIENT_PATHS: &[&str] = &[
    "/health",
    "/libraries",
    "/libraries/{library_id}",
    "/libraries/{library_id}/sources",
    "/items",
    "/items/{item_id}",
    "/items/{item_id}/credits",
    "/items/{item_id}/images",
    "/people",
    "/people/{person_id}",
    "/people/{person_id}/items",
    "/tags",
    "/tags/{tag_id}/items",
    "/genres",
    "/genres/{genre_id}/items",
    "/search",
    "/sources/{source_id}/probe",
    "/sources/{source_id}/playback/decision",
    "/playback/sessions/{session_id}",
    "/playback/sessions/{session_id}/cancel",
];

#[derive(Clone)]
pub struct TaruClient {
    base_url: Url,
    bearer_token: Option<String>,
    transport: Arc<dyn ClientTransport>,
}

impl TaruClient {
    /// Build a client with the default reqwest transport.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not an absolute URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, TaruClientError> {
        Self::with_transport(base_url, ReqwestTransport::default())
    }

    /// Build a client with a bearer token and the default reqwest transport.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not an absolute URL.
    pub fn with_bearer_token(
        base_url: impl AsRef<str>,
        token: impl Into<String>,
    ) -> Result<Self, TaruClientError> {
        Ok(Self::new(base_url)?.bearer_token(token))
    }

    /// Build a client with a custom transport for tests or alternate runtimes.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not an absolute URL.
    pub fn with_transport(
        base_url: impl AsRef<str>,
        transport: impl ClientTransport + 'static,
    ) -> Result<Self, TaruClientError> {
        let mut base_url =
            Url::parse(base_url.as_ref()).map_err(|source| TaruClientError::InvalidBaseUrl {
                reason: source.to_string(),
            })?;
        base_url.set_fragment(None);
        base_url.set_query(None);
        if !base_url.path().ends_with('/') {
            let mut path = base_url.path().to_owned();
            path.push('/');
            base_url.set_path(&path);
        }
        Ok(Self {
            base_url,
            bearer_token: None,
            transport: Arc::new(transport),
        })
    }

    #[must_use]
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    /// Get server health and public API version.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn health(&self) -> Result<HealthResponse, TaruClientError> {
        self.request_json_no_query(Method::GET, "/health", false)
            .await
    }

    /// List configured media libraries.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_libraries(
        &self,
        page: Option<PageQuery>,
    ) -> Result<LibraryListResponse, TaruClientError> {
        self.request_json(Method::GET, "/libraries", page.as_ref(), true)
            .await
    }

    /// Get one media library.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_library(
        &self,
        library_id: impl AsRef<str>,
    ) -> Result<LibraryResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/libraries/{}", encode_path_segment(library_id.as_ref())),
            true,
        )
        .await
    }

    /// List sources in one media library.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_library_sources(
        &self,
        library_id: impl AsRef<str>,
        page: Option<PageQuery>,
    ) -> Result<LibrarySourcesResponse, TaruClientError> {
        self.request_json(
            Method::GET,
            &format!(
                "/libraries/{}/sources",
                encode_path_segment(library_id.as_ref())
            ),
            page.as_ref(),
            true,
        )
        .await
    }

    /// List media items.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_items(
        &self,
        page: Option<PageQuery>,
    ) -> Result<ItemsResponse, TaruClientError> {
        self.request_json(Method::GET, "/items", page.as_ref(), true)
            .await
    }

    /// Get one media item with catalog relations.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_item(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<ItemDetailResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/items/{}", encode_path_segment(item_id.as_ref())),
            true,
        )
        .await
    }

    /// List credits for one media item.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_item_credits(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<ItemCreditsResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/items/{}/credits", encode_path_segment(item_id.as_ref())),
            true,
        )
        .await
    }

    /// List images for one media item.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_item_images(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<ImagesResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/items/{}/images", encode_path_segment(item_id.as_ref())),
            true,
        )
        .await
    }

    /// List people.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_people(
        &self,
        page: Option<PageQuery>,
    ) -> Result<PeopleResponse, TaruClientError> {
        self.request_json(Method::GET, "/people", page.as_ref(), true)
            .await
    }

    /// Get one person.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_person(
        &self,
        person_id: impl AsRef<str>,
    ) -> Result<PersonResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/people/{}", encode_path_segment(person_id.as_ref())),
            true,
        )
        .await
    }

    /// List media items linked to one person.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_person_items(
        &self,
        person_id: impl AsRef<str>,
        page: Option<PageQuery>,
    ) -> Result<PersonItemsResponse, TaruClientError> {
        self.request_json(
            Method::GET,
            &format!("/people/{}/items", encode_path_segment(person_id.as_ref())),
            page.as_ref(),
            true,
        )
        .await
    }

    /// List tags.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_tags(
        &self,
        page: Option<PageQuery>,
    ) -> Result<TagsResponse, TaruClientError> {
        self.request_json(Method::GET, "/tags", page.as_ref(), true)
            .await
    }

    /// List media items linked to one tag.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_tag_items(
        &self,
        tag_id: impl AsRef<str>,
        page: Option<PageQuery>,
    ) -> Result<TagItemsResponse, TaruClientError> {
        self.request_json(
            Method::GET,
            &format!("/tags/{}/items", encode_path_segment(tag_id.as_ref())),
            page.as_ref(),
            true,
        )
        .await
    }

    /// List genres.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_genres(
        &self,
        page: Option<PageQuery>,
    ) -> Result<GenreListResponse, TaruClientError> {
        self.request_json(Method::GET, "/genres", page.as_ref(), true)
            .await
    }

    /// List media items linked to one genre.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_genre_items(
        &self,
        genre_id: impl AsRef<str>,
        page: Option<PageQuery>,
    ) -> Result<GenreItemsResponse, TaruClientError> {
        self.request_json(
            Method::GET,
            &format!("/genres/{}/items", encode_path_segment(genre_id.as_ref())),
            page.as_ref(),
            true,
        )
        .await
    }

    /// Search media items.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn search_items(
        &self,
        query: SearchQuery<'_>,
    ) -> Result<SearchResponse, TaruClientError> {
        self.request_json(Method::GET, "/search", Some(&query), true)
            .await
    }

    /// Get persisted media probe data for one source.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_source_probe(
        &self,
        source_id: impl AsRef<str>,
    ) -> Result<SourceProbeResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!("/sources/{}/probe", encode_path_segment(source_id.as_ref())),
            true,
        )
        .await
    }

    /// Get playback decision for one source.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_playback_decision(
        &self,
        source_id: impl AsRef<str>,
        capabilities: Option<PlaybackCapabilitiesQuery<'_>>,
    ) -> Result<PlaybackDecisionResponse, TaruClientError> {
        self.request_json(
            Method::GET,
            &format!(
                "/sources/{}/playback/decision",
                encode_path_segment(source_id.as_ref())
            ),
            capabilities.as_ref(),
            true,
        )
        .await
    }

    /// Get one playback session.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_playback_session(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<TranscodeSessionResponse, TaruClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!(
                "/playback/sessions/{}",
                encode_path_segment(session_id.as_ref())
            ),
            true,
        )
        .await
    }

    /// Request playback session cancellation.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn cancel_playback_session(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<TranscodeSessionResponse, TaruClientError> {
        self.request_json_no_query(
            Method::POST,
            &format!(
                "/playback/sessions/{}/cancel",
                encode_path_segment(session_id.as_ref())
            ),
            true,
        )
        .await
    }

    async fn request_json_no_query<T>(
        &self,
        method: Method,
        path: &str,
        auth: bool,
    ) -> Result<T, TaruClientError>
    where
        T: DeserializeOwned,
    {
        self.request_json::<T, NoQuery>(method, path, None, auth)
            .await
    }

    async fn request_json<T, Q>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        auth: bool,
    ) -> Result<T, TaruClientError>
    where
        T: DeserializeOwned,
        Q: QueryParams + ?Sized,
    {
        let request = self.build_request(method, path, query, auth)?;
        let response = self.transport.send(request).await?;
        self.ensure_success(&response)?;
        self.ensure_version(&response)?;
        serde_json::from_slice(&response.body).map_err(|source| TaruClientError::Decode {
            path: path.to_owned(),
            source,
        })
    }

    fn build_request<Q>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        auth: bool,
    ) -> Result<ClientRequest, TaruClientError>
    where
        Q: QueryParams + ?Sized,
    {
        let mut url = self
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|source| TaruClientError::InvalidPath {
                path: path.to_owned(),
                reason: source.to_string(),
            })?;

        if let Some(query) = query {
            let mut query_pairs = Vec::new();
            query.append_query(&mut query_pairs);
            if !query_pairs.is_empty() {
                let mut pairs = url.query_pairs_mut();
                for (key, value) in query_pairs {
                    pairs.append_pair(&key, &value);
                }
            }
        }

        let mut headers = HeaderMap::new();
        if auth {
            if let Some(token) = &self.bearer_token {
                let value =
                    HeaderValue::from_str(&format!("Bearer {token}")).map_err(|source| {
                        TaruClientError::InvalidHeader {
                            name: AUTHORIZATION,
                            source,
                        }
                    })?;
                headers.insert(AUTHORIZATION, value);
            }
        }

        Ok(ClientRequest {
            method,
            url,
            headers,
        })
    }

    fn ensure_success(&self, response: &ClientResponse) -> Result<(), TaruClientError> {
        if response.status.is_success() {
            return Ok(());
        }

        let body = serde_json::from_slice::<ErrorResponse>(&response.body).unwrap_or_else(|_| {
            ErrorResponse {
                code: "invalid_input".to_owned(),
                message: response
                    .status
                    .canonical_reason()
                    .unwrap_or("HTTP error")
                    .to_owned(),
            }
        });
        Err(TaruClientError::Api {
            status: response.status,
            body,
        })
    }

    fn ensure_version(&self, response: &ClientResponse) -> Result<(), TaruClientError> {
        let header_name = HeaderName::from_static(API_VERSION_HEADER);
        if let Some(version) = response.headers.get(header_name) {
            let version = version
                .to_str()
                .map_err(|source| TaruClientError::InvalidVersionHeader { source })?;
            if version != API_VERSION {
                return Err(TaruClientError::UnsupportedApiVersion {
                    expected: API_VERSION,
                    actual: version.to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageQuery {
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

impl PageQuery {
    #[must_use]
    pub const fn new(limit: Option<u32>, offset: Option<u64>) -> Self {
        Self { limit, offset }
    }
}

impl QueryParams for PageQuery {
    fn append_query(&self, pairs: &mut Vec<(String, String)>) {
        if let Some(limit) = self.limit {
            pairs.push(("limit".to_owned(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            pairs.push(("offset".to_owned(), offset.to_string()));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SearchQuery<'a> {
    pub q: Option<&'a str>,
    pub facet: Option<&'a str>,
    pub page: Option<PageQuery>,
}

impl QueryParams for SearchQuery<'_> {
    fn append_query(&self, pairs: &mut Vec<(String, String)>) {
        if let Some(q) = self.q {
            pairs.push(("q".to_owned(), q.to_owned()));
        }
        if let Some(facet) = self.facet {
            pairs.push(("facet".to_owned(), facet.to_owned()));
        }
        if let Some(page) = self.page {
            page.append_query(pairs);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlaybackCapabilitiesQuery<'a> {
    pub direct_play: Option<bool>,
    pub container: Option<&'a str>,
    pub video_codec: Option<&'a str>,
    pub audio_codec: Option<&'a str>,
}

impl QueryParams for PlaybackCapabilitiesQuery<'_> {
    fn append_query(&self, pairs: &mut Vec<(String, String)>) {
        if let Some(direct_play) = self.direct_play {
            pairs.push((
                "direct_play".to_owned(),
                if direct_play { "true" } else { "false" }.to_owned(),
            ));
        }
        if let Some(container) = self.container {
            pairs.push(("container".to_owned(), container.to_owned()));
        }
        if let Some(video_codec) = self.video_codec {
            pairs.push(("video_codec".to_owned(), video_codec.to_owned()));
        }
        if let Some(audio_codec) = self.audio_codec {
            pairs.push(("audio_codec".to_owned(), audio_codec.to_owned()));
        }
    }
}

trait QueryParams {
    fn append_query(&self, pairs: &mut Vec<(String, String)>);
}

struct NoQuery;

impl QueryParams for NoQuery {
    fn append_query(&self, _pairs: &mut Vec<(String, String)>) {}
}

#[derive(Clone, Debug)]
pub struct ClientRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
}

#[derive(Clone, Debug)]
pub struct ClientResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[async_trait]
pub trait ClientTransport: Send + Sync {
    async fn send(&self, request: ClientRequest) -> Result<ClientResponse, TaruClientError>;
}

#[derive(Clone, Default)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    #[must_use]
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ClientTransport for ReqwestTransport {
    async fn send(&self, request: ClientRequest) -> Result<ClientResponse, TaruClientError> {
        let response = self
            .client
            .request(request.method, request.url)
            .headers(request.headers)
            .send()
            .await
            .map_err(TaruClientError::Transport)?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .map_err(TaruClientError::Transport)?
            .to_vec();

        Ok(ClientResponse {
            status,
            headers,
            body,
        })
    }
}

#[derive(Debug, Error)]
pub enum TaruClientError {
    #[error("invalid base URL: {reason}")]
    InvalidBaseUrl { reason: String },
    #[error("invalid request path {path}: {reason}")]
    InvalidPath { path: String, reason: String },
    #[error("invalid request header {name}")]
    InvalidHeader {
        name: HeaderName,
        source: reqwest::header::InvalidHeaderValue,
    },
    #[error("transport error")]
    Transport(#[source] reqwest::Error),
    #[error("Taru API returned {status}: {code}", code = body.code)]
    Api {
        status: StatusCode,
        body: ErrorResponse,
    },
    #[error("invalid API version header")]
    InvalidVersionHeader { source: reqwest::header::ToStrError },
    #[error("unsupported Taru API version {actual}, expected {expected}")]
    UnsupportedApiVersion {
        expected: &'static str,
        actual: String,
    },
    #[error("failed to decode response from {path}")]
    Decode {
        path: String,
        source: serde_json::Error,
    },
}

#[must_use]
fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;
    use serde_json::json;
    use std::{collections::VecDeque, sync::Mutex};
    use taru_client_protocol::ClientTranscodeSessionState;

    #[derive(Clone, Default)]
    struct MockTransport {
        requests: Arc<Mutex<Vec<ClientRequest>>>,
        responses: Arc<Mutex<VecDeque<ClientResponse>>>,
    }

    impl MockTransport {
        fn push_json(&self, status: StatusCode, body: serde_json::Value) {
            let mut headers = HeaderMap::new();
            headers.insert(
                HeaderName::from_static(API_VERSION_HEADER),
                HeaderValue::from_static(API_VERSION),
            );
            self.responses.lock().unwrap().push_back(ClientResponse {
                status,
                headers,
                body: serde_json::to_vec(&body).unwrap(),
            });
        }

        fn push_json_with_version(
            &self,
            status: StatusCode,
            version: &'static str,
            body: serde_json::Value,
        ) {
            let mut headers = HeaderMap::new();
            headers.insert(
                HeaderName::from_static(API_VERSION_HEADER),
                HeaderValue::from_static(version),
            );
            self.responses.lock().unwrap().push_back(ClientResponse {
                status,
                headers,
                body: serde_json::to_vec(&body).unwrap(),
            });
        }

        fn requests(&self) -> Vec<ClientRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ClientTransport for MockTransport {
        async fn send(&self, request: ClientRequest) -> Result<ClientResponse, TaruClientError> {
            self.requests.lock().unwrap().push(request);
            Ok(self
                .responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("mock response is queued"))
        }
    }

    #[tokio::test]
    async fn client_adds_auth_and_pagination_query_to_protected_routes() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "libraries": [],
                "page": {"limit": 25, "offset": 50, "returned": 0}
            }),
        );
        let client = TaruClient::with_transport("http://localhost:3000/api/", transport.clone())
            .unwrap()
            .bearer_token("secret");

        let response = client
            .list_libraries(Some(PageQuery::new(Some(25), Some(50))))
            .await
            .unwrap();

        assert_eq!(response.page, PageInfo::new(25, 50, 0));
        let request = transport.requests().pop().unwrap();
        assert_eq!(request.method, Method::GET);
        assert_eq!(
            request.url.as_str(),
            "http://localhost:3000/api/libraries?limit=25&offset=50"
        );
        assert_eq!(
            request.headers.get(AUTHORIZATION).unwrap(),
            HeaderValue::from_static("Bearer secret")
        );
    }

    #[tokio::test]
    async fn health_does_not_send_auth_header() {
        let transport = MockTransport::default();
        transport.push_json(StatusCode::OK, json!({"status": "ok", "version": "v1"}));
        let client = TaruClient::with_transport("http://localhost:3000", transport.clone())
            .unwrap()
            .bearer_token("secret");

        let response = client.health().await.unwrap();

        assert_eq!(response.version, "v1");
        let request = transport.requests().pop().unwrap();
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url.as_str(), "http://localhost:3000/health");
        assert!(request.headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn api_error_uses_public_error_envelope() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::UNAUTHORIZED,
            json!({"code": "unauthorized", "message": "authentication required"}),
        );
        let client = TaruClient::with_transport("http://localhost:3000", transport).unwrap();

        let error = client.list_libraries(None).await.unwrap_err();

        match error {
            TaruClientError::Api { status, body } => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(body.code, "unauthorized");
                assert_eq!(body.message, "authentication required");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn mismatched_api_version_is_rejected() {
        let transport = MockTransport::default();
        transport.push_json_with_version(
            StatusCode::OK,
            "v2",
            json!({"status": "ok", "version": "v2"}),
        );
        let client = TaruClient::with_transport("http://localhost:3000", transport).unwrap();

        let error = client.health().await.unwrap_err();

        match error {
            TaruClientError::UnsupportedApiVersion { expected, actual } => {
                assert_eq!(expected, "v1");
                assert_eq!(actual, "v2");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn playback_decision_query_and_session_cancel_paths_are_stable() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "source": {
                    "id": "source 1",
                    "library_id": "library-1",
                    "item_id": "item-1",
                    "locator": "local:///Demo.mp4",
                    "file_name": "Demo.mp4",
                    "size_bytes": 10,
                    "fingerprint": null
                },
                "probe": null,
                "decision": {
                    "mode": "direct_play",
                    "reason": "compatible",
                    "direct_play": {
                        "source_id": "source 1",
                        "content_type": "video/mp4",
                        "supports_range_requests": true
                    },
                    "transcode_plan": null
                }
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "session": {
                    "id": "session-1",
                    "source_id": "source 1",
                    "kind": "remux",
                    "request_key": "remux:source-1",
                    "state": "cancel_requested",
                    "failure_category": null,
                    "failure_message": null,
                    "created_at": "2026-05-17T00:00:00Z",
                    "updated_at": "2026-05-17T00:00:01Z",
                    "started_at": "2026-05-17T00:00:00Z",
                    "completed_at": null
                }
            }),
        );
        let client = TaruClient::with_transport("http://localhost:3000", transport.clone())
            .unwrap()
            .bearer_token("secret");

        let decision = client
            .get_playback_decision(
                "source 1",
                Some(PlaybackCapabilitiesQuery {
                    direct_play: Some(true),
                    container: Some("mp4,webm"),
                    video_codec: Some("h264"),
                    audio_codec: None,
                }),
            )
            .await
            .unwrap();
        let session = client.cancel_playback_session("session-1").await.unwrap();

        assert_eq!(decision.source.id, "source 1");
        assert_eq!(
            session.session.state,
            ClientTranscodeSessionState::CancelRequested
        );
        let requests = transport.requests();
        assert_eq!(
            requests[0].url.as_str(),
            "http://localhost:3000/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264"
        );
        assert_eq!(
            requests[1].url.as_str(),
            "http://localhost:3000/playback/sessions/session-1/cancel"
        );
        assert_eq!(requests[1].method, Method::POST);
    }

    #[test]
    fn sdk_inventory_covers_foundation_public_routes_without_streaming_methods() {
        for expected in [
            "/health",
            "/libraries",
            "/libraries/{library_id}",
            "/libraries/{library_id}/sources",
            "/items",
            "/items/{item_id}",
            "/items/{item_id}/credits",
            "/items/{item_id}/images",
            "/people",
            "/people/{person_id}",
            "/people/{person_id}/items",
            "/tags",
            "/tags/{tag_id}/items",
            "/genres",
            "/genres/{genre_id}/items",
            "/search",
            "/sources/{source_id}/probe",
            "/sources/{source_id}/playback/decision",
            "/playback/sessions/{session_id}",
            "/playback/sessions/{session_id}/cancel",
        ] {
            assert!(
                PUBLIC_CLIENT_PATHS.contains(&expected),
                "missing SDK route inventory entry {expected}"
            );
        }

        for deferred in [
            "/sources/{source_id}/stream",
            "/sources/{source_id}/stream/remux",
            "/sources/{source_id}/stream/hls/playlist.m3u8",
            "/playback/sessions/{session_id}/hls/segments/{segment_name}",
        ] {
            assert!(
                !PUBLIC_CLIENT_PATHS.contains(&deferred),
                "streaming route should stay deferred in M35: {deferred}"
            );
        }
    }

    #[test]
    fn sdk_inventory_rejects_admin_internal_and_secret_surfaces() {
        let joined = PUBLIC_CLIENT_PATHS.join("\n").to_ascii_lowercase();

        for forbidden in [
            "/addons",
            "/webhooks",
            "/automation",
            "/storage/backends",
            "/jobs",
            "secret_env",
            "output_path",
            "providerrawresponse",
            "taru_core",
            "taru-server",
            "taru_api",
        ] {
            assert!(
                !joined.contains(forbidden),
                "SDK leaked forbidden term: {forbidden}"
            );
        }
    }
}
