use std::sync::Arc;

use async_trait::async_trait;
use nako_client_core::{
    CoreHttpHeader, CoreHttpRequest, CoreHttpRequestSpec, CoreHttpResponse, CoreQueryParam,
    CoreRuntimeFailure, CoreRuntimeFailureKind,
};
pub use nako_client_protocol::{
    API_VERSION_HEADER, BrowserPlaybackCapabilitiesDto, BrowserPlaybackMode,
    BrowserPlaybackOutputContainer, BrowserPlaybackTicketRequest, BrowserPlaybackTicketResponse,
    BrowserPlaybackUrlDto, BrowserPlaybackUrlKind, CLIENT_PROTOCOL_VERSION as API_VERSION,
    ClientOutputContainer, ContinueWatchingResponse, CurrentUserResponse, ErrorResponse,
    GenreItemsResponse, GenreListResponse, HealthResponse, ImagesResponse, ItemCreditsResponse,
    ItemDetailResponse, ItemsResponse, LibraryListResponse, LibraryResponse,
    LibrarySourcesResponse, LoginRequest, LoginResponse, LogoutResponse,
    PLAYBACK_SESSION_ID_HEADER, PageInfo, PeopleResponse, PersonItemsResponse, PersonResponse,
    PlaybackDecisionResponse, PublicClientRustSdkExposure, SearchResponse, SetWatchedStateRequest,
    SourceProbeResponse, TagItemsResponse, TagsResponse, TranscodeSessionResponse,
    UpdatePlaybackProgressRequest, UserPlaybackStateResponse, public_client_json_routes,
    public_client_paths, public_client_streaming_routes,
};
use reqwest::{
    Method, StatusCode, Url,
    header::{HeaderMap, HeaderName, HeaderValue, RANGE},
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Clone)]
pub struct NakoClient {
    base_url: Url,
    bearer_token: Option<String>,
    transport: Arc<dyn ClientTransport>,
}

