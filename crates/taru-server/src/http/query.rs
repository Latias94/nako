use serde::Deserialize;
use taru_core::{AddonStatus, PageRequest, TaruError};

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
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct AddonListQuery {
    pub(super) status: Option<AddonStatus>,
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
