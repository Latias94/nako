use serde::{Deserialize, Serialize};
use taru_core::{LocalInferenceEvidenceSource, MediaKind};

pub const DEFAULT_PARSER_VERSION: &str = "taru-naming:default:v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParsedName {
    pub kind_hint: MediaKind,
    pub title: String,
    pub year: Option<u16>,
    pub season_number: Option<u16>,
    pub episode_number: Option<u16>,
    pub confidence_milli: u16,
    pub evidence_source: LocalInferenceEvidenceSource,
    pub evidence_value: String,
    pub parser_version: String,
}

pub trait NameParser: Send + Sync {
    fn parse_path(&self, path: &str) -> ParsedName;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultNameParser;

impl NameParser for DefaultNameParser {
    fn parse_path(&self, path: &str) -> ParsedName {
        parse_path(path)
    }
}

pub fn parse_path(path: &str) -> ParsedName {
    let file_name = file_name(path).to_owned();
    let file_stem = file_stem(&file_name);
    let cleaned = clean_separators(file_stem);

    if let Some(parsed) = parse_episode_name(&cleaned, &file_name) {
        return parsed;
    }

    parse_movie_or_unknown_name(&cleaned, &file_name)
}

fn parse_episode_name(cleaned: &str, evidence_value: &str) -> Option<ParsedName> {
    let tokens = tokenize(cleaned);

    for (index, token) in tokens.iter().enumerate() {
        if let Some((season_number, episode_number)) = parse_sxe_token(token) {
            return Some(ParsedName {
                kind_hint: MediaKind::Episode,
                title: title_before_token(cleaned, token, index),
                year: year_from_tokens(&tokens[..index]),
                season_number: Some(season_number),
                episode_number: Some(episode_number),
                confidence_milli: 900,
                evidence_source: LocalInferenceEvidenceSource::FileName,
                evidence_value: evidence_value.to_owned(),
                parser_version: DEFAULT_PARSER_VERSION.to_owned(),
            });
        }

        if let Some((season_number, episode_number)) = parse_1x02_token(token) {
            return Some(ParsedName {
                kind_hint: MediaKind::Episode,
                title: title_before_token(cleaned, token, index),
                year: year_from_tokens(&tokens[..index]),
                season_number: Some(season_number),
                episode_number: Some(episode_number),
                confidence_milli: 880,
                evidence_source: LocalInferenceEvidenceSource::FileName,
                evidence_value: evidence_value.to_owned(),
                parser_version: DEFAULT_PARSER_VERSION.to_owned(),
            });
        }
    }

    None
}

fn parse_movie_or_unknown_name(cleaned: &str, evidence_value: &str) -> ParsedName {
    let tokens = tokenize(cleaned);
    let year = year_from_tokens(&tokens);
    let title = if let Some(year) = year {
        trim_title_before_year(cleaned, year)
    } else {
        cleaned.trim().to_owned()
    };
    let kind_hint = if year.is_some() {
        MediaKind::Movie
    } else {
        MediaKind::Unknown
    };
    let confidence_milli = if year.is_some() { 760 } else { 350 };

    ParsedName {
        kind_hint,
        title: normalize_title(&title),
        year,
        season_number: None,
        episode_number: None,
        confidence_milli,
        evidence_source: LocalInferenceEvidenceSource::FileName,
        evidence_value: evidence_value.to_owned(),
        parser_version: DEFAULT_PARSER_VERSION.to_owned(),
    }
}

fn file_name(path: &str) -> &str {
    let normalized = path.trim_matches('/');

    normalized
        .rsplit_once('/')
        .map(|(_parent, file_name)| file_name)
        .unwrap_or(normalized)
}

fn file_stem(file_name: &str) -> &str {
    file_name
        .rsplit_once('.')
        .map(|(stem, _extension)| stem)
        .unwrap_or(file_name)
}

fn clean_separators(value: &str) -> String {
    value
        .replace(['.', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize(value: &str) -> Vec<&str> {
    value.split_whitespace().collect()
}

fn parse_sxe_token(token: &str) -> Option<(u16, u16)> {
    let token = token.trim_matches(|value: char| matches!(value, '[' | ']' | '(' | ')'));
    let lower = token.to_ascii_lowercase();
    let (season, episode) = lower.split_once('e')?;
    let season = season.strip_prefix('s')?;
    Some((season.parse().ok()?, episode.parse().ok()?))
}

fn parse_1x02_token(token: &str) -> Option<(u16, u16)> {
    let token = token.trim_matches(|value: char| matches!(value, '[' | ']' | '(' | ')'));
    let (season, episode) = token.split_once(['x', 'X'])?;
    Some((season.parse().ok()?, episode.parse().ok()?))
}

fn year_from_tokens(tokens: &[&str]) -> Option<u16> {
    tokens.iter().find_map(|token| {
        let token = token.trim_matches(|value: char| matches!(value, '[' | ']' | '(' | ')'));

        if token.len() == 4 && token.chars().all(|value| value.is_ascii_digit()) {
            let year = token.parse::<u16>().ok()?;
            (1888..=2100).contains(&year).then_some(year)
        } else {
            None
        }
    })
}

fn trim_title_before_year(cleaned: &str, year: u16) -> String {
    let marker = year.to_string();
    let Some(index) = cleaned.find(&marker) else {
        return cleaned.trim().to_owned();
    };

    cleaned[..index]
        .trim()
        .trim_end_matches(['-', '(', '['])
        .trim()
        .to_owned()
}

fn title_before_token(cleaned: &str, token: &str, token_index: usize) -> String {
    if token_index == 0 {
        return "unknown".to_owned();
    }

    let Some(index) = cleaned.find(token) else {
        return normalize_title(cleaned);
    };

    let title = cleaned[..index]
        .trim()
        .trim_end_matches(['-', '(', '['])
        .trim();

    if title.is_empty() {
        "unknown".to_owned()
    } else {
        normalize_title(title)
    }
}

fn normalize_title(value: &str) -> String {
    value
        .trim()
        .trim_matches(['-', '(', ')', '[', ']'])
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_movie_title_and_year() {
        let parsed = parse_path("Movies/The Matrix (1999).mkv");

        assert_eq!(parsed.kind_hint, MediaKind::Movie);
        assert_eq!(parsed.title, "The Matrix");
        assert_eq!(parsed.year, Some(1999));
    }

    #[test]
    fn parses_episode_sxe_pattern() {
        let parsed = parse_path("Shows/Firefly/Season 01/Firefly.S01E02.The Train Job.mkv");

        assert_eq!(parsed.kind_hint, MediaKind::Episode);
        assert_eq!(parsed.title, "Firefly");
        assert_eq!(parsed.season_number, Some(1));
        assert_eq!(parsed.episode_number, Some(2));
        assert_eq!(parsed.confidence_milli, 900);
        assert_eq!(
            parsed.evidence_source,
            LocalInferenceEvidenceSource::FileName
        );
        assert_eq!(parsed.evidence_value, "Firefly.S01E02.The Train Job.mkv");
        assert_eq!(parsed.parser_version, DEFAULT_PARSER_VERSION);
    }

    #[test]
    fn parses_episode_x_pattern() {
        let parsed = parse_path("Shows/Example/Example - 2x03 - Name.mp4");

        assert_eq!(parsed.kind_hint, MediaKind::Episode);
        assert_eq!(parsed.title, "Example");
        assert_eq!(parsed.season_number, Some(2));
        assert_eq!(parsed.episode_number, Some(3));
    }

    #[test]
    fn normalizes_dot_and_underscore_separators() {
        let parsed = parse_path("Some.Movie_Title.2024.1080p.mkv");

        assert_eq!(parsed.title, "Some Movie Title");
        assert_eq!(parsed.year, Some(2024));
    }

    #[test]
    fn handles_local_uri_path_parts_with_leading_slash() {
        let parsed = parse_path("/Sample Movie (2024).mp4");

        assert_eq!(parsed.kind_hint, MediaKind::Movie);
        assert_eq!(parsed.title, "Sample Movie");
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.confidence_milli, 760);
    }

    #[test]
    fn weak_file_name_evidence_returns_unknown_item() {
        let parsed = parse_path("Uploads/random.clip.mkv");

        assert_eq!(parsed.kind_hint, MediaKind::Unknown);
        assert_eq!(parsed.title, "random clip");
        assert_eq!(parsed.year, None);
        assert_eq!(parsed.confidence_milli, 350);
        assert_eq!(
            parsed.evidence_source,
            LocalInferenceEvidenceSource::FileName
        );
        assert_eq!(parsed.evidence_value, "random.clip.mkv");
        assert_eq!(parsed.parser_version, DEFAULT_PARSER_VERSION);
    }
}
