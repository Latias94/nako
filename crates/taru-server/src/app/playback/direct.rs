use std::{fmt, path::PathBuf};

use futures_util::StreamExt;
use taru_core::{MediaSource, Result, TaruError};
use taru_streaming::{
    DirectPlayRangeRequest, DirectPlayResponsePlan, content_type_for_file_name,
    plan_direct_play_response,
};
use taru_vfs::{ByteRange, ReadStream, StorageBackend, StorageUri};
use tokio::sync::OwnedSemaphorePermit;

use super::input::local_source_path_and_len;

pub struct DirectPlaySourcePlan {
    pub source: MediaSource,
    pub body: DirectPlaySourceBody,
    pub response: DirectPlayResponsePlan,
}

impl fmt::Debug for DirectPlaySourcePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectPlaySourcePlan")
            .field("source", &self.source)
            .field("body", &self.body)
            .field("response", &self.response)
            .finish()
    }
}

pub enum DirectPlaySourceBody {
    LocalPath(PathBuf),
    Stream(DirectPlayStreamBody),
    Empty,
}

pub struct DirectPlayStreamBody {
    pub stream: ReadStream,
    _permit: Option<OwnedSemaphorePermit>,
}

impl DirectPlayStreamBody {
    pub(super) fn new(stream: ReadStream, permit: Option<OwnedSemaphorePermit>) -> Self {
        Self {
            stream,
            _permit: permit,
        }
    }

    pub(crate) fn unbudgeted(stream: ReadStream) -> Self {
        Self::new(stream, None)
    }

    pub fn into_read_stream(self) -> ReadStream {
        let Self {
            stream,
            _permit: permit,
        } = self;
        let ReadStream { uri, range, body } = stream;
        let body = match permit {
            Some(permit) => body
                .map(move |chunk| {
                    let _permit = &permit;
                    chunk
                })
                .boxed(),
            None => body,
        };

        ReadStream::new(uri, range, body)
    }
}

impl fmt::Debug for DirectPlaySourceBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalPath(path) => formatter.debug_tuple("LocalPath").field(path).finish(),
            Self::Stream(stream) => formatter.debug_tuple("Stream").field(stream).finish(),
            Self::Empty => formatter.write_str("Empty"),
        }
    }
}

impl fmt::Debug for DirectPlayStreamBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectPlayStreamBody")
            .field("stream", &self.stream)
            .field("budgeted", &self._permit.is_some())
            .finish()
    }
}

pub(crate) async fn plan_direct_play_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    range_request: DirectPlayRangeRequest,
) -> Result<(DirectPlayResponsePlan, DirectPlaySourceBody)> {
    let response =
        plan_direct_play_response_with_backend(source, uri, backend, range_request).await?;

    if response.is_range_not_satisfiable() {
        return Ok((response, DirectPlaySourceBody::Empty));
    }

    match local_source_path_and_len(source, uri, backend).await {
        Ok((local_path, _total_len)) => {
            return Ok((response, DirectPlaySourceBody::LocalPath(local_path)));
        }
        Err(TaruError::Unsupported(_)) => {}
        Err(err) => return Err(err),
    }

    let range = response.range.map(|range| ByteRange {
        offset: range.start,
        length: Some(range.len()),
    });
    let stream = backend.stream_range(uri, range).await?;

    Ok((
        response,
        DirectPlaySourceBody::Stream(DirectPlayStreamBody::unbudgeted(stream)),
    ))
}

pub(super) async fn plan_direct_play_response_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    range_request: DirectPlayRangeRequest,
) -> Result<DirectPlayResponsePlan> {
    let metadata = backend.stat(uri).await?;
    let total_len = metadata.len.ok_or_else(|| TaruError::Storage {
        uri: source.locator.clone(),
        message: "direct play requires a known source length".to_owned(),
    })?;
    let content_type = content_type_for_file_name(&source.file_name).to_owned();

    Ok(plan_direct_play_response(
        total_len,
        content_type,
        range_request,
    ))
}

pub(super) fn should_budget_remote_stream(uri: &StorageUri) -> bool {
    uri.scheme() != "local"
}
