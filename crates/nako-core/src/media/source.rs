use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    LibraryId, LocalInferenceEvidenceId, MediaItemId, MediaSourceId, SourceDuplicateRelationshipId,
};

use super::item::MediaKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSource {
    pub id: MediaSourceId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub locator: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprintEvidenceKind {
    ContentHash,
    BackendFingerprint,
    SizeAndEtag,
    SizeAndModifiedTime,
    LocatorOnly,
}

impl SourceFingerprintEvidenceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContentHash => "content_hash",
            Self::BackendFingerprint => "backend_fingerprint",
            Self::SizeAndEtag => "size_etag",
            Self::SizeAndModifiedTime => "size_modified_time",
            Self::LocatorOnly => "locator_only",
        }
    }

    #[must_use]
    pub fn parse_redacted_fingerprint(value: &str) -> Option<Self> {
        let rest = value.strip_prefix("source:v1:")?;
        let (kind, digest) = rest.split_once(":sha256:")?;
        if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }

        match kind {
            "content_hash" => Some(Self::ContentHash),
            "backend_fingerprint" => Some(Self::BackendFingerprint),
            "size_etag" => Some(Self::SizeAndEtag),
            "size_modified_time" => Some(Self::SizeAndModifiedTime),
            "locator_only" => Some(Self::LocatorOnly),
            _ => None,
        }
    }

    #[must_use]
    pub const fn default_confidence_milli(self) -> u16 {
        match self {
            Self::ContentHash => 1_000,
            Self::SizeAndEtag => 800,
            Self::BackendFingerprint => 700,
            Self::SizeAndModifiedTime => 500,
            Self::LocatorOnly => 250,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceFingerprintPolicyInput<'a> {
    pub scheme: &'a str,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<&'a str>,
    pub etag: Option<&'a str>,
    pub backend_fingerprint: Option<&'a str>,
    pub stale: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintEvidence {
    pub kind: SourceFingerprintEvidenceKind,
    pub fingerprint: Option<String>,
    pub confidence_milli: u16,
    pub stale: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprintEscalationAction {
    None,
    PartialHash,
    FullHash,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprintEscalationReason {
    ExistingLocator,
    StrongEvidence,
    NoAmbiguousCandidate,
    ConfirmSingleWeakCandidate,
    DisambiguateMultipleCandidates,
    RefreshStaleAmbiguousEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceFingerprintEscalationDecision {
    pub action: SourceFingerprintEscalationAction,
    pub reason: SourceFingerprintEscalationReason,
    pub evidence_kind: SourceFingerprintEvidenceKind,
    pub confidence_milli: u16,
    pub stale: bool,
    pub candidate_count: u32,
}

impl SourceFingerprintEvidence {
    #[must_use]
    pub fn from_persisted_fingerprint(value: &str, stale: bool) -> Option<Self> {
        let kind = SourceFingerprintEvidenceKind::parse_redacted_fingerprint(value)?;

        Some(Self {
            kind,
            fingerprint: Some(value.to_owned()),
            confidence_milli: stale_adjusted_confidence(kind.default_confidence_milli(), stale),
            stale,
        })
    }

    #[must_use]
    pub fn from_scan_metadata(input: SourceFingerprintPolicyInput<'_>) -> Self {
        let etag = non_empty(input.etag);
        let modified_at = non_empty(input.modified_at);
        let backend_fingerprint = non_empty(input.backend_fingerprint);

        if let Some(value) = backend_fingerprint.filter(|value| looks_like_content_hash(value)) {
            return Self::from_parts(
                SourceFingerprintEvidenceKind::ContentHash,
                1_000,
                input.stale,
                &[input.scheme, value],
            );
        }

        if let Some((size_bytes, etag)) = input.size_bytes.zip(etag) {
            let size = size_bytes.to_string();
            return Self::from_parts(
                SourceFingerprintEvidenceKind::SizeAndEtag,
                800,
                input.stale,
                &[input.scheme, &size, etag],
            );
        }

        if let Some((size_bytes, modified_at)) = input.size_bytes.zip(modified_at) {
            let size = size_bytes.to_string();
            return Self::from_parts(
                SourceFingerprintEvidenceKind::SizeAndModifiedTime,
                500,
                input.stale,
                &[input.scheme, &size, modified_at],
            );
        }

        if let Some(value) = backend_fingerprint {
            return Self::from_parts(
                SourceFingerprintEvidenceKind::BackendFingerprint,
                700,
                input.stale,
                &[input.scheme, value],
            );
        }

        Self {
            kind: SourceFingerprintEvidenceKind::LocatorOnly,
            fingerprint: None,
            confidence_milli: stale_adjusted_confidence(250, input.stale),
            stale: input.stale,
        }
    }

    #[must_use]
    pub fn can_preserve_source_identity(&self) -> bool {
        !self.stale && self.confidence_milli >= 900 && self.fingerprint.is_some()
    }

    #[must_use]
    pub fn can_suggest_duplicate(&self) -> bool {
        self.confidence_milli >= 500 && self.fingerprint.is_some()
    }

    #[must_use]
    pub fn escalation_decision(
        &self,
        existing_locator: bool,
        candidate_count: usize,
    ) -> SourceFingerprintEscalationDecision {
        let candidate_count = usize_to_u32_saturating(candidate_count);
        let (action, reason) = if existing_locator {
            (
                SourceFingerprintEscalationAction::None,
                SourceFingerprintEscalationReason::ExistingLocator,
            )
        } else if self.can_preserve_source_identity() {
            (
                SourceFingerprintEscalationAction::None,
                SourceFingerprintEscalationReason::StrongEvidence,
            )
        } else if candidate_count == 0 {
            (
                SourceFingerprintEscalationAction::None,
                SourceFingerprintEscalationReason::NoAmbiguousCandidate,
            )
        } else if self.stale {
            (
                SourceFingerprintEscalationAction::FullHash,
                SourceFingerprintEscalationReason::RefreshStaleAmbiguousEvidence,
            )
        } else if candidate_count == 1 {
            (
                SourceFingerprintEscalationAction::PartialHash,
                SourceFingerprintEscalationReason::ConfirmSingleWeakCandidate,
            )
        } else {
            (
                SourceFingerprintEscalationAction::FullHash,
                SourceFingerprintEscalationReason::DisambiguateMultipleCandidates,
            )
        };

        SourceFingerprintEscalationDecision {
            action,
            reason,
            evidence_kind: self.kind,
            confidence_milli: self.confidence_milli,
            stale: self.stale,
            candidate_count,
        }
    }

    fn from_parts(
        kind: SourceFingerprintEvidenceKind,
        confidence_milli: u16,
        stale: bool,
        parts: &[&str],
    ) -> Self {
        Self {
            kind,
            fingerprint: Some(redacted_source_fingerprint(kind, parts)),
            confidence_milli: stale_adjusted_confidence(confidence_milli, stale),
            stale,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDuplicateEvidenceKind {
    StrongFingerprint,
    SizeAndEtag,
    PathEvidence,
    FilesystemLink,
    Manual,
    Other(String),
}

impl SourceDuplicateEvidenceKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::StrongFingerprint => ("strong_fingerprint", ""),
            Self::SizeAndEtag => ("size_and_etag", ""),
            Self::PathEvidence => ("path_evidence", ""),
            Self::FilesystemLink => ("filesystem_link", ""),
            Self::Manual => ("manual", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "strong_fingerprint" => Self::StrongFingerprint,
            "size_and_etag" => Self::SizeAndEtag,
            "path_evidence" => Self::PathEvidence,
            "filesystem_link" => Self::FilesystemLink,
            "manual" => Self::Manual,
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }

    #[must_use]
    pub fn from_source_fingerprint_evidence_kind(kind: SourceFingerprintEvidenceKind) -> Self {
        match kind {
            SourceFingerprintEvidenceKind::ContentHash => Self::StrongFingerprint,
            SourceFingerprintEvidenceKind::SizeAndEtag => Self::SizeAndEtag,
            SourceFingerprintEvidenceKind::SizeAndModifiedTime
            | SourceFingerprintEvidenceKind::LocatorOnly => Self::PathEvidence,
            SourceFingerprintEvidenceKind::BackendFingerprint => {
                Self::Other("backend_fingerprint".to_owned())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDuplicateRelationshipStatus {
    Suggested,
    Confirmed,
    Rejected,
}

impl SourceDuplicateRelationshipStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "suggested" => Ok(Self::Suggested),
            "confirmed" => Ok(Self::Confirmed),
            "rejected" => Ok(Self::Rejected),
            _ => Err(crate::NakoError::Database {
                message: format!(
                    "unknown source duplicate relationship status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceDuplicateRelationship {
    pub id: SourceDuplicateRelationshipId,
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub evidence_kind: SourceDuplicateEvidenceKind,
    pub evidence_value: Option<String>,
    pub status: SourceDuplicateRelationshipStatus,
    pub confidence_milli: Option<u16>,
}

impl SourceDuplicateRelationship {
    #[must_use]
    pub fn canonical_pair(
        source_id: MediaSourceId,
        duplicate_source_id: MediaSourceId,
    ) -> (MediaSourceId, MediaSourceId) {
        if source_id <= duplicate_source_id {
            (source_id, duplicate_source_id)
        } else {
            (duplicate_source_id, source_id)
        }
    }

    #[must_use]
    pub fn canonicalized(&self) -> Self {
        let mut relationship = self.clone();
        if relationship.source_id > relationship.duplicate_source_id {
            std::mem::swap(
                &mut relationship.source_id,
                &mut relationship.duplicate_source_id,
            );
        }
        relationship
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaSourceFingerprintMatch {
    pub source: MediaSource,
    pub stale: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDuplicateReconciliationAction {
    SuggestRelationship,
    ConfirmSuggested,
    RejectSuggested,
    PreserveSuggested,
    PreserveConfirmed,
    PreserveRejected,
    RefreshSourceFingerprint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceDuplicateReconciliationCandidate {
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub evidence_kind: SourceDuplicateEvidenceKind,
    pub confidence_milli: Option<u16>,
    pub stale: bool,
    pub relationship_id: Option<SourceDuplicateRelationshipId>,
    pub existing_status: Option<SourceDuplicateRelationshipStatus>,
    pub recommended_action: SourceDuplicateReconciliationAction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceDuplicateReconciliationPlan {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub fingerprint_evidence_kind: SourceFingerprintEvidenceKind,
    pub confidence_milli: u16,
    pub stale: bool,
    pub candidates: Vec<SourceDuplicateReconciliationCandidate>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceDuplicateReconciliationApplyResult {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub relationship_id: SourceDuplicateRelationshipId,
    pub relationship_status: SourceDuplicateRelationshipStatus,
    pub applied_action: SourceDuplicateReconciliationAction,
    pub created: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalInferenceEvidenceSource {
    Path,
    FileName,
    Directory,
    NearbyFile,
    MediaProbe,
    Other(String),
}

impl LocalInferenceEvidenceSource {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::Path => ("path", ""),
            Self::FileName => ("file_name", ""),
            Self::Directory => ("directory", ""),
            Self::NearbyFile => ("nearby_file", ""),
            Self::MediaProbe => ("media_probe", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(source: &str, source_key: String) -> Self {
        match source {
            "path" => Self::Path,
            "file_name" => Self::FileName,
            "directory" => Self::Directory,
            "nearby_file" => Self::NearbyFile,
            "media_probe" => Self::MediaProbe,
            "other" => Self::Other(source_key),
            _ => Self::Other(source.to_owned()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalInferenceEvidence {
    pub id: LocalInferenceEvidenceId,
    pub source_id: MediaSourceId,
    pub inferred_kind: MediaKind,
    pub inferred_title: Option<String>,
    pub inferred_year: Option<i32>,
    pub inferred_season: Option<u32>,
    pub inferred_episode: Option<u32>,
    pub confidence_milli: Option<u16>,
    pub evidence_source: LocalInferenceEvidenceSource,
    pub evidence_value: String,
    pub inference_version: String,
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn looks_like_content_hash(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    let Some((algorithm, digest)) = value.split_once(':') else {
        return false;
    };

    match algorithm {
        "sha256" | "blake3" => digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit()),
        "content" => !digest.is_empty(),
        _ => false,
    }
}

fn stale_adjusted_confidence(confidence_milli: u16, stale: bool) -> u16 {
    if stale {
        confidence_milli.saturating_sub(200)
    } else {
        confidence_milli
    }
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn redacted_source_fingerprint(kind: SourceFingerprintEvidenceKind, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    update_fingerprint_part(&mut hasher, "source-fingerprint-v1");
    update_fingerprint_part(&mut hasher, kind.as_str());
    for part in parts {
        update_fingerprint_part(&mut hasher, part);
    }
    let digest = hasher.finalize();

    format!(
        "source:v1:{}:sha256:{}",
        kind.as_str(),
        lowercase_hex(&digest)
    )
}

fn update_fingerprint_part(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_be_bytes());
    hasher.update(value.as_bytes());
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_fingerprint_policy_hashes_size_and_etag_without_leaking_raw_etag() {
        let evidence =
            SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
                scheme: "webdav",
                size_bytes: Some(42),
                modified_at: None,
                etag: Some("private-etag"),
                backend_fingerprint: None,
                stale: false,
            });

        let fingerprint = evidence.fingerprint.as_deref().unwrap();

        assert_eq!(evidence.kind, SourceFingerprintEvidenceKind::SizeAndEtag);
        assert_eq!(evidence.confidence_milli, 800);
        assert!(fingerprint.starts_with("source:v1:size_etag:sha256:"));
        assert!(!fingerprint.contains("private-etag"));
        assert!(evidence.can_suggest_duplicate());
        assert!(!evidence.can_preserve_source_identity());
    }

    #[test]
    fn source_fingerprint_policy_treats_content_hash_as_strong_identity_evidence() {
        let evidence =
            SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
                scheme: "local",
                size_bytes: Some(42),
                modified_at: Some("2026-05-29T00:00:00Z"),
                etag: None,
                backend_fingerprint: Some(
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                ),
                stale: false,
            });

        assert_eq!(evidence.kind, SourceFingerprintEvidenceKind::ContentHash);
        assert_eq!(evidence.confidence_milli, 1_000);
        assert!(evidence.can_suggest_duplicate());
        assert!(evidence.can_preserve_source_identity());
    }

    #[test]
    fn source_fingerprint_policy_recovers_kind_from_persisted_redacted_fingerprint() {
        let evidence = SourceFingerprintEvidence::from_persisted_fingerprint(
            "source:v1:content_hash:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            true,
        )
        .unwrap();

        assert_eq!(evidence.kind, SourceFingerprintEvidenceKind::ContentHash);
        assert_eq!(evidence.confidence_milli, 800);
        assert!(evidence.stale);
        assert_eq!(
            SourceFingerprintEvidence::from_persisted_fingerprint("sha256:0123456789abcdef", false),
            None
        );
    }

    #[test]
    fn source_fingerprint_policy_does_not_treat_malformed_hash_as_strong() {
        let evidence =
            SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
                scheme: "remote",
                size_bytes: None,
                modified_at: None,
                etag: None,
                backend_fingerprint: Some("sha256:not-a-content-hash"),
                stale: false,
            });

        assert_eq!(
            evidence.kind,
            SourceFingerprintEvidenceKind::BackendFingerprint
        );
        assert_eq!(evidence.confidence_milli, 700);
        assert!(evidence.can_suggest_duplicate());
        assert!(!evidence.can_preserve_source_identity());
    }

    #[test]
    fn source_fingerprint_policy_downgrades_stale_and_locator_only_evidence() {
        let stale = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
            scheme: "local",
            size_bytes: Some(42),
            modified_at: Some("2026-05-29T00:00:00Z"),
            etag: None,
            backend_fingerprint: None,
            stale: true,
        });
        let locator_only =
            SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
                scheme: "local",
                size_bytes: None,
                modified_at: None,
                etag: None,
                backend_fingerprint: None,
                stale: false,
            });

        assert_eq!(
            stale.kind,
            SourceFingerprintEvidenceKind::SizeAndModifiedTime
        );
        assert_eq!(stale.confidence_milli, 300);
        assert!(stale.fingerprint.is_some());
        assert!(!stale.can_suggest_duplicate());
        assert_eq!(
            locator_only.kind,
            SourceFingerprintEvidenceKind::LocatorOnly
        );
        assert_eq!(locator_only.fingerprint, None);
        assert!(!locator_only.can_suggest_duplicate());
    }

    #[test]
    fn source_fingerprint_escalation_skips_existing_strong_and_unmatched_evidence() {
        let strong = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
            scheme: "local",
            size_bytes: Some(42),
            modified_at: None,
            etag: None,
            backend_fingerprint: Some(
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
            stale: false,
        });
        let weak = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
            scheme: "webdav",
            size_bytes: Some(42),
            modified_at: None,
            etag: Some("private-etag"),
            backend_fingerprint: None,
            stale: false,
        });

        assert_eq!(
            weak.escalation_decision(true, 1),
            SourceFingerprintEscalationDecision {
                action: SourceFingerprintEscalationAction::None,
                reason: SourceFingerprintEscalationReason::ExistingLocator,
                evidence_kind: SourceFingerprintEvidenceKind::SizeAndEtag,
                confidence_milli: 800,
                stale: false,
                candidate_count: 1,
            }
        );
        assert_eq!(
            strong.escalation_decision(false, 1).reason,
            SourceFingerprintEscalationReason::StrongEvidence
        );
        assert_eq!(
            weak.escalation_decision(false, 0).reason,
            SourceFingerprintEscalationReason::NoAmbiguousCandidate
        );
    }

    #[test]
    fn source_fingerprint_escalation_recommends_partial_hash_for_single_weak_candidate() {
        let evidence =
            SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
                scheme: "remote",
                size_bytes: None,
                modified_at: None,
                etag: None,
                backend_fingerprint: Some("backend-fingerprint"),
                stale: false,
            });

        let decision = evidence.escalation_decision(false, 1);

        assert_eq!(
            decision.action,
            SourceFingerprintEscalationAction::PartialHash
        );
        assert_eq!(
            decision.reason,
            SourceFingerprintEscalationReason::ConfirmSingleWeakCandidate
        );
        assert_eq!(decision.candidate_count, 1);
        assert_eq!(
            decision.evidence_kind,
            SourceFingerprintEvidenceKind::BackendFingerprint
        );
        assert_eq!(decision.confidence_milli, 700);
    }

    #[test]
    fn source_fingerprint_escalation_recommends_full_hash_for_ambiguous_or_stale_candidates() {
        let fresh = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
            scheme: "webdav",
            size_bytes: Some(42),
            modified_at: None,
            etag: Some("private-etag"),
            backend_fingerprint: None,
            stale: false,
        });
        let stale = SourceFingerprintEvidence::from_scan_metadata(SourceFingerprintPolicyInput {
            scheme: "webdav",
            size_bytes: Some(42),
            modified_at: None,
            etag: Some("private-etag"),
            backend_fingerprint: None,
            stale: true,
        });

        assert_eq!(
            fresh.escalation_decision(false, 2).reason,
            SourceFingerprintEscalationReason::DisambiguateMultipleCandidates
        );
        assert_eq!(
            stale.escalation_decision(false, 1),
            SourceFingerprintEscalationDecision {
                action: SourceFingerprintEscalationAction::FullHash,
                reason: SourceFingerprintEscalationReason::RefreshStaleAmbiguousEvidence,
                evidence_kind: SourceFingerprintEvidenceKind::SizeAndEtag,
                confidence_milli: 600,
                stale: true,
                candidate_count: 1,
            }
        );
    }
}
