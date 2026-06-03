use async_trait::async_trait;
use nako_addon_protocol::{AddonResourceLink, AddonResourceLinkType, AddonResourceSearchResult};
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateListFilter,
    AcquisitionIntakeCandidateRecord, AcquisitionIntakeCandidateState, AcquisitionIntakeRepository,
    AcquisitionIntakeSourceKind, AddonId, Library, LibraryId, LibraryRepository,
    ManagedImportArtifactId, ManagedImportArtifactRecord, ManagedImportArtifactState,
    ManagedImportRepository, ManagedImportSourceKind, NakoError, NewAcquisitionIntakeCandidate,
    PageRequest, Result,
};
use nako_db::NakoDatabase;
use nako_library::{
    LibraryScannerOptions, STABLE_INTAKE_REQUIRED_OBSERVATIONS, StableIntakeCandidateEvidence,
    StableIntakeCandidateState, observe_stable_intake_candidate,
};
use nako_vfs::{ObjectKind, ObjectMetadata, StorageBackend, StorageUri};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::{
    managed_import::{CreateManagedImportArtifactRequest, ManagedImportAppService},
    storage::StorageBackendRegistry,
    watch_folder_suppression::{
        PlannedWatchFolderWriteSuppressionDiagnostic, WatchFolderSuppressionAppService,
    },
};

