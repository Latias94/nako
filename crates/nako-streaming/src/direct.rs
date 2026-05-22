use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RequestedByteRange {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedByteRange {
    pub start: u64,
    pub end: u64,
}

impl ResolvedByteRange {
    #[must_use]
    pub const fn len(self) -> u64 {
        self.end - self.start + 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectPlayRangeRequest {
    None,
    Range(RequestedByteRange),
    Invalid,
}

impl From<Option<RequestedByteRange>> for DirectPlayRangeRequest {
    fn from(value: Option<RequestedByteRange>) -> Self {
        match value {
            Some(range) => Self::Range(range),
            None => Self::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectPlayResponseStatus {
    Ok,
    PartialContent,
    RangeNotSatisfiable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectPlayResponsePlan {
    pub status: DirectPlayResponseStatus,
    pub content_type: String,
    pub total_len: u64,
    pub body_len: u64,
    pub range: Option<ResolvedByteRange>,
    pub content_range: Option<String>,
    pub seek_offset: u64,
}

impl DirectPlayResponsePlan {
    #[must_use]
    pub const fn is_range_not_satisfiable(&self) -> bool {
        matches!(self.status, DirectPlayResponseStatus::RangeNotSatisfiable)
    }
}

#[must_use]
pub fn plan_direct_play_response(
    total_len: u64,
    content_type: impl Into<String>,
    range_request: DirectPlayRangeRequest,
) -> DirectPlayResponsePlan {
    let content_type = content_type.into();

    match range_request {
        DirectPlayRangeRequest::None => DirectPlayResponsePlan {
            status: DirectPlayResponseStatus::Ok,
            content_type,
            total_len,
            body_len: total_len,
            range: None,
            content_range: None,
            seek_offset: 0,
        },
        DirectPlayRangeRequest::Invalid => range_not_satisfiable_response(total_len, content_type),
        DirectPlayRangeRequest::Range(requested) => {
            match resolve_byte_range(Some(requested), total_len) {
                Ok(Some(range)) => DirectPlayResponsePlan {
                    status: DirectPlayResponseStatus::PartialContent,
                    content_type,
                    total_len,
                    body_len: range.len(),
                    range: Some(range),
                    content_range: Some(format!(
                        "bytes {}-{}/{}",
                        range.start, range.end, total_len
                    )),
                    seek_offset: range.start,
                },
                Ok(None) => DirectPlayResponsePlan {
                    status: DirectPlayResponseStatus::Ok,
                    content_type,
                    total_len,
                    body_len: total_len,
                    range: None,
                    content_range: None,
                    seek_offset: 0,
                },
                Err(_) => range_not_satisfiable_response(total_len, content_type),
            }
        }
    }
}

pub fn parse_http_range_header(value: &str) -> Result<RequestedByteRange> {
    let Some(spec) = value.trim().strip_prefix("bytes=") else {
        return Err(NakoError::InvalidInput {
            message: "range header must use bytes unit".to_owned(),
        });
    };

    if spec.contains(',') {
        return Err(NakoError::InvalidInput {
            message: "multiple byte ranges are not supported".to_owned(),
        });
    }

    let Some((start, end)) = spec.split_once('-') else {
        return Err(NakoError::InvalidInput {
            message: "range header must include '-'".to_owned(),
        });
    };

    let start = parse_optional_u64(start.trim())?;
    let end = parse_optional_u64(end.trim())?;

    if start.is_none() && end.is_none() {
        return Err(NakoError::InvalidInput {
            message: "range header must include a start or suffix length".to_owned(),
        });
    }

    Ok(RequestedByteRange { start, end })
}

pub fn resolve_byte_range(
    requested: Option<RequestedByteRange>,
    total_len: u64,
) -> Result<Option<ResolvedByteRange>> {
    let Some(requested) = requested else {
        return Ok(None);
    };

    if total_len == 0 {
        return Err(NakoError::InvalidInput {
            message: "cannot satisfy range request for an empty source".to_owned(),
        });
    }

    let (start, end) = match (requested.start, requested.end) {
        (Some(start), Some(end)) => (start, end),
        (Some(start), None) => (start, total_len - 1),
        (None, Some(suffix_len)) => {
            if suffix_len == 0 {
                return Err(NakoError::InvalidInput {
                    message: "suffix byte range length must be greater than zero".to_owned(),
                });
            }

            let start = total_len.saturating_sub(suffix_len);
            (start, total_len - 1)
        }
        (None, None) => unreachable!("empty range is rejected by parser"),
    };

    if start > end || start >= total_len {
        return Err(NakoError::InvalidInput {
            message: format!("byte range {start}-{end} cannot be satisfied for length {total_len}"),
        });
    }

    Ok(Some(ResolvedByteRange {
        start,
        end: end.min(total_len - 1),
    }))
}

#[must_use]
pub fn content_type_for_file_name(file_name: &str) -> &'static str {
    match extension(file_name).as_deref().unwrap_or_default() {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "ts" | "m2ts" | "mts" => "video/mp2t",
        _ => "application/octet-stream",
    }
}

pub(crate) fn extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit_once('.')
        .map(|(_stem, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .map(str::to_ascii_lowercase)
}

fn parse_optional_u64(value: &str) -> Result<Option<u64>> {
    if value.is_empty() {
        return Ok(None);
    }

    value
        .parse::<u64>()
        .map(Some)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid byte range integer: {err}"),
        })
}

fn range_not_satisfiable_response(total_len: u64, content_type: String) -> DirectPlayResponsePlan {
    DirectPlayResponsePlan {
        status: DirectPlayResponseStatus::RangeNotSatisfiable,
        content_type,
        total_len,
        body_len: 0,
        range: None,
        content_range: Some(format!("bytes */{total_len}")),
        seek_offset: 0,
    }
}
