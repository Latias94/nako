use serde_json::{Map, Value, json};

use crate::public_client::{
    API_VERSION, API_VERSION_HEADER, ClientErrorCode, PLAYBACK_SESSION_ID_HEADER,
};

#[must_use]
pub fn public_openapi_v1() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Nako Public Client API",
            "version": API_VERSION,
            "description": "Public client HTTP contract for Nako API v1."
        },
        "paths": public_paths(),
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer"
                }
            },
            "headers": {
                "NakoApiVersion": {
                    "description": "Nako public API compatibility version.",
                    "schema": {
                        "type": "string",
                        "enum": [API_VERSION]
                    }
                },
                "WwwAuthenticate": {
                    "description": "Bearer authentication challenge.",
                    "schema": {
                        "type": "string",
                        "enum": ["Bearer"]
                    }
                },
                "NakoPlaybackSessionId": {
                    "description": "Public playback session id associated with a remux or HLS response.",
                    "schema": {
                        "type": "string",
                        "format": "uuid"
                    }
                }
            },
            "responses": common_responses(),
            "parameters": common_parameters(),
            "schemas": schemas()
        }
    })
}

#[must_use]
pub fn public_openapi_v1_json() -> String {
    serde_json::to_string_pretty(&public_openapi_v1()).expect("public OpenAPI document serializes")
}

