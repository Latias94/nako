mod browse;
mod connection;
mod encoding;
mod ids;
mod playback;
mod redaction;
mod request;
mod response;

pub use browse::{
    CoreBrowseEntityPagedRequestInput, CoreBrowseEntityRequestInput, CoreBrowsePagedRequestInput,
    CorePageQuery, CoreSearchItemsRequestInput, build_get_item_request, build_get_library_request,
    build_get_person_request, build_list_genre_items_request, build_list_genres_request,
    build_list_item_images_request, build_list_items_request, build_list_libraries_request,
    build_list_library_sources_request, build_list_person_items_request,
    build_list_tag_items_request, build_list_tags_request, build_search_items_request,
};
pub use connection::{
    CoreConnectionProbeInput, CoreConnectionProbeOutcome, CoreConnectionProbeOutcomeKind,
    CoreConnectionProbeSuccess, advance_connection_probe, start_connection_probe,
};
pub use encoding::{encode_path_segment, url_on};
pub use ids::{
    CONNECTION_AUTH_PROBE_REQUEST_ID, CONNECTION_HEALTH_REQUEST_ID, PLAYBACK_DECISION_REQUEST_ID,
    PLAYBACK_DIRECT_STREAM_HEAD_REQUEST_ID, PLAYBACK_DIRECT_STREAM_REQUEST_ID,
    PLAYBACK_HLS_PLAYLIST_REQUEST_ID, PLAYBACK_HLS_SEGMENT_REQUEST_ID,
    PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID, PLAYBACK_REMUX_STREAM_REQUEST_ID,
};
pub use playback::{
    CoreDirectPlaybackTargetInput, CoreHlsPlaylistTargetInput, CoreOutputContainer,
    CorePlaybackCapabilities, CorePlaybackDecisionRequestInput, CorePlaybackDecisionSummary,
    CorePlaybackMode, CorePlaybackSegmentInput, CorePlaybackTarget, CorePlaybackTargetInput,
    CoreRemuxPlaybackTargetInput, build_direct_playback_target, build_head_direct_playback_target,
    build_hls_playlist_target, build_hls_segment_request, build_playback_decision_request,
    build_recommended_playback_target, build_remux_playback_target,
};
pub use request::{
    CoreHttpHeader, CoreHttpRequest, CoreHttpRequestSpec, CoreQueryParam, CoreSafeRequestPreview,
    build_core_request,
};
pub use response::{
    CoreHttpResponse, CorePublicError, CoreRuntimeFailure, CoreRuntimeFailureKind,
    interpret_core_response,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input() -> CoreConnectionProbeInput {
        CoreConnectionProbeInput::new("https://taru.example/api/", "secret-token")
    }

    fn api_header() -> CoreHttpHeader {
        CoreHttpHeader::new(
            taru_client_protocol::API_VERSION_HEADER,
            taru_client_protocol::CLIENT_PROTOCOL_VERSION,
        )
    }

    #[test]
    fn connection_probe_starts_with_unauthenticated_health_request() {
        let outcome = start_connection_probe(&input());

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(request.request_id, CONNECTION_HEALTH_REQUEST_ID);
        assert_eq!(request.method, "GET");
        assert_eq!(request.url, "https://taru.example/api/health");
        assert!(request.headers.is_empty());
        assert_eq!(request.safe_preview.url, "https://taru.example/api/health");
    }

    #[test]
    fn generic_core_request_builds_encoded_url_auth_header_and_safe_preview() {
        let request = build_core_request(
            &CoreHttpRequestSpec::new(
                "playback.decision",
                "https://taru.example/api/",
                "GET",
                &format!(
                    "/sources/{}/playback/decision",
                    encode_path_segment("source 1")
                ),
            )
            .query(vec![
                CoreQueryParam::new("direct_play", "true"),
                CoreQueryParam::new("container", "mp4,webm"),
            ])
            .access_token(Some("secret-token".to_owned())),
        );

        assert_eq!(request.request_id, "playback.decision");
        assert_eq!(
            request.url,
            "https://taru.example/api/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm"
        );
        assert_eq!(
            request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn generic_response_interpreter_preserves_public_error_and_version_failures() {
        let request = build_core_request(
            &CoreHttpRequestSpec::new("demo", "https://taru.example", "GET", "/demo")
                .access_token(Some("secret-token".to_owned())),
        )
        .safe_preview;
        let http = CoreHttpResponse::new(
            "demo",
            403,
            vec![api_header()],
            json!({"code": "forbidden", "message": "secret-token cannot access this source"})
                .to_string(),
        );
        let failure =
            interpret_core_response(&http, Some(&request), &["secret-token"]).unwrap_err();

        assert_eq!(failure.kind, CoreRuntimeFailureKind::HttpError);
        assert_eq!(failure.status_code, Some(403));
        assert_eq!(
            failure.public_error,
            Some(CorePublicError {
                code: "forbidden".to_owned(),
                message: "<redacted> cannot access this source".to_owned(),
            })
        );

        let version = CoreHttpResponse::new(
            "demo",
            200,
            vec![CoreHttpHeader::new(
                taru_client_protocol::API_VERSION_HEADER,
                "v2",
            )],
            "{}",
        );
        let failure = interpret_core_response(&version, Some(&request), &[]).unwrap_err();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::UnsupportedApiVersion);
        assert_eq!(failure.observed_api_version.as_deref(), Some("v2"));
    }

    #[test]
    fn playback_decision_request_uses_core_route_query_auth_and_redaction() {
        let request = build_playback_decision_request(&CorePlaybackDecisionRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            source_id: "source 1".to_owned(),
            capabilities: CorePlaybackCapabilities {
                direct_play: Some(true),
                containers: vec!["mp4".to_owned(), "webm".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned(), "opus".to_owned()],
            },
        });

        assert_eq!(request.request_id, PLAYBACK_DECISION_REQUEST_ID);
        assert_eq!(
            request.url,
            "https://taru.example/api/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac%2Copus"
        );
        assert_eq!(
            request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn playback_targets_follow_mode_without_auth_or_media3_policy() {
        let input = CorePlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            decision: CorePlaybackDecisionSummary {
                source_id: "source 1".to_owned(),
                mode: CorePlaybackMode::Remux,
                transcode_output_container: Some(CoreOutputContainer::Mkv),
            },
            capabilities: CorePlaybackCapabilities {
                direct_play: Some(false),
                containers: vec!["mp4".to_owned(), "mkv".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned()],
            },
        };

        let target = build_recommended_playback_target(&input).unwrap();

        assert_eq!(target.request.request_id, PLAYBACK_REMUX_STREAM_REQUEST_ID);
        assert_eq!(
            target.request.url,
            "https://taru.example/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv"
        );
        assert!(target.request.headers.is_empty());
        let preflight = target.session_probe_request.unwrap();
        assert_eq!(
            preflight.request_id,
            PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID
        );
        assert_eq!(preflight.method, "HEAD");
        assert_eq!(preflight.url, target.request.url);

        let explicit_hls_remux = build_remux_playback_target(&CoreRemuxPlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            source_id: "source 1".to_owned(),
            capabilities: CorePlaybackCapabilities::empty(),
            output_container: Some(CoreOutputContainer::Hls),
        });
        assert_eq!(
            explicit_hls_remux.request.url,
            "https://taru.example/api/sources/source%201/stream/remux"
        );

        let explicit_direct = build_direct_playback_target(&CoreDirectPlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            source_id: "source 1".to_owned(),
        });
        assert_eq!(
            explicit_direct.request.url,
            "https://taru.example/api/sources/source%201/stream"
        );

        let unknown = build_recommended_playback_target(&CorePlaybackTargetInput {
            decision: CorePlaybackDecisionSummary {
                mode: CorePlaybackMode::Unknown,
                ..input.decision
            },
            ..input
        });
        assert_eq!(unknown, None);
    }

    #[test]
    fn connection_probe_reports_missing_token_before_auth_probe() {
        let outcome =
            start_connection_probe(&CoreConnectionProbeInput::new("https://taru.example", "  "));

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        assert_eq!(
            outcome.failure.unwrap().kind,
            CoreRuntimeFailureKind::MissingAccessToken
        );
    }

    #[test]
    fn health_success_advances_to_redacted_authenticated_probe_request() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"status": "ok", "version": "v1"}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(request.request_id, CONNECTION_AUTH_PROBE_REQUEST_ID);
        assert_eq!(
            request.url,
            "https://taru.example/api/libraries?limit=1&offset=0"
        );
        assert_eq!(
            request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn health_body_version_mismatch_is_unsupported_api_version() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"status": "ok", "version": "v2"}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        let failure = outcome.failure.unwrap();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::UnsupportedApiVersion);
        assert_eq!(failure.observed_api_version.as_deref(), Some("v2"));
    }

    #[test]
    fn auth_probe_http_error_preserves_public_error_and_redacts_token() {
        let response = CoreHttpResponse::new(
            CONNECTION_AUTH_PROBE_REQUEST_ID,
            401,
            vec![api_header()],
            json!({
                "code": "unauthorized",
                "message": "Bearer secret-token is expired"
            })
            .to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        let failure = outcome.failure.unwrap();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::HttpError);
        assert_eq!(failure.status_code, Some(401));
        assert_eq!(
            failure.public_error,
            Some(CorePublicError {
                code: "unauthorized".to_owned(),
                message: "Bearer <redacted> is expired".to_owned(),
            })
        );
        assert_eq!(
            failure.request.unwrap().headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn auth_probe_success_returns_connection_success_with_safe_previews() {
        let response = CoreHttpResponse::new(
            CONNECTION_AUTH_PROBE_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"libraries": [], "page": {"limit": 1, "offset": 0, "returned": 0}}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Success);
        let success = outcome.success.unwrap();
        assert_eq!(success.api_version, "v1");
        assert_eq!(
            success.health_request.url,
            "https://taru.example/api/health"
        );
        assert_eq!(
            success.auth_probe_request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn invalid_health_json_is_invalid_response() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            "not-json",
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        assert_eq!(
            outcome.failure.unwrap().kind,
            CoreRuntimeFailureKind::InvalidResponse
        );
    }

    #[test]
    fn browse_request_builders_use_stable_paths_pagination_auth_and_redaction() {
        let libraries = build_list_libraries_request(&CoreBrowsePagedRequestInput {
            base_url: "https://taru.example/api/".to_owned(),
            access_token: "secret-token".to_owned(),
            page: Some(CorePageQuery::new(Some(25), Some(50))),
        });
        assert_eq!(libraries.request_id, "browse.libraries");
        assert_eq!(
            libraries.url,
            "https://taru.example/api/libraries?limit=25&offset=50"
        );
        assert_eq!(
            libraries.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            libraries.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );

        let library_sources =
            build_list_library_sources_request(&CoreBrowseEntityPagedRequestInput {
                base_url: "https://taru.example/api".to_owned(),
                access_token: "secret-token".to_owned(),
                id: "library 1".to_owned(),
                page: Some(CorePageQuery::new(Some(24), Some(0))),
            });
        assert_eq!(
            library_sources.url,
            "https://taru.example/api/libraries/library%201/sources?limit=24&offset=0"
        );

        let item = build_get_item_request(&CoreBrowseEntityRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "item/1".to_owned(),
        });
        assert_eq!(item.url, "https://taru.example/api/items/item%2F1");
    }

    #[test]
    fn browse_facet_and_search_builders_encode_ids_facets_and_page() {
        let genre_items = build_list_genre_items_request(&CoreBrowseEntityPagedRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "genre 1".to_owned(),
            page: Some(CorePageQuery::new(Some(24), Some(12))),
        });
        assert_eq!(
            genre_items.url,
            "https://taru.example/api/genres/genre%201/items?limit=24&offset=12"
        );

        let tag_items = build_list_tag_items_request(&CoreBrowseEntityPagedRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "tag:favorite".to_owned(),
            page: Some(CorePageQuery::new(Some(10), Some(0))),
        });
        assert_eq!(
            tag_items.url,
            "https://taru.example/api/tags/tag%3Afavorite/items?limit=10&offset=0"
        );

        let person_items = build_list_person_items_request(&CoreBrowseEntityPagedRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            id: "person 1".to_owned(),
            page: None,
        });
        assert_eq!(
            person_items.url,
            "https://taru.example/api/people/person%201/items"
        );

        let search = build_search_items_request(&CoreSearchItemsRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            query: Some("route demo".to_owned()),
            facets: vec!["genre:test".to_owned(), "tag:favorite".to_owned()],
            page: Some(CorePageQuery::new(Some(12), Some(6))),
        });
        assert_eq!(
            search.url,
            "https://taru.example/api/search?q=route%20demo&facet=genre%3Atest%2Ctag%3Afavorite&limit=12&offset=6"
        );
    }
}
