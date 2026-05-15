use std::{io::SeekFrom, path::Path as FsPath};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use taru_core::TaruError;
use taru_streaming::{
    DirectPlayRangeRequest, DirectPlayResponsePlan, DirectPlayResponseStatus,
    parse_http_range_header,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::app::DirectPlaySourceBody;

use super::ApiResult;

pub(super) fn direct_play_range_request(headers: &HeaderMap) -> DirectPlayRangeRequest {
    let Some(value) = headers.get(header::RANGE) else {
        return DirectPlayRangeRequest::None;
    };

    let Ok(value) = value.to_str() else {
        return DirectPlayRangeRequest::Invalid;
    };

    match parse_http_range_header(value) {
        Ok(range) => DirectPlayRangeRequest::Range(range),
        Err(_) => DirectPlayRangeRequest::Invalid,
    }
}

pub(super) fn empty_direct_play_response(plan: &DirectPlayResponsePlan) -> Response {
    let mut response = Body::empty().into_response();
    apply_direct_play_headers(&mut response, plan);
    response
}

pub(super) fn hls_playlist_response(body: String) -> Response {
    let body_len = body.len();
    let mut response = Body::from(body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&body_len.to_string()).expect("content length is a valid header"),
    );
    response
}

pub(super) async fn stream_local_file_response(
    path: &FsPath,
    uri: &str,
    plan: &DirectPlayResponsePlan,
) -> ApiResult<Response> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|err| TaruError::Storage {
            uri: uri.to_owned(),
            message: format!("failed to open stream source: {err}"),
        })?;

    if plan.seek_offset > 0 {
        file.seek(SeekFrom::Start(plan.seek_offset))
            .await
            .map_err(|err| TaruError::Storage {
                uri: uri.to_owned(),
                message: format!("failed to seek stream source: {err}"),
            })?;
    }

    let stream = ReaderStream::new(file.take(plan.body_len));
    let mut response = Body::from_stream(stream).into_response();
    apply_direct_play_headers(&mut response, plan);

    Ok(response)
}

pub(super) async fn stream_direct_play_response(
    body: DirectPlaySourceBody,
    uri: &str,
    plan: &DirectPlayResponsePlan,
) -> ApiResult<Response> {
    match body {
        DirectPlaySourceBody::LocalPath(path) => stream_local_file_response(&path, uri, plan).await,
        DirectPlaySourceBody::Stream(stream) => {
            let mut response = Body::from_stream(stream.body).into_response();
            apply_direct_play_headers(&mut response, plan);
            Ok(response)
        }
        DirectPlaySourceBody::Empty => Ok(empty_direct_play_response(plan)),
    }
}

fn apply_direct_play_headers(response: &mut Response, plan: &DirectPlayResponsePlan) {
    *response.status_mut() = direct_play_status_code(plan.status);
    let headers = response.headers_mut();
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&plan.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&plan.body_len.to_string())
            .expect("content length is a valid header"),
    );

    if let Some(content_range) = &plan.content_range {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(content_range).expect("content range is a valid header"),
        );
    }
}

fn direct_play_status_code(status: DirectPlayResponseStatus) -> StatusCode {
    match status {
        DirectPlayResponseStatus::Ok => StatusCode::OK,
        DirectPlayResponseStatus::PartialContent => StatusCode::PARTIAL_CONTENT,
        DirectPlayResponseStatus::RangeNotSatisfiable => StatusCode::RANGE_NOT_SATISFIABLE,
    }
}
