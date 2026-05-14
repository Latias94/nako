use serde::{Deserialize, Serialize};
use taru_core::{CanonicalMetadata, ExternalId, Result, TaruError};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoDocument {
    pub metadata: CanonicalMetadata,
    pub external_ids: Vec<ExternalId>,
}

pub trait NfoCodec: Send + Sync {
    fn parse(&self, xml: &str) -> Result<NfoDocument>;

    fn render(&self, document: &NfoDocument) -> Result<String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MovieNfoCodec;

impl NfoCodec for MovieNfoCodec {
    fn parse(&self, xml: &str) -> Result<NfoDocument> {
        let metadata = CanonicalMetadata {
            title: required_tag(xml, "title")?,
            original_title: optional_tag(xml, "originaltitle"),
            sort_title: optional_tag(xml, "sorttitle"),
            overview: optional_tag(xml, "plot"),
            release_date: optional_tag(xml, "releasedate").or_else(|| optional_tag(xml, "year")),
            runtime_minutes: optional_tag(xml, "runtime").and_then(|value| value.parse().ok()),
            tagline: optional_tag(xml, "tagline"),
            genres: tags(xml, "genre"),
            ..CanonicalMetadata::default()
        };

        Ok(NfoDocument {
            metadata,
            external_ids: Vec::new(),
        })
    }

    fn render(&self, document: &NfoDocument) -> Result<String> {
        let metadata = &document.metadata;
        let mut output = String::from("<movie>\n");

        push_tag(&mut output, "title", Some(&metadata.title));
        push_tag(
            &mut output,
            "originaltitle",
            metadata.original_title.as_deref(),
        );
        push_tag(&mut output, "sorttitle", metadata.sort_title.as_deref());
        push_tag(&mut output, "plot", metadata.overview.as_deref());
        push_tag(&mut output, "releasedate", metadata.release_date.as_deref());
        if let Some(runtime) = metadata.runtime_minutes {
            push_tag(&mut output, "runtime", Some(&runtime.to_string()));
        }
        push_tag(&mut output, "tagline", metadata.tagline.as_deref());
        for genre in &metadata.genres {
            push_tag(&mut output, "genre", Some(genre));
        }

        output.push_str("</movie>\n");
        Ok(output)
    }
}

fn required_tag(xml: &str, name: &str) -> Result<String> {
    optional_tag(xml, name).ok_or_else(|| TaruError::InvalidInput {
        message: format!("NFO is missing required <{name}> tag"),
    })
}

fn optional_tag(xml: &str, name: &str) -> Option<String> {
    tags(xml, name).into_iter().next()
}

fn tags(xml: &str, name: &str) -> Vec<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let mut values = Vec::new();
    let mut remaining = xml;

    while let Some((_before, after_open)) = remaining.split_once(&open) {
        let Some((value, after_close)) = after_open.split_once(&close) else {
            break;
        };
        values.push(unescape_xml(value.trim()));
        remaining = after_close;
    }

    values
}

fn push_tag(output: &mut String, name: &str, value: Option<&str>) {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return;
    };

    output.push_str("  <");
    output.push_str(name);
    output.push('>');
    output.push_str(&escape_xml(value));
    output.push_str("</");
    output.push_str(name);
    output.push_str(">\n");
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movie_nfo_round_trips_core_fields() {
        let document = NfoDocument {
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                original_title: Some("The Matrix".to_owned()),
                sort_title: Some("Matrix, The".to_owned()),
                overview: Some("A hacker discovers reality.".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                runtime_minutes: Some(136),
                tagline: Some("Welcome to the Real World".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                ..CanonicalMetadata::default()
            },
            external_ids: Vec::new(),
        };
        let codec = MovieNfoCodec;

        let xml = codec.render(&document).unwrap();
        let parsed = codec.parse(&xml).unwrap();

        assert_eq!(parsed.metadata.title, "The Matrix");
        assert_eq!(parsed.metadata.sort_title, Some("Matrix, The".to_owned()));
        assert_eq!(parsed.metadata.runtime_minutes, Some(136));
        assert_eq!(
            parsed.metadata.genres,
            vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
    }
}
