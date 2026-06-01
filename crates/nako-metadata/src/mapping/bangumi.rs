use nako_core::{
    ContentRating, ExternalId, ExternalProvider, ImageKind, MetadataCandidateRecord, StudioRef,
};

use crate::providers::{
    BangumiEpisode, BangumiInfoBoxItem, BangumiSubject, first_non_empty, non_empty_string,
    push_provider_image_uri,
};

pub(crate) fn bangumi_subject_to_metadata(
    subject: BangumiSubject,
    image_base_url: &str,
) -> MetadataCandidateRecord {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.common.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
            subject_images.grid.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url,
                ExternalProvider::Bangumi,
                None,
                None,
                None,
            );
        }
    }

    let studios = bangumi_infobox_strings(&subject.infobox, &["动画制作", "制作", "製作"])
        .into_iter()
        .map(|name| StudioRef {
            name,
            external_ids: Vec::new(),
        })
        .collect();
    let tags = subject
        .tags
        .into_iter()
        .map(|tag| tag.name)
        .filter(|name| !name.trim().is_empty())
        .collect();

    MetadataCandidateRecord {
        title: first_non_empty(&[Some(subject.name_cn.as_str()), Some(subject.name.as_str())]),
        original_title: non_empty_string(subject.name),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date: subject.date.filter(|value| !value.trim().is_empty()),
        runtime_minutes: None,
        tags,
        ratings: subject
            .rating
            .and_then(|rating| rating.score)
            .map(|score| ContentRating {
                source: "Bangumi:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        studios,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: subject.id.to_string(),
        }],
        ..MetadataCandidateRecord::default()
    }
}

pub(crate) fn bangumi_episode_to_metadata(episode: &BangumiEpisode) -> MetadataCandidateRecord {
    MetadataCandidateRecord {
        title: first_non_empty(&[Some(episode.name_cn.as_str()), Some(episode.name.as_str())]),
        original_title: non_empty_string(episode.name.clone()),
        overview: episode
            .desc
            .clone()
            .filter(|value| !value.trim().is_empty()),
        release_date: episode
            .airdate
            .clone()
            .filter(|value| !value.trim().is_empty()),
        runtime_minutes: episode
            .duration_seconds
            .and_then(|seconds| (seconds > 0).then_some(seconds.div_ceil(60))),
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: episode.id.to_string(),
        }],
        ..MetadataCandidateRecord::default()
    }
}

fn bangumi_infobox_strings(items: &[BangumiInfoBoxItem], keys: &[&str]) -> Vec<String> {
    items
        .iter()
        .filter(|item| keys.iter().any(|key| item.key == *key))
        .flat_map(|item| metadata_strings_from_json(&item.value))
        .collect()
}

fn metadata_strings_from_json(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(value) => non_empty_string(value.clone()).into_iter().collect(),
        serde_json::Value::Array(values) => values
            .iter()
            .flat_map(metadata_strings_from_json)
            .collect::<Vec<_>>(),
        serde_json::Value::Object(map) => map
            .get("v")
            .or_else(|| map.get("value"))
            .into_iter()
            .flat_map(metadata_strings_from_json)
            .collect(),
        _ => Vec::new(),
    }
}
