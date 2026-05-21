use sha2::{Digest, Sha256};
use taru_core::{LocalMetadataPolicy, MediaKind, MediaRepository, Result, TaruError};
use taru_vfs::{StorageBackend, StorageCapabilities, StorageUri};

use super::{NfoCodec, NfoDocument, NfoHierarchy, NfoService, workflow::nfo_uri_for_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NfoAuthorityPreviewOperation {
    Import,
    Export,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NfoAuthorityPreviewRequest {
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub operation: NfoAuthorityPreviewOperation,
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NfoAuthorityPreviewSummary {
    pub library_id: taru_core::LibraryId,
    pub operation: NfoAuthorityPreviewOperation,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
    pub scanned_sources: u64,
    pub create_items: u64,
    pub skip_items: u64,
    pub update_items: u64,
    pub backup_required_items: u64,
    pub policy_rejected_items: u64,
    pub failure_items: u64,
    pub decisions: Vec<NfoAuthorityPreviewDecision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NfoAuthorityPreviewDecision {
    pub source_id: taru_core::MediaSourceId,
    pub item_id: taru_core::MediaItemId,
    pub locator: String,
    pub nfo_uri: Option<StorageUri>,
    pub content_fingerprint: Option<String>,
    pub action: NfoAuthorityPreviewAction,
    pub reason: NfoAuthorityPreviewReason,
    pub backup_required: bool,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NfoAuthorityPreviewAction {
    Create,
    Skip,
    Update,
    PolicyRejected,
    Fail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NfoAuthorityPreviewReason {
    ExportWouldCreateSidecar,
    ExportWouldSkipExistingSidecar,
    ExportWouldUpdateExistingSidecar,
    ImportWouldReadSidecar,
    ImportSidecarMissing,
    PolicyDoesNotAllowOperation,
    UnsupportedMediaKind,
    MissingMediaItem,
    InvalidSidecarPath,
    StorageReadFailed,
    StorageUnsupported,
    NfoParseFailed,
    NfoRenderFailed,
    NfoPreservationFailed,
}

impl<B, R, C> NfoService<B, R, C>
where
    B: StorageBackend,
    R: MediaRepository,
    C: NfoCodec,
{
    pub async fn preview_authority(
        &self,
        request: NfoAuthorityPreviewRequest,
    ) -> Result<NfoAuthorityPreviewSummary> {
        let sources = self.list_all_sources(request.library_id).await?;
        let mut summary = NfoAuthorityPreviewSummary {
            library_id: request.library_id,
            operation: request.operation,
            policy: request.policy,
            force: request.force,
            scanned_sources: sources.len() as u64,
            create_items: 0,
            skip_items: 0,
            update_items: 0,
            backup_required_items: 0,
            policy_rejected_items: 0,
            failure_items: 0,
            decisions: Vec::with_capacity(sources.len()),
        };

        for source in sources {
            let decision = match request.operation {
                NfoAuthorityPreviewOperation::Import => {
                    self.preview_import_source(source, request.policy).await
                }
                NfoAuthorityPreviewOperation::Export => {
                    self.preview_export_source(source, request.policy, request.force)
                        .await
                }
            };
            summary.record(decision);
        }

        summary
            .decisions
            .sort_by(|left, right| left.locator.cmp(&right.locator));

        Ok(summary)
    }

    async fn preview_import_source(
        &self,
        source: taru_core::MediaSource,
        policy: LocalMetadataPolicy,
    ) -> NfoAuthorityPreviewDecision {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => {
                return preview_failure(
                    &source,
                    None,
                    NfoAuthorityPreviewReason::InvalidSidecarPath,
                    err,
                );
            }
        };

        if !import_policy_allowed(policy) {
            return NfoAuthorityPreviewDecision {
                source_id: source.id,
                item_id: source.item_id,
                locator: source.locator,
                nfo_uri: Some(nfo_uri),
                content_fingerprint: None,
                action: NfoAuthorityPreviewAction::PolicyRejected,
                reason: NfoAuthorityPreviewReason::PolicyDoesNotAllowOperation,
                backup_required: false,
                message:
                    "NFO import requires read-only, local-first, or remote-first local metadata policy"
                        .to_owned(),
            };
        }

        let xml = match self.backend.read_to_string(&nfo_uri).await {
            Ok(xml) => xml,
            Err(TaruError::NotFound { .. }) => {
                return NfoAuthorityPreviewDecision {
                    source_id: source.id,
                    item_id: source.item_id,
                    locator: source.locator,
                    nfo_uri: Some(nfo_uri),
                    content_fingerprint: None,
                    action: NfoAuthorityPreviewAction::Skip,
                    reason: NfoAuthorityPreviewReason::ImportSidecarMissing,
                    backup_required: false,
                    message: "NFO import would skip because no sidecar exists".to_owned(),
                };
            }
            Err(err) => {
                return preview_failure(
                    &source,
                    Some(nfo_uri),
                    classify_preview_read_failure(&err),
                    err,
                );
            }
        };

        if let Err(err) = self.codec.parse(&xml) {
            return preview_failure(
                &source,
                Some(nfo_uri),
                NfoAuthorityPreviewReason::NfoParseFailed,
                err,
            );
        }

        NfoAuthorityPreviewDecision {
            source_id: source.id,
            item_id: source.item_id,
            locator: source.locator,
            nfo_uri: Some(nfo_uri),
            content_fingerprint: Some(xml_content_fingerprint(&xml)),
            action: NfoAuthorityPreviewAction::Update,
            reason: NfoAuthorityPreviewReason::ImportWouldReadSidecar,
            backup_required: false,
            message: "NFO import would read this sidecar and update metadata according to policy"
                .to_owned(),
        }
    }

    async fn preview_export_source(
        &self,
        source: taru_core::MediaSource,
        policy: LocalMetadataPolicy,
        force: bool,
    ) -> NfoAuthorityPreviewDecision {
        let nfo_uri = match nfo_uri_for_source(&source) {
            Ok(uri) => uri,
            Err(err) => {
                return preview_failure(
                    &source,
                    None,
                    NfoAuthorityPreviewReason::InvalidSidecarPath,
                    err,
                );
            }
        };

        if !export_policy_allowed(policy) {
            return NfoAuthorityPreviewDecision {
                source_id: source.id,
                item_id: source.item_id,
                locator: source.locator,
                nfo_uri: Some(nfo_uri),
                content_fingerprint: None,
                action: NfoAuthorityPreviewAction::PolicyRejected,
                reason: NfoAuthorityPreviewReason::PolicyDoesNotAllowOperation,
                backup_required: false,
                message: "NFO export requires write-sidecar local metadata policy".to_owned(),
            };
        }

        let item = match self.repository.get_media_item(source.item_id).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                return preview_failure(
                    &source,
                    Some(nfo_uri),
                    NfoAuthorityPreviewReason::MissingMediaItem,
                    TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    },
                );
            }
            Err(err) => {
                return preview_failure(
                    &source,
                    Some(nfo_uri),
                    NfoAuthorityPreviewReason::StorageReadFailed,
                    err,
                );
            }
        };

        if item.kind != MediaKind::Movie {
            return NfoAuthorityPreviewDecision {
                source_id: source.id,
                item_id: source.item_id,
                locator: source.locator,
                nfo_uri: Some(nfo_uri),
                content_fingerprint: None,
                action: NfoAuthorityPreviewAction::Skip,
                reason: NfoAuthorityPreviewReason::UnsupportedMediaKind,
                backup_required: false,
                message: "NFO export currently supports movie sidecars only".to_owned(),
            };
        }

        let existing_xml = match self.backend.stat(&nfo_uri).await {
            Ok(metadata) => {
                if !metadata
                    .capabilities
                    .contains(StorageCapabilities::WRITABLE)
                {
                    return preview_failure(
                        &source,
                        Some(nfo_uri),
                        NfoAuthorityPreviewReason::StorageUnsupported,
                        TaruError::Unsupported("NFO export requires a writable sidecar target"),
                    );
                }

                if force {
                    match self.backend.read_to_string(&nfo_uri).await {
                        Ok(xml) => Some(xml),
                        Err(err) => {
                            return preview_failure(
                                &source,
                                Some(nfo_uri),
                                classify_preview_read_failure(&err),
                                err,
                            );
                        }
                    }
                } else {
                    return NfoAuthorityPreviewDecision {
                        source_id: source.id,
                        item_id: source.item_id,
                        locator: source.locator,
                        nfo_uri: Some(nfo_uri),
                        content_fingerprint: None,
                        action: NfoAuthorityPreviewAction::Skip,
                        reason: NfoAuthorityPreviewReason::ExportWouldSkipExistingSidecar,
                        backup_required: false,
                        message:
                            "NFO export would skip because a sidecar already exists and force is false"
                                .to_owned(),
                    };
                }
            }
            Err(TaruError::NotFound { .. }) => None,
            Err(err) => {
                return preview_failure(
                    &source,
                    Some(nfo_uri),
                    classify_preview_read_failure(&err),
                    err,
                );
            }
        };

        let document =
            NfoDocument::from_metadata(item.kind, item.metadata, NfoHierarchy::default());
        match existing_xml {
            Some(existing_xml) => match self.codec.render_preserving(&document, &existing_xml) {
                Ok(_) => NfoAuthorityPreviewDecision {
                    source_id: source.id,
                    item_id: source.item_id,
                    locator: source.locator,
                    nfo_uri: Some(nfo_uri),
                    content_fingerprint: Some(xml_content_fingerprint(&existing_xml)),
                    action: NfoAuthorityPreviewAction::Update,
                    reason: NfoAuthorityPreviewReason::ExportWouldUpdateExistingSidecar,
                    backup_required: true,
                    message:
                        "NFO export would update the existing sidecar with a same-directory backup"
                            .to_owned(),
                },
                Err(err) => preview_failure(
                    &source,
                    Some(nfo_uri),
                    NfoAuthorityPreviewReason::NfoPreservationFailed,
                    err,
                ),
            },
            None => match self.codec.render(&document) {
                Ok(_) => NfoAuthorityPreviewDecision {
                    source_id: source.id,
                    item_id: source.item_id,
                    locator: source.locator,
                    nfo_uri: Some(nfo_uri),
                    content_fingerprint: None,
                    action: NfoAuthorityPreviewAction::Create,
                    reason: NfoAuthorityPreviewReason::ExportWouldCreateSidecar,
                    backup_required: false,
                    message: "NFO export would create a new sidecar".to_owned(),
                },
                Err(err) => preview_failure(
                    &source,
                    Some(nfo_uri),
                    NfoAuthorityPreviewReason::NfoRenderFailed,
                    err,
                ),
            },
        }
    }
}

impl NfoAuthorityPreviewSummary {
    fn record(&mut self, decision: NfoAuthorityPreviewDecision) {
        match decision.action {
            NfoAuthorityPreviewAction::Create => self.create_items += 1,
            NfoAuthorityPreviewAction::Skip => self.skip_items += 1,
            NfoAuthorityPreviewAction::Update => self.update_items += 1,
            NfoAuthorityPreviewAction::PolicyRejected => self.policy_rejected_items += 1,
            NfoAuthorityPreviewAction::Fail => self.failure_items += 1,
        }

        if decision.backup_required {
            self.backup_required_items += 1;
        }

        self.decisions.push(decision);
    }
}

fn preview_failure(
    source: &taru_core::MediaSource,
    nfo_uri: Option<StorageUri>,
    reason: NfoAuthorityPreviewReason,
    err: impl ToString,
) -> NfoAuthorityPreviewDecision {
    NfoAuthorityPreviewDecision {
        source_id: source.id,
        item_id: source.item_id,
        locator: source.locator.clone(),
        nfo_uri,
        content_fingerprint: None,
        action: NfoAuthorityPreviewAction::Fail,
        reason,
        backup_required: false,
        message: err.to_string(),
    }
}

fn xml_content_fingerprint(xml: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(xml.as_bytes()))
}

fn import_policy_allowed(policy: LocalMetadataPolicy) -> bool {
    matches!(
        policy,
        LocalMetadataPolicy::ReadOnly
            | LocalMetadataPolicy::LocalFirst
            | LocalMetadataPolicy::RemoteFirst
    )
}

fn export_policy_allowed(policy: LocalMetadataPolicy) -> bool {
    policy == LocalMetadataPolicy::WriteSidecar
}

fn classify_preview_read_failure(err: &TaruError) -> NfoAuthorityPreviewReason {
    match err {
        TaruError::Unsupported(_) => NfoAuthorityPreviewReason::StorageUnsupported,
        _ => NfoAuthorityPreviewReason::StorageReadFailed,
    }
}
