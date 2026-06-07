use nako_core::{NakoError, Result};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

pub(super) fn canonical_retry_next_attempt(
    next_attempt_at: &Option<String>,
    invalid_timestamp_message: &'static str,
    canonicalization_message: &'static str,
) -> Result<Option<String>> {
    let Some(next_attempt_at) = next_attempt_at else {
        return Ok(None);
    };

    let parsed = OffsetDateTime::parse(next_attempt_at, &Rfc3339).map_err(|_err| {
        NakoError::InvalidInput {
            message: invalid_timestamp_message.to_owned(),
        }
    })?;
    let canonical = parsed
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .map_err(|_err| NakoError::InvalidInput {
            message: canonicalization_message.to_owned(),
        })?;

    Ok(Some(canonical))
}