#[async_trait]
trait AcquisitionIntakeWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;

    async fn upsert_acquisition_intake_candidate(
        &self,
        candidate: NewAcquisitionIntakeCandidate,
    ) -> Result<AcquisitionIntakeCandidateRecord>;

    async fn get_acquisition_intake_candidate(
        &self,
        id: AcquisitionIntakeCandidateId,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;

    async fn find_acquisition_intake_candidate_by_source_key(
        &self,
        target_library_id: LibraryId,
        source_kind: &AcquisitionIntakeSourceKind,
        source_key: &str,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;

    async fn list_acquisition_intake_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<Vec<AcquisitionIntakeCandidateRecord>>;

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>>;

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>>;

    async fn link_acquisition_intake_candidate_managed_import_artifact(
        &self,
        id: AcquisitionIntakeCandidateId,
        managed_import_artifact_id: ManagedImportArtifactId,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;
}

#[async_trait]
impl<T> AcquisitionIntakeWorkflowStore for T
where
    T: AcquisitionIntakeRepository
        + LibraryRepository
        + ManagedImportRepository
        + std::fmt::Debug
        + Send
        + Sync,
{
    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        LibraryRepository::get_library(self, id).await
    }

    async fn upsert_acquisition_intake_candidate(
        &self,
        candidate: NewAcquisitionIntakeCandidate,
    ) -> Result<AcquisitionIntakeCandidateRecord> {
        AcquisitionIntakeRepository::upsert_acquisition_intake_candidate(self, candidate).await
    }

    async fn get_acquisition_intake_candidate(
        &self,
        id: AcquisitionIntakeCandidateId,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        AcquisitionIntakeRepository::get_acquisition_intake_candidate(self, id).await
    }

    async fn find_acquisition_intake_candidate_by_source_key(
        &self,
        target_library_id: LibraryId,
        source_kind: &AcquisitionIntakeSourceKind,
        source_key: &str,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        AcquisitionIntakeRepository::find_acquisition_intake_candidate_by_source_key(
            self,
            target_library_id,
            source_kind,
            source_key,
        )
        .await
    }

    async fn list_acquisition_intake_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<Vec<AcquisitionIntakeCandidateRecord>> {
        AcquisitionIntakeRepository::list_acquisition_intake_candidates(self, filter, page).await
    }

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        ManagedImportRepository::get_managed_import_artifact(self, id).await
    }

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        ManagedImportRepository::find_managed_import_artifact_by_source(
            self,
            target_library_id,
            source_kind,
            source_uri,
        )
        .await
    }

    async fn link_acquisition_intake_candidate_managed_import_artifact(
        &self,
        id: AcquisitionIntakeCandidateId,
        managed_import_artifact_id: ManagedImportArtifactId,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        AcquisitionIntakeRepository::link_acquisition_intake_candidate_managed_import_artifact(
            self,
            id,
            managed_import_artifact_id,
            updated_at_ms,
            diagnostics_json,
        )
        .await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AcquisitionIntakeAppService {
    store: Arc<dyn AcquisitionIntakeWorkflowStore>,
    managed_import: ManagedImportAppService,
    storage_backends: Option<StorageBackendRegistry>,
    watch_folder_suppression: WatchFolderSuppressionAppService,
}

impl AcquisitionIntakeAppService {
    pub(super) fn new_with_storage(
        store: NakoDatabase,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self::new_with_storage_and_suppression(
            store,
            storage_backends,
            WatchFolderSuppressionAppService::new(),
        )
    }

    pub(super) fn new_with_storage_and_suppression(
        store: NakoDatabase,
        storage_backends: StorageBackendRegistry,
        watch_folder_suppression: WatchFolderSuppressionAppService,
    ) -> Self {
        let managed_import = ManagedImportAppService::new(store.clone());
        Self {
            managed_import,
            storage_backends: Some(storage_backends),
            watch_folder_suppression,
            store: Arc::new(store),
        }
    }

    pub(crate) async fn record_candidate(
        &self,
        request: RecordAcquisitionIntakeCandidateRequest,
    ) -> Result<AcquisitionIntakeCandidateDiagnostic> {
        self.store
            .get_library(request.target_library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: request.target_library_id.to_string(),
            })?;

        let source_key = require_non_empty("acquisition intake source_key", request.source_key)?;
        let source_uri = require_non_empty("acquisition intake source_uri", request.source_uri)?;
        let state = request
            .state
            .unwrap_or(AcquisitionIntakeCandidateState::Discovered);
        let now_ms = super::current_time_ms()?;
        let record = self
            .store
            .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
                id: request.id.unwrap_or_else(AcquisitionIntakeCandidateId::new),
                target_library_id: request.target_library_id,
                source_kind: request.source_kind,
                source_key,
                source_uri,
                display_name: optional_non_empty(request.display_name),
                intended_locator: optional_non_empty(request.intended_locator),
                size_bytes: request.size_bytes,
                fingerprint: optional_non_empty(request.fingerprint),
                managed_import_artifact_id: request.managed_import_artifact_id,
                state,
                diagnostics_json: optional_non_empty(request.diagnostics_json),
                first_seen_at_ms: now_ms,
                last_seen_at_ms: now_ms,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            })
            .await?;

        Ok(AcquisitionIntakeCandidateDiagnostic::from_record(record))
    }

    pub(crate) async fn list_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<AcquisitionIntakeCandidateDiagnostics> {
        let page = page.clamped();
        if let Some(library_id) = filter.target_library_id {
            self.store
                .get_library(library_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "library",
                    id: library_id.to_string(),
                })?;
        }

        let records = self
            .store
            .list_acquisition_intake_candidates(filter, page)
            .await?;
        let returned = records.len();
        let candidates = records
            .into_iter()
            .map(AcquisitionIntakeCandidateDiagnostic::from_record)
            .collect();

        Ok(AcquisitionIntakeCandidateDiagnostics {
            limit: page.limit,
            offset: page.offset,
            returned,
            candidates,
        })
    }

    pub(crate) async fn record_resource_search_selection(
        &self,
        request: RecordResourceSearchSelectionRequest,
    ) -> Result<RecordResourceSearchSelectionDiagnostic> {
        let query = require_non_empty("resource search selection query", request.query.clone())?;
        let manifest_id = require_non_empty(
            "resource search selection manifest_id",
            request.manifest_id.clone(),
        )?;
        let result_id = require_non_empty(
            "resource search selection result id",
            request.result.id.clone(),
        )?;
        let source_uri = selected_resource_search_link_uri(&request.selected_link)?;
        let source_key = resource_search_selection_source_key(
            request.addon_id,
            &manifest_id,
            request.selected_link.link_type,
            &source_uri,
        );
        let existing = self
            .store
            .find_acquisition_intake_candidate_by_source_key(
                request.target_library_id,
                &AcquisitionIntakeSourceKind::ResourceSearchSelection,
                &source_key,
            )
            .await?;
        let diagnostics_json = resource_search_selection_diagnostics_json(
            &request,
            &query,
            &manifest_id,
            &result_id,
            &source_uri,
        )?;

        let candidate = self
            .record_candidate(RecordAcquisitionIntakeCandidateRequest {
                id: None,
                target_library_id: request.target_library_id,
                source_kind: AcquisitionIntakeSourceKind::ResourceSearchSelection,
                source_key,
                source_uri,
                display_name: Some(request.result.title),
                intended_locator: None,
                size_bytes: None,
                fingerprint: None,
                managed_import_artifact_id: None,
                state: Some(AcquisitionIntakeCandidateState::Ready),
                diagnostics_json: Some(diagnostics_json),
            })
            .await?;

        Ok(RecordResourceSearchSelectionDiagnostic {
            candidate,
            idempotent_replay: existing.is_some(),
        })
    }

    pub(crate) async fn discover_watch_folder_candidates(
        &self,
        request: DiscoverWatchFolderCandidatesRequest,
    ) -> Result<WatchFolderDiscoveryDiagnostic> {
        let library = self
            .store
            .get_library(request.target_library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: request.target_library_id.to_string(),
            })?;
        let root_uri = request
            .root_uri
            .or_else(|| {
                library
                    .roots
                    .first()
                    .and_then(|root| StorageUri::parse(root).ok())
            })
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!(
                    "library {} does not have a valid watch-folder root uri",
                    library.id
                ),
            })?;
        let backend = self
            .storage_backends
            .as_ref()
            .ok_or(NakoError::Unsupported(
                "watch-folder discovery requires configured storage backends",
            ))?
            .backend_for_library_root(&library)
            .await?;
        let max_depth = request.max_depth.unwrap_or_else(|| {
            library
                .options
                .scan
                .max_depth
                .unwrap_or_else(|| LibraryScannerOptions::default().max_depth)
        });
        let mut stack = vec![(root_uri.clone(), 0_usize)];
        let mut ready_candidates = 0_u64;
        let mut inspecting_candidates = 0_u64;
        let mut blocked_candidates = 0_u64;
        let mut incomplete_candidates = 0_u64;
        let mut unsupported_candidates = 0_u64;
        let mut suppressed_candidates = 0_u64;
        let mut recorded_candidates = 0_u64;
        let mut newly_ready_candidates = 0_u64;
        let mut failures = Vec::new();

        while let Some((uri, depth)) = stack.pop() {
            if depth > max_depth {
                continue;
            }

            let metadata = match backend.stat(&uri).await {
                Ok(metadata) => metadata,
                Err(err) => {
                    failures.push(WatchFolderDiscoveryFailureDiagnostic::from_error(uri, err));
                    continue;
                }
            };

            if self
                .watch_folder_suppression
                .match_suppression(library.id, &metadata.uri)
                .await?
                .is_some()
            {
                suppressed_candidates += 1;
                continue;
            }

            match metadata.kind {
                ObjectKind::Directory => match backend.list(&metadata.uri).await {
                    Ok(mut entries) => {
                        entries.sort_by(|left, right| right.uri.as_str().cmp(left.uri.as_str()));
                        for entry in entries {
                            stack.push((entry.uri, depth + 1));
                        }
                    }
                    Err(err) => {
                        failures.push(WatchFolderDiscoveryFailureDiagnostic::from_error(
                            metadata.uri,
                            err,
                        ));
                    }
                },
                ObjectKind::File | ObjectKind::Symlink => {
                    let (source_key, existing) = self
                        .find_existing_watch_folder_candidate(library.id, &metadata)
                        .await?;
                    let classification =
                        classify_watch_folder_candidate(&metadata, existing.as_ref());
                    match classification.reason {
                        WatchFolderCandidateReason::Inspecting => inspecting_candidates += 1,
                        WatchFolderCandidateReason::Ready => ready_candidates += 1,
                        WatchFolderCandidateReason::Incomplete => {
                            blocked_candidates += 1;
                            incomplete_candidates += 1;
                        }
                        WatchFolderCandidateReason::Unsupported => {
                            blocked_candidates += 1;
                            unsupported_candidates += 1;
                        }
                    }
                    if classification.newly_ready {
                        newly_ready_candidates += 1;
                    }
                    self.record_candidate(RecordAcquisitionIntakeCandidateRequest {
                        id: None,
                        target_library_id: library.id,
                        source_kind: AcquisitionIntakeSourceKind::WatchFolder,
                        source_key,
                        source_uri: metadata.uri.to_string(),
                        display_name: file_name(&metadata.uri).map(str::to_owned),
                        intended_locator: None,
                        size_bytes: metadata.len,
                        fingerprint: metadata.fingerprint.clone(),
                        managed_import_artifact_id: None,
                        state: Some(classification.state),
                        diagnostics_json: Some(classification.diagnostics_json()),
                    })
                    .await?;
                    recorded_candidates += 1;
                }
                ObjectKind::Other => {}
            }
        }

        Ok(WatchFolderDiscoveryDiagnostic {
            target_library_id: library.id,
            root_scheme: Some(root_uri.scheme().to_owned()),
            root_uri_redacted: redact_uri(root_uri.as_str()),
            ready_candidates,
            inspecting_candidates,
            blocked_candidates,
            incomplete_candidates,
            unsupported_candidates,
            suppressed_candidates,
            recorded_candidates,
            newly_ready_candidates,
            active_suppressions: self
                .watch_folder_suppression
                .list_active_for_library(library.id)
                .await?,
            failures,
            writes_library: false,
            managed_import_artifacts_created: false,
            promotion_apply: false,
        })
    }

    async fn find_existing_watch_folder_candidate(
        &self,
        library_id: LibraryId,
        metadata: &ObjectMetadata,
    ) -> Result<(String, Option<AcquisitionIntakeCandidateRecord>)> {
        let source_key = watch_folder_candidate_source_key(&metadata.uri);
        let existing = self
            .store
            .find_acquisition_intake_candidate_by_source_key(
                library_id,
                &AcquisitionIntakeSourceKind::WatchFolder,
                &source_key,
            )
            .await?;
        if existing.is_some() {
            return Ok((source_key, existing));
        }

        let legacy_source_key = legacy_watch_folder_source_key(metadata);
        if legacy_source_key != source_key {
            let existing = self
                .store
                .find_acquisition_intake_candidate_by_source_key(
                    library_id,
                    &AcquisitionIntakeSourceKind::WatchFolder,
                    &legacy_source_key,
                )
                .await?;
            if existing.is_some() {
                return Ok((legacy_source_key, existing));
            }
        }

        Ok((source_key, None))
    }

    pub(crate) async fn accept_candidate(
        &self,
        request: AcceptAcquisitionIntakeCandidateRequest,
    ) -> Result<AcquisitionIntakeAcceptanceDiagnostic> {
        let candidate = self
            .store
            .get_acquisition_intake_candidate(request.candidate_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "acquisition_intake_candidate",
                id: request.candidate_id.to_string(),
            })?;
        validate_acceptance_state(&candidate)?;

        if let (Some(linked), Some(requested)) = (
            candidate.managed_import_artifact_id,
            request.managed_import_artifact_id,
        ) {
            if linked != requested {
                return Err(NakoError::Conflict {
                    message: format!(
                        "acquisition intake candidate is already linked to managed import artifact {linked}"
                    ),
                });
            }
        }

        let (artifact, artifact_reused) = match candidate.managed_import_artifact_id {
            Some(artifact_id) => {
                let artifact = self
                    .store
                    .get_managed_import_artifact(artifact_id)
                    .await?
                    .ok_or_else(|| NakoError::NotFound {
                        entity: "managed_import_artifact",
                        id: artifact_id.to_string(),
                    })?;
                (artifact, true)
            }
            None => self.resolve_or_create_artifact(&candidate, request).await?,
        };

        let updated = self
            .store
            .link_acquisition_intake_candidate_managed_import_artifact(
                candidate.id,
                artifact.id,
                super::current_time_ms()?,
                Some(
                    serde_json::json!({
                        "accepted": true,
                        "managed_import_artifact_id": artifact.id,
                        "writes_library": false,
                        "promotion_apply": false
                    })
                    .to_string(),
                ),
            )
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "acquisition_intake_candidate",
                id: candidate.id.to_string(),
            })?;

        Ok(AcquisitionIntakeAcceptanceDiagnostic {
            candidate: AcquisitionIntakeCandidateDiagnostic::from_record(updated),
            artifact_id: artifact.id,
            artifact_state: artifact.state,
            replayed: candidate.managed_import_artifact_id == Some(artifact.id),
            artifact_reused,
            writes_library: false,
            promotion_apply: false,
            media_source_created: false,
        })
    }

    async fn resolve_or_create_artifact(
        &self,
        candidate: &AcquisitionIntakeCandidateRecord,
        request: AcceptAcquisitionIntakeCandidateRequest,
    ) -> Result<(ManagedImportArtifactRecord, bool)> {
        if let Some(artifact_id) = request.managed_import_artifact_id {
            let artifact = self
                .store
                .get_managed_import_artifact(artifact_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "managed_import_artifact",
                    id: artifact_id.to_string(),
                })?;
            validate_link_target(candidate, &artifact)?;
            return Ok((artifact, true));
        }

        let source_kind = managed_import_source_kind(&candidate.source_kind);
        if let Some(existing) = self
            .store
            .find_managed_import_artifact_by_source(
                candidate.target_library_id,
                &source_kind,
                &candidate.source_uri,
            )
            .await?
        {
            validate_link_target(candidate, &existing)?;
            return Ok((existing, true));
        }

        let diagnostic = self
            .managed_import
            .create_artifact(CreateManagedImportArtifactRequest {
                id: None,
                target_library_id: candidate.target_library_id,
                source_kind,
                source_uri: candidate.source_uri.clone(),
                staging_manifest_id: None,
                artifact_uri: None,
                original_file_name: candidate.display_name.clone(),
                intended_locator: candidate.intended_locator.clone(),
                size_bytes: candidate.size_bytes,
                fingerprint: candidate.fingerprint.clone(),
                state: Some(ManagedImportArtifactState::Proposed),
                diagnostics_json: candidate.diagnostics_json.clone(),
            })
            .await?;
        let artifact = self
            .store
            .get_managed_import_artifact(diagnostic.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_import_artifact",
                id: diagnostic.id.to_string(),
            })?;
        Ok((artifact, false))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordAcquisitionIntakeCandidateRequest {
    pub(crate) id: Option<AcquisitionIntakeCandidateId>,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: AcquisitionIntakeSourceKind,
    pub(crate) source_key: String,
    pub(crate) source_uri: String,
    pub(crate) display_name: Option<String>,
    pub(crate) intended_locator: Option<String>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) fingerprint: Option<String>,
    pub(crate) managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub(crate) state: Option<AcquisitionIntakeCandidateState>,
    pub(crate) diagnostics_json: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordResourceSearchSelectionRequest {
    pub(crate) target_library_id: LibraryId,
    pub(crate) addon_id: AddonId,
    pub(crate) manifest_id: String,
    pub(crate) query: String,
    pub(crate) result: AddonResourceSearchResult,
    pub(crate) selected_link: AddonResourceLink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordResourceSearchSelectionDiagnostic {
    pub(crate) candidate: AcquisitionIntakeCandidateDiagnostic,
    pub(crate) idempotent_replay: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AcceptAcquisitionIntakeCandidateRequest {
    pub(crate) candidate_id: AcquisitionIntakeCandidateId,
    pub(crate) managed_import_artifact_id: Option<ManagedImportArtifactId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoverWatchFolderCandidatesRequest {
    pub(crate) target_library_id: LibraryId,
    pub(crate) root_uri: Option<StorageUri>,
    pub(crate) max_depth: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct AcquisitionIntakeCandidateDiagnostics {
    pub(crate) limit: u32,
    pub(crate) offset: u64,
    pub(crate) returned: usize,
    pub(crate) candidates: Vec<AcquisitionIntakeCandidateDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct AcquisitionIntakeCandidateDiagnostic {
    pub(crate) id: AcquisitionIntakeCandidateId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: String,
    pub(crate) custom_source_kind: bool,
    pub(crate) source_scheme: Option<String>,
    pub(crate) source_uri_redacted: String,
    pub(crate) source_key_fingerprint: String,
    pub(crate) has_display_name: bool,
    pub(crate) has_intended_locator: bool,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) has_fingerprint: bool,
    pub(crate) managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub(crate) state: AcquisitionIntakeCandidateState,
    pub(crate) has_diagnostics: bool,
    pub(crate) first_seen_at_ms: i64,
    pub(crate) last_seen_at_ms: i64,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct AcquisitionIntakeAcceptanceDiagnostic {
    pub(crate) candidate: AcquisitionIntakeCandidateDiagnostic,
    pub(crate) artifact_id: ManagedImportArtifactId,
    pub(crate) artifact_state: ManagedImportArtifactState,
    pub(crate) replayed: bool,
    pub(crate) artifact_reused: bool,
    pub(crate) writes_library: bool,
    pub(crate) promotion_apply: bool,
    pub(crate) media_source_created: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WatchFolderDiscoveryDiagnostic {
    pub(crate) target_library_id: LibraryId,
    pub(crate) root_scheme: Option<String>,
    pub(crate) root_uri_redacted: String,
    pub(crate) ready_candidates: u64,
    pub(crate) inspecting_candidates: u64,
    pub(crate) blocked_candidates: u64,
    pub(crate) incomplete_candidates: u64,
    pub(crate) unsupported_candidates: u64,
    pub(crate) suppressed_candidates: u64,
    pub(crate) recorded_candidates: u64,
    pub(crate) newly_ready_candidates: u64,
    pub(crate) active_suppressions: Vec<PlannedWatchFolderWriteSuppressionDiagnostic>,
    pub(crate) failures: Vec<WatchFolderDiscoveryFailureDiagnostic>,
    pub(crate) writes_library: bool,
    pub(crate) managed_import_artifacts_created: bool,
    pub(crate) promotion_apply: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WatchFolderDiscoveryFailureDiagnostic {
    pub(crate) uri_redacted: String,
    pub(crate) safe_message: String,
}

impl WatchFolderDiscoveryFailureDiagnostic {
    fn from_error(uri: StorageUri, err: NakoError) -> Self {
        Self {
            uri_redacted: redact_uri(uri.as_str()),
            safe_message: safe_error_message(&err),
        }
    }
}

impl AcquisitionIntakeCandidateDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: AcquisitionIntakeCandidateRecord) -> Self {
        let (source_kind, source_kind_key) = record.source_kind.as_parts();
        Self {
            id: record.id,
            target_library_id: record.target_library_id,
            source_kind: source_kind.to_owned(),
            custom_source_kind: !source_kind_key.is_empty(),
            source_scheme: uri_scheme(&record.source_uri).map(str::to_owned),
            source_uri_redacted: redact_uri(&record.source_uri),
            source_key_fingerprint: fingerprint_key(&record.source_key),
            has_display_name: record.display_name.is_some(),
            has_intended_locator: record.intended_locator.is_some(),
            size_bytes: record.size_bytes,
            has_fingerprint: record.fingerprint.is_some(),
            managed_import_artifact_id: record.managed_import_artifact_id,
            state: record.state,
            has_diagnostics: record.diagnostics_json.is_some(),
            first_seen_at_ms: record.first_seen_at_ms,
            last_seen_at_ms: record.last_seen_at_ms,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WatchFolderCandidateClassification {
    state: AcquisitionIntakeCandidateState,
    reason: WatchFolderCandidateReason,
    stable_candidate: Option<StableIntakeCandidateEvidence>,
    newly_ready: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum WatchFolderCandidateReason {
    Inspecting,
    Ready,
    Incomplete,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct WatchFolderCandidateDiagnostics {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    watch_folder: bool,
    classification: WatchFolderCandidateReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stable_candidate: Option<StableIntakeCandidateEvidence>,
    #[serde(default)]
    writes_library: bool,
    #[serde(default)]
    managed_import_artifact_created: bool,
    #[serde(default)]
    promotion_apply: bool,
}

impl WatchFolderCandidateClassification {
    fn diagnostics_json(self) -> String {
        serde_json::to_string(&WatchFolderCandidateDiagnostics {
            schema: Some("nako.watch_folder_candidate.v2".to_owned()),
            watch_folder: true,
            classification: self.reason,
            stable_candidate: self.stable_candidate,
            writes_library: false,
            managed_import_artifact_created: false,
            promotion_apply: false,
        })
        .expect("watch-folder diagnostics should serialize")
    }
}

fn classify_watch_folder_candidate(
    metadata: &ObjectMetadata,
    existing: Option<&AcquisitionIntakeCandidateRecord>,
) -> WatchFolderCandidateClassification {
    if is_incomplete_candidate(metadata.uri.as_str()) {
        return WatchFolderCandidateClassification {
            state: AcquisitionIntakeCandidateState::Blocked,
            reason: WatchFolderCandidateReason::Incomplete,
            stable_candidate: None,
            newly_ready: false,
        };
    }

    if is_supported_media(metadata.uri.as_str()) {
        let previous = previous_watch_folder_stable_candidate(existing, metadata);
        let decision = observe_stable_intake_candidate(
            previous.as_ref(),
            watch_folder_observation_key(metadata),
        );
        let state = match decision.state {
            StableIntakeCandidateState::Inspecting => AcquisitionIntakeCandidateState::Inspecting,
            StableIntakeCandidateState::Stable => AcquisitionIntakeCandidateState::Ready,
        };
        WatchFolderCandidateClassification {
            state,
            reason: match state {
                AcquisitionIntakeCandidateState::Inspecting => {
                    WatchFolderCandidateReason::Inspecting
                }
                AcquisitionIntakeCandidateState::Ready => WatchFolderCandidateReason::Ready,
                _ => unreachable!(
                    "supported watch-folder candidates only classify as inspecting or ready"
                ),
            },
            stable_candidate: Some(decision.evidence),
            newly_ready: state == AcquisitionIntakeCandidateState::Ready
                && existing.is_none_or(|candidate| {
                    candidate.state != AcquisitionIntakeCandidateState::Ready
                }),
        }
    } else {
        WatchFolderCandidateClassification {
            state: AcquisitionIntakeCandidateState::Blocked,
            reason: WatchFolderCandidateReason::Unsupported,
            stable_candidate: None,
            newly_ready: false,
        }
    }
}

fn is_supported_media(value: &str) -> bool {
    extension(value).is_some_and(|extension| {
        DEFAULT_MEDIA_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(extension))
    })
}

fn is_incomplete_candidate(value: &str) -> bool {
    extension(value).is_some_and(|extension| {
        INCOMPLETE_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(extension))
    })
}

fn watch_folder_candidate_source_key(uri: &StorageUri) -> String {
    format!("watch_folder:{}", uri)
}

fn legacy_watch_folder_source_key(metadata: &ObjectMetadata) -> String {
    match (&metadata.fingerprint, metadata.len) {
        (Some(fingerprint), Some(size_bytes)) => {
            format!(
                "{}|size={size_bytes}|fingerprint={fingerprint}",
                metadata.uri
            )
        }
        (Some(fingerprint), None) => format!("{}|fingerprint={fingerprint}", metadata.uri),
        (None, Some(size_bytes)) => format!("{}|size={size_bytes}", metadata.uri),
        (None, None) => metadata.uri.to_string(),
    }
}

fn watch_folder_observation_key(metadata: &ObjectMetadata) -> String {
    let mut hasher = Sha256::new();
    update_watch_folder_hash_part(&mut hasher, "watch-folder-observation-v1");
    update_watch_folder_hash_part(&mut hasher, metadata.uri.as_str());
    update_watch_folder_hash_part(
        &mut hasher,
        match metadata.kind {
            ObjectKind::File => "file",
            ObjectKind::Directory => "directory",
            ObjectKind::Symlink => "symlink",
            ObjectKind::Other => "other",
        },
    );
    update_watch_folder_hash_part(
        &mut hasher,
        &metadata
            .len
            .map_or_else(String::new, |value| value.to_string()),
    );
    update_watch_folder_hash_part(&mut hasher, metadata.modified_at.as_deref().unwrap_or(""));
    update_watch_folder_hash_part(&mut hasher, metadata.etag.as_deref().unwrap_or(""));
    update_watch_folder_hash_part(&mut hasher, metadata.fingerprint.as_deref().unwrap_or(""));

    format!("watch_folder_observation:v1:sha256:{:x}", hasher.finalize())
}

fn previous_watch_folder_stable_candidate(
    existing: Option<&AcquisitionIntakeCandidateRecord>,
    metadata: &ObjectMetadata,
) -> Option<StableIntakeCandidateEvidence> {
    let parsed = existing.and_then(parse_watch_folder_candidate_diagnostics);
    parsed
        .and_then(|diagnostics| diagnostics.stable_candidate)
        .or_else(|| {
            existing
                .filter(|candidate| candidate.state == AcquisitionIntakeCandidateState::Ready)
                .map(|_| StableIntakeCandidateEvidence {
                    observation_key: watch_folder_observation_key(metadata),
                    consecutive_stable_observations: STABLE_INTAKE_REQUIRED_OBSERVATIONS,
                })
        })
}

fn parse_watch_folder_candidate_diagnostics(
    candidate: &AcquisitionIntakeCandidateRecord,
) -> Option<WatchFolderCandidateDiagnostics> {
    candidate
        .diagnostics_json
        .as_deref()
        .and_then(|value| serde_json::from_str(value).ok())
}

fn update_watch_folder_hash_part(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_string().as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    hasher.update([0xff]);
}

fn file_name(uri: &StorageUri) -> Option<&str> {
    uri.path_part()
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
}

fn extension(path: &str) -> Option<&str> {
    let file_name = path.rsplit('/').next()?;
    let (_stem, extension) = file_name.rsplit_once('.')?;
    (!extension.is_empty()).then_some(extension)
}

fn validate_acceptance_state(candidate: &AcquisitionIntakeCandidateRecord) -> Result<()> {
    if matches!(
        candidate.state,
        AcquisitionIntakeCandidateState::Rejected
            | AcquisitionIntakeCandidateState::Failed
            | AcquisitionIntakeCandidateState::Superseded
    ) {
        return Err(NakoError::Conflict {
            message: format!(
                "acquisition intake candidate cannot be accepted from state: {}",
                candidate.state.as_str()
            ),
        });
    }

    Ok(())
}

fn validate_link_target(
    candidate: &AcquisitionIntakeCandidateRecord,
    artifact: &ManagedImportArtifactRecord,
) -> Result<()> {
    if artifact.target_library_id != candidate.target_library_id {
        return Err(NakoError::Conflict {
            message: "managed import artifact targets a different library".to_owned(),
        });
    }

    Ok(())
}

fn managed_import_source_kind(
    source_kind: &AcquisitionIntakeSourceKind,
) -> ManagedImportSourceKind {
    match source_kind {
        AcquisitionIntakeSourceKind::WatchFolder => ManagedImportSourceKind::WatchedCandidate,
        AcquisitionIntakeSourceKind::OperatorSubmitted => ManagedImportSourceKind::LocalFile,
        AcquisitionIntakeSourceKind::ExternalDownloadOutput => {
            ManagedImportSourceKind::Other("external_download_output".to_owned())
        }
        AcquisitionIntakeSourceKind::AddonProposed => ManagedImportSourceKind::AddonProposed,
        AcquisitionIntakeSourceKind::ResourceSearchSelection => {
            ManagedImportSourceKind::ResourceSearchSelection
        }
        AcquisitionIntakeSourceKind::Other(value) => ManagedImportSourceKind::Other(value.clone()),
    }
}

fn selected_resource_search_link_uri(link: &AddonResourceLink) -> Result<String> {
    optional_non_empty(Some(link.normalized_url.clone()))
        .or_else(|| optional_non_empty(Some(link.url.clone())))
        .ok_or_else(|| NakoError::InvalidInput {
            message: "resource search selection link uri cannot be empty".to_owned(),
        })
}

fn resource_search_selection_source_key(
    addon_id: AddonId,
    manifest_id: &str,
    link_type: AddonResourceLinkType,
    source_uri: &str,
) -> String {
    let material = format!(
        "nako.resource-search-selection.v1\0{addon_id}\0{manifest_id}\0{}\0{source_uri}",
        link_type.as_str()
    );
    format!("resource_search_selection:sha256:{}", sha256_hex(&material))
}

fn resource_search_selection_diagnostics_json(
    request: &RecordResourceSearchSelectionRequest,
    query: &str,
    manifest_id: &str,
    result_id: &str,
    source_uri: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "schema": "nako.acquisition_intake.resource_search_selection.v1",
        "resource_search_selection": true,
        "addon_id": request.addon_id,
        "manifest_id": manifest_id,
        "query_fingerprint": fingerprint_key(query),
        "result": {
            "id_fingerprint": fingerprint_key(result_id),
            "source": optional_trimmed(&request.result.source),
            "score": request.result.score,
            "has_title": !request.result.title.trim().is_empty(),
            "has_content": request
                .result
                .content
                .as_ref()
                .is_some_and(|content| !content.trim().is_empty()),
            "tag_count": request.result.tags.len(),
            "image_count": request.result.images.len(),
            "link_count": request.result.links.len(),
        },
        "link": {
            "type": request.selected_link.link_type.as_str(),
            "source": optional_trimmed(&request.selected_link.source),
            "source_ref_redacted": redact_uri(source_uri),
            "source_ref_fingerprint": fingerprint_key(source_uri),
            "has_password": request.selected_link.password.is_some(),
            "has_note": request
                .selected_link
                .note
                .as_ref()
                .is_some_and(|note| !note.trim().is_empty()),
        },
        "writes_library": false,
        "managed_import_artifact_created": false,
        "promotion_apply": false,
    }))
    .map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize resource search selection diagnostics: {err}"),
    })
}

fn require_non_empty(label: &str, value: String) -> Result<String> {
    optional_non_empty(Some(value)).ok_or_else(|| NakoError::InvalidInput {
        message: format!("{label} cannot be empty"),
    })
}

fn optional_non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn optional_trimmed(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn uri_scheme(value: &str) -> Option<&str> {
    value
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.is_empty())
}

fn redact_uri(value: &str) -> String {
    uri_scheme(value)
        .map(|scheme| format!("{scheme}://<redacted>"))
        .unwrap_or_else(|| "<redacted>".to_owned())
}

fn fingerprint_key(value: &str) -> String {
    let digest = sha256_hex(value);
    format!("sha256:{}", &digest[..32])
}

fn sha256_hex(value: &str) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn safe_error_message(err: &NakoError) -> String {
    match err {
        NakoError::NotFound { entity, .. } => format!("{entity} was not found"),
        NakoError::InvalidInput { .. } => "invalid watch-folder input".to_owned(),
        NakoError::Conflict { .. } => "watch-folder conflict".to_owned(),
        NakoError::Unauthorized { .. } => "watch-folder access is unauthorized".to_owned(),
        NakoError::Forbidden { .. } => "watch-folder access is forbidden".to_owned(),
        NakoError::Unsupported(_) => "watch-folder operation is unsupported".to_owned(),
        NakoError::Provider { provider, .. } => format!("{provider} provider error"),
        NakoError::Storage { kind, .. } => format!("storage error: {kind:?}"),
        NakoError::Database { .. } => "database error".to_owned(),
    }
}

const DEFAULT_MEDIA_EXTENSIONS: &[&str] = &[
    "3gp", "avi", "flv", "m2ts", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "mts", "ts", "webm",
    "wmv",
];

const INCOMPLETE_EXTENSIONS: &[&str] = &["part", "partial", "crdownload", "tmp", "download"];
