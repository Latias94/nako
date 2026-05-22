use nako_addon_protocol::{AddonArtworkKind, AddonArtworkSourceKind, AddonArtworkWritePayload};
use nako_core::{
    AddonSideEffectRecord, AddonSideEffectTargetKind, ArtworkCandidateId,
    ArtworkCandidateRepository, ArtworkCandidateSourceKind, ImageKind, NakoError,
    NewArtworkCandidate, Result,
};
use nako_db::NakoDatabase;

use super::{
    side_effect_apply::AddonSideEffectApplyCommand, target::resolve_side_effect_media_item,
};

#[derive(Clone, Debug)]
pub(super) struct AddonArtworkWriteAdapter {
    store: NakoDatabase,
}

impl AddonArtworkWriteAdapter {
    pub(super) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(super) async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectApplyCommand> {
        let payload = parse_addon_artwork_write_payload(&side_effect.payload_json)?;
        if side_effect.target.kind != AddonSideEffectTargetKind::MediaItem {
            return Err(NakoError::InvalidInput {
                message: "addon artwork_write candidate proposal requires a media_item target"
                    .to_owned(),
            });
        }
        let item = resolve_side_effect_media_item(&self.store, side_effect).await?;
        let source_uri = normalize_remote_artwork_url(&payload.source.url)?;
        let language = normalize_artwork_language(payload.language.as_deref())?;
        let kind = payload.kind.into_image_kind();

        let existing = self
            .store
            .find_artwork_candidate_by_source(
                side_effect.addon_id,
                side_effect.library_id,
                item.id,
                &kind,
                ArtworkCandidateSourceKind::RemoteUrl,
                &source_uri,
            )
            .await?;
        let (candidate_id, created) = if let Some(existing) = existing {
            (existing.id, false)
        } else {
            let candidate = self
                .store
                .create_artwork_candidate(NewArtworkCandidate {
                    id: ArtworkCandidateId::new(),
                    addon_id: side_effect.addon_id,
                    side_effect_id: side_effect.id,
                    library_id: side_effect.library_id,
                    item_id: item.id,
                    kind: kind.clone(),
                    source_kind: ArtworkCandidateSourceKind::RemoteUrl,
                    source_uri,
                    width: payload.width,
                    height: payload.height,
                    language,
                })
                .await?;
            (candidate.id, true)
        };

        Ok(AddonSideEffectApplyCommand::applied(
            side_effect.id,
            item.id,
            "artwork_candidate",
            Some(artwork_candidate_apply_report(
                candidate_id,
                &kind,
                created,
            )?),
        ))
    }
}

trait AddonArtworkKindExt {
    fn into_image_kind(self) -> ImageKind;
}

impl AddonArtworkKindExt for AddonArtworkKind {
    fn into_image_kind(self) -> ImageKind {
        match self {
            Self::Poster => ImageKind::Poster,
            Self::Backdrop => ImageKind::Backdrop,
            Self::Logo => ImageKind::Logo,
            Self::Banner => ImageKind::Banner,
            Self::Thumbnail => ImageKind::Thumbnail,
        }
    }
}

fn image_kind_report_value(kind: &ImageKind) -> &'static str {
    match kind {
        ImageKind::Poster => "poster",
        ImageKind::Backdrop => "backdrop",
        ImageKind::Logo => "logo",
        ImageKind::Banner => "banner",
        ImageKind::Thumbnail => "thumbnail",
        ImageKind::Other(_) => "other",
    }
}

fn parse_addon_artwork_write_payload(payload_json: &str) -> Result<AddonArtworkWritePayload> {
    let payload = serde_json::from_str::<AddonArtworkWritePayload>(payload_json).map_err(|_| {
        NakoError::InvalidInput {
            message: "invalid addon artwork_write payload".to_owned(),
        }
    })?;
    match payload.source.kind {
        AddonArtworkSourceKind::RemoteUrl => {}
    }
    validate_artwork_dimension("width", payload.width)?;
    validate_artwork_dimension("height", payload.height)?;
    Ok(payload)
}

fn validate_artwork_dimension(field: &str, value: Option<u32>) -> Result<()> {
    if matches!(value, Some(0 | 20001..)) {
        return Err(NakoError::InvalidInput {
            message: format!("addon artwork_write {field} must be between 1 and 20000"),
        });
    }
    Ok(())
}

fn normalize_remote_artwork_url(value: &str) -> Result<String> {
    let value = value.trim();
    if value.len() > 2048 {
        return Err(NakoError::InvalidInput {
            message: "addon artwork_write remote URL must be at most 2048 bytes".to_owned(),
        });
    }
    let url = reqwest::Url::parse(value).map_err(|_| NakoError::InvalidInput {
        message: "invalid addon artwork_write remote URL".to_owned(),
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(NakoError::InvalidInput {
            message: "addon artwork_write remote URL must use http or https".to_owned(),
        });
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(NakoError::InvalidInput {
            message: "addon artwork_write remote URL must not contain credentials".to_owned(),
        });
    }
    Ok(url.to_string())
}

fn normalize_artwork_language(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 32 {
        return Err(NakoError::InvalidInput {
            message: "addon artwork_write language must be at most 32 bytes".to_owned(),
        });
    }
    Ok(Some(value.to_ascii_lowercase()))
}

fn artwork_candidate_apply_report(
    candidate_id: ArtworkCandidateId,
    kind: &ImageKind,
    created: bool,
) -> Result<String> {
    let report = serde_json::json!({
        "kind": "artwork_candidate",
        "candidate_id": candidate_id.to_string(),
        "image_kind": image_kind_report_value(kind),
        "status": "proposed",
        "candidate_created": u8::from(created),
        "candidate_existing": u8::from(!created),
    });

    serde_json::to_string(&report).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon artwork candidate report: {err}"),
    })
}
