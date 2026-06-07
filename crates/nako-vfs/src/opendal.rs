use async_trait::async_trait;
use futures_util::StreamExt;
use nako_core::{NakoError, Result, StorageErrorKind};

use crate::{
    ByteRange, ObjectKind, ObjectMetadata, ReadRange, ReadStream, StorageBackend,
    StorageCapabilities, StorageUri, VirtualFile,
};

#[derive(Clone)]
pub struct OpenDalStorageBackend {
    scheme: &'static str,
    operator: opendal::Operator,
}

impl OpenDalStorageBackend {
    pub fn memory_for_proof(scheme: &'static str) -> Result<Self> {
        if scheme.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "OpenDAL proof scheme cannot be empty".to_owned(),
            });
        }

        let operator = opendal::Operator::new(opendal::services::Memory::default())
            .map_err(|err| map_opendal_error("opendal", &err))?
            .finish();

        Ok(Self { scheme, operator })
    }

    #[cfg(test)]
    async fn write_proof_object(
        &self,
        uri: &StorageUri,
        content: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let path = self.path_for(uri)?;
        self.operator
            .write(&path, content.into())
            .await
            .map(|_| ())
            .map_err(|err| map_opendal_error(uri.as_str(), &err))
    }

    fn path_for(&self, uri: &StorageUri) -> Result<String> {
        if uri.scheme() != self.scheme {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "OpenDAL proof backend only accepts '{}' uris, got '{}'",
                    self.scheme,
                    uri.scheme()
                ),
            });
        }
        let raw_path = uri.path_part();
        if !raw_path.starts_with('/')
            || uri.as_str().contains('@')
            || raw_path.contains('?')
            || raw_path.contains('#')
        {
            return Err(NakoError::InvalidInput {
                message:
                    "OpenDAL proof storage uri must not contain authority, query, or fragment material"
                        .to_owned(),
            });
        }

        let path = raw_path.trim_start_matches('/');
        if path
            .split('/')
            .any(|segment| segment == ".." || segment == "." || segment.contains('\\'))
        {
            return Err(NakoError::storage_security_violation(
                uri.to_string(),
                "OpenDAL proof storage path failed authority validation",
            ));
        }

        Ok(path.to_owned())
    }

    fn uri_for_path(&self, path: &str) -> Result<StorageUri> {
        StorageUri::from_parts(self.scheme, path)
    }
}

#[async_trait]
impl StorageBackend for OpenDalStorageBackend {
    fn scheme(&self) -> &'static str {
        self.scheme
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        let path = self.path_for(uri)?;
        let metadata = self
            .operator
            .stat(&path)
            .await
            .map_err(|err| map_opendal_error(uri.as_str(), &err))?;

        Ok(metadata_for(self, path.as_str(), metadata)?)
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        let path = normalize_directory_path(&self.path_for(uri)?);
        let mut entries = self
            .operator
            .list(&path)
            .await
            .map_err(|err| map_opendal_error(uri.as_str(), &err))?;
        entries.retain(|entry| is_direct_child(&path, entry.path()));

        let mut output = Vec::with_capacity(entries.len());
        for entry in entries {
            output.push(metadata_for(self, entry.path(), entry.metadata().clone())?);
        }
        output.sort_by(|left, right| left.uri.as_str().cmp(right.uri.as_str()));
        Ok(output)
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        let metadata = self.stat(uri).await?;
        if metadata.kind != ObjectKind::File {
            return Err(NakoError::InvalidInput {
                message: format!("cannot open non-file OpenDAL proof uri: {uri}"),
            });
        }
        if let (Some(range), Some(len)) = (range, metadata.len) {
            range.validate_for_len(uri, len)?;
        }

        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        let metadata = self.stat(uri).await?;
        if metadata.kind != ObjectKind::File {
            return Err(NakoError::InvalidInput {
                message: format!("cannot read non-file OpenDAL proof uri: {uri}"),
            });
        }
        if let (Some(range), Some(len)) = (range, metadata.len) {
            range.validate_for_len(uri, len)?;
        }

        let path = self.path_for(uri)?;
        let bytes = match range {
            Some(range) => {
                self.operator
                    .read_with(&path)
                    .range(range_to_std(range, uri, metadata.len)?)
                    .await
            }
            None => self.operator.read(&path).await,
        }
        .map_err(|err| map_opendal_error(uri.as_str(), &err))?
        .to_vec();

        validate_read_length(uri, range, metadata.len, bytes.len())?;
        Ok(ReadRange {
            uri: uri.clone(),
            range,
            bytes,
        })
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        let metadata = self.stat(uri).await?;
        if metadata.kind != ObjectKind::File {
            return Err(NakoError::InvalidInput {
                message: format!("cannot stream non-file OpenDAL proof uri: {uri}"),
            });
        }
        if let (Some(range), Some(len)) = (range, metadata.len) {
            range.validate_for_len(uri, len)?;
        }

