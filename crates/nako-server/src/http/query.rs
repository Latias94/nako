use nako_core::{
    AcquisitionIntakeCandidateListFilter, AcquisitionIntakeCandidateState,
    AcquisitionIntakeSourceKind, AddonStatus, CatalogGovernanceItemListFilter,
    DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI, DomainEventKind, IngestionFailurePhase,
    IngestionFailureStatus, JobKind, JobListFilter, JobStatus, LibraryId, LibraryItemBrowseFacet,
    LibraryItemBrowseQuery, LibraryItemBrowseSortKey, LibraryItemBrowseSortOrder,
    LibraryItemWatchStateFilter, ManagedArtworkArtifactLifecycleFilter, ManagedImportArtifactId,
    MediaKind, MediaSourceId, NakoError, OutboxEventListFilter, OutboxEventStatus, PageRequest,
    PlaybackSessionListFilter, PlaybackSessionState, StagingPurpose, StagingState,
    TranscodeSessionId,
};
use serde::Deserialize;

use crate::app::ImageVariantRequest;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct PageQuery {
    pub(super) limit: Option<u32>,
    pub(super) offset: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct SearchPageQuery {
    #[serde(default)]
    pub(super) q: String,
    pub(super) facet: Option<String>,
    pub(super) limit: Option<u32>,
    pub(super) offset: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct LibraryItemsQuery {
    pub(super) limit: Option<u32>,
    pub(super) offset: Option<u64>,
    pub(super) sort: Option<String>,
    pub(super) order: Option<String>,
    pub(super) facets: Vec<String>,
    pub(super) watch_state: Option<String>,
}

impl LibraryItemsQuery {
    pub(super) fn from_raw_query(raw_query: Option<&str>) -> Result<Self, NakoError> {
        let mut query = LibraryItemsQuery::default();

        let Some(raw_query) = raw_query else {
            return Ok(query);
        };

        for (name, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            let value = value.into_owned();
            match name.as_ref() {
                "limit" => query.limit = Some(parse_u32_filter("limit", value)?),
                "offset" => query.offset = Some(parse_u64_filter("offset", value)?),
                "sort" => query.sort = Some(value),
                "order" => query.order = Some(value),
                "facet" => query.facets.push(value),
                "watch_state" => query.watch_state = Some(value),
                _ => {}
            }
        }

        Ok(query)
    }

    pub(super) fn into_browse_query(self) -> Result<LibraryItemBrowseQuery, NakoError> {
        Ok(LibraryItemBrowseQuery {
            page: PageQuery {
                limit: self.limit,
                offset: self.offset,
            }
            .try_into()?,
            sort: parse_library_item_sort(self.sort)?,
            order: parse_library_item_order(self.order)?,
            facets: parse_library_item_facets(self.facets)?,
            watch_state: parse_library_item_watch_state(self.watch_state)?,
        })
    }
}

impl SearchPageQuery {
    pub(super) fn page(&self) -> PageQuery {
        PageQuery {
            limit: self.limit,
            offset: self.offset,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ImageVariantQuery {
    pub(super) width: Option<String>,
    pub(super) height: Option<String>,
}

impl ImageVariantQuery {
    pub(super) fn into_variant_request(self) -> Result<ImageVariantRequest, NakoError> {
        if self.width.is_none() && self.height.is_none() {
            return Ok(ImageVariantRequest::original());
        }

        ImageVariantRequest::bounded(
            self.width
                .map(|value| parse_u32_filter("width", value))
                .transpose()?,
            self.height
                .map(|value| parse_u32_filter("height", value))
                .transpose()?,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct AddonListQuery {
    pub(super) status: Option<AddonStatus>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct IngestionFailureQuery {
    pub(super) phase: Option<IngestionFailurePhase>,
    pub(super) status: Option<IngestionFailureStatus>,
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct JobListQuery {
    pub(super) status: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) resource_class: Option<String>,
    pub(super) library_id: Option<String>,
    pub(super) source_id: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl JobListQuery {
    pub(super) fn into_filter_and_page(self) -> Result<(JobListFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            JobListFilter {
                status: self.status.map(parse_job_status_filter).transpose()?,
                kind: self.kind.map(parse_job_kind_filter).transpose()?,
                resource_class: self.resource_class,
                library_id: self
                    .library_id
                    .map(|value| {
                        value
                            .parse::<LibraryId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid source_id filter: {err}"),
                            })
                    })
                    .transpose()?,
            },
            page.try_into()?,
        ))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct OutboxEventListQuery {
    pub(super) kind: Option<String>,
    pub(super) status: Option<String>,
    pub(super) library_id: Option<String>,
    pub(super) source_id: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl OutboxEventListQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(OutboxEventListFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            OutboxEventListFilter {
                kind: self.kind.map(parse_domain_event_kind_filter).transpose()?,
                status: self
                    .status
                    .map(parse_outbox_event_status_filter)
                    .transpose()?,
                library_id: self
                    .library_id
                    .map(|value| {
                        value
                            .parse::<LibraryId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid source_id filter: {err}"),
                            })
                    })
                    .transpose()?,
            },
            page.try_into()?,
        ))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct PlaybackSessionListQuery {
    pub(super) principal_id: Option<String>,
    pub(super) source_id: Option<String>,
    pub(super) state: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct PlaybackSupportEvidenceQuery {
    pub(super) session_id: Option<String>,
    pub(super) source_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct AcquisitionIntakeCandidateListQuery {
    pub(super) library_id: Option<String>,
    pub(super) state: Option<String>,
    pub(super) source_kind: Option<String>,
    pub(super) managed_import_artifact_id: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl AcquisitionIntakeCandidateListQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(AcquisitionIntakeCandidateListFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            AcquisitionIntakeCandidateListFilter {
                target_library_id: self
                    .library_id
                    .map(|value| {
                        value
                            .parse::<LibraryId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                state: self
                    .state
                    .map(parse_acquisition_intake_candidate_state_filter)
                    .transpose()?,
                source_kind: self
                    .source_kind
                    .map(parse_acquisition_intake_source_kind_filter),
                managed_import_artifact_id: self
                    .managed_import_artifact_id
                    .map(|value| {
                        value.parse::<ManagedImportArtifactId>().map_err(|err| {
                            NakoError::InvalidInput {
                                message: format!(
                                    "invalid managed_import_artifact_id filter: {err}"
                                ),
                            }
                        })
                    })
                    .transpose()?,
            },
            page.try_into()?,
        ))
    }
}

impl PlaybackSupportEvidenceQuery {
    pub(super) fn into_context(
        self,
    ) -> Result<(Option<TranscodeSessionId>, Option<MediaSourceId>), NakoError> {
        Ok((
            self.session_id
                .map(|value| {
                    value
                        .parse::<TranscodeSessionId>()
                        .map_err(|err| NakoError::InvalidInput {
                            message: format!("invalid session_id filter: {err}"),
                        })
                })
                .transpose()?,
            self.source_id
                .map(|value| {
                    value
                        .parse::<MediaSourceId>()
                        .map_err(|err| NakoError::InvalidInput {
                            message: format!("invalid source_id filter: {err}"),
                        })
                })
                .transpose()?,
        ))
    }
}

impl PlaybackSessionListQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(PlaybackSessionListFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            PlaybackSessionListFilter {
                principal_id: self
                    .principal_id
                    .map(|value| {
                        value.parse().map_err(|err| NakoError::InvalidInput {
                            message: format!("invalid principal_id filter: {err}"),
                        })
                    })
                    .transpose()?,
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid source_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                state: self
                    .state
                    .map(parse_playback_session_state_filter)
                    .transpose()?,
            },
            page.try_into()?,
        ))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct StorageStagingQuery {
    pub(super) purpose: Option<String>,
    pub(super) state: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl StorageStagingQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(Option<StagingPurpose>, Option<StagingState>, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            self.purpose.map(parse_staging_purpose_filter).transpose()?,
            self.state.map(parse_staging_state_filter).transpose()?,
            page.try_into()?,
        ))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ArtworkArtifactLifecycleQuery {
    pub(super) cleanup_candidates_only: Option<bool>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl ArtworkArtifactLifecycleQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(ManagedArtworkArtifactLifecycleFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };
        let filter = if self.cleanup_candidates_only.unwrap_or(false) {
            ManagedArtworkArtifactLifecycleFilter::CleanupCandidates
        } else {
            ManagedArtworkArtifactLifecycleFilter::All
        };

        Ok((filter, page.try_into()?))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ArtworkGalleryQuery {
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl ArtworkGalleryQuery {
    pub(super) fn into_page(self) -> Result<PageRequest, NakoError> {
        PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        }
        .try_into()
    }
}

const DEFAULT_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT: u32 = 500;
const MAX_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT: u32 = 5_000;

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ArtworkArtifactStorageDriftQuery {
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
    pub(super) file_scan_limit: Option<String>,
}

impl ArtworkArtifactStorageDriftQuery {
    pub(super) fn into_page_and_file_scan_limit(self) -> Result<(PageRequest, u32), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };
        let file_scan_limit = self
            .file_scan_limit
            .map(|value| parse_u32_filter("file_scan_limit", value))
            .transpose()?
            .unwrap_or(DEFAULT_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT);
        let file_scan_limit = if file_scan_limit == 0 {
            DEFAULT_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT
        } else {
            file_scan_limit
        };
        if file_scan_limit > MAX_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "file_scan_limit must be less than or equal to {}",
                    MAX_ARTWORK_STORAGE_DRIFT_FILE_SCAN_LIMIT
                ),
            });
        }

        Ok((page.try_into()?, file_scan_limit))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ArtworkArtifactRemediationQuery {
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
    pub(super) file_scan_limit: Option<String>,
    pub(super) confirm: Option<bool>,
}

impl ArtworkArtifactRemediationQuery {
    pub(super) fn into_page_and_file_scan_limit(self) -> Result<(PageRequest, u32), NakoError> {
        ArtworkArtifactStorageDriftQuery {
            limit: self.limit,
            offset: self.offset,
            file_scan_limit: self.file_scan_limit,
        }
        .into_page_and_file_scan_limit()
    }

    pub(super) fn into_confirmed_file_scan_limit(self) -> Result<u32, NakoError> {
        if self.confirm != Some(true) {
            return Err(NakoError::InvalidInput {
                message: "confirm=true is required for managed artwork stray file cleanup"
                    .to_owned(),
            });
        }

        let (_page, file_scan_limit) = self.into_page_and_file_scan_limit()?;
        Ok(file_scan_limit)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct CatalogGovernanceItemsQuery {
    pub(super) library_id: Option<String>,
    pub(super) max_confidence_milli: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl CatalogGovernanceItemsQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(CatalogGovernanceItemListFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };
        let max_confidence_milli = self
            .max_confidence_milli
            .map(parse_confidence_milli_filter)
            .transpose()?
            .unwrap_or(DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI);

        Ok((
            CatalogGovernanceItemListFilter {
                library_id: self
                    .library_id
                    .map(|value| {
                        value
                            .parse::<LibraryId>()
                            .map_err(|err| NakoError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                max_confidence_milli,
            },
            page.try_into()?,
        ))
    }
}

fn parse_job_status_filter(value: String) -> Result<JobStatus, NakoError> {
    JobStatus::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid status filter: {value}"),
    })
}

fn parse_job_kind_filter(value: String) -> Result<JobKind, NakoError> {
    JobKind::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid kind filter: {value}"),
    })
}

fn parse_domain_event_kind_filter(value: String) -> Result<DomainEventKind, NakoError> {
    DomainEventKind::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid kind filter: {value}"),
    })
}

fn parse_outbox_event_status_filter(value: String) -> Result<OutboxEventStatus, NakoError> {
    OutboxEventStatus::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid status filter: {value}"),
    })
}

