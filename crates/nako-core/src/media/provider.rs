use serde::{Deserialize, Serialize};

use crate::{MediaItemId, ProviderMappingId, ProviderSubjectId};

use super::metadata::MetadataSource;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalProvider {
    Tmdb,
    Douban,
    Bangumi,
    Imdb,
    Local,
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ExternalId {
    pub provider: ExternalProvider,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSubjectKind {
    Movie,
    Series,
    Season,
    Episode,
    Collection,
    Subject,
    Person,
    Other(String),
}

impl ProviderSubjectKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::Movie => ("movie", ""),
            Self::Series => ("series", ""),
            Self::Season => ("season", ""),
            Self::Episode => ("episode", ""),
            Self::Collection => ("collection", ""),
            Self::Subject => ("subject", ""),
            Self::Person => ("person", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "movie" => Self::Movie,
            "series" => Self::Series,
            "season" => Self::Season,
            "episode" => Self::Episode,
            "collection" => Self::Collection,
            "subject" => Self::Subject,
            "person" => Self::Person,
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderSubject {
    pub id: ProviderSubjectId,
    pub provider: ExternalProvider,
    pub subject_kind: ProviderSubjectKind,
    pub subject_key: String,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMappingStatus {
    Candidate,
    Accepted,
    Rejected,
}

impl ProviderMappingStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "candidate" => Ok(Self::Candidate),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            _ => Err(crate::NakoError::Database {
                message: format!("unknown provider mapping status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderMapping {
    pub id: ProviderMappingId,
    pub item_id: MediaItemId,
    pub subject_id: ProviderSubjectId,
    pub status: ProviderMappingStatus,
    pub confidence_milli: Option<u16>,
    pub source: MetadataSource,
}