        let path = self.path_for(uri)?;
        let stream_range = match range {
            Some(range) => range_to_std(range, uri, metadata.len)?,
            None => {
                0..metadata.len.ok_or_else(|| NakoError::InvalidInput {
                    message: format!("OpenDAL proof stream requires known object length: {uri}"),
                })?
            }
        };
        let body_uri = uri.to_string();
        let body = self
            .operator
            .reader(&path)
            .await
            .map_err(|err| map_opendal_error(uri.as_str(), &err))?
            .into_bytes_stream(stream_range)
            .await
            .map_err(|err| map_opendal_error(uri.as_str(), &err))?
            .map(move |chunk| {
                chunk.map_err(|err| {
                    NakoError::storage(
                        body_uri.clone(),
                        storage_kind_for_io_error(&err),
                        "OpenDAL proof storage stream failed",
                    )
                })
            })
            .boxed();

        Ok(ReadStream::new(uri.clone(), range, body))
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        let read = self.read_range(uri, None).await?;
        String::from_utf8(read.bytes).map_err(|err| NakoError::InvalidInput {
            message: format!("OpenDAL proof object is not valid utf-8: {err}"),
        })
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        let _ = (uri, content);
        Err(NakoError::Unsupported(
            "OpenDAL proof backend does not expose product writes",
        ))
    }
}

fn metadata_for(
    backend: &OpenDalStorageBackend,
    path: &str,
    metadata: opendal::Metadata,
) -> Result<ObjectMetadata> {
    let kind = match metadata.mode() {
        opendal::EntryMode::FILE => ObjectKind::File,
        opendal::EntryMode::DIR => ObjectKind::Directory,
        _ => ObjectKind::Other,
    };
    let len = (kind == ObjectKind::File).then_some(metadata.content_length());
    let etag = metadata.etag().map(ToOwned::to_owned);
    let modified_at = metadata.last_modified().map(|value| value.to_string());
    let fingerprint = etag
        .as_ref()
        .map(|etag| format!("opendal:etag={etag}"))
        .or_else(|| len.map(|len| format!("opendal:size={len}")));

    Ok(ObjectMetadata {
        uri: backend.uri_for_path(path)?,
        kind,
        len,
        modified_at,
        etag,
        fingerprint,
        capabilities: opendal_capabilities(kind),
        cache: None,
    })
}

fn normalize_directory_path(path: &str) -> String {
    if path.is_empty() || path.ends_with('/') {
        path.to_owned()
    } else {
        format!("{path}/")
    }
}

fn is_direct_child(parent: &str, child: &str) -> bool {
    if child == parent {
        return false;
    }
    let Some(rest) = child.strip_prefix(parent) else {
        return false;
    };
    !rest.is_empty() && !rest.trim_end_matches('/').contains('/')
}

fn opendal_capabilities(kind: ObjectKind) -> StorageCapabilities {
    let base = StorageCapabilities::SEEKABLE
        | StorageCapabilities::RANGE_READABLE
        | StorageCapabilities::EXPENSIVE_LISTING
        | StorageCapabilities::REMOTE_LATENCY;

    match kind {
        ObjectKind::File | ObjectKind::Directory => base,
        ObjectKind::Symlink | ObjectKind::Other => StorageCapabilities::REMOTE_LATENCY,
    }
}

fn range_to_std(
    range: ByteRange,
    uri: &StorageUri,
    object_len: Option<u64>,
) -> Result<std::ops::Range<u64>> {
    let end = match (range.length, object_len) {
        (Some(length), _) => {
            range
                .offset
                .checked_add(length)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: format!("range overflows file length: {uri}"),
                })?
        }
        (None, Some(len)) => len,
        (None, None) => {
            return Err(NakoError::InvalidInput {
                message: format!("open-ended range requires known object length: {uri}"),
            });
        }
    };
    Ok(range.offset..end)
}

fn validate_read_length(
    uri: &StorageUri,
    range: Option<ByteRange>,
    total_len: Option<u64>,
    actual_len: usize,
) -> Result<()> {
    let expected_len = match (range, total_len) {
        (Some(range), Some(total_len)) => Some(range.resolved_len(uri, total_len)?),
        (Some(range), None) => range.length,
        (None, total_len) => total_len,
    };
    let Some(expected_len) = expected_len else {
        return Ok(());
    };
    let actual_len = u64::try_from(actual_len).map_err(|err| {
        NakoError::storage(
            uri.to_string(),
            StorageErrorKind::Unknown,
            format!("read length does not fit u64: {err}"),
        )
    })?;

    if actual_len != expected_len {
        return Err(NakoError::storage_staging_validation_mismatch(
            uri.to_string(),
            format!("OpenDAL proof read returned {actual_len} bytes, expected {expected_len}"),
        ));
    }

    Ok(())
}

