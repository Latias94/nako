use nako_core::{
    LocalInferenceEvidence, LocalInferenceEvidenceId, LocalInferenceEvidenceSource, MediaKind,
};
use nako_naming::{NameEvidenceSource, ParsedMediaKind, ParsedName};

use super::types::LocalInferenceRequest;

pub(super) fn local_inference_evidence_from_parsed(
    request: &LocalInferenceRequest<'_>,
    parsed_name: &ParsedName,
) -> LocalInferenceEvidence {
    LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id: request.source_id,
        inferred_kind: media_kind_from_parsed(parsed_name.kind_hint),
        inferred_title: Some(parsed_name.title.clone()),
        inferred_year: parsed_name.year.map(i32::from),
        inferred_season: parsed_name.season_number.map(u32::from),
        inferred_episode: parsed_name.episode_number.map(u32::from),
        confidence_milli: Some(parsed_name.confidence_milli),
        evidence_source: evidence_source_from_name(parsed_name.evidence_source.clone()),
        evidence_value: parsed_name.evidence_value.clone(),
        inference_version: parsed_name.parser_version.clone(),
    }
}

pub(super) fn media_kind_from_parsed(kind: ParsedMediaKind) -> MediaKind {
    match kind {
        ParsedMediaKind::Movie => MediaKind::Movie,
        ParsedMediaKind::Series => MediaKind::Series,
        ParsedMediaKind::Season => MediaKind::Season,
        ParsedMediaKind::Episode => MediaKind::Episode,
        ParsedMediaKind::Extra => MediaKind::Extra,
        ParsedMediaKind::Collection => MediaKind::Collection,
        ParsedMediaKind::Unknown => MediaKind::Unknown,
    }
}

fn evidence_source_from_name(source: NameEvidenceSource) -> LocalInferenceEvidenceSource {
    match source {
        NameEvidenceSource::Path => LocalInferenceEvidenceSource::Path,
        NameEvidenceSource::FileName => LocalInferenceEvidenceSource::FileName,
        NameEvidenceSource::Directory => LocalInferenceEvidenceSource::Directory,
        NameEvidenceSource::NearbyFile => LocalInferenceEvidenceSource::NearbyFile,
        NameEvidenceSource::MediaProbe => LocalInferenceEvidenceSource::MediaProbe,
        NameEvidenceSource::Other(value) => LocalInferenceEvidenceSource::Other(value),
    }
}