fn parse_playback_session_state_filter(value: String) -> Result<PlaybackSessionState, NakoError> {
    PlaybackSessionState::parse(&value).ok_or_else(|| NakoError::InvalidInput {
        message: format!("invalid state filter: {value}"),
    })
}

fn parse_staging_purpose_filter(value: String) -> Result<StagingPurpose, NakoError> {
    StagingPurpose::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid purpose filter: {value}"),
    })
}

fn parse_staging_state_filter(value: String) -> Result<StagingState, NakoError> {
    StagingState::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid state filter: {value}"),
    })
}

fn parse_acquisition_intake_candidate_state_filter(
    value: String,
) -> Result<AcquisitionIntakeCandidateState, NakoError> {
    AcquisitionIntakeCandidateState::parse(&value).map_err(|_err| NakoError::InvalidInput {
        message: format!("invalid state filter: {value}"),
    })
}

fn parse_acquisition_intake_source_kind_filter(value: String) -> AcquisitionIntakeSourceKind {
    match value.as_str() {
        "watch_folder" => AcquisitionIntakeSourceKind::WatchFolder,
        "operator_submitted" => AcquisitionIntakeSourceKind::OperatorSubmitted,
        "external_download_output" => AcquisitionIntakeSourceKind::ExternalDownloadOutput,
        "addon_proposed" => AcquisitionIntakeSourceKind::AddonProposed,
        "resource_search_selection" => AcquisitionIntakeSourceKind::ResourceSearchSelection,
        _ => AcquisitionIntakeSourceKind::Other(value),
    }
}

