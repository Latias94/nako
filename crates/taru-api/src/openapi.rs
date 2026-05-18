use serde_json::{Map, Value, json};

use crate::{API_VERSION, API_VERSION_HEADER, ClientErrorCode};

#[must_use]
pub fn public_openapi_v1() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Taru Public Client API",
            "version": API_VERSION,
            "description": "Public client HTTP contract for Taru API v1."
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
                "TaruApiVersion": {
                    "description": "Taru public API compatibility version.",
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
        "/sources/{source_id}/stream".to_owned(),
        json!({
            "get": binary_get("streamSource", "Stream direct-play bytes for one source.", vec![path_parameter("source_id", "Media source id."), range_header_parameter()]),
            "head": empty_head("headStreamSource", "Preflight direct-play stream headers for one source.", vec![path_parameter("source_id", "Media source id."), range_header_parameter()])
        }),
    );
    paths.insert(
        "/sources/{source_id}/stream/remux".to_owned(),
        json!({
            "get": binary_get(
                "remuxStreamSource",
                "Run or reuse remux output and stream bytes.",
                remux_parameters("source_id")
            )
        }),
    );
    paths.insert(
        "/sources/{source_id}/stream/hls/playlist.m3u8".to_owned(),
        json!({
            "get": text_get(
                "hlsPlaylistSource",
                "Start or reuse HLS transcode and return a playlist.",
                playback_parameters("source_id")
            )
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}".to_owned(),
        json!({
            "get": json_get("getPlaybackSession", "Get one playback session.", "playback", vec![path_parameter("session_id", "Playback session id.")], schema_ref("TranscodeSessionResponse"))
        }),
    );
    paths.insert(
        "/playback/sessions/{session_id}/cancel".to_owned(),
        json!({
            "post": json_post("cancelPlaybackSession", "Request playback session cancellation.", "playback", vec![path_parameter("session_id", "Playback session id.")], schema_ref("TranscodeSessionResponse"))
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

fn binary_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
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

fn text_get(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
        parameters,
        json!({
            "description": "Text response.",
            "headers": api_version_headers(),
            "content": {
                "application/vnd.apple.mpegurl": {
                    "schema": string_schema()
                }
            }
        }),
    )
}

fn empty_head(operation_id: &str, summary: &str, parameters: Vec<Value>) -> Value {
    operation(
        operation_id,
        summary,
        "playback",
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
                API_VERSION_HEADER: header_ref("TaruApiVersion"),
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
    ]
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
    json!({API_VERSION_HEADER: header_ref("TaruApiVersion")})
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
        "MetadataProfileDto": object_schema(&["item_kinds", "local_readers", "metadata_providers", "image_providers", "language", "country", "refresh_mode", "local_metadata_policy"], json!({
            "item_kinds": array_schema(schema_ref("ClientMediaKind")),
            "local_readers": array_schema(string_schema()),
            "metadata_providers": array_schema(string_schema()),
            "image_providers": array_schema(string_schema()),
            "language": nullable_string_schema(),
            "country": nullable_string_schema(),
            "refresh_mode": enum_schema(&["none", "validation_only", "default", "missing_only", "full_refresh"]),
            "local_metadata_policy": enum_schema(&["disabled", "read_only", "local_first", "remote_first", "write_sidecar"])
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
            "images": array_schema(schema_ref("ImageAssetDto"))
        })),
        "ItemCreditsResponse": object_schema(&["item_id", "credits", "people"], json!({
            "item_id": uuid_schema(),
            "credits": array_schema(schema_ref("ItemCreditDto")),
            "people": array_schema(schema_ref("PersonDto"))
        })),
        "ImagesResponse": object_schema(&["item_id", "images"], json!({
            "item_id": uuid_schema(),
            "images": array_schema(schema_ref("ImageAssetDto"))
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
        "SourceProbeResponse": object_schema(&["source_id", "probe"], json!({
            "source_id": uuid_schema(),
            "probe": schema_ref("MediaProbeDto")
        })),
        "PlaybackDecisionResponse": object_schema(&["source", "probe", "decision"], json!({
            "source": schema_ref("MediaSourceDto"),
            "probe": nullable_ref("MediaProbeDto"),
            "decision": schema_ref("ClientPlaybackDecision")
        })),
        "ClientPlaybackDecision": object_schema(&["mode", "reason", "direct_play", "transcode_plan"], json!({
            "mode": enum_schema(&["direct_play", "remux", "transcode"]),
            "reason": string_schema(),
            "direct_play": nullable_ref("ClientDirectPlayPlan"),
            "transcode_plan": nullable_ref("ClientTranscodePlan")
        })),
        "ClientDirectPlayPlan": object_schema(&["source_id", "content_type", "supports_range_requests"], json!({
            "source_id": uuid_schema(),
            "content_type": string_schema(),
            "supports_range_requests": boolean_schema()
        })),
        "ClientTranscodePlan": object_schema(&["output_container", "video_codec", "audio_codec", "hardware_acceleration"], json!({
            "output_container": enum_schema(&["hls", "mp4", "mkv"]),
            "video_codec": nullable_string_schema(),
            "audio_codec": nullable_string_schema(),
            "hardware_acceleration": enum_schema(&["none", "vaapi", "nvenc", "quick_sync"])
        })),
        "TranscodeSessionResponse": object_schema(&["session"], json!({
            "session": schema_ref("TranscodeSessionDto")
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
        "MediaItemDto": object_schema(&["id", "kind", "parent_id", "metadata"], json!({
            "id": uuid_schema(),
            "kind": schema_ref("ClientMediaKind"),
            "parent_id": nullable_string_schema(),
            "metadata": schema_ref("CanonicalMetadataDto")
        })),
        "ClientMediaKind": enum_schema(&["movie", "series", "season", "episode", "collection", "extra", "unknown"]),
        "CanonicalMetadataDto": object_schema(&["title", "original_title", "sort_title", "overview", "release_date", "runtime_minutes", "tagline", "genres", "tags", "ratings", "images", "credits", "collections", "studios", "external_ids"], json!({
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
            "images": array_schema(schema_ref("ImageRefDto")),
            "credits": array_schema(schema_ref("CreditDto")),
            "collections": array_schema(schema_ref("CollectionRefDto")),
            "studios": array_schema(schema_ref("StudioRefDto")),
            "external_ids": array_schema(schema_ref("ExternalIdDto"))
        })),
        "ContentRatingDto": object_schema(&["source", "value"], json!({"source": string_schema(), "value": string_schema()})),
        "ImageRefDto": object_schema(&["kind", "uri", "provider", "width", "height", "language"], json!({
            "kind": string_schema(),
            "uri": string_schema(),
            "provider": string_schema(),
            "width": json!({"type": "integer", "format": "int32", "nullable": true}),
            "height": json!({"type": "integer", "format": "int32", "nullable": true}),
            "language": nullable_string_schema()
        })),
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
        "MediaStreamDto": object_schema(&["index", "kind", "codec", "language", "duration_ms", "bit_rate", "width", "height", "channels", "sample_rate"], json!({
            "index": integer_schema("int32"),
            "kind": string_schema(),
            "codec": nullable_string_schema(),
            "language": nullable_string_schema(),
            "duration_ms": json!({"type": "integer", "format": "int64", "nullable": true}),
            "bit_rate": json!({"type": "integer", "format": "int64", "nullable": true}),
            "width": json!({"type": "integer", "format": "int32", "nullable": true}),
            "height": json!({"type": "integer", "format": "int32", "nullable": true}),
            "channels": json!({"type": "integer", "format": "int32", "nullable": true}),
            "sample_rate": json!({"type": "integer", "format": "int32", "nullable": true})
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
        "ImageAssetDto": object_schema(&["id", "owner", "kind", "source_uri", "provider", "cache_uri", "width", "height", "language", "selected", "content_hash", "etag"], json!({
            "id": uuid_schema(),
            "owner": string_schema(),
            "kind": string_schema(),
            "source_uri": string_schema(),
            "provider": string_schema(),
            "cache_uri": nullable_string_schema(),
            "width": json!({"type": "integer", "format": "int32", "nullable": true}),
            "height": json!({"type": "integer", "format": "int32", "nullable": true}),
            "language": nullable_string_schema(),
            "selected": boolean_schema(),
            "content_hash": nullable_string_schema(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use taru_client_protocol::public_client_paths;

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
            "/admin/v1/playback/runtime",
            "/admin/v1/playback/sessions",
            "/admin/v1/storage/staging",
            "/admin/v1/system/config",
            "/storage/backends",
            "/jobs/{job_id}",
            "/addons",
            "/addon/v1/access-check",
            "/addon/v1/side-effects",
            "/webhooks/endpoints",
            "/automation/providers",
            "/metadata/providers",
        ] {
            assert!(
                !paths.contains_key(excluded),
                "excluded path leaked: {excluded}"
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
            document["components"]["headers"]["TaruApiVersion"]["schema"]["enum"][0],
            API_VERSION
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
            "taru_core",
            "taru-server",
            "taru_server",
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
}