fn map_opendal_error(uri: &str, err: &opendal::Error) -> NakoError {
    let kind = match err.kind() {
        opendal::ErrorKind::NotFound => {
            return NakoError::NotFound {
                entity: "storage_object",
                id: uri.to_owned(),
            };
        }
        opendal::ErrorKind::PermissionDenied => StorageErrorKind::Unauthorized,
        opendal::ErrorKind::RateLimited => StorageErrorKind::RateLimited,
        opendal::ErrorKind::RangeNotSatisfied => StorageErrorKind::StagingValidationMismatch,
        opendal::ErrorKind::IsADirectory
        | opendal::ErrorKind::NotADirectory
        | opendal::ErrorKind::Unexpected
        | opendal::ErrorKind::Unsupported => StorageErrorKind::Unknown,
        _ => StorageErrorKind::Unknown,
    };
    NakoError::storage(uri.to_owned(), kind, "OpenDAL proof storage request failed")
}

fn storage_kind_for_io_error(err: &std::io::Error) -> StorageErrorKind {
    match err.kind() {
        std::io::ErrorKind::PermissionDenied => StorageErrorKind::Unauthorized,
        std::io::ErrorKind::TimedOut => StorageErrorKind::Timeout,
        std::io::ErrorKind::UnexpectedEof => StorageErrorKind::StagingValidationMismatch,
        _ => StorageErrorKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn opendal_backend_stats_lists_and_reads_ranges() {
        let backend = OpenDalStorageBackend::memory_for_proof("opendal").unwrap();
        let movie = StorageUri::from_parts("opendal", "Movies/Demo.mkv").unwrap();
        backend
            .write_proof_object(&movie, b"nako-media")
            .await
            .unwrap();

        let stat = backend.stat(&movie).await.unwrap();
        assert_eq!(stat.uri, movie);
        assert_eq!(stat.kind, ObjectKind::File);
        assert_eq!(stat.len, Some(10));
        assert!(
            stat.capabilities
                .contains(StorageCapabilities::RANGE_READABLE)
        );
        assert!(
            stat.capabilities
                .contains(StorageCapabilities::REMOTE_LATENCY)
        );

        let read = backend
            .read_range(
                &movie,
                Some(ByteRange {
                    offset: 5,
                    length: Some(5),
                }),
            )
            .await
            .unwrap();
        assert_eq!(read.bytes, b"media");

        let mut stream = backend.stream_range(&movie, None).await.unwrap();
        let mut streamed = Vec::new();
        while let Some(chunk) = stream.body.next().await {
            streamed.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(streamed, b"nako-media");
    }

    #[tokio::test]
    async fn opendal_backend_lists_direct_children_only() {
        let backend = OpenDalStorageBackend::memory_for_proof("opendal").unwrap();
        backend
            .write_proof_object(
                &StorageUri::from_parts("opendal", "Movies/Demo.mkv").unwrap(),
                b"demo",
            )
            .await
            .unwrap();
        backend
            .write_proof_object(
                &StorageUri::from_parts("opendal", "Movies/Nested/Hidden.mkv").unwrap(),
                b"hidden",
            )
            .await
            .unwrap();

        let entries = backend
            .list(&StorageUri::from_parts("opendal", "Movies").unwrap())
            .await
            .unwrap();
        let uris = entries
            .iter()
            .map(|entry| entry.uri.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            uris,
            vec!["opendal:///Movies/Demo.mkv", "opendal:///Movies/Nested/"]
        );
        assert!(!uris.contains(&"opendal:///Movies/Nested/Hidden.mkv"));
    }

    #[tokio::test]
    async fn opendal_backend_rejects_unsafe_uri_authority() {
        let backend = OpenDalStorageBackend::memory_for_proof("opendal").unwrap();

        let traversal = StorageUri::parse("opendal:///../secret.mkv").unwrap();
        let err = backend.stat(&traversal).await.unwrap_err();
        assert_eq!(
            err.storage_failure_class(),
            Some(nako_core::StorageFailureClass::Security)
        );

        let authority =
            StorageUri::parse("opendal://user:password@example.test/movie.mkv").unwrap();
        let err = backend.stat(&authority).await.unwrap_err();
        assert!(err.to_string().contains("must not contain authority"));

        let naked_authority = StorageUri::parse("opendal://example.test/movie.mkv").unwrap();
        let err = backend.stat(&naked_authority).await.unwrap_err();
        assert!(err.to_string().contains("must not contain authority"));
    }
}
