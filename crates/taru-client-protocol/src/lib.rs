use serde::{Deserialize, Serialize};

pub const CLIENT_PROTOCOL_VERSION: &str = "v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageInfo {
    pub limit: u32,
    pub offset: u64,
    pub returned: u32,
}

impl PageInfo {
    #[must_use]
    pub const fn new(limit: u32, offset: u64, returned: u32) -> Self {
        Self {
            limit,
            offset,
            returned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_system_envelopes_serialize_without_server_types() {
        let health = HealthResponse {
            status: "ok".to_owned(),
            version: CLIENT_PROTOCOL_VERSION.to_owned(),
        };
        let error = ErrorResponse {
            code: "not_found".to_owned(),
            message: "missing".to_owned(),
        };
        let page = PageInfo::new(50, 100, 3);

        let health_json = serde_json::to_value(&health).unwrap();
        let error_json = serde_json::to_value(&error).unwrap();
        let page_json = serde_json::to_value(page).unwrap();

        assert_eq!(health_json["version"], "v1");
        assert_eq!(error_json["code"], "not_found");
        assert_eq!(page_json["limit"], 50);
        assert_eq!(page_json["offset"], 100);
        assert_eq!(page_json["returned"], 3);
    }
}
