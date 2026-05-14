use serde::{Deserialize, Serialize};
use taru_core::{CanonicalMetadata, ExternalId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoDocument {
    pub metadata: CanonicalMetadata,
    pub external_ids: Vec<ExternalId>,
}

pub trait NfoCodec: Send + Sync {
    fn parse(&self, xml: &str) -> taru_core::Result<NfoDocument>;

    fn render(&self, document: &NfoDocument) -> taru_core::Result<String>;
}
