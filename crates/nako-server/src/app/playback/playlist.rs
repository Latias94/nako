use std::path::Path;

use nako_core::{NakoError, Result, TranscodeSessionId};

pub(super) fn rewrite_hls_playlist(body: &str, session_id: TranscodeSessionId) -> String {
    let mut rewritten = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                line.to_owned()
            } else {
                format!("/playback/sessions/{session_id}/hls/segments/{trimmed}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if body.ends_with('\n') {
        rewritten.push('\n');
    }

    rewritten
}

pub(super) fn validate_hls_segment_name(segment_name: &str) -> Result<()> {
    if segment_name.is_empty()
        || segment_name.contains('/')
        || segment_name.contains('\\')
        || segment_name.contains("..")
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    let path = Path::new(segment_name);
    if !path
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(NakoError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    Ok(())
}
