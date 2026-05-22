use nako_core::{
    AddonSideEffectRecord, AddonSideEffectTargetKind, MediaItem, MediaRepository, NakoError, Result,
};
use nako_db::NakoDatabase;

pub(super) async fn resolve_side_effect_media_item(
    store: &NakoDatabase,
    side_effect: &AddonSideEffectRecord,
) -> Result<MediaItem> {
    match side_effect.target.kind {
        AddonSideEffectTargetKind::MediaItem => {
            let item_id = side_effect
                .target
                .id
                .parse()
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid addon side effect media item target id: {err}"),
                })?;
            store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })
        }
        AddonSideEffectTargetKind::MediaSource => {
            let source_id =
                side_effect
                    .target
                    .id
                    .parse()
                    .map_err(|err| NakoError::InvalidInput {
                        message: format!("invalid addon side effect media source target id: {err}"),
                    })?;
            let source =
                store
                    .get_media_source(source_id)
                    .await?
                    .ok_or_else(|| NakoError::NotFound {
                        entity: "media_source",
                        id: source_id.to_string(),
                    })?;
            store
                .get_media_item(source.item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: source.item_id.to_string(),
                })
        }
    }
}
