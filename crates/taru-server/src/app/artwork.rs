use serde::Serialize;
use taru_api::AcceptManagedArtworkCandidateResponse;
use taru_core::{
    ArtworkCandidateId, ArtworkCandidateRepository, ArtworkCandidateStatus, JobId, JobKind,
    LibraryItemRepository, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    ManagedArtworkRepository, MediaRepository, NewJob, NewManagedArtworkIngest, Result, TaruError,
};
use taru_db::SqliteStore;

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkAppService {
    store: SqliteStore,
}

impl ManagedArtworkAppService {
    pub(crate) fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    pub(crate) async fn accept_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<AcceptManagedArtworkCandidateResponse> {
        let candidate = self
            .store
            .get_artwork_candidate(candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: candidate_id.to_string(),
            })?;

        if candidate.status == ArtworkCandidateStatus::Rejected {
            return Err(TaruError::InvalidInput {
                message: "rejected artwork candidates cannot be accepted".to_owned(),
            });
        }

        self.store
            .get_media_item(candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: candidate.item_id.to_string(),
            })?;
        self.store
            .get_library_item_state(candidate.library_id, candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library_item_state",
                id: format!("{}:{}", candidate.library_id, candidate.item_id),
            })?;

        let job_id = JobId::new();
        let input = ManagedArtworkIngestJobInput {
            candidate_id,
            library_id: candidate.library_id,
            item_id: candidate.item_id,
            image_kind: image_kind_label(&candidate.kind).to_owned(),
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize managed artwork ingest job input: {err}"),
        })?;
        let acceptance = self
            .store
            .accept_managed_artwork_candidate_ingest(
                candidate_id,
                NewManagedArtworkIngest {
                    id: ManagedArtworkIngestId::new(),
                    candidate_id,
                    job_id,
                    library_id: candidate.library_id,
                    item_id: candidate.item_id,
                    kind: candidate.kind,
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    library_id: Some(candidate.library_id),
                    source_id: None,
                    input_json: Some(input_json),
                },
            )
            .await?;

        Ok(AcceptManagedArtworkCandidateResponse::from_acceptance(
            acceptance,
        ))
    }
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobInput {
    candidate_id: ArtworkCandidateId,
    library_id: taru_core::LibraryId,
    item_id: taru_core::MediaItemId,
    image_kind: String,
}

fn image_kind_label(kind: &taru_core::ImageKind) -> &'static str {
    match kind {
        taru_core::ImageKind::Poster => "poster",
        taru_core::ImageKind::Backdrop => "backdrop",
        taru_core::ImageKind::Logo => "logo",
        taru_core::ImageKind::Thumbnail => "thumbnail",
        taru_core::ImageKind::Banner => "banner",
        taru_core::ImageKind::Other(_) => "other",
    }
}
