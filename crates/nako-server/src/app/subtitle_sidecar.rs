use nako_core::{
    MediaSource, MediaStreamDisposition, MediaStreamInfo, MediaStreamKind, MediaStreamOrigin,
    NakoError, Result,
};
use nako_vfs::StorageUri;

pub(crate) const SUBTITLE_SIDECAR_MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SubtitleSidecarRole {
    Default,
    Forced,
    Sdh,
    Commentary,
}

impl SubtitleSidecarRole {
    fn segment(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Forced => Some("forced"),
            Self::Sdh => Some("sdh"),
            Self::Commentary => Some("commentary"),
        }
    }
}

pub(crate) fn subtitle_sidecar_file_name(
    media_file_name: &str,
    language: &str,
    role: SubtitleSidecarRole,
    extension: &str,
) -> Result<String> {
    let stem = media_file_stem(media_file_name);
    let language = safe_language_segment(language)?;
    let extension = safe_extension_segment(extension)?;
    let mut parts = vec![stem, language];
    if let Some(role_segment) = role.segment() {
        parts.push(role_segment.to_owned());
    }

    Ok(format!("{}.{}", parts.join("."), extension))
}

pub(crate) fn subtitle_sidecar_file_name_for_stream(
    source: &MediaSource,
    stream: &MediaStreamInfo,
) -> Result<String> {
    ensure_sidecar_subtitle_stream(stream)?;
    let language = stream
        .language
        .as_deref()
        .ok_or_else(|| NakoError::InvalidInput {
            message: "sidecar subtitle stream is missing language".to_owned(),
        })?;
    let extension = stream
        .codec
        .as_deref()
        .ok_or_else(|| NakoError::InvalidInput {
            message: "sidecar subtitle stream is missing format".to_owned(),
        })?;
    let role = role_from_disposition(&stream.technical.disposition);

    subtitle_sidecar_file_name(&source.file_name, language, role, extension)
}

pub(crate) fn subtitle_sidecar_uri_for_source(
    source_uri: &StorageUri,
    file_name: &str,
) -> Result<StorageUri> {
    let file_name = safe_leaf_name(file_name)?;
    let source_path = source_uri
        .path_part()
        .trim_start_matches(['/', '\\'])
        .replace('\\', "/");
    if source_path.is_empty() || source_path.ends_with('/') {
        return Err(NakoError::InvalidInput {
            message: "media source locator does not point at a file".to_owned(),
        });
    }

    let sidecar_path = source_path
        .rsplit_once('/')
        .map(|(dir, _leaf)| {
            if dir.is_empty() {
                file_name.clone()
            } else {
                format!("{dir}/{file_name}")
            }
        })
        .unwrap_or(file_name);

    StorageUri::from_parts(source_uri.scheme(), &sidecar_path)
}

pub(crate) fn subtitle_content_type_for_extension(extension: &str) -> Result<&'static str> {
    match safe_extension_segment(extension)?.as_str() {
        "vtt" => Ok("text/vtt; charset=utf-8"),
        "srt" => Ok("application/x-subrip; charset=utf-8"),
        _ => Err(NakoError::Unsupported(
            "sidecar subtitle format is not supported for playback serving",
        )),
    }
}

fn ensure_sidecar_subtitle_stream(stream: &MediaStreamInfo) -> Result<()> {
    if stream.kind != MediaStreamKind::Subtitle {
        return Err(NakoError::InvalidInput {
            message: "requested stream is not a subtitle stream".to_owned(),
        });
    }
    if stream.technical.origin.as_ref() != Some(&MediaStreamOrigin::Sidecar) {
        return Err(NakoError::Unsupported(
            "only sidecar subtitle streams can be served directly",
        ));
    }

    Ok(())
}

fn role_from_disposition(disposition: &MediaStreamDisposition) -> SubtitleSidecarRole {
    if disposition.forced {
        SubtitleSidecarRole::Forced
    } else if disposition.hearing_impaired {
        SubtitleSidecarRole::Sdh
    } else if disposition.commentary {
        SubtitleSidecarRole::Commentary
    } else {
        SubtitleSidecarRole::Default
    }
}