fn public_paths() -> Value {
    let mut paths = Map::new();
    paths.insert(
        "/health".to_owned(),
        json!({
            "get": {
                "operationId": "getHealth",
                "summary": "Get server health and public API version.",
                "tags": ["system"],
                "responses": {
                    "200": json_response("Server is healthy.", schema_ref("HealthResponse")),
                    "500": response_ref("InternalServerError")
                }
            }
        }),
    );
    paths.insert(
        "/auth/login".to_owned(),
        json!({
            "post": public_json_post_with_body(
                "login",
                "Create a local user session.",
                "account",
                vec![],
                schema_ref("LoginRequest"),
                schema_ref("LoginResponse")
            )
        }),
    );
    paths.insert(
        "/auth/invitations/redeem".to_owned(),
        json!({
            "post": public_json_post_with_body(
                "redeemInvitation",
                "Redeem a local invitation into a user session.",
                "account",
                vec![],
                schema_ref("RedeemInvitationRequest"),
                schema_ref("LoginResponse")
            )
        }),
    );
    paths.insert(
        "/auth/logout".to_owned(),
        json!({
            "post": json_post("logout", "Revoke the current local user session.", "account", vec![], schema_ref("LogoutResponse"))
        }),
    );
    paths.insert(
        "/users/me".to_owned(),
        json!({
            "get": json_get("getCurrentUser", "Get the current user account.", "account", vec![], schema_ref("CurrentUserResponse"))
        }),
    );
    paths.insert(
        "/users/me/playlists".to_owned(),
        json!({
            "get": json_get(
                "listUserPlaylists",
                "List the current user's private playlists.",
                "user-playlist",
                vec![parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("UserPlaylistsResponse")
            ),
            "post": json_post_with_body(
                "createUserPlaylist",
                "Create a private playlist for the current user.",
                "user-playlist",
                vec![],
                schema_ref("CreateUserPlaylistRequest"),
                schema_ref("UserPlaylistResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playlists/{playlist_id}".to_owned(),
        json!({
            "get": json_get(
                "getUserPlaylist",
                "Get one current-user playlist summary.",
                "user-playlist",
                vec![path_parameter("playlist_id", "User playlist id.")],
                schema_ref("UserPlaylistResponse")
            ),
            "patch": json_patch(
                "updateUserPlaylist",
                "Rename a current-user playlist.",
                "user-playlist",
                vec![path_parameter("playlist_id", "User playlist id.")],
                schema_ref("UpdateUserPlaylistRequest"),
                schema_ref("UserPlaylistResponse")
            ),
            "delete": json_delete(
                "deleteUserPlaylist",
                "Delete a current-user playlist and its membership rows.",
                "user-playlist",
                vec![path_parameter("playlist_id", "User playlist id.")],
                schema_ref("UserPlaylistDeleteResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playlists/{playlist_id}/items".to_owned(),
        json!({
            "get": json_get(
                "listUserPlaylistItems",
                "List accessible items in one current-user playlist.",
                "user-playlist",
                vec![
                    path_parameter("playlist_id", "User playlist id."),
                    parameter_ref("Limit"),
                    parameter_ref("Offset")
                ],
                schema_ref("UserPlaylistItemsResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playlists/{playlist_id}/items/{item_id}".to_owned(),
        json!({
            "put": json_put(
                "addUserPlaylistItem",
                "Add or idempotently keep one media item in a current-user playlist.",
                "user-playlist",
                vec![
                    path_parameter("playlist_id", "User playlist id."),
                    path_parameter("item_id", "Media item id.")
                ],
                schema_ref("AddUserPlaylistItemRequest"),
                schema_ref("UserPlaylistResponse")
            ),
            "delete": json_delete(
                "removeUserPlaylistItem",
                "Remove one media item from a current-user playlist.",
                "user-playlist",
                vec![
                    path_parameter("playlist_id", "User playlist id."),
                    path_parameter("item_id", "Media item id.")
                ],
                schema_ref("UserPlaylistResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playlists/{playlist_id}/items/reorder".to_owned(),
        json!({
            "put": json_put(
                "reorderUserPlaylistItems",
                "Replace the current-user playlist item order.",
                "user-playlist",
                vec![path_parameter("playlist_id", "User playlist id.")],
                schema_ref("ReorderUserPlaylistItemsRequest"),
                schema_ref("UserPlaylistResponse")
            )
        }),
    );
    paths.insert(
        "/libraries".to_owned(),
        json!({
            "get": json_get("listLibraries", "List configured media libraries.", "library", vec![parameter_ref("Limit"), parameter_ref("Offset")], schema_ref("LibraryListResponse"))
        }),
    );
    paths.insert(
        "/libraries/{library_id}".to_owned(),
        json!({
            "get": json_get("getLibrary", "Get one media library.", "library", vec![path_parameter("library_id", "Library id.")], schema_ref("LibraryResponse"))
        }),
    );
    paths.insert(
        "/libraries/{library_id}/sources".to_owned(),
        json!({
            "get": json_get(
                "listLibrarySources",
                "List sources in one media library.",
                "library",
                vec![path_parameter("library_id", "Library id."), parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("LibrarySourcesResponse")
            )
        }),
    );
    paths.insert(
        "/libraries/{library_id}/items".to_owned(),
        json!({
            "get": json_get(
                "listLibraryItems",
                "List media items in one library.",
                "library",
                vec![
                    path_parameter("library_id", "Library id."),
                    parameter_ref("Limit"),
                    parameter_ref("Offset"),
                    query_parameter("sort", "Library browse sort key.", schema_ref("ClientBrowseSortKey"), false),
                    query_parameter("order", "Library browse sort order.", schema_ref("ClientSortOrder"), false),
                    query_parameter("facet", "Comma-separated public browse facet tokens.", string_schema(), false),
                    query_parameter("watch_state", "Current user's watch-state filter.", schema_ref("ClientWatchStateFilter"), false),
                ],
                schema_ref("LibraryItemsResponse")
            )
        }),
    );
    paths.insert(
        "/items".to_owned(),
        json!({
            "get": json_get("listItems", "List media items.", "catalog", vec![parameter_ref("Limit"), parameter_ref("Offset")], schema_ref("ItemsResponse"))
        }),
    );
    paths.insert(
        "/items/{item_id}".to_owned(),
        json!({
            "get": json_get("getItem", "Get one media item with catalog relations.", "catalog", vec![path_parameter("item_id", "Media item id.")], schema_ref("ItemDetailResponse"))
        }),
    );
    paths.insert(
        "/items/{item_id}/credits".to_owned(),
        json!({
            "get": json_get("listItemCredits", "List credits for one media item.", "catalog", vec![path_parameter("item_id", "Media item id.")], schema_ref("ItemCreditsResponse"))
        }),
    );
    paths.insert(
        "/items/{item_id}/images".to_owned(),
        json!({
            "get": json_get("listItemImages", "List images for one media item.", "catalog", vec![path_parameter("item_id", "Media item id.")], schema_ref("ImagesResponse"))
        }),
    );
    paths.insert(
        "/images/{image_id}".to_owned(),
        json!({
            "get": binary_get_with_tag(
                "getImage",
                "Serve one selected artwork image.",
                "catalog",
                image_parameters()
            ),
            "head": empty_head_with_tag(
                "headImage",
                "Preflight selected artwork image headers.",
                "catalog",
                image_parameters()
            )
        }),
    );
    paths.insert(
        "/people".to_owned(),
        json!({
            "get": json_get("listPeople", "List people.", "catalog", vec![parameter_ref("Limit"), parameter_ref("Offset")], schema_ref("PeopleResponse"))
        }),
    );
    paths.insert(
        "/people/{person_id}".to_owned(),
        json!({
            "get": json_get("getPerson", "Get one person.", "catalog", vec![path_parameter("person_id", "Person id.")], schema_ref("PersonResponse"))
        }),
    );
    paths.insert(
        "/people/{person_id}/items".to_owned(),
        json!({
            "get": json_get(
                "listPersonItems",
                "List media items linked to one person.",
                "catalog",
                vec![path_parameter("person_id", "Person id."), parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("PersonItemsResponse")
            )
        }),
    );
    paths.insert(
        "/tags".to_owned(),
        json!({
            "get": json_get("listTags", "List tags.", "catalog", vec![parameter_ref("Limit"), parameter_ref("Offset")], schema_ref("TagsResponse"))
        }),
    );
    paths.insert(
        "/tags/{tag_id}/items".to_owned(),
        json!({
            "get": json_get(
                "listTagItems",
                "List media items linked to one tag.",
                "catalog",
                vec![path_parameter("tag_id", "Tag id."), parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("TagItemsResponse")
            )
        }),
    );
    paths.insert(
        "/genres".to_owned(),
        json!({
            "get": json_get("listGenres", "List genres.", "catalog", vec![parameter_ref("Limit"), parameter_ref("Offset")], schema_ref("GenreListResponse"))
        }),
    );
    paths.insert(
        "/genres/{genre_id}/items".to_owned(),
        json!({
            "get": json_get(
                "listGenreItems",
                "List media items linked to one genre.",
                "catalog",
                vec![path_parameter("genre_id", "Genre id."), parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("GenreItemsResponse")
            )
        }),
    );
    paths.insert(
        "/search".to_owned(),
        json!({
            "get": json_get(
                "searchItems",
                "Search media items.",
                "catalog",
                vec![query_parameter("q", "Search query.", string_schema(), false), query_parameter("facet", "Comma-separated lightweight facets.", string_schema(), false), parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("SearchResponse")
            )
        }),
    );
    paths.insert(
        "/management/context-links".to_owned(),
        json!({
            "get": json_get(
                "getManagementContextLinks",
                "List management actions available from the current media context.",
                "management",
                vec![
                    query_parameter("library_id", "Optional library id for the current context.", uuid_schema(), false),
                    query_parameter("item_id", "Optional media item id for the current context.", uuid_schema(), false),
                    query_parameter("source_id", "Optional media source id for the current context.", uuid_schema(), false),
                    query_parameter("playback_session_id", "Optional playback session id for the current context.", uuid_schema(), false)
                ],
                schema_ref("ManagementContextLinksResponse")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/probe".to_owned(),
        json!({
            "get": json_get("getSourceProbe", "Get persisted media probe data for one source.", "playback", vec![path_parameter("source_id", "Media source id.")], schema_ref("SourceProbeResponse"))
        }),
    );
    paths.insert(
        "/sources/{source_id}/playback/decision".to_owned(),
        json!({
            "get": json_get(
                "getSourcePlaybackDecision",
                "Get playback decision for one source.",
                "playback",
                playback_parameters("source_id"),
                schema_ref("PlaybackDecisionResponse")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/playback/browser-ticket".to_owned(),
        json!({
            "post": json_post_with_body(
                "createBrowserPlaybackTicket",
                "Issue a browser playback ticket for one source.",
                "playback",
                vec![path_parameter("source_id", "Media source id.")],
                schema_ref("BrowserPlaybackTicketRequest"),
                schema_ref("BrowserPlaybackTicketResponse")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/stream".to_owned(),
        json!({
            "get": binary_get("streamSource", "Stream direct-play bytes for one source.", vec![path_parameter("source_id", "Media source id."), range_header_parameter()]),
            "head": empty_head("headStreamSource", "Preflight direct-play stream headers for one source.", vec![path_parameter("source_id", "Media source id."), range_header_parameter()])
        }),
    );
    paths.insert(
        "/sources/{source_id}/stream/remux".to_owned(),
        json!({
            "get": session_binary_get(
                "remuxStreamSource",
                "Run or reuse remux output and stream bytes.",
                remux_parameters("source_id")
            ),
            "head": session_empty_head(
                "headRemuxStreamSource",
                "Preflight remux stream headers and expose playback session identity.",
                remux_parameters("source_id")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/stream/hls/playlist.m3u8".to_owned(),
        json!({
            "get": session_text_get(
                "hlsPlaylistSource",
                "Start or reuse HLS transcode and return a playlist.",
                hls_parameters("source_id")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/subtitles/{stream_index}".to_owned(),
        json!({
            "get": subtitle_text_get(
                "getSourceSubtitle",
                "Serve one sidecar subtitle stream.",
                vec![
                    path_parameter("source_id", "Media source id."),
                    path_integer_parameter("stream_index", "Subtitle stream index.")
                ]
            )
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}".to_owned(),
        json!({
            "get": json_get("getPlaybackSession", "Get one playback session.", "playback", vec![path_parameter("session_id", "Playback session id.")], schema_ref("PlaybackSessionResponse"))
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}/cancel".to_owned(),
        json!({
            "post": json_post("cancelPlaybackSession", "Request playback session cancellation.", "playback", vec![path_parameter("session_id", "Playback session id.")], schema_ref("PlaybackSessionResponse"))
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}/heartbeat".to_owned(),
        json!({
            "post": json_post_with_body(
                "heartbeatPlaybackSession",
                "Record playback session heartbeat and position.",
                "playback",
                vec![path_parameter("session_id", "Playback session id.")],
                schema_ref("PlaybackSessionHeartbeatRequest"),
                schema_ref("PlaybackSessionResponse")
            )
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}/hls/segments/{segment_name}".to_owned(),
        json!({
            "get": binary_get(
                "hlsSegment",
                "Stream one generated HLS segment.",
                vec![path_parameter("session_id", "Playback session id."), path_parameter("segment_name", "HLS segment file name.")]
            )
        }),
    );
    paths.insert(
        "/renderers".to_owned(),
        json!({
            "get": json_get(
                "listRenderers",
                "List controllable renderer sessions owned by the current user.",
                "renderer",
                vec![parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("RendererSessionsResponse")
            ),
            "post": json_post_with_body(
                "registerRenderer",
                "Register or refresh a Nako remote renderer session.",
                "renderer",
                vec![],
                schema_ref("RendererRegistrationRequest"),
                schema_ref("RendererSessionResponse")
            )
        }),
    );
    paths.insert(
        "/renderers/{renderer_session_id}/heartbeat".to_owned(),
        json!({
            "post": json_post_with_body(
                "heartbeatRenderer",
                "Record renderer heartbeat and optional capability updates.",
                "renderer",
                vec![path_parameter("renderer_session_id", "Renderer session id.")],
                schema_ref("RendererHeartbeatRequest"),
                schema_ref("RendererSessionResponse")
            )
        }),
    );
    paths.insert(
        "/renderers/{renderer_session_id}/commands/next".to_owned(),
        json!({
            "post": json_post(
                "pollNextRendererCommand",
                "Claim the next queued command for a renderer session.",
                "renderer",
                vec![path_parameter("renderer_session_id", "Renderer session id.")],
                schema_ref("RendererCommandPollResponse")
            )
        }),
    );
    paths.insert(
        "/renderers/{renderer_session_id}/commands/play".to_owned(),
        json!({
            "post": json_post_with_body(
                "playOnRenderer",
                "Create a policy-checked playback session and queue a play command for a renderer.",
                "renderer",
                vec![path_parameter("renderer_session_id", "Renderer session id.")],
                schema_ref("RendererPlayCommandRequest"),
                schema_ref("RendererPlayCommandResponse")
            )
        }),
    );
    paths.insert(
        "/renderers/{renderer_session_id}/commands/{command_id}/complete".to_owned(),
        json!({
            "post": json_post_with_body(
                "completeRendererCommand",
                "Acknowledge or fail a delivered renderer command.",
                "renderer",
                vec![
                    path_parameter("renderer_session_id", "Renderer session id."),
                    path_parameter("command_id", "Renderer command id.")
                ],
                schema_ref("RendererCommandCompletionRequest"),
                schema_ref("RendererCommandResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playback-state/items/{item_id}".to_owned(),
        json!({
            "get": json_get(
                "getUserPlaybackState",
                "Get the current user's playback state for one media item.",
                "user-playback",
                vec![path_parameter("item_id", "Media item id.")],
                schema_ref("UserPlaybackStateResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playback-state/continue-watching".to_owned(),
        json!({
            "get": json_get(
                "listContinueWatching",
                "List the current user's continue-watching items.",
                "user-playback",
                vec![parameter_ref("Limit"), parameter_ref("Offset")],
                schema_ref("ContinueWatchingResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playback-state/items/{item_id}/progress".to_owned(),
        json!({
            "put": json_put(
                "updateUserPlaybackProgress",
                "Update the current user's playback progress for one media item.",
                "user-playback",
                vec![path_parameter("item_id", "Media item id.")],
                schema_ref("UpdatePlaybackProgressRequest"),
                schema_ref("UserPlaybackStateResponse")
            )
        }),
    );
    paths.insert(
        "/users/me/playback-state/items/{item_id}/watched".to_owned(),
        json!({
            "put": json_put(
                "setUserWatchedState",
                "Set the current user's watched state for one media item.",
                "user-playback",
                vec![path_parameter("item_id", "Media item id.")],
                schema_ref("SetWatchedStateRequest"),
                schema_ref("UserPlaybackStateResponse")
            )
        }),
    );
    Value::Object(paths)
}

fn json_get(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    response_schema: Value,
) -> Value {
    operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    )
}

fn json_post(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    response_schema: Value,
) -> Value {
    operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    )
}

fn json_post_with_body(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    request_schema: Value,
    response_schema: Value,
) -> Value {
    let mut value = operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    );
    value["requestBody"] = json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": request_schema
            }
        }
    });
    value
}

fn public_json_post_with_body(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    request_schema: Value,
    response_schema: Value,
) -> Value {
    let mut value = public_operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    );
    value["requestBody"] = json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": request_schema
            }
        }
    });
    value
}

fn json_put(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    request_schema: Value,
    response_schema: Value,
) -> Value {
    let mut value = operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    );
    value["requestBody"] = json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": request_schema
            }
        }
    });
    value
}

fn json_patch(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    request_schema: Value,
    response_schema: Value,
) -> Value {
    let mut value = operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    );
    value["requestBody"] = json!({
        "required": true,
        "content": {
            "application/json": {
                "schema": request_schema
            }
        }
    });
    value
}

fn json_delete(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    response_schema: Value,
) -> Value {
    operation(
        operation_id,
        summary,
        tag,
        parameters,
        json_response("OK.", response_schema),
    )
}

fn binary_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    binary_get_with_tag(operation_id, summary, "playback", parameters)
}

fn session_binary_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
        parameters,
        json!({
            "description": "Binary stream.",
            "headers": playback_session_headers(),
            "content": {
                "application/octet-stream": {
                    "schema": {
                        "type": "string",
                        "format": "binary"
                    }
                }
            }
        }),
    )
}

fn binary_get_with_tag(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
) -> Value {
    operation(
        operation_id,
        summary,
        tag,
        parameters,
        json!({
            "description": "Binary stream.",
            "headers": api_version_headers(),
            "content": {
                "application/octet-stream": {
                    "schema": {
                        "type": "string",
                        "format": "binary"
                    }
                }
            }
        }),
    )
}

fn session_text_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
        parameters,
        json!({
            "description": "Text response.",
            "headers": playback_session_headers(),
            "content": {
                "application/vnd.apple.mpegurl": {
                    "schema": string_schema()
                }
            }
        }),
    )
}

fn subtitle_text_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
        parameters,
        json!({
            "description": "Subtitle text.",
            "headers": api_version_headers(),
            "content": {
                "text/vtt": {
                    "schema": string_schema()
                },
                "application/x-subrip": {
                    "schema": string_schema()
                }
            }
        }),
    )
}

fn session_empty_head(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
        parameters,
        json!({
            "description": "Headers only.",
            "headers": playback_session_headers()
        }),
    )
}

fn empty_head(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    empty_head_with_tag(operation_id, summary, "playback", parameters)
}

fn empty_head_with_tag(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
) -> Value {
    operation(
        operation_id,
        summary,
        tag,
        parameters,
        json!({
            "description": "Headers only.",
            "headers": api_version_headers()
        }),
    )
}

fn operation(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    success_response: Value,
) -> Value {
    let mut responses = Map::new();
    responses.insert("200".to_owned(), success_response);
    responses.insert("400".to_owned(), response_ref("BadRequest"));
    responses.insert("401".to_owned(), response_ref("Unauthorized"));
    responses.insert("404".to_owned(), response_ref("NotFound"));
    responses.insert("409".to_owned(), response_ref("Conflict"));
    responses.insert("416".to_owned(), response_ref("RangeNotSatisfiable"));
    responses.insert("500".to_owned(), response_ref("InternalServerError"));

    json!({
        "operationId": operation_id,
        "summary": summary,
        "tags": [tag],
        "security": [{"BearerAuth": []}],
        "parameters": parameters,
        "responses": responses
    })
}

fn public_operation(
    operation_id: &str,
    summary: &str,
    tag: &str,
    parameters: Vec<Value>,
    success_response: Value,
) -> Value {
    let mut responses = Map::new();
    responses.insert("200".to_owned(), success_response);
    responses.insert("400".to_owned(), response_ref("BadRequest"));
    responses.insert("401".to_owned(), response_ref("Unauthorized"));
    responses.insert("409".to_owned(), response_ref("Conflict"));
    responses.insert("500".to_owned(), response_ref("InternalServerError"));

    json!({
        "operationId": operation_id,
        "summary": summary,
        "tags": [tag],
        "parameters": parameters,
        "responses": responses
    })
}

fn json_response(description: &str, schema: Value) -> Value {
    json!({
        "description": description,
        "headers": api_version_headers(),
        "content": {
            "application/json": {
                "schema": schema
            }
        }
    })
}

fn common_responses() -> Value {
    json!({
        "BadRequest": error_response("Bad request."),
        "Unauthorized": {
            "description": "Authentication is required.",
            "headers": {
                API_VERSION_HEADER: header_ref("NakoApiVersion"),
                "www-authenticate": header_ref("WwwAuthenticate")
            },
            "content": {
                "application/json": {
                    "schema": schema_ref("ErrorResponse")
                }
            }
        },
        "NotFound": error_response("Resource was not found."),
        "Conflict": error_response("Request conflicts with current state."),
        "RangeNotSatisfiable": error_response("Requested byte range is not satisfiable."),
        "InternalServerError": error_response("Internal server error.")
    })
}

fn error_response(description: &str) -> Value {
    json!({
        "description": description,
        "headers": api_version_headers(),
        "content": {
            "application/json": {
                "schema": schema_ref("ErrorResponse")
            }
        }
    })
}

fn common_parameters() -> Value {
    json!({
        "Limit": query_parameter("limit", "Maximum number of records to return.", integer_schema("int32"), false),
        "Offset": query_parameter("offset", "Offset of the first record to return.", integer_schema("int64"), false)
    })
}

fn playback_parameters(source_id_name: &str) -> Vec<Value> {
    vec![
        path_parameter(source_id_name, "Media source id."),
        query_parameter(
            "direct_play",
            "Whether the client can direct-play compatible sources.",
            boolean_schema(),
            false,
        ),
        query_parameter(
            "container",
            "Comma-separated playable containers.",
            string_schema(),
            false,
        ),
        query_parameter(
            "video_codec",
            "Comma-separated playable video codecs.",
            string_schema(),
            false,
        ),
        query_parameter(
            "audio_codec",
            "Comma-separated playable audio codecs.",
            string_schema(),
            false,
        ),
        query_parameter(
            "max_video_bitrate",
            "Maximum direct-play video bitrate in bits per second.",
            integer_schema("int64"),
            false,
        ),
        query_parameter(
            "max_width",
            "Maximum direct-play video width.",
            integer_schema("int32"),
            false,
        ),
        query_parameter(
            "max_height",
            "Maximum direct-play video height.",
            integer_schema("int32"),
            false,
        ),
        query_parameter(
            "max_audio_channels",
            "Maximum direct-play audio channel count.",
            integer_schema("int32"),
            false,
        ),
        query_parameter(
            "supports_hdr",
            "Whether the client can direct-play HDR video.",
            boolean_schema(),
            false,
        ),
        query_parameter(
            "supports_subtitles",
            "Whether the client can directly deliver selected subtitles.",
            boolean_schema(),
            false,
        ),
        query_parameter(
            "hls_variant_policy",
            "Requested HLS rendition variant policy.",
            enum_schema(&["single_variant", "adaptive"]),
            false,
        ),
        query_parameter(
            "hls_segment_container",
            "Requested HLS segment container.",
            enum_schema(&["mpeg_ts", "fmp4"]),
            false,
        ),
    ]
}

fn hls_parameters(source_id_name: &str) -> Vec<Value> {
    let mut parameters = playback_parameters(source_id_name);
    parameters.push(query_parameter(
        "start_position_ms",
        "Optional HLS playback start position in milliseconds for seek/restart.",
        integer_schema("int64"),
        false,
    ));
    parameters.push(query_parameter(
        "preferred_audio_language",
        "Comma-separated ordered preferred audio language tags for HLS default audio selection. Explicit audio_stream still wins.",
        string_schema(),
        false,
    ));
    parameters.push(query_parameter(
        "preferred_subtitle_language",
        "Comma-separated ordered preferred subtitle language tags for HLS default subtitle selection. Explicit subtitle_stream still wins.",
        string_schema(),
        false,
    ));
    parameters
}

fn remux_parameters(source_id_name: &str) -> Vec<Value> {
    let mut parameters = playback_parameters(source_id_name);
    parameters.push(query_parameter(
        "output_container",
        "Requested remux output container.",
        enum_schema(&["mp4", "mkv"]),
        false,
    ));
    parameters.push(range_header_parameter());
    parameters
}

fn image_parameters() -> Vec<Value> {
    vec![
        path_parameter("image_id", "Selected artwork image id."),
        query_parameter(
            "width",
            "Optional bounded image variant width. Must be a positive integer within the server artwork limit.",
            integer_schema("int32"),
            false,
        ),
        query_parameter(
            "height",
            "Optional bounded image variant height. Must be a positive integer within the server artwork limit.",
            integer_schema("int32"),
            false,
        ),
    ]
}

fn range_header_parameter() -> Value {
    json!({
        "name": "Range",
        "in": "header",
        "required": false,
        "schema": string_schema(),
        "description": "Optional HTTP byte range."
    })
}

fn path_parameter(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "schema": {
            "type": "string",
            "format": "uuid"
        },
        "description": description
    })
}

fn path_integer_parameter(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "schema": integer_schema("int32"),
        "description": description
    })
}

fn query_parameter(name: &str, description: &str, schema: Value, required: bool) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": required,
        "schema": schema,
        "description": description
    })
}

fn parameter_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/parameters/{name}")})
}

fn response_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/responses/{name}")})
}

fn header_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/headers/{name}")})
}

fn schema_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/schemas/{name}")})
}

fn api_version_headers() -> Value {
    json!({API_VERSION_HEADER: header_ref("NakoApiVersion")})
}

fn playback_session_headers() -> Value {
    json!({
        API_VERSION_HEADER: header_ref("NakoApiVersion"),
        PLAYBACK_SESSION_ID_HEADER: header_ref("NakoPlaybackSessionId")
    })
}

fn string_schema() -> Value {
    json!({"type": "string"})
}

fn nullable_string_schema() -> Value {
    json!({"type": "string", "nullable": true})
}

fn boolean_schema() -> Value {
    json!({"type": "boolean"})
}

fn integer_schema(format: &str) -> Value {
    json!({"type": "integer", "format": format})
}

fn enum_schema(values: &[&str]) -> Value {
    json!({"type": "string", "enum": values})
}

fn array_schema(item_schema: Value) -> Value {
    json!({"type": "array", "items": item_schema})
}

fn nullable_ref(name: &str) -> Value {
    json!({"allOf": [schema_ref(name)], "nullable": true})
}

fn schemas() -> Value {
    json!({
        "HealthResponse": object_schema(&["status", "version"], json!({
            "status": string_schema(),
            "version": enum_schema(&[API_VERSION])
        })),
        "ErrorResponse": object_schema(&["code", "message"], json!({
            "code": enum_schema(ClientErrorCode::ALL.iter().map(|code| code.as_str()).collect::<Vec<_>>().as_slice()),
            "message": string_schema()
        })),
        "LoginRequest": object_schema(&["username", "password"], json!({
            "username": string_schema(),
            "password": string_schema()
        })),
        "LoginResponse": object_schema(&["session", "account"], json!({
            "session": schema_ref("UserSessionDto"),
            "account": schema_ref("CurrentUserResponse")
        })),
        "RedeemInvitationRequest": object_schema(&["token", "username", "display_name", "password"], json!({
            "token": string_schema(),
            "username": string_schema(),
            "display_name": string_schema(),
            "password": string_schema()
        })),
        "LogoutResponse": object_schema(&["revoked"], json!({
            "revoked": boolean_schema()
        })),
        "UserSessionDto": object_schema(&["token", "expires_at_ms"], json!({
            "token": string_schema(),
            "expires_at_ms": integer_schema("int64")
        })),
        "CurrentUserResponse": object_schema(&["user"], json!({
            "user": schema_ref("CurrentUserDto")
        })),
        "CurrentUserDto": object_schema(&["id", "username", "display_name", "roles", "bootstrap"], json!({
            "id": uuid_schema(),
            "username": string_schema(),
            "display_name": string_schema(),
            "roles": array_schema(enum_schema(&["administrator", "library_manager", "viewer"])),
            "bootstrap": boolean_schema()
        })),
        "PageInfo": object_schema(&["limit", "offset", "returned"], json!({
            "limit": integer_schema("int32"),
            "offset": integer_schema("int64"),
            "returned": integer_schema("int32")
        })),
        "LibraryListResponse": object_schema(&["libraries", "page"], json!({
            "libraries": array_schema(schema_ref("LibraryDto")),
            "page": schema_ref("PageInfo")
        })),
        "LibraryResponse": object_schema(&["library"], json!({
            "library": schema_ref("LibraryDto")
        })),
        "LibrarySourcesResponse": object_schema(&["library", "sources", "page"], json!({
            "library": schema_ref("LibraryDto"),
            "sources": array_schema(schema_ref("LibrarySourceResponse")),
            "page": schema_ref("PageInfo")
        })),
        "LibraryItemsResponse": object_schema(&["library", "items", "page"], json!({
            "library": schema_ref("LibraryDto"),
            "items": array_schema(schema_ref("MediaItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "ClientBrowseSortKey": enum_schema(&["title", "release_date", "date_added", "last_played"]),
        "ClientSortOrder": enum_schema(&["asc", "desc"]),
        "ClientWatchStateFilter": enum_schema(&["any", "watched", "unwatched", "in_progress"]),
        "LibrarySourceResponse": object_schema(&["source", "item", "probe"], json!({
            "source": schema_ref("MediaSourceDto"),
            "item": nullable_ref("MediaItemDto"),
            "probe": nullable_ref("MediaProbeDto")
        })),
        "LibraryDto": object_schema(&["id", "name", "roots", "options"], json!({
            "id": uuid_schema(),
            "name": string_schema(),
            "roots": array_schema(string_schema()),
            "options": schema_ref("LibraryOptionsDto")
        })),
        "LibraryOptionsDto": object_schema(&["domain", "preset", "scan", "naming_strategy", "metadata_profile"], json!({
            "domain": enum_schema(&["video", "audio", "image", "document", "mixed", "online"]),
            "preset": enum_schema(&["movies", "tv", "anime", "music", "podcast", "photos", "home_video", "mixed_video", "online_catalog", "custom"]),
            "scan": schema_ref("LibraryScanOptionsDto"),
            "naming_strategy": enum_schema(&["movie", "series", "anime", "music", "podcast", "photo", "home_video", "mixed", "online_catalog"]),
            "metadata_profile": schema_ref("MetadataProfileDto")
        })),
        "LibraryScanOptionsDto": object_schema(&["realtime_monitor", "max_depth"], json!({
            "realtime_monitor": boolean_schema(),
            "max_depth": json!({"type": "integer", "format": "int32", "nullable": true})
        })),
        "MetadataProfileDto": object_schema(&["item_kinds", "local_readers", "metadata_providers", "image_providers", "language", "country", "refresh_mode", "local_metadata_policy", "scan"], json!({
            "item_kinds": array_schema(schema_ref("ClientMediaKind")),
            "local_readers": array_schema(string_schema()),
            "metadata_providers": array_schema(string_schema()),
            "image_providers": array_schema(string_schema()),
            "language": nullable_string_schema(),
            "country": nullable_string_schema(),
            "refresh_mode": enum_schema(&["none", "validation_only", "default", "missing_only", "full_refresh"]),
            "local_metadata_policy": enum_schema(&["disabled", "read_only", "local_first", "remote_first", "write_sidecar"]),
            "scan": schema_ref("MetadataScanPolicyDto")
        })),
        "MetadataScanPolicyDto": object_schema(&["enabled", "addon_scrape", "addon_writeback"], json!({
            "enabled": boolean_schema(),
            "addon_scrape": boolean_schema(),
            "addon_writeback": boolean_schema()
        })),
        "ItemsResponse": object_schema(&["items", "page"], json!({
            "items": array_schema(schema_ref("MediaItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "ItemDetailResponse": object_schema(&["item", "sources", "credits", "genres", "tags", "collections", "studios", "images"], json!({
            "item": schema_ref("MediaItemDto"),
            "sources": array_schema(schema_ref("MediaSourceDto")),
            "credits": array_schema(schema_ref("ItemCreditDto")),
            "genres": array_schema(schema_ref("ItemGenreDto")),
            "tags": array_schema(schema_ref("ItemTagDto")),
            "collections": array_schema(schema_ref("CollectionItemDto")),
            "studios": array_schema(schema_ref("ItemStudioDto")),
            "images": array_schema(schema_ref("PublicImageRefDto"))
        })),
        "ItemCreditsResponse": object_schema(&["item_id", "credits", "people"], json!({
            "item_id": uuid_schema(),
            "credits": array_schema(schema_ref("ItemCreditDto")),
            "people": array_schema(schema_ref("PersonDto"))
        })),
        "ImagesResponse": object_schema(&["item_id", "images"], json!({
            "item_id": uuid_schema(),
            "images": array_schema(schema_ref("PublicImageRefDto"))
        })),
        "PeopleResponse": object_schema(&["people", "page"], json!({
            "people": array_schema(schema_ref("PersonDto")),
            "page": schema_ref("PageInfo")
        })),
        "PersonResponse": object_schema(&["person"], json!({
            "person": schema_ref("PersonDto")
        })),
        "PersonItemsResponse": object_schema(&["person", "items", "page"], json!({
            "person": schema_ref("PersonDto"),
            "items": array_schema(schema_ref("MediaItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "TagsResponse": object_schema(&["tags", "page"], json!({
            "tags": array_schema(schema_ref("TagDto")),
            "page": schema_ref("PageInfo")
        })),
        "TagItemsResponse": object_schema(&["tag", "items", "page"], json!({
            "tag": schema_ref("TagDto"),
            "items": array_schema(schema_ref("MediaItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "GenreListResponse": object_schema(&["genres", "page"], json!({
            "genres": array_schema(schema_ref("GenreDto")),
            "page": schema_ref("PageInfo")
        })),
        "GenreItemsResponse": object_schema(&["genre", "items", "page"], json!({
            "genre": schema_ref("GenreDto"),
            "items": array_schema(schema_ref("MediaItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "SearchResponse": object_schema(&["hits", "page"], json!({
            "hits": array_schema(schema_ref("SearchItemHit")),
            "page": schema_ref("PageInfo")
        })),
        "SearchItemHit": object_schema(&["item", "score"], json!({
            "item": schema_ref("MediaItemDto"),
            "score": json!({"type": "number", "format": "float"})
        })),
        "ManagementContextLinksResponse": object_schema(&["context", "links"], json!({
            "context": schema_ref("ManagementContextDto"),
            "links": array_schema(schema_ref("ManagementContextLinkDto"))
        })),
        "ManagementContextDto": object_schema(&["library_id", "item_id", "source_id", "playback_session_id"], json!({
            "library_id": nullable_uuid_schema(),
            "item_id": nullable_uuid_schema(),
            "source_id": nullable_uuid_schema(),
            "playback_session_id": nullable_uuid_schema()
        })),
        "ManagementContextLinkDto": object_schema(&["route_name", "method", "surface", "action", "target", "enabled", "required_access", "disabled_reason"], json!({
            "route_name": string_schema(),
            "method": enum_schema(&["GET", "POST", "PUT", "DELETE"]),
            "surface": enum_schema(&["management", "media"]),
            "action": enum_schema(&[
                "scan_library",
                "update_library_metadata_profile",
                "refresh_item_metadata",
                "view_jobs",
                "view_playback_diagnostics",
                "view_playback_runtime",
                "manage_library_access"
            ]),
            "target": schema_ref("ManagementContextDto"),
            "enabled": boolean_schema(),
            "required_access": enum_schema(&["library_manage", "administrator"]),
            "disabled_reason": json!({"type": "string", "nullable": true, "enum": ["missing_context", "insufficient_permission"]})
        })),
        "SourceProbeResponse": object_schema(&["source_id", "probe"], json!({
            "source_id": uuid_schema(),
            "probe": schema_ref("MediaProbeDto")
        })),
        "BrowserPlaybackTicketRequest": object_schema(&["mode"], json!({
            "mode": enum_schema(&["direct", "remux", "hls", "subtitle"]),
            "capabilities": schema_ref("BrowserPlaybackCapabilitiesDto"),
            "subtitle_stream_index": integer_schema("int32")
        })),
        "BrowserPlaybackCapabilitiesDto": object_schema(&[], json!({
            "direct_play": boolean_schema(),
            "container": array_schema(string_schema()),
            "video_codec": array_schema(string_schema()),
            "audio_codec": array_schema(string_schema()),
            "max_video_bitrate": integer_schema("int64"),
            "max_width": integer_schema("int32"),
            "max_height": integer_schema("int32"),
            "max_audio_channels": integer_schema("int32"),
            "supports_hdr": boolean_schema(),
            "supports_subtitles": boolean_schema(),
            "hls_variant_policy": schema_ref("ClientHlsVariantPolicy"),
            "hls_segment_container": schema_ref("ClientHlsSegmentContainer"),
            "output_container": enum_schema(&["mp4", "mkv"])
        })),
        "BrowserPlaybackTicketResponse": object_schema(&["source_id", "playback_session_id", "mode", "expires_at", "urls"], json!({
            "source_id": uuid_schema(),
            "item_id": nullable_uuid_schema(),
            "playback_session_id": nullable_uuid_schema(),
            "mode": enum_schema(&["direct", "remux", "hls", "subtitle"]),
            "expires_at": string_schema(),
            "urls": non_empty_array_schema(schema_ref("BrowserPlaybackUrlDto"))
        })),
        "BrowserPlaybackUrlDto": object_schema(&["kind", "url", "content_type", "supports_range_requests"], json!({
            "kind": enum_schema(&["stream", "playlist", "subtitle"]),
            "url": string_schema(),
            "content_type": string_schema(),
            "supports_range_requests": boolean_schema()
        })),
        "ClientPlaybackTargetKind": enum_schema(&[
            "browser",
            "native_desktop",
            "native_mobile",
            "nako_remote_client",
            "chromecast",
            "dlna_renderer",
            "airplay"
        ]),
        "ClientPlaybackTargetNetworkScope": enum_schema(&["local", "remote", "unknown"]),
        "ClientPlaybackTargetTransportAuth": enum_schema(&["bearer", "browser_ticket", "cast_ticket", "none"]),
        "ClientRendererControlCommand": enum_schema(&["show_item", "play", "pause", "resume", "seek", "stop", "set_volume"]),
        "ClientRendererSessionState": enum_schema(&["online", "offline", "revoked"]),
        "ClientRendererCommandState": enum_schema(&["queued", "delivered", "acknowledged", "failed", "cancelled"]),
        "ClientPlaybackPermission": enum_schema(&[
            "media_playback",
            "direct_play",
            "remux",
            "audio_transcode",
            "video_transcode",
            "remote_playback",
            "remote_control",
            "cast"
        ]),
        "ClientPlaybackPermissionDecisionReason": enum_schema(&[
            "allowed",
            "library_access_does_not_allow_play",
            "media_playback_disabled",
            "direct_play_disabled",
            "remux_disabled",
            "audio_transcode_disabled",
            "video_transcode_disabled",
            "remote_playback_disabled",
            "remote_control_disabled",
            "cast_disabled"
        ]),
        "ClientPlaybackTargetDto": object_schema(&["kind", "network_scope", "transport_auth", "media_capabilities", "control_capabilities"], json!({
            "kind": schema_ref("ClientPlaybackTargetKind"),
            "network_scope": schema_ref("ClientPlaybackTargetNetworkScope"),
            "transport_auth": schema_ref("ClientPlaybackTargetTransportAuth"),
            "media_capabilities": schema_ref("ClientPlaybackCapabilitiesDto"),
            "control_capabilities": schema_ref("ClientRendererControlCapabilitiesDto")
        })),
        "ClientRendererControlCapabilitiesDto": object_schema(&["commands"], json!({
            "commands": array_schema(schema_ref("ClientRendererControlCommand"))
        })),
        "ClientPlaybackDenialDto": object_schema(&["permission", "reason"], json!({
            "permission": schema_ref("ClientPlaybackPermission"),
            "reason": schema_ref("ClientPlaybackPermissionDecisionReason")
        })),
        "PlaybackDecisionResponse": object_schema(&["source", "probe", "target", "decision"], json!({
            "source": schema_ref("MediaSourceDto"),
            "probe": nullable_ref("MediaProbeDto"),
            "target": schema_ref("ClientPlaybackTargetDto"),
            "decision": schema_ref("ClientPlaybackDecision")
        })),
        "ClientPlaybackDecision": object_schema(&["mode", "reason", "report", "denial", "direct_play", "transcode_plan"], json!({
            "mode": enum_schema(&["direct_play", "remux", "transcode", "denied"]),
            "reason": schema_ref("ClientPlaybackDecisionReason"),
            "report": schema_ref("ClientPlaybackDecisionReport"),
            "denial": nullable_ref("ClientPlaybackDenialDto"),
            "direct_play": nullable_ref("ClientDirectPlayPlan"),
            "transcode_plan": nullable_ref("ClientTranscodePlan")
        })),
        "ClientPlaybackDecisionReport": object_schema(&["selected_mode", "direct_play", "remux", "transcode"], json!({
            "selected_mode": enum_schema(&["direct_play", "remux", "transcode", "denied"]),
            "direct_play": schema_ref("ClientPlaybackCapabilityEvaluation"),
            "remux": schema_ref("ClientPlaybackCapabilityEvaluation"),
            "transcode": schema_ref("ClientPlaybackCapabilityEvaluation"),
            "denial": nullable_ref("ClientPlaybackDenialDto")
        })),
        "ClientPlaybackCapabilityEvaluation": object_schema(&["supported", "reasons"], json!({
            "supported": boolean_schema(),
            "reasons": array_schema(schema_ref("ClientPlaybackCompatibilityCondition"))
        })),
        "ClientPlaybackCompatibilityCondition": enum_schema(&[
            "compatible",
            "direct_play_disabled",
            "media_technical_facts_missing",
            "container_unknown",
            "container_unsupported",
            "remux_container_unsupported",
            "video_codec_unsupported",
            "audio_codec_unsupported",
            "video_bitrate_unsupported",
            "video_resolution_unsupported",
            "video_hdr_unsupported",
            "audio_channels_unsupported",
            "subtitle_delivery_unsupported",
            "requested_transcode_output",
            "transcode_profile_unsupported",
            "policy_denied"
        ]),
        "ClientPlaybackDecisionReason": enum_schema(&[
            "compatible",
            "requested_transcode_output",
            "client_disabled_direct_play",
            "source_container_unknown",
            "client_container_unsupported",
            "source_codecs_unsupported",
            "policy_denied"
        ]),
        "ClientDirectPlayPlan": object_schema(&["source_id", "content_type", "supports_range_requests"], json!({
            "source_id": uuid_schema(),
            "content_type": string_schema(),
            "supports_range_requests": boolean_schema()
        })),
        "ClientTranscodePlan": object_schema(&["output_container", "video_codec", "audio_codec"], json!({
            "output_container": enum_schema(&["hls", "mp4", "mkv"]),
            "video_codec": nullable_string_schema(),
            "audio_codec": nullable_string_schema()
        })),
        "TranscodeSessionResponse": object_schema(&["session"], json!({
            "session": schema_ref("TranscodeSessionDto")
        })),
        "PlaybackSessionResponse": object_schema(&["session"], json!({
            "session": schema_ref("PlaybackSessionDto")
        })),
        "PlaybackSessionDto": object_schema(&["id", "source_id", "item_id", "mode", "state", "updated_at"], json!({
            "id": uuid_schema(),
            "source_id": uuid_schema(),
            "item_id": uuid_schema(),
            "mode": enum_schema(&["direct", "remux", "hls"]),
            "state": enum_schema(&["active", "paused", "cancel_requested", "cancelled", "ended", "failed"]),
            "transcode_session_id": nullable_uuid_schema(),
            "position_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "client_capabilities": nullable_ref("ClientPlaybackCapabilitiesDto"),
            "last_heartbeat_at": nullable_string_schema(),
            "started_at": nullable_string_schema(),
            "ended_at": nullable_string_schema(),
            "updated_at": string_schema()
        })),
        "ClientPlaybackCapabilitiesDto": object_schema(&["direct_play", "containers", "video_codecs", "audio_codecs"], json!({
            "direct_play": boolean_schema(),
            "containers": array_schema(string_schema()),
            "video_codecs": array_schema(string_schema()),
            "audio_codecs": array_schema(string_schema()),
            "max_video_bitrate": integer_schema("int64"),
            "max_width": integer_schema("int32"),
            "max_height": integer_schema("int32"),
            "max_audio_channels": integer_schema("int32"),
            "supports_hdr": boolean_schema(),
            "supports_subtitles": boolean_schema(),
            "hls_variant_policy": schema_ref("ClientHlsVariantPolicy"),
            "hls_segment_container": schema_ref("ClientHlsSegmentContainer")
        })),
        "ClientHlsVariantPolicy": enum_schema(&["single_variant", "adaptive"]),
        "ClientHlsSegmentContainer": enum_schema(&["mpeg_ts", "fmp4"]),
        "PlaybackSessionHeartbeatRequest": object_schema(&["state"], json!({
            "state": enum_schema(&["active", "paused", "cancel_requested", "cancelled", "ended", "failed"]),
            "position_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true})
        })),
        "RendererRegistrationRequest": object_schema(&["display_name", "target_kind", "network_scope", "transport_auth", "control_capabilities"], json!({
            "display_name": string_schema(),
            "target_kind": schema_ref("ClientPlaybackTargetKind"),
            "network_scope": schema_ref("ClientPlaybackTargetNetworkScope"),
            "transport_auth": schema_ref("ClientPlaybackTargetTransportAuth"),
            "media_capabilities": nullable_ref("ClientPlaybackCapabilitiesDto"),
            "control_capabilities": schema_ref("ClientRendererControlCapabilitiesDto"),
            "ttl_ms": json!({"type": "integer", "format": "int64", "nullable": true})
        })),
        "RendererHeartbeatRequest": object_schema(&["state"], json!({
            "state": schema_ref("ClientRendererSessionState"),
            "media_capabilities": nullable_ref("ClientPlaybackCapabilitiesDto"),
            "control_capabilities": nullable_ref("ClientRendererControlCapabilitiesDto"),
            "ttl_ms": json!({"type": "integer", "format": "int64", "nullable": true})
        })),
        "RendererSessionResponse": object_schema(&["renderer"], json!({
            "renderer": schema_ref("RendererSessionDto")
        })),
        "RendererSessionsResponse": object_schema(&["renderers", "page"], json!({
            "renderers": array_schema(schema_ref("RendererSessionDto")),
            "page": schema_ref("PageInfo")
        })),
        "RendererSessionDto": object_schema(&["id", "target_kind", "display_name", "network_scope", "transport_auth", "control_capabilities", "state", "updated_at"], json!({
            "id": uuid_schema(),
            "target_kind": schema_ref("ClientPlaybackTargetKind"),
            "display_name": string_schema(),
            "network_scope": schema_ref("ClientPlaybackTargetNetworkScope"),
            "transport_auth": schema_ref("ClientPlaybackTargetTransportAuth"),
            "media_capabilities": nullable_ref("ClientPlaybackCapabilitiesDto"),
            "control_capabilities": schema_ref("ClientRendererControlCapabilitiesDto"),
            "state": schema_ref("ClientRendererSessionState"),
            "active_playback_session_id": nullable_uuid_schema(),
            "last_seen_at": nullable_string_schema(),
            "expires_at": nullable_string_schema(),
            "updated_at": string_schema()
        })),
        "RendererCommandPollResponse": object_schema(&["command"], json!({
            "command": nullable_ref("RendererCommandDto")
        })),
        "RendererCommandResponse": object_schema(&["command"], json!({
            "command": schema_ref("RendererCommandDto")
        })),
        "RendererCommandDto": object_schema(&["id", "renderer_session_id", "command", "state", "created_at", "updated_at"], json!({
            "id": uuid_schema(),
            "renderer_session_id": uuid_schema(),
            "command": schema_ref("ClientRendererControlCommand"),
            "state": schema_ref("ClientRendererCommandState"),
            "item_id": nullable_uuid_schema(),
            "source_id": nullable_uuid_schema(),
            "playback_session_id": nullable_uuid_schema(),
            "position_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "volume_percent": json!({"type": "integer", "format": "int32", "nullable": true, "minimum": 0, "maximum": 100}),
            "transport": nullable_ref("RendererCommandTransportDto"),
            "created_at": string_schema(),
            "updated_at": string_schema()
        })),
        "RendererCommandTransportDto": object_schema(&["mode", "expires_at", "urls"], json!({
            "mode": schema_ref("RendererTransportMode"),
            "expires_at": string_schema(),
            "urls": array_schema(schema_ref("RendererCommandTransportUrlDto"))
        })),
        "RendererCommandTransportUrlDto": object_schema(&["kind", "url", "content_type", "supports_range_requests"], json!({
            "kind": schema_ref("RendererTransportUrlKind"),
            "url": string_schema(),
            "content_type": string_schema(),
            "supports_range_requests": boolean_schema()
        })),
        "RendererTransportMode": enum_schema(&["direct", "remux", "hls"]),
        "RendererTransportUrlKind": enum_schema(&["stream", "playlist", "segment_base"]),
        "RendererCommandCompletionRequest": object_schema(&["state"], json!({
            "state": schema_ref("ClientRendererCommandState"),
            "failure_message": nullable_string_schema()
        })),
        "RendererPlayCommandRequest": object_schema(&["source_id"], json!({
            "source_id": uuid_schema(),
            "position_ms": json!({"type": "integer", "format": "int64", "nullable": true})
        })),
        "RendererPlayCommandResponse": object_schema(&["command", "session"], json!({
            "command": schema_ref("RendererCommandDto"),
            "session": schema_ref("PlaybackSessionDto")
        })),
        "TranscodeSessionDto": object_schema(&["id", "source_id", "kind", "request_key", "state", "failure_category", "failure_message", "created_at", "updated_at", "started_at", "completed_at"], json!({
            "id": uuid_schema(),
            "source_id": uuid_schema(),
            "kind": enum_schema(&["remux", "hls_transcode"]),
            "request_key": string_schema(),
            "state": enum_schema(&["planned", "starting", "running", "cancel_requested", "cancelled", "failed", "finished"]),
            "failure_category": json!({"type": "string", "nullable": true, "enum": ["invalid_request", "runner", "timeout", "storage", "stale", "cancelled", "unknown"]}),
            "failure_message": nullable_string_schema(),
            "created_at": string_schema(),
            "updated_at": string_schema(),
            "started_at": nullable_string_schema(),
            "completed_at": nullable_string_schema()
        })),
        "UserPlaylistsResponse": object_schema(&["playlists", "page"], json!({
            "playlists": array_schema(schema_ref("UserPlaylistDto")),
            "page": schema_ref("PageInfo")
        })),
        "UserPlaylistResponse": object_schema(&["playlist"], json!({
            "playlist": schema_ref("UserPlaylistDto")
        })),
        "UserPlaylistItemsResponse": object_schema(&["playlist", "items", "page"], json!({
            "playlist": schema_ref("UserPlaylistDto"),
            "items": array_schema(schema_ref("UserPlaylistItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "UserPlaylistDeleteResponse": object_schema(&["playlist_id", "deleted"], json!({
            "playlist_id": uuid_schema(),
            "deleted": boolean_schema()
        })),
        "UserPlaylistDto": object_schema(&["id", "name", "visibility", "item_count", "created_at", "updated_at", "version"], json!({
            "id": uuid_schema(),
            "name": string_schema(),
            "visibility": enum_schema(&["private"]),
            "item_count": integer_schema("int32"),
            "created_at": string_schema(),
            "updated_at": string_schema(),
            "version": integer_schema("int64")
        })),
        "UserPlaylistItemDto": object_schema(&["playlist_id", "item_id", "position", "added_at", "item", "images"], json!({
            "playlist_id": uuid_schema(),
            "item_id": uuid_schema(),
            "position": integer_schema("int32"),
            "added_at": string_schema(),
            "item": schema_ref("MediaItemDto"),
            "images": array_schema(schema_ref("PublicImageRefDto"))
        })),
        "CreateUserPlaylistRequest": object_schema(&["name"], json!({
            "name": string_schema()
        })),
        "UpdateUserPlaylistRequest": object_schema(&["name"], json!({
            "name": string_schema(),
            "expected_version": integer_schema("int64")
        })),
        "AddUserPlaylistItemRequest": object_schema(&[], json!({
            "position": integer_schema("int32"),
            "expected_version": integer_schema("int64")
        })),
        "ReorderUserPlaylistItemsRequest": object_schema(&["item_ids"], json!({
            "item_ids": array_schema(uuid_schema()),
            "expected_version": integer_schema("int64")
        })),
        "UserPlaybackStateResponse": object_schema(&["state"], json!({
            "state": schema_ref("UserPlaybackStateDto")
        })),
        "ContinueWatchingResponse": object_schema(&["items", "page"], json!({
            "items": array_schema(schema_ref("ContinueWatchingItemDto")),
            "page": schema_ref("PageInfo")
        })),
        "ContinueWatchingItemDto": object_schema(&["item", "state", "images"], json!({
            "item": schema_ref("MediaItemDto"),
            "state": schema_ref("UserPlaybackStateDto"),
            "images": array_schema(schema_ref("PublicImageRefDto"))
        })),
        "UserPlaybackStateDto": object_schema(&["item_id", "source_id", "resume_position_ms", "duration_ms", "progress_percent", "watched", "watched_at", "last_played_at", "updated_at", "version"], json!({
            "item_id": uuid_schema(),
            "source_id": nullable_string_schema(),
            "resume_position_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "progress_percent": json!({"type": "number", "format": "float", "nullable": true}),
            "watched": boolean_schema(),
            "watched_at": nullable_string_schema(),
            "last_played_at": nullable_string_schema(),
            "updated_at": nullable_string_schema(),
            "version": integer_schema("int64")
        })),
        "UpdatePlaybackProgressRequest": object_schema(&["position_ms"], json!({
            "source_id": nullable_string_schema(),
            "position_ms": integer_schema("int64"),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "reported_at": nullable_string_schema()
        })),
        "SetWatchedStateRequest": object_schema(&["watched"], json!({
            "watched": boolean_schema(),
            "source_id": nullable_string_schema(),
            "position_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "marked_at": nullable_string_schema()
        })),
        "MediaItemDto": object_schema(&["id", "kind", "parent_id", "metadata"], json!({
            "id": uuid_schema(),
            "kind": schema_ref("ClientMediaKind"),
            "parent_id": nullable_string_schema(),
            "metadata": schema_ref("CanonicalMetadataDto")
        })),
        "ClientMediaKind": enum_schema(&["movie", "series", "season", "episode", "collection", "extra", "unknown"]),
        "CanonicalMetadataDto": object_schema(&["title", "original_title", "sort_title", "overview", "release_date", "runtime_minutes", "tagline", "genres", "tags", "ratings", "credits", "collections", "studios", "external_ids"], json!({
            "title": string_schema(),
            "original_title": nullable_string_schema(),
            "sort_title": nullable_string_schema(),
            "overview": nullable_string_schema(),
            "release_date": nullable_string_schema(),
            "runtime_minutes": json!({"type": "integer", "format": "int32", "nullable": true}),
            "tagline": nullable_string_schema(),
            "genres": array_schema(string_schema()),
            "tags": array_schema(string_schema()),
            "ratings": array_schema(schema_ref("ContentRatingDto")),
            "credits": array_schema(schema_ref("CreditDto")),
            "collections": array_schema(schema_ref("CollectionRefDto")),
            "studios": array_schema(schema_ref("StudioRefDto")),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "ContentRatingDto": object_schema(&["source", "value"], json!({"source": string_schema(), "value": string_schema()})),
        "CreditDto": object_schema(&["name", "role", "character", "order", "external_ids"], json!({
            "name": string_schema(),
            "role": string_schema(),
            "character": nullable_string_schema(),
            "order": json!({"type": "integer", "format": "int32", "nullable": true}),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "CollectionRefDto": object_schema(&["name", "overview", "sort_order", "external_ids"], json!({
            "name": string_schema(),
            "overview": nullable_string_schema(),
            "sort_order": json!({"type": "integer", "format": "int32", "nullable": true}),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "StudioRefDto": object_schema(&["name", "external_ids"], json!({
            "name": string_schema(),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "ExternalIdDto": object_schema(&["provider", "value"], json!({"provider": string_schema(), "value": string_schema()})),
        "MediaSourceDto": object_schema(&["id", "library_id", "item_id", "file_name", "size_bytes", "fingerprint"], json!({
            "id": uuid_schema(),
            "library_id": uuid_schema(),
            "item_id": uuid_schema(),
            "file_name": string_schema(),
            "size_bytes": json!({"type": "integer", "format": "int64", "nullable": true}),
            "fingerprint": nullable_string_schema()
        })),
        "MediaProbeDto": object_schema(&["duration_ms", "container", "bit_rate", "streams"], json!({
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "container": nullable_string_schema(),
            "bit_rate": json!({"type": "integer", "format": "int64", "nullable": true}),
            "streams": array_schema(schema_ref("MediaStreamDto"))
        })),
        "MediaStreamDto": object_schema(&["index", "kind", "origin", "codec", "language", "duration_ms", "bit_rate", "width", "height", "channels", "sample_rate", "disposition"], json!({
            "index": integer_schema("int32"),
            "kind": string_schema(),
            "origin": nullable_string_schema(),
            "codec": nullable_string_schema(),
            "language": nullable_string_schema(),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "bit_rate": json!({"type": "integer", "format": "int64", "nullable": true}),
            "width": json!({"type": "integer", "format": "int32", "nullable": true}),
            "height": json!({"type": "integer", "format": "int32", "nullable": true}),
            "channels": json!({"type": "integer", "format": "int32", "nullable": true}),
            "sample_rate": json!({"type": "integer", "format": "int32", "nullable": true}),
            "disposition": schema_ref("MediaStreamDispositionDto")
        })),
        "MediaStreamDispositionDto": object_schema(&["default", "forced", "hearing_impaired", "visual_impaired", "commentary", "attached_pic", "captions", "descriptions"], json!({
            "default": boolean_schema(),
            "forced": boolean_schema(),
            "hearing_impaired": boolean_schema(),
            "visual_impaired": boolean_schema(),
            "commentary": boolean_schema(),
            "attached_pic": boolean_schema(),
            "captions": boolean_schema(),
            "descriptions": boolean_schema()
        })),
        "PersonDto": object_schema(&["id", "name", "sort_name", "overview", "external_ids"], json!({
            "id": uuid_schema(),
            "name": string_schema(),
            "sort_name": nullable_string_schema(),
            "overview": nullable_string_schema(),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "ItemCreditDto": object_schema(&["item_id", "person_id", "role", "character", "sort_order"], json!({
            "item_id": uuid_schema(),
            "person_id": uuid_schema(),
            "role": string_schema(),
            "character": nullable_string_schema(),
            "sort_order": json!({"type": "integer", "format": "int32", "nullable": true})
        })),
        "GenreDto": object_schema(&["id", "name", "source"], json!({"id": uuid_schema(), "name": string_schema(), "source": string_schema()})),
        "ItemGenreDto": object_schema(&["item_id", "genre_id"], json!({"item_id": uuid_schema(), "genre_id": uuid_schema()})),
        "TagDto": object_schema(&["id", "name", "source"], json!({"id": uuid_schema(), "name": string_schema(), "source": string_schema()})),
        "ItemTagDto": object_schema(&["item_id", "tag_id"], json!({"item_id": uuid_schema(), "tag_id": uuid_schema()})),
        "CollectionItemDto": object_schema(&["collection_id", "item_id", "sort_order"], json!({
            "collection_id": uuid_schema(),
            "item_id": uuid_schema(),
            "sort_order": json!({"type": "integer", "format": "int32", "nullable": true})
        })),
        "ItemStudioDto": object_schema(&["item_id", "studio_id"], json!({"item_id": uuid_schema(), "studio_id": uuid_schema()})),
        "PublicImageRefDto": object_schema(&["id", "owner", "kind", "url", "width", "height", "language", "media_type", "etag"], json!({
            "id": uuid_schema(),
            "owner": json!({"type": "object", "additionalProperties": string_schema()}),
            "kind": string_schema(),
            "url": string_schema(),
            "width": json!({"type": "integer", "format": "int32", "nullable": true}),
            "height": json!({"type": "integer", "format": "int32", "nullable": true}),
            "language": nullable_string_schema(),
            "media_type": nullable_string_schema(),
            "etag": nullable_string_schema()
        }))
    })
}

fn object_schema(required: &[&str], properties: Value) -> Value {
    json!({
        "type": "object",
        "required": required,
        "properties": properties,
        "additionalProperties": false
    })
}

fn uuid_schema() -> Value {
    json!({"type": "string", "format": "uuid"})
}

fn nullable_uuid_schema() -> Value {
    json!({"type": "string", "format": "uuid", "nullable": true})
}

fn non_empty_array_schema(item_schema: Value) -> Value {
    json!({"type": "array", "items": item_schema, "minItems": 1})
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_addon_protocol::addon_runtime_paths;
    use nako_client_protocol::public_client_paths;

    #[test]
    fn public_openapi_paths_match_public_client_scope() {
        let document = public_openapi_v1();
        let paths = document["paths"].as_object().unwrap();

        assert_eq!(paths.len(), public_client_paths().len());
        for path in public_client_paths() {
            assert!(paths.contains_key(path), "missing public path {path}");
        }

        for excluded in [
            "/admin/v1/catalog/governance/items",
            "/admin/v1/overview",
            "/admin/v1/events",
            "/admin/v1/jobs",
            "/admin/v1/artwork/candidates/{candidate_id}/accept",
            "/admin/v1/artwork/ingests/process-next",
            "/admin/v1/playback/runtime",
            "/admin/v1/playback/sessions",
            "/admin/v1/playback/renderers",
            "/admin/v1/storage/staging",
            "/admin/v1/system/config",
            "/storage/backends",
            "/jobs/{job_id}",
            "/addons",
            "/webhooks/endpoints",
            "/automation/providers",
            "/metadata/providers",
        ] {
            assert!(
                !paths.contains_key(excluded),
                "excluded path leaked: {excluded}"
            );
        }

        for path in addon_runtime_paths() {
            assert!(
                !paths.contains_key(path),
                "Addon runtime path leaked into Public Client OpenAPI: {path}"
            );
        }
    }

    #[test]
    fn public_openapi_expresses_auth_version_errors_and_pagination() {
        let document = public_openapi_v1();

        assert_eq!(
            document["components"]["securitySchemes"]["BearerAuth"]["scheme"],
            "bearer"
        );
        assert_eq!(
            document["components"]["headers"]["NakoApiVersion"]["schema"]["enum"][0],
            API_VERSION
        );
        assert_eq!(
            document["components"]["headers"]["NakoPlaybackSessionId"]["schema"]["format"],
            "uuid"
        );
        assert!(
            document["components"]["responses"]["Unauthorized"]["headers"]
                .get("www-authenticate")
                .is_some()
        );
        assert!(
            document["components"]["parameters"]
                .as_object()
                .unwrap()
                .contains_key("Limit")
        );

        let health = &document["paths"]["/health"]["get"];
        assert!(health.get("security").is_none());

        let libraries = &document["paths"]["/libraries"]["get"];
        assert_eq!(libraries["security"][0]["BearerAuth"], json!([]));
        assert!(libraries["responses"].get("401").is_some());
    }

    #[test]
    fn public_openapi_does_not_reference_internal_or_admin_surfaces() {
        let serialized = public_openapi_v1_json().to_ascii_lowercase();

        for forbidden in [
            "nako_core",
            "nako-server",
            "nako_server",
            "output_path",
            "secret_env",
            "providerrawresponse",
            "raw cache",
            "/addons",
            "/addon",
            "/addon/v1",
            "/webhooks",
            "/automation",
            "/storage/backends",
            "/jobs",
            "/admin",
            "/admin/v1",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "public OpenAPI leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn public_openapi_image_contract_uses_public_refs_without_raw_locators() {
        managed_artwork_variant_openapi_contract_uses_safe_query_parameters();
    }

    #[test]
    fn managed_artwork_variant_openapi_contract_uses_safe_query_parameters() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();
        let serialized = public_openapi_v1_json().to_ascii_lowercase();

        assert!(schemas.contains_key("PublicImageRefDto"));
        assert!(!schemas.contains_key("ImageAssetDto"));
        assert!(!schemas.contains_key("ImageRefDto"));
        assert_eq!(
            document["paths"]["/images/{image_id}"]["get"]["responses"]["200"]["content"]["application/octet-stream"]
                ["schema"]["format"],
            "binary"
        );
        assert_eq!(
            document["paths"]["/sources/{source_id}/stream/hls/playlist.m3u8"]["get"]["responses"]
                ["200"]["headers"][PLAYBACK_SESSION_ID_HEADER]["$ref"],
            "#/components/headers/NakoPlaybackSessionId"
        );
        assert!(
            document["paths"]["/sources/{source_id}/stream/hls/playlist.m3u8"]["get"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|parameter| parameter["name"] == "start_position_ms")
        );
        assert!(
            document["paths"]["/sources/{source_id}/stream/hls/playlist.m3u8"]["get"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|parameter| parameter["name"] == "preferred_audio_language")
        );
        assert!(
            document["paths"]["/sources/{source_id}/stream/hls/playlist.m3u8"]["get"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|parameter| parameter["name"] == "preferred_subtitle_language")
        );
        assert_eq!(
            document["paths"]["/sources/{source_id}/stream/remux"]["get"]["responses"]["200"]["headers"]
                [PLAYBACK_SESSION_ID_HEADER]["$ref"],
            "#/components/headers/NakoPlaybackSessionId"
        );
        assert_eq!(
            document["paths"]["/sources/{source_id}/stream/remux"]["head"]["responses"]["200"]["headers"]
                [PLAYBACK_SESSION_ID_HEADER]["$ref"],
            "#/components/headers/NakoPlaybackSessionId"
        );
        assert!(
            document["paths"]["/images/{image_id}"]
                .get("head")
                .is_some()
        );
        let image_parameters = document["paths"]["/images/{image_id}"]["get"]["parameters"]
            .as_array()
            .unwrap();
        assert!(
            image_parameters
                .iter()
                .any(|parameter| { parameter["name"] == "width" && parameter["in"] == "query" })
        );
        assert!(
            image_parameters
                .iter()
                .any(|parameter| { parameter["name"] == "height" && parameter["in"] == "query" })
        );
        assert_eq!(
            document["components"]["schemas"]["ItemDetailResponse"]["properties"]["images"]["items"]
                ["$ref"],
            "#/components/schemas/PublicImageRefDto"
        );
        assert_eq!(
            document["components"]["schemas"]["ImagesResponse"]["properties"]["images"]["items"]["$ref"],
            "#/components/schemas/PublicImageRefDto"
        );
        assert!(
            document["components"]["schemas"]["CanonicalMetadataDto"]["properties"]
                .get("images")
                .is_none()
        );

        for forbidden in [
            "source_uri",
            "cache_uri",
            "storage_uri",
            "managed-artwork://",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "public OpenAPI image contract leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn public_openapi_browser_playback_ticket_contract_uses_safe_ticket_urls() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();
        let serialized = public_openapi_v1_json().to_ascii_lowercase();

        assert!(schemas.contains_key("BrowserPlaybackCapabilitiesDto"));
        assert!(schemas.contains_key("BrowserPlaybackTicketRequest"));
        assert!(schemas.contains_key("BrowserPlaybackTicketResponse"));
        assert!(schemas.contains_key("BrowserPlaybackUrlDto"));
        assert_eq!(
            document["paths"]["/sources/{source_id}/playback/browser-ticket"]["post"]["requestBody"]
                ["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/BrowserPlaybackTicketRequest"
        );
        assert_eq!(
            document["paths"]["/sources/{source_id}/playback/browser-ticket"]["post"]["responses"]
                ["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/BrowserPlaybackTicketResponse"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketRequest"]["properties"]["capabilities"]
                ["$ref"],
            "#/components/schemas/BrowserPlaybackCapabilitiesDto"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["properties"]["urls"]
                ["items"]["$ref"],
            "#/components/schemas/BrowserPlaybackUrlDto"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackUrlDto"]["properties"]["kind"]["enum"],
            json!(["stream", "playlist", "subtitle"])
        );
        assert_eq!(
            document["paths"]["/sources/{source_id}/subtitles/{stream_index}"]["get"]["operationId"],
            "getSourceSubtitle"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["properties"]["item_id"]
                ["format"],
            "uuid"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["properties"]["item_id"]
                ["nullable"],
            true
        );
        assert!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!("playback_session_id"))
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["properties"]["playback_session_id"]
                ["format"],
            "uuid"
        );
        assert_eq!(
            document["components"]["schemas"]["BrowserPlaybackTicketResponse"]["properties"]["playback_session_id"]
                ["nullable"],
            true
        );

        for forbidden in ["locator", "source_uri", "cache_uri", "storage_uri"] {
            assert!(
                !serialized.contains(forbidden),
                "browser playback ticket contract leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn public_openapi_playback_decision_uses_typed_reason_and_capabilities() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();

        assert!(schemas.contains_key("ClientPlaybackDecisionReason"));
        assert!(schemas.contains_key("ClientPlaybackDecisionReport"));
        assert!(schemas.contains_key("ClientPlaybackCapabilityEvaluation"));
        assert!(schemas.contains_key("ClientPlaybackCompatibilityCondition"));
        assert!(schemas.contains_key("ClientPlaybackCapabilitiesDto"));
        assert!(schemas.contains_key("ClientPlaybackTargetDto"));
        assert!(schemas.contains_key("ClientPlaybackDenialDto"));
        assert!(schemas.contains_key("ClientPlaybackPermission"));
        assert!(!schemas.contains_key("PlaybackSessionClientCapabilitiesDto"));
        assert_eq!(
            document["components"]["schemas"]["PlaybackDecisionResponse"]["properties"]["target"]["$ref"],
            "#/components/schemas/ClientPlaybackTargetDto"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDecision"]["properties"]["reason"]["$ref"],
            "#/components/schemas/ClientPlaybackDecisionReason"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDecision"]["properties"]["report"]["$ref"],
            "#/components/schemas/ClientPlaybackDecisionReport"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDecisionReport"]["properties"]["direct_play"]
                ["$ref"],
            "#/components/schemas/ClientPlaybackCapabilityEvaluation"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackCapabilityEvaluation"]["properties"]["reasons"]
                ["items"]["$ref"],
            "#/components/schemas/ClientPlaybackCompatibilityCondition"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDecision"]["properties"]["denial"]["allOf"]
                [0]["$ref"],
            "#/components/schemas/ClientPlaybackDenialDto"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackTargetDto"]["properties"]["kind"]["$ref"],
            "#/components/schemas/ClientPlaybackTargetKind"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackTargetDto"]["properties"]["transport_auth"]
                ["$ref"],
            "#/components/schemas/ClientPlaybackTargetTransportAuth"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDenialDto"]["properties"]["permission"]
                ["$ref"],
            "#/components/schemas/ClientPlaybackPermission"
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackDecisionReason"]["enum"],
            json!([
                "compatible",
                "requested_transcode_output",
                "client_disabled_direct_play",
                "source_container_unknown",
                "client_container_unsupported",
                "source_codecs_unsupported",
                "policy_denied"
            ])
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackCompatibilityCondition"]["enum"],
            json!([
                "compatible",
                "direct_play_disabled",
                "media_technical_facts_missing",
                "container_unknown",
                "container_unsupported",
                "remux_container_unsupported",
                "video_codec_unsupported",
                "audio_codec_unsupported",
                "video_bitrate_unsupported",
                "video_resolution_unsupported",
                "video_hdr_unsupported",
                "audio_channels_unsupported",
                "subtitle_delivery_unsupported",
                "requested_transcode_output",
                "transcode_profile_unsupported",
                "policy_denied"
            ])
        );
        assert_eq!(
            document["components"]["schemas"]["ClientPlaybackPermission"]["enum"],
            json!([
                "media_playback",
                "direct_play",
                "remux",
                "audio_transcode",
                "video_transcode",
                "remote_playback",
                "remote_control",
                "cast"
            ])
        );
        assert_eq!(
            document["components"]["schemas"]["PlaybackSessionDto"]["properties"]["client_capabilities"]
                ["allOf"][0]["$ref"],
            "#/components/schemas/ClientPlaybackCapabilitiesDto"
        );
        assert_eq!(
            document["components"]["schemas"]["PlaybackSessionDto"]["properties"]["client_capabilities"]
                ["nullable"],
            true
        );
    }

    #[test]
    fn public_openapi_renderer_contract_exposes_control_surface_without_principals() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();

        assert!(schemas.contains_key("RendererRegistrationRequest"));
        assert!(schemas.contains_key("RendererSessionDto"));
        assert!(schemas.contains_key("RendererCommandDto"));
        assert!(schemas.contains_key("RendererCommandTransportDto"));
        assert!(schemas.contains_key("RendererPlayCommandRequest"));
        assert_eq!(
            document["paths"]["/renderers"]["post"]["requestBody"]["content"]["application/json"]["schema"]
                ["$ref"],
            "#/components/schemas/RendererRegistrationRequest"
        );
        assert_eq!(
            document["paths"]["/renderers/{renderer_session_id}/commands/next"]["post"]["responses"]
                ["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/RendererCommandPollResponse"
        );
        assert_eq!(
            document["paths"]["/renderers/{renderer_session_id}/commands/play"]["post"]["responses"]
                ["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/RendererPlayCommandResponse"
        );
        assert_eq!(
            document["components"]["schemas"]["RendererSessionDto"]["properties"]["control_capabilities"]
                ["$ref"],
            "#/components/schemas/ClientRendererControlCapabilitiesDto"
        );
        assert_eq!(
            document["components"]["schemas"]["RendererCommandDto"]["properties"]["state"]["$ref"],
            "#/components/schemas/ClientRendererCommandState"
        );
        assert_eq!(
            document["components"]["schemas"]["RendererCommandDto"]["properties"]["transport"]["allOf"]
                [0]["$ref"],
            "#/components/schemas/RendererCommandTransportDto"
        );
        assert_eq!(
            document["components"]["schemas"]["RendererCommandTransportDto"]["properties"]["urls"]
                ["items"]["$ref"],
            "#/components/schemas/RendererCommandTransportUrlDto"
        );

        let renderer_contract = json!({
            "paths": {
                "/renderers": document["paths"]["/renderers"].clone(),
                "/renderers/{renderer_session_id}/heartbeat": document["paths"]["/renderers/{renderer_session_id}/heartbeat"].clone(),
                "/renderers/{renderer_session_id}/commands/next": document["paths"]["/renderers/{renderer_session_id}/commands/next"].clone(),
                "/renderers/{renderer_session_id}/commands/play": document["paths"]["/renderers/{renderer_session_id}/commands/play"].clone(),
                "/renderers/{renderer_session_id}/commands/{command_id}/complete": document["paths"]["/renderers/{renderer_session_id}/commands/{command_id}/complete"].clone()
            },
            "schemas": {
                "RendererRegistrationRequest": document["components"]["schemas"]["RendererRegistrationRequest"].clone(),
                "RendererHeartbeatRequest": document["components"]["schemas"]["RendererHeartbeatRequest"].clone(),
                "RendererSessionDto": document["components"]["schemas"]["RendererSessionDto"].clone(),
                "RendererCommandDto": document["components"]["schemas"]["RendererCommandDto"].clone(),
                "RendererCommandTransportDto": document["components"]["schemas"]["RendererCommandTransportDto"].clone(),
                "RendererCommandTransportUrlDto": document["components"]["schemas"]["RendererCommandTransportUrlDto"].clone(),
                "RendererTransportMode": document["components"]["schemas"]["RendererTransportMode"].clone(),
                "RendererTransportUrlKind": document["components"]["schemas"]["RendererTransportUrlKind"].clone(),
                "RendererCommandCompletionRequest": document["components"]["schemas"]["RendererCommandCompletionRequest"].clone(),
                "RendererPlayCommandRequest": document["components"]["schemas"]["RendererPlayCommandRequest"].clone(),
                "RendererPlayCommandResponse": document["components"]["schemas"]["RendererPlayCommandResponse"].clone()
            }
        })
        .to_string()
        .to_ascii_lowercase();

        for forbidden in ["principal_id", "owner_principal", "payload_json", "token"] {
            assert!(
                !renderer_contract.contains(forbidden),
                "public renderer contract leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn public_openapi_user_playback_contract_uses_me_routes_without_principal_ids() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();
        let serialized = public_openapi_v1_json().to_ascii_lowercase();

        assert_eq!(
            document["paths"]["/users/me/playback-state/items/{item_id}"]["get"]["responses"]["200"]
                ["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/UserPlaybackStateResponse"
        );
        assert_eq!(
            document["paths"]["/users/me/playback-state/items/{item_id}/progress"]["put"]["requestBody"]
                ["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/UpdatePlaybackProgressRequest"
        );
        assert_eq!(
            document["paths"]["/users/me/playback-state/items/{item_id}/watched"]["put"]["requestBody"]
                ["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/SetWatchedStateRequest"
        );
        assert!(schemas.contains_key("ContinueWatchingResponse"));
        assert!(schemas.contains_key("UserPlaybackStateDto"));
        assert_eq!(
            document["components"]["schemas"]["ContinueWatchingItemDto"]["properties"]["item"]["$ref"],
            "#/components/schemas/MediaItemDto"
        );
        assert_eq!(
            document["components"]["schemas"]["ContinueWatchingItemDto"]["properties"]["images"]["items"]
                ["$ref"],
            "#/components/schemas/PublicImageRefDto"
        );

        for forbidden in ["principal_id", "user_id", "local-admin"] {
            assert!(
                !serialized.contains(forbidden),
                "public user playback contract leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn public_openapi_user_playlist_contract_uses_me_routes_without_collection_or_hls_state() {
        let document = public_openapi_v1();
        let schemas = document["components"]["schemas"].as_object().unwrap();

        assert_eq!(
            document["paths"]["/users/me/playlists"]["get"]["responses"]["200"]["content"]["application/json"]
                ["schema"]["$ref"],
            "#/components/schemas/UserPlaylistsResponse"
        );
        assert_eq!(
            document["paths"]["/users/me/playlists"]["post"]["requestBody"]["content"]["application/json"]
                ["schema"]["$ref"],
            "#/components/schemas/CreateUserPlaylistRequest"
        );
        assert_eq!(
            document["paths"]["/users/me/playlists/{playlist_id}/items/reorder"]["put"]["requestBody"]
                ["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/ReorderUserPlaylistItemsRequest"
        );
        assert!(schemas.contains_key("UserPlaylistDto"));
        assert!(schemas.contains_key("UserPlaylistItemDto"));
        assert_eq!(
            document["components"]["schemas"]["UserPlaylistItemDto"]["properties"]["item"]["$ref"],
            "#/components/schemas/MediaItemDto"
        );
        assert_eq!(
            document["components"]["schemas"]["UserPlaylistItemDto"]["properties"]["images"]["items"]
                ["$ref"],
            "#/components/schemas/PublicImageRefDto"
        );

        let playlist_contract = json!({
            "paths": {
                "/users/me/playlists": document["paths"]["/users/me/playlists"].clone(),
                "/users/me/playlists/{playlist_id}": document["paths"]["/users/me/playlists/{playlist_id}"].clone(),
                "/users/me/playlists/{playlist_id}/items": document["paths"]["/users/me/playlists/{playlist_id}/items"].clone(),
                "/users/me/playlists/{playlist_id}/items/{item_id}": document["paths"]["/users/me/playlists/{playlist_id}/items/{item_id}"].clone(),
                "/users/me/playlists/{playlist_id}/items/reorder": document["paths"]["/users/me/playlists/{playlist_id}/items/reorder"].clone()
            },
            "schemas": {
                "UserPlaylistsResponse": document["components"]["schemas"]["UserPlaylistsResponse"].clone(),
                "UserPlaylistResponse": document["components"]["schemas"]["UserPlaylistResponse"].clone(),
                "UserPlaylistDto": document["components"]["schemas"]["UserPlaylistDto"].clone(),
                "UserPlaylistItemDto": document["components"]["schemas"]["UserPlaylistItemDto"].clone()
            }
        })
        .to_string()
        .to_ascii_lowercase();

        for forbidden in ["principal_id", "user_id", "collection_id", "playlist.m3u8"] {
            assert!(
                !playlist_contract.contains(forbidden),
                "public user playlist contract leaked forbidden term: {forbidden}"
            );
        }
    }
}
