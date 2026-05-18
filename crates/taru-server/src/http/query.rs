use serde::Deserialize;
use taru_core::{
    AddonStatus, CatalogGovernanceItemListFilter,
    DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI, DomainEventKind, IngestionFailurePhase,
    IngestionFailureStatus, JobKind, JobListFilter, JobStatus, LibraryId, MediaSourceId,
    OutboxEventListFilter, OutboxEventStatus, PageRequest, StagingPurpose, StagingState, TaruError,
    TranscodeSessionKind, TranscodeSessionListFilter, TranscodeSessionState,
};

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

impl SearchPageQuery {
    pub(super) fn page(&self) -> PageQuery {
        PageQuery {
            limit: self.limit,
            offset: self.offset,
        }
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
    pub(super) fn into_filter_and_page(self) -> Result<(JobListFilter, PageRequest), TaruError> {
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
                            .map_err(|err| TaruError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| TaruError::InvalidInput {
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
    ) -> Result<(OutboxEventListFilter, PageRequest), TaruError> {
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
                            .map_err(|err| TaruError::InvalidInput {
                                message: format!("invalid library_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| TaruError::InvalidInput {
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
    pub(super) source_id: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) state: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl PlaybackSessionListQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(TranscodeSessionListFilter, PageRequest), TaruError> {
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
            TranscodeSessionListFilter {
                source_id: self
                    .source_id
                    .map(|value| {
                        value
                            .parse::<MediaSourceId>()
                            .map_err(|err| TaruError::InvalidInput {
                                message: format!("invalid source_id filter: {err}"),
                            })
                    })
                    .transpose()?,
                kind: self
                    .kind
                    .map(parse_transcode_session_kind_filter)
                    .transpose()?,
                state: self
                    .state
                    .map(parse_transcode_session_state_filter)
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
    ) -> Result<(Option<StagingPurpose>, Option<StagingState>, PageRequest), TaruError> {
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
pub(super) struct CatalogGovernanceItemsQuery {
    pub(super) library_id: Option<String>,
    pub(super) max_confidence_milli: Option<String>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl CatalogGovernanceItemsQuery {
    pub(super) fn into_filter_and_page(
        self,
    ) -> Result<(CatalogGovernanceItemListFilter, PageRequest), TaruError> {
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
                            .map_err(|err| TaruError::InvalidInput {
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

fn parse_job_status_filter(value: String) -> Result<JobStatus, TaruError> {
    JobStatus::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid status filter: {value}"),
    })
}

fn parse_job_kind_filter(value: String) -> Result<JobKind, TaruError> {
    JobKind::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid kind filter: {value}"),
    })
}

fn parse_domain_event_kind_filter(value: String) -> Result<DomainEventKind, TaruError> {
    DomainEventKind::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid kind filter: {value}"),
    })
}

fn parse_outbox_event_status_filter(value: String) -> Result<OutboxEventStatus, TaruError> {
    OutboxEventStatus::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid status filter: {value}"),
    })
}

fn parse_transcode_session_kind_filter(value: String) -> Result<TranscodeSessionKind, TaruError> {
    TranscodeSessionKind::parse(&value).ok_or_else(|| TaruError::InvalidInput {
        message: format!("invalid kind filter: {value}"),
    })
}

fn parse_transcode_session_state_filter(value: String) -> Result<TranscodeSessionState, TaruError> {
    TranscodeSessionState::parse(&value).ok_or_else(|| TaruError::InvalidInput {
        message: format!("invalid state filter: {value}"),
    })
}

fn parse_staging_purpose_filter(value: String) -> Result<StagingPurpose, TaruError> {
    StagingPurpose::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid purpose filter: {value}"),
    })
}

fn parse_staging_state_filter(value: String) -> Result<StagingState, TaruError> {
    StagingState::parse(&value).map_err(|_err| TaruError::InvalidInput {
        message: format!("invalid state filter: {value}"),
    })
}

fn parse_u32_filter(name: &str, value: String) -> Result<u32, TaruError> {
    value.parse::<u32>().map_err(|err| TaruError::InvalidInput {
        message: format!("invalid {name} filter: {err}"),
    })
}

fn parse_u64_filter(name: &str, value: String) -> Result<u64, TaruError> {
    value.parse::<u64>().map_err(|err| TaruError::InvalidInput {
        message: format!("invalid {name} filter: {err}"),
    })
}

fn parse_confidence_milli_filter(value: String) -> Result<u16, TaruError> {
    let confidence = value
        .parse::<u16>()
        .map_err(|err| TaruError::InvalidInput {
            message: format!("invalid max_confidence_milli filter: {err}"),
        })?;

    if confidence > 1_000 {
        return Err(TaruError::InvalidInput {
            message: "max_confidence_milli must be less than or equal to 1000".to_owned(),
        });
    }

    Ok(confidence)
}

impl TryFrom<PageQuery> for PageRequest {
    type Error = TaruError;

    fn try_from(value: PageQuery) -> Result<Self, Self::Error> {
        let limit = value.limit.unwrap_or(PageRequest::DEFAULT_LIMIT);

        if limit > PageRequest::MAX_LIMIT {
            return Err(TaruError::InvalidInput {
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