fn safe_leaf_name(file_name: &str) -> Result<String> {
    let file_name = file_name.trim();
    if file_name.is_empty() || file_name.contains(['/', '\\']) {
        return Err(NakoError::InvalidInput {
            message: "subtitle sidecar file name must be a safe leaf name".to_owned(),
        });
    }

    Ok(file_name.to_owned())
}

fn media_file_stem(file_name: &str) -> String {
    let file_name = safe_media_file_name(file_name);
    let stem = file_name
        .rsplit_once('.')
        .and_then(|(stem, _)| optional_non_empty(stem))
        .unwrap_or(file_name);

    optional_non_empty(&stem).unwrap_or_else(|| "media".to_owned())
}

pub(crate) fn safe_media_file_name(file_name: &str) -> String {
    let leaf = file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_name)
        .trim();
    optional_non_empty(leaf).unwrap_or_else(|| "media".to_owned())
}

fn safe_language_segment(language: &str) -> Result<String> {
    let Some(language) = optional_non_empty(language) else {
        return Err(NakoError::InvalidInput {
            message: "subtitle language cannot be empty".to_owned(),
        });
    };
    let language = language.to_ascii_lowercase();
    let valid = language.len() <= 35
        && !language.starts_with('-')
        && !language.ends_with('-')
        && language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-');
    if !valid {
        return Err(NakoError::InvalidInput {
            message: "subtitle language must be a safe BCP-47-like tag".to_owned(),
        });
    }

    Ok(language)
}

fn safe_extension_segment(extension: &str) -> Result<String> {
    let Some(extension) = optional_non_empty(extension) else {
        return Err(NakoError::InvalidInput {
            message: "subtitle format cannot be empty".to_owned(),
        });
    };
    let extension = extension.to_ascii_lowercase();
    let extension = match extension.as_str() {
        "webvtt" => "vtt".to_owned(),
        value => value.to_owned(),
    };
    let valid = matches!(extension.as_str(), "srt" | "vtt");
    if !valid {
        return Err(NakoError::Unsupported(
            "sidecar subtitle format is not supported for playback serving",
        ));
    }

    Ok(extension)
}

fn optional_non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_core::{MediaSourceId, MediaStreamTechnicalFacts};

    #[test]
    fn sidecar_file_name_uses_safe_leaf_stem_language_role_and_extension() {
        let name = subtitle_sidecar_file_name(
            "../Movie.Name.mkv",
            "zh-Hant",
            SubtitleSidecarRole::Forced,
            "webvtt",
        )
        .unwrap();

        assert_eq!(name, "Movie.Name.zh-hant.forced.vtt");
    }

    #[test]
    fn sidecar_file_name_for_stream_reconstructs_imported_default_sidecar() {
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: nako_core::LibraryId::new(),
            item_id: nako_core::MediaItemId::new(),
            locator: "local:///Movies/Demo.mkv".to_owned(),
            file_name: "Demo.mkv".to_owned(),
            size_bytes: None,
            fingerprint: None,
        };
        let stream = MediaStreamInfo {
            index: 2,
            kind: MediaStreamKind::Subtitle,
            codec: Some("srt".to_owned()),
            language: Some("en".to_owned()),
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
            technical: MediaStreamTechnicalFacts {
                origin: Some(MediaStreamOrigin::Sidecar),
                ..MediaStreamTechnicalFacts::default()
            },
        };

        assert_eq!(
            subtitle_sidecar_file_name_for_stream(&source, &stream).unwrap(),
            "Demo.en.srt"
        );
    }

    #[test]
    fn sidecar_uri_stays_next_to_source_without_accepting_paths_as_leaf_names() {
        let source = StorageUri::parse("local:///Movies/Demo.mkv").unwrap();

        assert_eq!(
            subtitle_sidecar_uri_for_source(&source, "Demo.en.srt")
                .unwrap()
                .to_string(),
            "local:///Movies/Demo.en.srt"
        );
        assert!(subtitle_sidecar_uri_for_source(&source, "../Demo.en.srt").is_err());
    }
}