impl NakoClient {
    /// Build a client with the default reqwest transport.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not an absolute URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, NakoClientError> {
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
    ) -> Result<Self, NakoClientError> {
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
    ) -> Result<Self, NakoClientError> {
        let mut base_url =
            Url::parse(base_url.as_ref()).map_err(|source| NakoClientError::InvalidBaseUrl {
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
    pub async fn health(&self) -> Result<HealthResponse, NakoClientError> {
        self.request_json_no_query(Method::GET, "/health", false)
            .await
    }

    /// Create a local user session.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, encode, or decode errors.
    pub async fn login(&self, request: &LoginRequest) -> Result<LoginResponse, NakoClientError> {
        self.request_json_body(Method::POST, "/auth/login", request, false)
            .await
    }

    /// Revoke the current local user session.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn logout(&self) -> Result<LogoutResponse, NakoClientError> {
        self.request_json_no_query(Method::POST, "/auth/logout", true)
            .await
    }

    /// Get the current user account.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn current_user(&self) -> Result<CurrentUserResponse, NakoClientError> {
        self.request_json_no_query(Method::GET, "/users/me", true)
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
    ) -> Result<LibraryListResponse, NakoClientError> {
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
    ) -> Result<LibraryResponse, NakoClientError> {
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
    ) -> Result<LibrarySourcesResponse, NakoClientError> {
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
    ) -> Result<ItemsResponse, NakoClientError> {
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
    ) -> Result<ItemDetailResponse, NakoClientError> {
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
    ) -> Result<ItemCreditsResponse, NakoClientError> {
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
    ) -> Result<ImagesResponse, NakoClientError> {
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
    ) -> Result<PeopleResponse, NakoClientError> {
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
    ) -> Result<PersonResponse, NakoClientError> {
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
    ) -> Result<PersonItemsResponse, NakoClientError> {
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
    ) -> Result<TagsResponse, NakoClientError> {
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
    ) -> Result<TagItemsResponse, NakoClientError> {
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
    ) -> Result<GenreListResponse, NakoClientError> {
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
    ) -> Result<GenreItemsResponse, NakoClientError> {
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
    ) -> Result<SearchResponse, NakoClientError> {
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
    ) -> Result<SourceProbeResponse, NakoClientError> {
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
    ) -> Result<PlaybackDecisionResponse, NakoClientError> {
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

    /// Issue a browser playback ticket for one source.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, encode, or decode errors.
    pub async fn create_browser_playback_ticket(
        &self,
        source_id: impl AsRef<str>,
        request: &BrowserPlaybackTicketRequest,
    ) -> Result<BrowserPlaybackTicketResponse, NakoClientError> {
        self.request_json_body(
            Method::POST,
            &format!(
                "/sources/{}/playback/browser-ticket",
                encode_path_segment(source_id.as_ref())
            ),
            request,
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
    ) -> Result<TranscodeSessionResponse, NakoClientError> {
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
    ) -> Result<TranscodeSessionResponse, NakoClientError> {
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

    /// Get this principal's playback state for one media item.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn get_user_playback_state(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<UserPlaybackStateResponse, NakoClientError> {
        self.request_json_no_query(
            Method::GET,
            &format!(
                "/users/me/playback-state/items/{}",
                encode_path_segment(item_id.as_ref())
            ),
            true,
        )
        .await
    }

    /// List this principal's continue-watching items.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn list_continue_watching(
        &self,
        page: Option<PageQuery>,
    ) -> Result<ContinueWatchingResponse, NakoClientError> {
        self.request_json(
            Method::GET,
            "/users/me/playback-state/continue-watching",
            page.as_ref(),
            true,
        )
        .await
    }

    /// Update this principal's playback progress for one media item.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn update_user_playback_progress(
        &self,
        item_id: impl AsRef<str>,
        request: &UpdatePlaybackProgressRequest,
    ) -> Result<UserPlaybackStateResponse, NakoClientError> {
        self.request_json_body(
            Method::PUT,
            &format!(
                "/users/me/playback-state/items/{}/progress",
                encode_path_segment(item_id.as_ref())
            ),
            request,
            true,
        )
        .await
    }

    /// Set this principal's watched state for one media item.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, version, or decode errors.
    pub async fn set_user_watched_state(
        &self,
        item_id: impl AsRef<str>,
        request: &SetWatchedStateRequest,
    ) -> Result<UserPlaybackStateResponse, NakoClientError> {
        self.request_json_body(
            Method::PUT,
            &format!(
                "/users/me/playback-state/items/{}/watched",
                encode_path_segment(item_id.as_ref())
            ),
            request,
            true,
        )
        .await
    }

    /// Build a direct-play byte stream request for one source.
    ///
    /// This only constructs the request. It does not execute or own the
    /// response body.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn stream_source_request(
        &self,
        source_id: impl AsRef<str>,
        range: Option<&str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::GET,
            &format!(
                "/sources/{}/stream",
                encode_path_segment(source_id.as_ref())
            ),
            Option::<&NoQuery>::None,
            range,
        )
    }

    /// Build a direct-play stream header preflight request for one source.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn head_stream_source_request(
        &self,
        source_id: impl AsRef<str>,
        range: Option<&str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::HEAD,
            &format!(
                "/sources/{}/stream",
                encode_path_segment(source_id.as_ref())
            ),
            Option::<&NoQuery>::None,
            range,
        )
    }

    /// Build a selected artwork image byte request.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn image_request(
        &self,
        image_id: impl AsRef<str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.image_variant_request(image_id, None)
    }

    /// Build a selected artwork image byte request for an optional bounded variant.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn image_variant_request(
        &self,
        image_id: impl AsRef<str>,
        variant: Option<ImageVariantQuery>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::GET,
            &format!("/images/{}", encode_path_segment(image_id.as_ref())),
            variant.as_ref(),
            None,
        )
    }

    /// Build a selected artwork image header preflight request.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn head_image_request(
        &self,
        image_id: impl AsRef<str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.head_image_variant_request(image_id, None)
    }

    /// Build a selected artwork image variant header preflight request.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn head_image_variant_request(
        &self,
        image_id: impl AsRef<str>,
        variant: Option<ImageVariantQuery>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::HEAD,
            &format!("/images/{}", encode_path_segment(image_id.as_ref())),
            variant.as_ref(),
            None,
        )
    }

    /// Build a remux byte stream request for one source.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn remux_stream_source_request(
        &self,
        source_id: impl AsRef<str>,
        query: Option<RemuxPlaybackQuery<'_>>,
        range: Option<&str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::GET,
            &format!(
                "/sources/{}/stream/remux",
                encode_path_segment(source_id.as_ref())
            ),
            query.as_ref(),
            range,
        )
    }

    /// Build a remux stream header preflight request for one source.
    ///
    /// The response exposes the public playback session id header without
    /// transferring stream bytes.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn head_remux_stream_source_request(
        &self,
        source_id: impl AsRef<str>,
        query: Option<RemuxPlaybackQuery<'_>>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_streaming_request(
            Method::HEAD,
            &format!(
                "/sources/{}/stream/remux",
                encode_path_segment(source_id.as_ref())
            ),
            query.as_ref(),
            None,
        )
    }

    /// Build an HLS playlist request for one source.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn hls_playlist_request(
        &self,
        source_id: impl AsRef<str>,
        capabilities: Option<PlaybackCapabilitiesQuery<'_>>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_request(
            Method::GET,
            &format!(
                "/sources/{}/stream/hls/playlist.m3u8",
                encode_path_segment(source_id.as_ref())
            ),
            capabilities.as_ref(),
            true,
        )
    }

    /// Build an HLS segment request for one playback session.
    ///
    /// # Errors
    ///
    /// Returns URL or header construction errors.
    pub fn hls_segment_request(
        &self,
        session_id: impl AsRef<str>,
        segment_name: impl AsRef<str>,
    ) -> Result<ClientRequest, NakoClientError> {
        self.build_request(
            Method::GET,
            &format!(
                "/playback/sessions/{}/hls/segments/{}",
                encode_path_segment(session_id.as_ref()),
                encode_path_segment(segment_name.as_ref())
            ),
            Option::<&NoQuery>::None,
            true,
        )
    }

    async fn request_json_no_query<T>(
        &self,
        method: Method,
        path: &str,
        auth: bool,
    ) -> Result<T, NakoClientError>
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
    ) -> Result<T, NakoClientError>
    where
        T: DeserializeOwned,
        Q: QueryParams + ?Sized,
    {
        let request = self.build_request(method, path, query, auth)?;
        let response = self.transport.send(request).await?;
        self.ensure_response_policy(&response)?;
        serde_json::from_slice(&response.body).map_err(|source| NakoClientError::Decode {
            path: path.to_owned(),
            source,
        })
    }

    async fn request_json_body<T, B>(
        &self,
        method: Method,
        path: &str,
        body: &B,
        auth: bool,
    ) -> Result<T, NakoClientError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let mut request = self.build_request(method, path, Option::<&NoQuery>::None, auth)?;
        request.headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        request.body = serde_json::to_vec(body).map_err(|source| NakoClientError::Encode {
            path: path.to_owned(),
            source,
        })?;
        let response = self.transport.send(request).await?;
        self.ensure_response_policy(&response)?;
        serde_json::from_slice(&response.body).map_err(|source| NakoClientError::Decode {
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
    ) -> Result<ClientRequest, NakoClientError>
    where
        Q: QueryParams + ?Sized,
    {
        let mut query_pairs = Vec::new();
        if let Some(query) = query {
            query.append_query(&mut query_pairs);
        }
        let spec = CoreHttpRequestSpec::new(path, self.base_url.as_str(), method.as_str(), path)
            .query(
                query_pairs
                    .into_iter()
                    .map(|(name, value)| CoreQueryParam::new(name, value))
                    .collect(),
            )
            .access_token(if auth {
                self.bearer_token.clone()
            } else {
                None
            });
        core_request_to_client_request(nako_client_core::build_core_request(&spec), path)
    }

    fn build_streaming_request<Q>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        range: Option<&str>,
    ) -> Result<ClientRequest, NakoClientError>
    where
        Q: QueryParams + ?Sized,
    {
        let mut request = self.build_request(method, path, query, true)?;
        if let Some(range) = range {
            let value =
                HeaderValue::from_str(range).map_err(|source| NakoClientError::InvalidHeader {
                    name: RANGE,
                    source,
                })?;
            request.headers.insert(RANGE, value);
        }
        Ok(request)
    }

    fn ensure_response_policy(&self, response: &ClientResponse) -> Result<(), NakoClientError> {
        match nako_client_core::interpret_core_response(
            &core_response_from_client_response(response)?,
            None,
            &[self.bearer_token.as_deref().unwrap_or_default()],
        ) {
            Ok(()) => Ok(()),
            Err(failure) => Err(nako_error_from_core_failure(response.status, failure)),
        }
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImageVariantQuery {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl QueryParams for ImageVariantQuery {
    fn append_query(&self, pairs: &mut Vec<(String, String)>) {
        if let Some(width) = self.width {
            pairs.push(("width".to_owned(), width.to_string()));
        }
        if let Some(height) = self.height {
            pairs.push(("height".to_owned(), height.to_string()));
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RemuxPlaybackQuery<'a> {
    pub capabilities: PlaybackCapabilitiesQuery<'a>,
    pub output_container: Option<ClientOutputContainer>,
}

impl QueryParams for RemuxPlaybackQuery<'_> {
    fn append_query(&self, pairs: &mut Vec<(String, String)>) {
        self.capabilities.append_query(pairs);
        if let Some(output_container) = &self.output_container {
            pairs.push((
                "output_container".to_owned(),
                output_container.wire_value().to_owned(),
            ));
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
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ClientResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

fn core_request_to_client_request(
    request: CoreHttpRequest,
    path: &str,
) -> Result<ClientRequest, NakoClientError> {
    let method = Method::from_bytes(request.method.as_bytes()).map_err(|source| {
        NakoClientError::InvalidPath {
            path: path.to_owned(),
            reason: source.to_string(),
        }
    })?;
    let url = Url::parse(&request.url).map_err(|source| NakoClientError::InvalidPath {
        path: path.to_owned(),
        reason: source.to_string(),
    })?;
    let mut headers = HeaderMap::new();
    for header in request.headers {
        let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|source| {
            NakoClientError::InvalidPath {
                path: path.to_owned(),
                reason: source.to_string(),
            }
        })?;
        let value = HeaderValue::from_str(&header.value).map_err(|source| {
            NakoClientError::InvalidHeader {
                name: name.clone(),
                source,
            }
        })?;
        headers.insert(name, value);
    }
    Ok(ClientRequest {
        method,
        url,
        headers,
        body: request.body_utf8.unwrap_or_default().into_bytes(),
    })
}

fn core_response_from_client_response(
    response: &ClientResponse,
) -> Result<CoreHttpResponse, NakoClientError> {
    let mut headers = Vec::new();
    for (name, value) in &response.headers {
        let value = value.to_str().map_err(|source| {
            if name.as_str().eq_ignore_ascii_case(API_VERSION_HEADER) {
                NakoClientError::InvalidVersionHeader { source }
            } else {
                NakoClientError::InvalidPath {
                    path: String::new(),
                    reason: source.to_string(),
                }
            }
        })?;
        headers.push(CoreHttpHeader::new(name.as_str(), value));
    }
    Ok(CoreHttpResponse {
        request_id: String::new(),
        status_code: i32::from(response.status.as_u16()),
        headers,
        body_utf8: String::from_utf8_lossy(&response.body).into_owned(),
    })
}

fn nako_error_from_core_failure(
    status: StatusCode,
    failure: CoreRuntimeFailure,
) -> NakoClientError {
    match failure.kind {
        CoreRuntimeFailureKind::HttpError => NakoClientError::Api {
            status,
            body: failure
                .public_error
                .map(|error| ErrorResponse {
                    code: error.code,
                    message: error.message,
                })
                .unwrap_or_else(|| ErrorResponse {
                    code: "invalid_input".to_owned(),
                    message: status.canonical_reason().unwrap_or("HTTP error").to_owned(),
                }),
        },
        CoreRuntimeFailureKind::UnsupportedApiVersion => NakoClientError::UnsupportedApiVersion {
            expected: API_VERSION,
            actual: failure.observed_api_version.unwrap_or_default(),
        },
        CoreRuntimeFailureKind::InvalidResponse => NakoClientError::InvalidCoreResponse,
        CoreRuntimeFailureKind::MissingAccessToken => NakoClientError::MissingAccessToken,
    }
}

#[async_trait]
pub trait ClientTransport: Send + Sync {
    async fn send(&self, request: ClientRequest) -> Result<ClientResponse, NakoClientError>;
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
    async fn send(&self, request: ClientRequest) -> Result<ClientResponse, NakoClientError> {
        let mut builder = self
            .client
            .request(request.method, request.url)
            .headers(request.headers);
        if !request.body.is_empty() {
            builder = builder.body(request.body);
        }
        let response = builder.send().await.map_err(NakoClientError::Transport)?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .map_err(NakoClientError::Transport)?
            .to_vec();

        Ok(ClientResponse {
            status,
            headers,
            body,
        })
    }
}

#[derive(Debug, Error)]
pub enum NakoClientError {
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
    #[error("failed to encode request body for {path}")]
    Encode {
        path: String,
        source: serde_json::Error,
    },
    #[error("Nako API returned {status}: {code}", code = body.code)]
    Api {
        status: StatusCode,
        body: ErrorResponse,
    },
    #[error("invalid API version header")]
    InvalidVersionHeader { source: reqwest::header::ToStrError },
    #[error("unsupported Nako API version {actual}, expected {expected}")]
    UnsupportedApiVersion {
        expected: &'static str,
        actual: String,
    },
    #[error("failed to decode response from {path}")]
    Decode {
        path: String,
        source: serde_json::Error,
    },
    #[error("missing access token")]
    MissingAccessToken,
    #[error("invalid response")]
    InvalidCoreResponse,
}

#[must_use]
fn encode_path_segment(value: &str) -> String {
    nako_client_core::encode_path_segment(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_client_protocol::{
        ClientTranscodeSessionState, PUBLIC_CLIENT_ROUTES, PublicClientHttpMethod,
    };
    use reqwest::header::{AUTHORIZATION, HeaderValue};
    use serde_json::json;
    use std::{collections::VecDeque, sync::Mutex};

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
        async fn send(&self, request: ClientRequest) -> Result<ClientResponse, NakoClientError> {
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
        let client = NakoClient::with_transport("http://localhost:3000/api/", transport.clone())
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
        let client = NakoClient::with_transport("http://localhost:3000", transport.clone())
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
    async fn account_login_skips_auth_and_me_logout_use_bearer() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "session": {
                    "token": "nako_sess_returned",
                    "expires_at_ms": 1234
                },
                "account": {
                    "user": {
                        "id": "018f0000-0000-7000-8000-000000000001",
                        "username": "viewer",
                        "display_name": "Viewer",
                        "roles": ["viewer"],
                        "bootstrap": false
                    }
                }
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "user": {
                    "id": "018f0000-0000-7000-8000-000000000001",
                    "username": "viewer",
                    "display_name": "Viewer",
                    "roles": ["viewer"],
                    "bootstrap": false
                }
            }),
        );
        transport.push_json(StatusCode::OK, json!({"revoked": true}));
        let client = NakoClient::with_transport("http://localhost:3000", transport.clone())
            .unwrap()
            .bearer_token("session-token");

        let login = client
            .login(&LoginRequest {
                username: "viewer".to_owned(),
                password: "password".to_owned(),
            })
            .await
            .unwrap();
        let me = client.current_user().await.unwrap();
        let logout = client.logout().await.unwrap();

        assert_eq!(login.session.token, "nako_sess_returned");
        assert_eq!(me.user.username, "viewer");
        assert!(logout.revoked);
        let requests = transport.requests();
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].url.as_str(), "http://localhost:3000/auth/login");
        assert!(requests[0].headers.get(AUTHORIZATION).is_none());
        assert!(String::from_utf8_lossy(&requests[0].body).contains("\"password\""));
        assert_eq!(requests[1].method, Method::GET);
        assert_eq!(requests[1].url.as_str(), "http://localhost:3000/users/me");
        assert_eq!(
            requests[1].headers.get(AUTHORIZATION).unwrap(),
            HeaderValue::from_static("Bearer session-token")
        );
        assert_eq!(requests[2].method, Method::POST);
        assert_eq!(
            requests[2].url.as_str(),
            "http://localhost:3000/auth/logout"
        );
        assert_eq!(
            requests[2].headers.get(AUTHORIZATION).unwrap(),
            HeaderValue::from_static("Bearer session-token")
        );
    }

    #[tokio::test]
    async fn api_error_uses_public_error_envelope() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::UNAUTHORIZED,
            json!({"code": "unauthorized", "message": "authentication required"}),
        );
        let client = NakoClient::with_transport("http://localhost:3000", transport).unwrap();

        let error = client.list_libraries(None).await.unwrap_err();

        match error {
            NakoClientError::Api { status, body } => {
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
        let client = NakoClient::with_transport("http://localhost:3000", transport).unwrap();

        let error = client.health().await.unwrap_err();

        match error {
            NakoClientError::UnsupportedApiVersion { expected, actual } => {
                assert_eq!(expected, "v1");
                assert_eq!(actual, "v2");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn playback_decision_ticket_and_session_cancel_paths_are_stable() {
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
                "target": {
                    "kind": "browser",
                    "network_scope": "local",
                    "transport_auth": "browser_ticket",
                    "media_capabilities": {
                        "direct_play": true,
                        "containers": ["mp4", "webm"],
                        "video_codecs": ["h264"],
                        "audio_codecs": ["aac"]
                    },
                    "control_capabilities": {
                        "commands": ["play", "pause", "seek", "stop"]
                    }
                },
                "decision": {
                    "mode": "direct_play",
                    "reason": "compatible",
                    "report": {
                        "selected_mode": "direct_play",
                        "direct_play": {
                            "supported": true,
                            "reasons": ["compatible"]
                        },
                        "remux": {
                            "supported": false,
                            "reasons": ["client_container_unsupported"]
                        },
                        "transcode": {
                            "supported": false,
                            "reasons": ["requested_transcode_output"]
                        },
                        "denial": null
                    },
                    "denial": null,
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
                "source_id": "source 1",
                "item_id": "item-1",
                "mode": "hls",
                "expires_at": "2026-05-26T12:00:00Z",
                "urls": [{
                    "kind": "playlist",
                    "url": "/sources/source%201/stream/hls/playlist.m3u8?ticket=opaque",
                    "content_type": "application/vnd.apple.mpegurl",
                    "supports_range_requests": false
                }]
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "session": {
                    "id": "session-1",
                    "source_id": "source 1",
                    "kind": "remux",
                    "request_key": "test-transcode-profile:remux-source-1",
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
        let client = NakoClient::with_transport("http://localhost:3000", transport.clone())
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
        let ticket = client
            .create_browser_playback_ticket(
                "source 1",
                &BrowserPlaybackTicketRequest {
                    mode: BrowserPlaybackMode::Hls,
                    capabilities: Some(BrowserPlaybackCapabilitiesDto {
                        direct_play: Some(true),
                        container: Some(vec!["mp4".to_owned(), "webm".to_owned()]),
                        video_codec: Some(vec!["h264".to_owned()]),
                        audio_codec: Some(vec!["aac".to_owned()]),
                        output_container: Some(BrowserPlaybackOutputContainer::Mp4),
                    }),
                },
            )
            .await
            .unwrap();
        let session = client.cancel_playback_session("session-1").await.unwrap();

        assert_eq!(decision.source.id, "source 1");
        assert_eq!(ticket.mode, BrowserPlaybackMode::Hls);
        assert_eq!(ticket.urls[0].kind, BrowserPlaybackUrlKind::Playlist);
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
            "http://localhost:3000/sources/source%201/playback/browser-ticket"
        );
        assert_eq!(requests[1].method, Method::POST);
        assert_eq!(
            requests[1]
                .headers
                .get(reqwest::header::CONTENT_TYPE)
                .unwrap(),
            HeaderValue::from_static("application/json")
        );
        let ticket_body = serde_json::from_slice::<serde_json::Value>(&requests[1].body).unwrap();
        assert_eq!(ticket_body["mode"], "hls");
        assert_eq!(ticket_body["capabilities"]["container"][0], "mp4");
        assert_eq!(ticket_body["capabilities"]["output_container"], "mp4");
        assert_eq!(
            requests[2].url.as_str(),
            "http://localhost:3000/playback/sessions/session-1/cancel"
        );
        assert_eq!(requests[2].method, Method::POST);
    }

    #[tokio::test]
    async fn user_playback_methods_use_me_routes_json_bodies_and_pagination() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "state": {
                    "item_id": "item 1",
                    "source_id": "source 1",
                    "resume_position_ms": 120000,
                    "duration_ms": 600000,
                    "progress_percent": 0.2,
                    "watched": false,
                    "watched_at": null,
                    "last_played_at": "2026-05-19T00:00:00Z",
                    "updated_at": "2026-05-19T00:00:00Z",
                    "version": 1
                }
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "items": [],
                "page": {"limit": 10, "offset": 20, "returned": 0}
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "state": {
                    "item_id": "item 1",
                    "source_id": "source 1",
                    "resume_position_ms": 120000,
                    "duration_ms": 600000,
                    "progress_percent": 0.2,
                    "watched": false,
                    "watched_at": null,
                    "last_played_at": "2026-05-19T00:00:00Z",
                    "updated_at": "2026-05-19T00:00:00Z",
                    "version": 1
                }
            }),
        );
        transport.push_json(
            StatusCode::OK,
            json!({
                "state": {
                    "item_id": "item 1",
                    "source_id": "source 1",
                    "resume_position_ms": null,
                    "duration_ms": 600000,
                    "progress_percent": null,
                    "watched": true,
                    "watched_at": "2026-05-19T00:01:00Z",
                    "last_played_at": "2026-05-19T00:01:00Z",
                    "updated_at": "2026-05-19T00:01:00Z",
                    "version": 2
                }
            }),
        );
        let client = NakoClient::with_transport("http://localhost:3000", transport.clone())
            .unwrap()
            .bearer_token("secret");

        let state = client.get_user_playback_state("item 1").await.unwrap();
        let list = client
            .list_continue_watching(Some(PageQuery::new(Some(10), Some(20))))
            .await
            .unwrap();
        let progress = client
            .update_user_playback_progress(
                "item 1",
                &UpdatePlaybackProgressRequest {
                    source_id: Some("source 1".to_owned()),
                    position_ms: 120_000,
                    duration_ms: Some(600_000),
                    reported_at: Some("2026-05-19T00:00:00Z".to_owned()),
                },
            )
            .await
            .unwrap();
        let watched = client
            .set_user_watched_state(
                "item 1",
                &SetWatchedStateRequest {
                    watched: true,
                    source_id: Some("source 1".to_owned()),
                    position_ms: Some(600_000),
                    duration_ms: Some(600_000),
                    marked_at: Some("2026-05-19T00:01:00Z".to_owned()),
                },
            )
            .await
            .unwrap();

        assert_eq!(state.state.progress_percent, Some(0.2));
        assert_eq!(list.page, PageInfo::new(10, 20, 0));
        assert_eq!(progress.state.resume_position_ms, Some(120_000));
        assert!(watched.state.watched);
        let requests = transport.requests();
        assert_eq!(
            requests[0].url.as_str(),
            "http://localhost:3000/users/me/playback-state/items/item%201"
        );
        assert_eq!(
            requests[1].url.as_str(),
            "http://localhost:3000/users/me/playback-state/continue-watching?limit=10&offset=20"
        );
        assert_eq!(
            requests[2].url.as_str(),
            "http://localhost:3000/users/me/playback-state/items/item%201/progress"
        );
        assert_eq!(
            requests[3].url.as_str(),
            "http://localhost:3000/users/me/playback-state/items/item%201/watched"
        );
        assert_eq!(requests[2].method, Method::PUT);
        assert_eq!(requests[3].method, Method::PUT);
        assert_eq!(
            requests[2]
                .headers
                .get(reqwest::header::CONTENT_TYPE)
                .unwrap(),
            HeaderValue::from_static("application/json")
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&requests[2].body).unwrap()["position_ms"],
            120_000
        );
    }

    #[test]
    fn streaming_request_builders_use_stable_paths_methods_headers_and_queries() {
        let client =
            NakoClient::with_transport("http://localhost:3000/api", MockTransport::default())
                .unwrap()
                .bearer_token("secret");

        let direct = client
            .stream_source_request("source 1", Some("bytes=10-20"))
            .unwrap();
        let head = client.head_stream_source_request("source 1", None).unwrap();
        let image = client.image_request("image 1").unwrap();
        let image_head = client.head_image_request("image 1").unwrap();
        let image_variant = client
            .image_variant_request(
                "image 1",
                Some(ImageVariantQuery {
                    width: Some(300),
                    height: Some(450),
                }),
            )
            .unwrap();
        let image_variant_head = client
            .head_image_variant_request(
                "image 1",
                Some(ImageVariantQuery {
                    width: Some(300),
                    height: None,
                }),
            )
            .unwrap();
        let remux = client
            .remux_stream_source_request(
                "source 1",
                Some(RemuxPlaybackQuery {
                    capabilities: PlaybackCapabilitiesQuery {
                        direct_play: Some(false),
                        container: Some("mp4,mkv"),
                        video_codec: Some("h264"),
                        audio_codec: Some("aac"),
                    },
                    output_container: Some(ClientOutputContainer::Mkv),
                }),
                Some("bytes=0-"),
            )
            .unwrap();
        let remux_head = client
            .head_remux_stream_source_request(
                "source 1",
                Some(RemuxPlaybackQuery {
                    capabilities: PlaybackCapabilitiesQuery {
                        direct_play: Some(false),
                        container: Some("mp4,mkv"),
                        video_codec: Some("h264"),
                        audio_codec: Some("aac"),
                    },
                    output_container: Some(ClientOutputContainer::Mkv),
                }),
            )
            .unwrap();
        let playlist = client
            .hls_playlist_request(
                "source 1",
                Some(PlaybackCapabilitiesQuery {
                    direct_play: None,
                    container: Some("hls"),
                    video_codec: Some("h264"),
                    audio_codec: None,
                }),
            )
            .unwrap();
        let segment = client
            .hls_segment_request("session 1", "seg 001.ts")
            .unwrap();

        assert_eq!(direct.method, Method::GET);
        assert_eq!(head.method, Method::HEAD);
        assert_eq!(image.method, Method::GET);
        assert_eq!(image_head.method, Method::HEAD);
        assert_eq!(image_variant.method, Method::GET);
        assert_eq!(image_variant_head.method, Method::HEAD);
        assert_eq!(remux.method, Method::GET);
        assert_eq!(remux_head.method, Method::HEAD);
        assert_eq!(playlist.method, Method::GET);
        assert_eq!(segment.method, Method::GET);
        assert_eq!(
            direct.url.as_str(),
            "http://localhost:3000/api/sources/source%201/stream"
        );
        assert_eq!(
            head.url.as_str(),
            "http://localhost:3000/api/sources/source%201/stream"
        );
        assert_eq!(
            image.url.as_str(),
            "http://localhost:3000/api/images/image%201"
        );
        assert_eq!(
            image_head.url.as_str(),
            "http://localhost:3000/api/images/image%201"
        );
        assert_eq!(
            image_variant.url.as_str(),
            "http://localhost:3000/api/images/image%201?width=300&height=450"
        );
        assert_eq!(
            image_variant_head.url.as_str(),
            "http://localhost:3000/api/images/image%201?width=300"
        );
        assert_eq!(
            remux.url.as_str(),
            "http://localhost:3000/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv"
        );
        assert_eq!(
            remux_head.url.as_str(),
            "http://localhost:3000/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv"
        );
        assert_eq!(
            playlist.url.as_str(),
            "http://localhost:3000/api/sources/source%201/stream/hls/playlist.m3u8?container=hls&video_codec=h264"
        );
        assert_eq!(
            segment.url.as_str(),
            "http://localhost:3000/api/playback/sessions/session%201/hls/segments/seg%20001.ts"
        );

        for request in [
            &direct,
            &head,
            &image,
            &image_head,
            &image_variant,
            &image_variant_head,
            &remux,
            &remux_head,
            &playlist,
            &segment,
        ] {
            assert_eq!(
                request.headers.get(AUTHORIZATION).unwrap(),
                HeaderValue::from_static("Bearer secret")
            );
        }
        assert_eq!(
            direct.headers.get(RANGE).unwrap(),
            HeaderValue::from_static("bytes=10-20")
        );
        assert!(head.headers.get(RANGE).is_none());
        assert_eq!(
            remux.headers.get(RANGE).unwrap(),
            HeaderValue::from_static("bytes=0-")
        );
        assert!(remux_head.headers.get(RANGE).is_none());
        assert_eq!(PLAYBACK_SESSION_ID_HEADER, "x-nako-playback-session-id");
    }

    #[test]
    fn sdk_inventory_uses_shared_protocol_routes_and_exposure() {
        let all_paths = public_client_paths().collect::<Vec<_>>();
        let json_paths = public_client_json_routes()
            .map(|route| route.path)
            .collect::<Vec<_>>();
        let streaming_paths = public_client_streaming_routes()
            .map(|route| route.path)
            .collect::<Vec<_>>();

        assert_eq!(all_paths.len(), PUBLIC_CLIENT_ROUTES.len());
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
            "/sources/{source_id}/playback/browser-ticket",
            "/playback/sessions/{session_id}",
            "/playback/sessions/{session_id}/cancel",
        ] {
            assert!(
                json_paths.contains(&expected),
                "missing JSON SDK route inventory entry {expected}"
            );
        }

        for expected in [
            "/images/{image_id}",
            "/sources/{source_id}/stream",
            "/sources/{source_id}/stream/remux",
            "/sources/{source_id}/stream/hls/playlist.m3u8",
            "/playback/sessions/{session_id}/hls/segments/{segment_name}",
        ] {
            assert!(
                streaming_paths.contains(&expected),
                "missing streaming builder inventory entry {expected}"
            );
        }

        let direct_stream = PUBLIC_CLIENT_ROUTES
            .iter()
            .find(|route| route.path == "/sources/{source_id}/stream")
            .unwrap();
        assert_eq!(
            direct_stream.rust_sdk_exposure,
            PublicClientRustSdkExposure::StreamingBuilder
        );
        assert_eq!(
            direct_stream.methods,
            &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Head]
        );

        let remux_stream = PUBLIC_CLIENT_ROUTES
            .iter()
            .find(|route| route.path == "/sources/{source_id}/stream/remux")
            .unwrap();
        assert_eq!(
            remux_stream.rust_sdk_exposure,
            PublicClientRustSdkExposure::StreamingBuilder
        );
        assert_eq!(
            remux_stream.methods,
            &[PublicClientHttpMethod::Get, PublicClientHttpMethod::Head]
        );
    }

    #[test]
    fn sdk_inventory_rejects_admin_internal_and_secret_surfaces() {
        let joined = public_client_paths()
            .collect::<Vec<_>>()
            .join("\n")
            .to_ascii_lowercase();

        for forbidden in [
            "/addons",
            "/webhooks",
            "/automation",
            "/storage/backends",
            "/jobs",
            "secret_env",
            "output_path",
            "providerrawresponse",
            "nako_core",
            "nako-server",
            "nako_api",
        ] {
            assert!(
                !joined.contains(forbidden),
                "SDK leaked forbidden term: {forbidden}"
            );
        }
    }
}
