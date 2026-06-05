use futures_util::StreamExt;
use nako_core::{
    LibraryId, MediaSourceId, NakoError, Result, SourceFingerprintEscalationAction,
    SourceFingerprintEscalationDecision, SourceFingerprintEscalationReason,
    SourceFingerprintEvidence, SourceFingerprintEvidenceKind, SourceFingerprintPolicyInput,
};
use nako_vfs::{ByteRange, StorageBackend, StorageUri};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS: &str = "disk.scan.source_fingerprint_hash";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprintHashMode {
    Partial { prefix_bytes: u64 },
    Full,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintHashRequest {
    pub uri: StorageUri,
    pub mode: SourceFingerprintHashMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintHashJobInput {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub source_scheme: String,
    pub mode: SourceFingerprintHashMode,
}

impl SourceFingerprintHashJobInput {
    pub fn new(
        library_id: LibraryId,
        source_id: MediaSourceId,
        source_scheme: impl Into<String>,
        mode: SourceFingerprintHashMode,
    ) -> Result<Self> {
        let source_scheme = source_scheme.into();
        validate_source_scheme(&source_scheme)?;

        Ok(Self {
            library_id,
            source_id,
            source_scheme,
            mode,
        })
    }

    pub fn from_request(
        library_id: LibraryId,
        source_id: MediaSourceId,
        request: &SourceFingerprintHashRequest,
    ) -> Result<Self> {
        Self::new(library_id, source_id, request.uri.scheme(), request.mode)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintHashReport {
    pub mode: SourceFingerprintHashMode,
    pub evidence: SourceFingerprintEvidence,
    pub bytes_hashed: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprintHashSchedulingPolicy {
    Disabled,
    Enabled { partial_prefix_bytes: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintHashSchedulingInput {
    pub source_uri: StorageUri,
    pub decision: SourceFingerprintEscalationDecision,
    pub policy: SourceFingerprintHashSchedulingPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFingerprintHashSchedulingPlan {
    pub diagnostic: SourceFingerprintHashSchedulingDiagnostic,
    pub request: Option<SourceFingerprintHashRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintHashSchedulingDiagnostic {
    pub source_scheme: String,
    pub action: SourceFingerprintEscalationAction,
    pub reason: SourceFingerprintEscalationReason,
    pub evidence_kind: SourceFingerprintEvidenceKind,
    pub confidence_milli: u16,
    pub stale: bool,
    pub candidate_count: u32,
    pub scheduled: bool,
    pub mode: Option<SourceFingerprintHashMode>,
}

pub fn plan_source_fingerprint_hash_scheduling(
    input: SourceFingerprintHashSchedulingInput,
) -> Result<SourceFingerprintHashSchedulingPlan> {
    let source_scheme = input.source_uri.scheme().to_owned();
    let mode = hash_mode_for_decision(input.decision.action, input.policy)?;
    let request = mode.map(|mode| SourceFingerprintHashRequest {
        uri: input.source_uri,
        mode,
    });

    Ok(SourceFingerprintHashSchedulingPlan {
        diagnostic: SourceFingerprintHashSchedulingDiagnostic {
            source_scheme,
            action: input.decision.action,
            reason: input.decision.reason,
            evidence_kind: input.decision.evidence_kind,
            confidence_milli: input.decision.confidence_milli,
            stale: input.decision.stale,
            candidate_count: input.decision.candidate_count,
            scheduled: request.is_some(),
            mode,
        },
        request,
    })
}

#[derive(Debug)]
pub struct SourceFingerprintHashExecutor<B> {
    backend: B,
}

impl<B> SourceFingerprintHashExecutor<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    #[must_use]
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

impl<B> SourceFingerprintHashExecutor<B>
where
    B: StorageBackend,
{
    pub async fn execute(
        &self,
        request: SourceFingerprintHashRequest,
    ) -> Result<SourceFingerprintHashReport> {
        match request.mode {
            SourceFingerprintHashMode::Partial { prefix_bytes } => {
                self.execute_partial(request.uri, prefix_bytes).await
            }
            SourceFingerprintHashMode::Full => self.execute_full(request.uri).await,
        }
    }

    async fn execute_partial(
        &self,
        uri: StorageUri,
        prefix_bytes: u64,
    ) -> Result<SourceFingerprintHashReport> {
        if prefix_bytes == 0 {
            return Err(NakoError::InvalidInput {
                message: "partial source fingerprint hash prefix must be greater than zero"
                    .to_owned(),
            });
        }

        let range = ByteRange {
            offset: 0,
            length: Some(prefix_bytes),
        };
        let read = self.backend.read_range(&uri, Some(range)).await?;
        let bytes_hashed = usize_to_u64(read.bytes.len(), &uri)?;
        let digest = sha256_hex(&read.bytes);
        let backend_fingerprint =
            format!("partial-sha256:{digest}:bytes={bytes_hashed}:prefix={prefix_bytes}");
        let evidence = source_fingerprint_evidence(
            self.backend.scheme(),
            SourceFingerprintEvidenceKind::BackendFingerprint,
            &backend_fingerprint,
        );

        Ok(SourceFingerprintHashReport {
            mode: SourceFingerprintHashMode::Partial { prefix_bytes },
            evidence,
            bytes_hashed,
        })
    }

    async fn execute_full(&self, uri: StorageUri) -> Result<SourceFingerprintHashReport> {
        let mut stream = self.backend.stream_range(&uri, None).await?;
        let mut hasher = Sha256::new();
        let mut bytes_hashed = 0_u64;

        while let Some(chunk) = stream.body.next().await {
            let chunk = chunk?;
            bytes_hashed = bytes_hashed
                .checked_add(usize_to_u64(chunk.len(), &uri)?)
                .ok_or_else(|| {
                    NakoError::storage_unknown(
                        uri.to_string(),
                        "source fingerprint stream length overflowed u64",
                    )
                })?;
            hasher.update(chunk.as_ref());
        }

        let digest = format!("{:x}", hasher.finalize());
        let backend_fingerprint = format!("sha256:{digest}");
        let evidence = source_fingerprint_evidence(
            self.backend.scheme(),
            SourceFingerprintEvidenceKind::ContentHash,
            &backend_fingerprint,
        );

        Ok(SourceFingerprintHashReport {
            mode: SourceFingerprintHashMode::Full,
            evidence,
            bytes_hashed,
        })
    }
}

fn source_fingerprint_evidence(
    scheme: &str,
    expected_kind: SourceFingerprintEvidenceKind,
    backend_fingerprint: &str,
) -> SourceFingerprintEvidence {
    let evidence = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
        scheme,
        size_bytes: None,
        modified_at: None,
        etag: None,
        backend_fingerprint: Some(backend_fingerprint),
        stale: false,
    });

    debug_assert_eq!(evidence.kind, expected_kind);
    evidence
}

fn hash_mode_for_decision(
    action: SourceFingerprintEscalationAction,
    policy: SourceFingerprintHashSchedulingPolicy,
) -> Result<Option<SourceFingerprintHashMode>> {
    let partial_prefix_bytes = match policy {
        SourceFingerprintHashSchedulingPolicy::Disabled => return Ok(None),
        SourceFingerprintHashSchedulingPolicy::Enabled {
            partial_prefix_bytes,
        } => partial_prefix_bytes,
    };

    if partial_prefix_bytes == 0 {
        return Err(NakoError::InvalidInput {
            message: "source fingerprint hash partial prefix must be greater than zero".to_owned(),
        });
    }

    Ok(match action {
        SourceFingerprintEscalationAction::None => None,
        SourceFingerprintEscalationAction::PartialHash => {
            Some(SourceFingerprintHashMode::Partial {
                prefix_bytes: partial_prefix_bytes,
            })
        }
        SourceFingerprintEscalationAction::FullHash => Some(SourceFingerprintHashMode::Full),
    })
}

fn validate_source_scheme(source_scheme: &str) -> Result<()> {
    if source_scheme.is_empty() || source_scheme.trim() != source_scheme {
        return Err(NakoError::InvalidInput {
            message: "source fingerprint hash job source scheme must be non-empty and trimmed"
                .to_owned(),
        });
    }
    if !source_scheme
        .chars()
        .next()
        .is_some_and(|value| value.is_ascii_alphabetic())
        || !source_scheme.chars().all(is_storage_scheme_character)
    {
        return Err(NakoError::InvalidInput {
            message:
                "source fingerprint hash job source scheme must contain only scheme-safe ASCII characters"
                    .to_owned(),
        });
    }

    Ok(())
}

fn is_storage_scheme_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '+' | '-' | '.' | '_')
}

fn usize_to_u64(value: usize, uri: &StorageUri) -> Result<u64> {
    u64::try_from(value).map_err(|err| {
        NakoError::storage_unknown(
            uri.to_string(),
            format!("source fingerprint byte count does not fit u64: {err}"),
        )
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use nako_core::{NakoError, StorageErrorKind};
    use nako_vfs::{
        ObjectKind, ObjectMetadata, ReadRange, ReadStream, StorageCapabilities, VirtualFile,
    };

    use super::*;

    #[test]
    fn hash_scheduling_disabled_reports_decision_without_request() {
        let uri = StorageUri::from_parts("local", "Movies/Secret Path/Demo.mkv").unwrap();
        let plan = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: uri,
            decision: partial_hash_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Disabled,
        })
        .unwrap();

        assert_eq!(plan.request, None);
        assert_eq!(plan.diagnostic.source_scheme, "local");
        assert_eq!(
            plan.diagnostic.action,
            SourceFingerprintEscalationAction::PartialHash
        );
        assert_eq!(
            plan.diagnostic.reason,
            SourceFingerprintEscalationReason::ConfirmSingleWeakCandidate
        );
        assert_eq!(
            plan.diagnostic.evidence_kind,
            SourceFingerprintEvidenceKind::BackendFingerprint
        );
        assert_eq!(plan.diagnostic.confidence_milli, 700);
        assert!(!plan.diagnostic.stale);
        assert_eq!(plan.diagnostic.candidate_count, 1);
        assert!(!plan.diagnostic.scheduled);
        assert_eq!(plan.diagnostic.mode, None);
    }

    #[test]
    fn hash_scheduling_none_escalation_remains_diagnostic_only() {
        let uri = StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap();
        let plan = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: uri,
            decision: no_escalation_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: 4_096,
            },
        })
        .unwrap();

        assert_eq!(plan.request, None);
        assert_eq!(
            plan.diagnostic.action,
            SourceFingerprintEscalationAction::None
        );
        assert_eq!(plan.diagnostic.mode, None);
        assert!(!plan.diagnostic.scheduled);
    }

    #[test]
    fn hash_scheduling_partial_builds_configured_prefix_request() {
        let uri = StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap();
        let plan = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: uri.clone(),
            decision: partial_hash_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: 8_192,
            },
        })
        .unwrap();

        let mode = SourceFingerprintHashMode::Partial {
            prefix_bytes: 8_192,
        };
        assert_eq!(plan.diagnostic.source_scheme, "local");
        assert_eq!(plan.diagnostic.mode, Some(mode));
        assert!(plan.diagnostic.scheduled);
        assert_eq!(
            plan.request,
            Some(SourceFingerprintHashRequest { uri, mode })
        );
    }

    #[test]
    fn hash_scheduling_full_builds_full_hash_request() {
        let uri = StorageUri::from_parts("webdav", "Movies/Demo.mkv").unwrap();
        let plan = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: uri.clone(),
            decision: full_hash_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: 8_192,
            },
        })
        .unwrap();

        assert_eq!(plan.diagnostic.source_scheme, "webdav");
        assert_eq!(plan.diagnostic.mode, Some(SourceFingerprintHashMode::Full));
        assert!(plan.diagnostic.scheduled);
        assert_eq!(
            plan.request,
            Some(SourceFingerprintHashRequest {
                uri,
                mode: SourceFingerprintHashMode::Full,
            })
        );
    }

    #[test]
    fn hash_scheduling_rejects_zero_partial_prefix_without_locator_leak() {
        let err = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: StorageUri::from_parts("local", "Users/Frankorz/Secret Path/Demo.mkv")
                .unwrap(),
            decision: partial_hash_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: 0,
            },
        })
        .unwrap_err();

        let NakoError::InvalidInput { message } = err else {
            panic!("expected invalid input");
        };
        assert_eq!(
            message,
            "source fingerprint hash partial prefix must be greater than zero"
        );
        assert!(!message.contains("Secret Path"));
        assert!(!message.contains("local:///"));
    }

    #[test]
    fn hash_scheduling_diagnostic_serialization_redacts_locator_path() {
        let plan = plan_source_fingerprint_hash_scheduling(SourceFingerprintHashSchedulingInput {
            source_uri: StorageUri::from_parts(
                "local",
                "Users/Frankorz/Secret Path/Hidden Movie.mkv",
            )
            .unwrap(),
            decision: partial_hash_decision(),
            policy: SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: 16_384,
            },
        })
        .unwrap();

        let diagnostic = serde_json::to_string(&plan.diagnostic).unwrap();
        assert!(diagnostic.contains(r#""source_scheme":"local""#));
        assert!(!diagnostic.contains("Hidden Movie"));
        assert!(!diagnostic.contains("Secret Path"));
        assert!(!diagnostic.contains("Frankorz"));
        assert!(!diagnostic.contains("local:///"));
    }

    #[test]
    fn hash_job_input_from_request_serializes_without_locator_path() {
        let library_id = LibraryId::new();
        let source_id = MediaSourceId::new();
        let request = SourceFingerprintHashRequest {
            uri: StorageUri::from_parts("local", "Users/Frankorz/Secret Path/Hidden Movie.mkv")
                .unwrap(),
            mode: SourceFingerprintHashMode::Full,
        };

        let input =
            SourceFingerprintHashJobInput::from_request(library_id, source_id, &request).unwrap();

        assert_eq!(
            input,
            SourceFingerprintHashJobInput {
                library_id,
                source_id,
                source_scheme: "local".to_owned(),
                mode: SourceFingerprintHashMode::Full,
            }
        );

        let serialized = serde_json::to_string(&input).unwrap();
        let round_trip: SourceFingerprintHashJobInput = serde_json::from_str(&serialized).unwrap();
        assert_eq!(round_trip, input);
        assert!(serialized.contains(r#""source_scheme":"local""#));
        assert!(serialized.contains(r#""mode":"full""#));
        assert!(!serialized.contains("Hidden Movie"));
        assert!(!serialized.contains("Secret Path"));
        assert!(!serialized.contains("Frankorz"));
        assert!(!serialized.contains("local:///"));
        assert!(!serialized.contains(r#""uri""#));
        assert!(!serialized.contains("sha256"));
        assert!(!serialized.contains("etag"));
    }

    #[test]
    fn hash_job_input_carries_partial_mode() {
        let input = SourceFingerprintHashJobInput::new(
            LibraryId::new(),
            MediaSourceId::new(),
            "webdav",
            SourceFingerprintHashMode::Partial {
                prefix_bytes: 65_536,
            },
        )
        .unwrap();

        let serialized = serde_json::to_string(&input).unwrap();
        let round_trip: SourceFingerprintHashJobInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(round_trip, input);
        assert_eq!(round_trip.source_scheme, "webdav");
        assert_eq!(
            round_trip.mode,
            SourceFingerprintHashMode::Partial {
                prefix_bytes: 65_536,
            }
        );
        assert!(serialized.contains(r#""partial""#));
        assert!(serialized.contains(r#""prefix_bytes":65536"#));
    }

    #[test]
    fn hash_job_input_rejects_locator_like_source_scheme_without_leaking_value() {
        let err = SourceFingerprintHashJobInput::new(
            LibraryId::new(),
            MediaSourceId::new(),
            "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv",
            SourceFingerprintHashMode::Full,
        )
        .unwrap_err();

        let NakoError::InvalidInput { message } = err else {
            panic!("expected invalid input");
        };
        assert_eq!(
            message,
            "source fingerprint hash job source scheme must contain only scheme-safe ASCII characters"
        );
        assert!(!message.contains("Hidden Movie"));
        assert!(!message.contains("Secret Path"));
        assert!(!message.contains("Frankorz"));
        assert!(!message.contains("local:///"));
    }

    #[tokio::test]
    async fn partial_hash_reads_configured_prefix_and_returns_redacted_backend_evidence() {
        let backend = HashTestBackend::new(b"abcdef".to_vec());
        let executor = SourceFingerprintHashExecutor::new(backend.clone());
        let uri = StorageUri::from_parts("test", "Movies/Demo.mkv").unwrap();

        let report = executor
            .execute(SourceFingerprintHashRequest {
                uri: uri.clone(),
                mode: SourceFingerprintHashMode::Partial { prefix_bytes: 3 },
            })
            .await
            .unwrap();

        assert_eq!(
            backend.read_ranges(),
            vec![Some(ByteRange {
                offset: 0,
                length: Some(3)
            })]
        );
        assert_eq!(backend.stream_ranges(), Vec::<Option<ByteRange>>::new());
        assert_eq!(
            report.mode,
            SourceFingerprintHashMode::Partial { prefix_bytes: 3 }
        );
        assert_eq!(report.bytes_hashed, 3);
        assert_eq!(
            report.evidence.kind,
            SourceFingerprintEvidenceKind::BackendFingerprint
        );
        assert_eq!(report.evidence.confidence_milli, 700);
        let fingerprint = report.evidence.fingerprint.as_deref().unwrap();
        assert!(fingerprint.starts_with("source:v1:backend_fingerprint:sha256:"));
        assert!(!fingerprint.contains(&sha256_hex(b"abc")));
        assert!(!fingerprint.contains("abc"));
        assert!(!fingerprint.contains(uri.as_str()));
    }

    #[tokio::test]
    async fn full_hash_streams_without_read_range_and_returns_redacted_content_hash() {
        let backend = HashTestBackend::new(b"abcdef".to_vec());
        let executor = SourceFingerprintHashExecutor::new(backend.clone());
        let uri = StorageUri::from_parts("test", "Movies/Demo.mkv").unwrap();

        let report = executor
            .execute(SourceFingerprintHashRequest {
                uri: uri.clone(),
                mode: SourceFingerprintHashMode::Full,
            })
            .await
            .unwrap();

        assert_eq!(backend.read_ranges(), Vec::<Option<ByteRange>>::new());
        assert_eq!(backend.stream_ranges(), vec![None]);
        assert_eq!(report.mode, SourceFingerprintHashMode::Full);
        assert_eq!(report.bytes_hashed, 6);
        assert_eq!(
            report.evidence.kind,
            SourceFingerprintEvidenceKind::ContentHash
        );
        assert_eq!(report.evidence.confidence_milli, 1_000);
        let fingerprint = report.evidence.fingerprint.as_deref().unwrap();
        assert!(fingerprint.starts_with("source:v1:content_hash:sha256:"));
        assert!(!fingerprint.contains(&sha256_hex(b"abcdef")));
        assert!(!fingerprint.contains(uri.as_str()));
    }

    #[tokio::test]
    async fn hash_execution_propagates_backend_read_failures() {
        let expected = NakoError::storage_timeout("test:///Movies/Demo.mkv", "range timed out");
        let backend = HashTestBackend::new(b"abcdef".to_vec()).with_read_error(expected.clone());
        let executor = SourceFingerprintHashExecutor::new(backend);

        let err = executor
            .execute(SourceFingerprintHashRequest {
                uri: StorageUri::from_parts("test", "Movies/Demo.mkv").unwrap(),
                mode: SourceFingerprintHashMode::Partial { prefix_bytes: 3 },
            })
            .await
            .unwrap_err();

        assert_eq!(err, expected);
    }

    #[tokio::test]
    async fn hash_execution_propagates_unsupported_and_stream_failures() {
        let unsupported_executor = SourceFingerprintHashExecutor::new(UnsupportedRangeBackend);
        let unsupported_partial = unsupported_executor
            .execute(SourceFingerprintHashRequest {
                uri: StorageUri::from_parts("unsupported", "Movies/Demo.mkv").unwrap(),
                mode: SourceFingerprintHashMode::Partial { prefix_bytes: 3 },
            })
            .await
            .unwrap_err();
        assert!(matches!(unsupported_partial, NakoError::Unsupported(_)));

        let unsupported_full = unsupported_executor
            .execute(SourceFingerprintHashRequest {
                uri: StorageUri::from_parts("unsupported", "Movies/Demo.mkv").unwrap(),
                mode: SourceFingerprintHashMode::Full,
            })
            .await
            .unwrap_err();
        assert!(matches!(unsupported_full, NakoError::Unsupported(_)));

        let expected = NakoError::storage_timeout("test:///Movies/Demo.mkv", "stream timed out");
        let backend = HashTestBackend::new(b"abcdef".to_vec()).with_stream_error(expected.clone());
        let executor = SourceFingerprintHashExecutor::new(backend);
        let err = executor
            .execute(SourceFingerprintHashRequest {
                uri: StorageUri::from_parts("test", "Movies/Demo.mkv").unwrap(),
                mode: SourceFingerprintHashMode::Full,
            })
            .await
            .unwrap_err();

        assert_eq!(err, expected);
    }

    fn no_escalation_decision() -> SourceFingerprintEscalationDecision {
        SourceFingerprintEscalationDecision {
            action: SourceFingerprintEscalationAction::None,
            reason: SourceFingerprintEscalationReason::StrongEvidence,
            evidence_kind: SourceFingerprintEvidenceKind::ContentHash,
            confidence_milli: 1_000,
            stale: false,
            candidate_count: 1,
        }
    }

    fn partial_hash_decision() -> SourceFingerprintEscalationDecision {
        SourceFingerprintEscalationDecision {
            action: SourceFingerprintEscalationAction::PartialHash,
            reason: SourceFingerprintEscalationReason::ConfirmSingleWeakCandidate,
            evidence_kind: SourceFingerprintEvidenceKind::BackendFingerprint,
            confidence_milli: 700,
            stale: false,
            candidate_count: 1,
        }
    }

    fn full_hash_decision() -> SourceFingerprintEscalationDecision {
        SourceFingerprintEscalationDecision {
            action: SourceFingerprintEscalationAction::FullHash,
            reason: SourceFingerprintEscalationReason::DisambiguateMultipleCandidates,
            evidence_kind: SourceFingerprintEvidenceKind::SizeAndEtag,
            confidence_milli: 800,
            stale: false,
            candidate_count: 2,
        }
    }

    #[derive(Clone)]
    struct HashTestBackend {
        state: Arc<HashTestState>,
    }

    struct HashTestState {
        bytes: Vec<u8>,
        read_ranges: Mutex<Vec<Option<ByteRange>>>,
        stream_ranges: Mutex<Vec<Option<ByteRange>>>,
        read_error: Mutex<Option<NakoError>>,
        stream_error: Mutex<Option<NakoError>>,
    }

    impl HashTestBackend {
        fn new(bytes: Vec<u8>) -> Self {
            Self {
                state: Arc::new(HashTestState {
                    bytes,
                    read_ranges: Mutex::new(Vec::new()),
                    stream_ranges: Mutex::new(Vec::new()),
                    read_error: Mutex::new(None),
                    stream_error: Mutex::new(None),
                }),
            }
        }

        fn with_read_error(self, err: NakoError) -> Self {
            *self.state.read_error.lock().unwrap() = Some(err);
            self
        }

        fn with_stream_error(self, err: NakoError) -> Self {
            *self.state.stream_error.lock().unwrap() = Some(err);
            self
        }

        fn read_ranges(&self) -> Vec<Option<ByteRange>> {
            self.state.read_ranges.lock().unwrap().clone()
        }

        fn stream_ranges(&self) -> Vec<Option<ByteRange>> {
            self.state.stream_ranges.lock().unwrap().clone()
        }

        fn bytes_for_range(&self, range: Option<ByteRange>) -> Vec<u8> {
            let Some(range) = range else {
                return self.state.bytes.clone();
            };
            let start = usize::try_from(range.offset).unwrap();
            let end = match range.length {
                Some(length) => start + usize::try_from(length).unwrap(),
                None => self.state.bytes.len(),
            };
            self.state.bytes[start..end].to_vec()
        }
    }

    #[async_trait]
    impl StorageBackend for HashTestBackend {
        fn scheme(&self) -> &'static str {
            "test"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(ObjectMetadata {
                uri: uri.clone(),
                kind: ObjectKind::File,
                len: Some(self.state.bytes.len() as u64),
                modified_at: None,
                etag: None,
                fingerprint: None,
                capabilities: StorageCapabilities::RANGE_READABLE,
                cache: None,
            })
        }

        async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(Vec::new())
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Ok(VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            self.state.read_ranges.lock().unwrap().push(range);
            if let Some(err) = self.state.read_error.lock().unwrap().clone() {
                return Err(err);
            }
            Ok(ReadRange {
                uri: uri.clone(),
                range,
                bytes: self.bytes_for_range(range),
            })
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            self.state.stream_ranges.lock().unwrap().push(range);
            if let Some(err) = self.state.stream_error.lock().unwrap().clone() {
                return Err(err);
            }
            Ok(ReadStream::from_bytes(
                uri.clone(),
                range,
                self.bytes_for_range(range),
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(NakoError::storage(
                uri.to_string(),
                StorageErrorKind::Unknown,
                "test backend does not support text reads",
            ))
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::storage(
                uri.to_string(),
                StorageErrorKind::Unknown,
                "test backend does not support text writes",
            ))
        }
    }

    struct UnsupportedRangeBackend;

    #[async_trait]
    impl StorageBackend for UnsupportedRangeBackend {
        fn scheme(&self) -> &'static str {
            "unsupported"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Err(NakoError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            })
        }

        async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(Vec::new())
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Ok(VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "test backend does not support text reads",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "test backend does not support text writes",
            ))
        }
    }
}
