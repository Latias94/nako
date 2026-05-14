use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