fn parse_library_item_sort(value: Option<String>) -> Result<LibraryItemBrowseSortKey, NakoError> {
    match value.as_deref().unwrap_or("date_added") {
        "title" => Ok(LibraryItemBrowseSortKey::Title),
        "release_date" => Ok(LibraryItemBrowseSortKey::ReleaseDate),
        "date_added" => Ok(LibraryItemBrowseSortKey::DateAdded),
        "last_played" => Ok(LibraryItemBrowseSortKey::LastPlayed),
        other => Err(NakoError::InvalidInput {
            message: format!("unsupported library item sort: {other}"),
        }),
    }
}

fn parse_library_item_order(
    value: Option<String>,
) -> Result<LibraryItemBrowseSortOrder, NakoError> {
    match value.as_deref().unwrap_or("desc") {
        "asc" => Ok(LibraryItemBrowseSortOrder::Asc),
        "desc" => Ok(LibraryItemBrowseSortOrder::Desc),
        other => Err(NakoError::InvalidInput {
            message: format!("unsupported library item order: {other}"),
        }),
    }
}

fn parse_library_item_facets(
    values: Vec<String>,
) -> Result<Vec<LibraryItemBrowseFacet>, NakoError> {
    let mut facets = Vec::new();
    for token in values
        .into_iter()
        .flat_map(|value| {
            value
                .split(',')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|value| !value.is_empty())
    {
        let Some((prefix, value)) = token.split_once(':') else {
            return Err(NakoError::InvalidInput {
                message: format!("invalid library item facet: {token}"),
            });
        };
        match prefix {
            "kind" => facets.push(LibraryItemBrowseFacet::Kind(parse_library_item_kind(
                value,
            )?)),
            other => {
                return Err(NakoError::InvalidInput {
                    message: format!("unsupported library item facet: {other}"),
                });
            }
        }
    }

    Ok(facets)
}

fn parse_library_item_kind(value: &str) -> Result<MediaKind, NakoError> {
    match value {
        "movie" => Ok(MediaKind::Movie),
        "series" => Ok(MediaKind::Series),
        "season" => Ok(MediaKind::Season),
        "episode" => Ok(MediaKind::Episode),
        "collection" => Ok(MediaKind::Collection),
        "extra" => Ok(MediaKind::Extra),
        "unknown" => Ok(MediaKind::Unknown),
        other => Err(NakoError::InvalidInput {
            message: format!("unsupported library item kind facet: {other}"),
        }),
    }
}

fn parse_library_item_watch_state(
    value: Option<String>,
) -> Result<LibraryItemWatchStateFilter, NakoError> {
    match value.as_deref().unwrap_or("any") {
        "any" => Ok(LibraryItemWatchStateFilter::Any),
        "watched" => Ok(LibraryItemWatchStateFilter::Watched),
        "unwatched" => Ok(LibraryItemWatchStateFilter::Unwatched),
        "in_progress" => Ok(LibraryItemWatchStateFilter::InProgress),
        other => Err(NakoError::InvalidInput {
            message: format!("unsupported library item watch_state: {other}"),
        }),
    }
}

pub(super) fn parse_u32_filter(name: &str, value: String) -> Result<u32, NakoError> {
    value.parse::<u32>().map_err(|err| NakoError::InvalidInput {
        message: format!("invalid {name} filter: {err}"),
    })
}

pub(super) fn parse_u64_filter(name: &str, value: String) -> Result<u64, NakoError> {
    value.parse::<u64>().map_err(|err| NakoError::InvalidInput {
        message: format!("invalid {name} filter: {err}"),
    })
}

fn parse_confidence_milli_filter(value: String) -> Result<u16, NakoError> {
    let confidence = value
        .parse::<u16>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid max_confidence_milli filter: {err}"),
        })?;

    if confidence > 1_000 {
        return Err(NakoError::InvalidInput {
            message: "max_confidence_milli must be less than or equal to 1000".to_owned(),
        });
    }

    Ok(confidence)
}

impl TryFrom<PageQuery> for PageRequest {
    type Error = NakoError;

    fn try_from(value: PageQuery) -> Result<Self, Self::Error> {
        let limit = value.limit.unwrap_or(PageRequest::DEFAULT_LIMIT);

        if limit > PageRequest::MAX_LIMIT {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "limit must be less than or equal to {}",
                    PageRequest::MAX_LIMIT
                ),
            });
        }

        Ok(PageRequest {
            limit,
            offset: value.offset.unwrap_or_default(),
        }
        .clamped())
    }
}
