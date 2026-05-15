use std::{fmt, path::PathBuf};

use futures_util::StreamExt;
use taru_api::PlaybackDecisionResponse;
use taru_core::{MediaProbeRepository, MediaSource, MediaSourceId, Result, TaruError};
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan,
    content_type_for_file_name, decide_playback, plan_direct_play_response,
};
use taru_vfs::{ByteRange, ReadStream, StorageBackend, StorageUri};
use tokio::sync::OwnedSemaphorePermit;

use super::{TaruApp, local_source_path_and_len};

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
    fn new(stream: ReadStream, permit: Option<OwnedSemaphorePermit>) -> Self {
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

impl TaruApp {
    pub async fn get_source_playback_decision(
        &self,
        source_id: MediaSourceId,
        client: ClientPlaybackCapabilities,
    ) -> Result<PlaybackDecisionResponse> {
        let source = self.get_source_or_not_found(source_id).await?;
        let probe = self.inner.store.get_media_probe(source.id).await?;
        let decision = decide_playback(&source, probe.as_ref(), &client);

        Ok(PlaybackDecisionResponse {
            source,
            probe,
            decision,
        })
    }

    pub async fn plan_direct_play(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlaySourcePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let stream_permit = if should_budget_remote_stream(&uri) {
            Some(self.acquire_remote_stream_permit().await?)
        } else {
            None
        };
        let (response, body) =
            plan_direct_play_with_backend(&source, &uri, backend.as_ref(), range_request).await?;
        let body = match body {
            DirectPlaySourceBody::Stream(stream) => DirectPlaySourceBody::Stream(
                DirectPlayStreamBody::new(stream.stream, stream_permit),
            ),
            other => other,
        };

        Ok(DirectPlaySourcePlan {
            source,
            body,
            response,
        })
    }

    pub async fn plan_direct_play_preflight(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlayResponsePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;

        plan_direct_play_response_with_backend(&source, &uri, backend.as_ref(), range_request).await
    }
}

pub(super) async fn plan_direct_play_with_backend(
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

async fn plan_direct_play_response_with_backend(
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

fn should_budget_remote_stream(uri: &StorageUri) -> bool {
    uri.scheme() != "local"
}
