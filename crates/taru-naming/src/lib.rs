use serde::{Deserialize, Serialize};
use taru_core::MediaKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParsedName {
    pub kind_hint: MediaKind,
    pub title: String,
    pub year: Option<u16>,
    pub season_number: Option<u16>,
    pub episode_number: Option<u16>,
}

pub trait NameParser: Send + Sync {
    fn parse_path(&self, path: &str) -> ParsedName;
}
