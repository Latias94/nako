use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use taru_core::{
    CanonicalMetadata, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind, ImageRef,
    MediaKind, Result, TaruError,
};

type XmlNode<'a, 'input> = roxmltree::Node<'a, 'input>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoPreservedRender {
    pub xml: String,
    pub report: NfoPreservationReport,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoPreservationReport {
    pub preserved_unknown_fields: Vec<String>,
    pub updated_owned_fields: Vec<String>,
    pub conflicts: Vec<NfoFieldConflict>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoFieldConflict {
    pub field: String,
    pub existing_value: Option<String>,
    pub replacement_value: Option<String>,
    pub reason: NfoFieldConflictReason,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoFieldConflictReason {
    DuplicateOwnedField,
    OwnedFieldAlias,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoDocument {
    pub metadata: CanonicalMetadata,
    pub external_ids: Vec<ExternalId>,
    pub hierarchy: NfoHierarchy,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoHierarchy {
    pub kind: Option<MediaKind>,
    pub series_title: Option<String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
}

pub trait NfoCodec: Send + Sync {
    fn parse(&self, xml: &str) -> Result<NfoDocument>;

    fn render(&self, document: &NfoDocument) -> Result<String>;

    fn render_preserving(
        &self,
        _document: &NfoDocument,
        _existing_xml: &str,
    ) -> Result<NfoPreservedRender> {
        Err(TaruError::Unsupported(
            "NFO codec does not support preservation-aware rendering",
        ))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MovieNfoCodec;

impl NfoCodec for MovieNfoCodec {
    fn parse(&self, xml: &str) -> Result<NfoDocument> {
        let document = roxmltree::Document::parse(xml).map_err(|err| TaruError::InvalidInput {
            message: format!("invalid NFO XML: {err}"),
        })?;
        let root = document.root_element();
        let hierarchy = hierarchy_from_nfo(root);
        let metadata = CanonicalMetadata {
            title: required_child_text(root, "title")?,
            original_title: optional_child_text(root, "originaltitle"),
            sort_title: optional_child_text(root, "sorttitle"),
            overview: optional_child_text(root, "plot"),
            release_date: optional_child_text(root, "releasedate")
                .or_else(|| optional_child_text(root, "aired"))
                .or_else(|| optional_child_text(root, "premiered"))
                .or_else(|| optional_child_text(root, "year")),
            runtime_minutes: optional_child_text(root, "runtime")
                .and_then(|value| value.parse().ok()),
            tagline: optional_child_text(root, "tagline"),
            genres: child_texts(root, "genre"),
            tags: child_texts(root, "tag"),
            images: images_from_nfo(root),
            credits: credits_from_nfo(root),
            ..CanonicalMetadata::default()
        };

        Ok(NfoDocument {
            metadata,
            external_ids: Vec::new(),
            hierarchy,
        })
    }

    fn render(&self, document: &NfoDocument) -> Result<String> {
        Ok(render_movie_xml(document, &[]).xml)
    }

    fn render_preserving(
        &self,
        document: &NfoDocument,
        existing_xml: &str,
    ) -> Result<NfoPreservedRender> {
        let parsed =
            roxmltree::Document::parse(existing_xml).map_err(|err| TaruError::InvalidInput {
                message: format!("invalid NFO XML: {err}"),
            })?;
        let root = parsed.root_element();
        if root.tag_name().name() != "movie" {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "movie NFO preservation requires <movie> root, found <{}>",
                    root.tag_name().name()
                ),
            });
        }

        let preserved = collect_preserved_movie_children(existing_xml, root, document);
        let mut rendered = render_movie_xml(document, &preserved.fragments);
        rendered.report.conflicts = preserved.conflicts;
        rendered.report.preserved_unknown_fields = preserved.fields;
        Ok(rendered)
    }
}

struct PreservedMovieChildren {
    fragments: Vec<PreservedXmlFragment>,
    fields: Vec<String>,
    conflicts: Vec<NfoFieldConflict>,
}

struct PreservedXmlFragment {
    xml: String,
}

fn collect_preserved_movie_children(
    existing_xml: &str,
    root: XmlNode<'_, '_>,
    document: &NfoDocument,
) -> PreservedMovieChildren {
    let mut fragments = Vec::new();
    let mut fields = Vec::new();
    let mut conflicts = Vec::new();
    let mut seen_single_fields = HashSet::new();

    for child in root.children() {
        match child.node_type() {
            roxmltree::NodeType::Element => {
                let tag = child.tag_name().name();
                let Some(canonical_field) = canonical_owned_movie_field(tag) else {
                    push_preserved_child(existing_xml, child, tag, &mut fields, &mut fragments);
                    continue;
                };

                if is_release_date_alias(tag) {
                    conflicts.push(NfoFieldConflict {
                        field: canonical_field.to_owned(),
                        existing_value: node_text(child),
                        replacement_value: replacement_value_for_field(document, canonical_field),
                        reason: NfoFieldConflictReason::OwnedFieldAlias,
                    });
                    continue;
                }

                if !is_repeatable_owned_movie_field(canonical_field)
                    && !seen_single_fields.insert(canonical_field)
                {
                    conflicts.push(NfoFieldConflict {
                        field: canonical_field.to_owned(),
                        existing_value: node_text(child),
                        replacement_value: replacement_value_for_field(document, canonical_field),
                        reason: NfoFieldConflictReason::DuplicateOwnedField,
                    });
                }
            }
            roxmltree::NodeType::Comment => {
                push_preserved_child(existing_xml, child, "#comment", &mut fields, &mut fragments);
            }
            roxmltree::NodeType::PI => {
                push_preserved_child(
                    existing_xml,
                    child,
                    "#processing-instruction",
                    &mut fields,
                    &mut fragments,
                );
            }
            roxmltree::NodeType::Root | roxmltree::NodeType::Text => {}
        }
    }

    PreservedMovieChildren {
        fragments,
        fields,
        conflicts,
    }
}

fn push_preserved_child(
    existing_xml: &str,
    child: XmlNode<'_, '_>,
    field: &str,
    fields: &mut Vec<String>,
    fragments: &mut Vec<PreservedXmlFragment>,
) {
    let fragment = existing_xml[child.range()].trim();
    if fragment.is_empty() {
        return;
    }

    fields.push(field.to_owned());
    fragments.push(PreservedXmlFragment {
        xml: fragment.to_owned(),
    });
}

fn render_movie_xml(
    document: &NfoDocument,
    preserved_fragments: &[PreservedXmlFragment],
) -> NfoPreservedRender {
    let metadata = &document.metadata;
    let mut output = String::from("<movie>\n");
    let mut updated_owned_fields = Vec::new();

    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "title",
        Some(&metadata.title),
    );
    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "originaltitle",
        metadata.original_title.as_deref(),
    );
    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "sorttitle",
        metadata.sort_title.as_deref(),
    );
    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "plot",
        metadata.overview.as_deref(),
    );
    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "releasedate",
        metadata.release_date.as_deref(),
    );
    if let Some(runtime) = metadata.runtime_minutes {
        push_owned_tag(
            &mut output,
            &mut updated_owned_fields,
            "runtime",
            Some(&runtime.to_string()),
        );
    }
    push_owned_tag(
        &mut output,
        &mut updated_owned_fields,
        "tagline",
        metadata.tagline.as_deref(),
    );
    for genre in &metadata.genres {
        push_owned_tag(&mut output, &mut updated_owned_fields, "genre", Some(genre));
    }
    for tag in &metadata.tags {
        push_owned_tag(&mut output, &mut updated_owned_fields, "tag", Some(tag));
    }
    for credit in &metadata.credits {
        match &credit.role {
            CreditRole::Actor => {
                record_updated_field(&mut updated_owned_fields, "actor");
                output.push_str("  <actor>\n");
                push_tag(&mut output, "name", Some(&credit.name));
                push_tag(&mut output, "role", credit.character.as_deref());
                if let Some(order) = credit.order {
                    push_tag(&mut output, "order", Some(&order.to_string()));
                }
                output.push_str("  </actor>\n");
            }
            CreditRole::Director => push_owned_tag(
                &mut output,
                &mut updated_owned_fields,
                "director",
                Some(&credit.name),
            ),
            CreditRole::Writer => push_owned_tag(
                &mut output,
                &mut updated_owned_fields,
                "writer",
                Some(&credit.name),
            ),
            _ => {}
        }
    }
    for image in &metadata.images {
        match &image.kind {
            ImageKind::Poster => push_owned_tag(
                &mut output,
                &mut updated_owned_fields,
                "poster",
                Some(&image.uri),
            ),
            ImageKind::Backdrop => push_owned_tag(
                &mut output,
                &mut updated_owned_fields,
                "fanart",
                Some(&image.uri),
            ),
            ImageKind::Thumbnail => push_owned_tag(
                &mut output,
                &mut updated_owned_fields,
                "thumb",
                Some(&image.uri),
            ),
            _ => {}
        }
    }

    for fragment in preserved_fragments {
        push_preserved_fragment(&mut output, fragment);
    }

    output.push_str("</movie>\n");
    NfoPreservedRender {
        xml: output,
        report: NfoPreservationReport {
            preserved_unknown_fields: Vec::new(),
            updated_owned_fields,
            conflicts: Vec::new(),
        },
    }
}

fn push_preserved_fragment(output: &mut String, fragment: &PreservedXmlFragment) {
    if fragment.xml.trim().is_empty() {
        return;
    }

    for line in fragment.xml.lines() {
        output.push_str("  ");
        output.push_str(line.trim_end());
        output.push('\n');
    }
}

fn canonical_owned_movie_field(name: &str) -> Option<&'static str> {
    match name {
        "title" => Some("title"),
        "originaltitle" => Some("original_title"),
        "sorttitle" => Some("sort_title"),
        "plot" => Some("plot"),
        "releasedate" | "aired" | "premiered" | "year" => Some("release_date"),
        "runtime" => Some("runtime"),
        "tagline" => Some("tagline"),
        "genre" => Some("genre"),
        "tag" => Some("tag"),
        "actor" => Some("actor"),
        "director" => Some("director"),
        "writer" => Some("writer"),
        "poster" => Some("poster"),
        "fanart" => Some("fanart"),
        "thumb" => Some("thumb"),
        _ => None,
    }
}

fn is_repeatable_owned_movie_field(field: &str) -> bool {
    matches!(
        field,
        "genre" | "tag" | "actor" | "director" | "writer" | "poster" | "fanart" | "thumb"
    )
}

fn is_release_date_alias(name: &str) -> bool {
    matches!(name, "aired" | "premiered" | "year")
}

fn replacement_value_for_field(document: &NfoDocument, field: &str) -> Option<String> {
    let metadata = &document.metadata;
    match field {
        "title" => Some(metadata.title.clone()),
        "original_title" => metadata.original_title.clone(),
        "sort_title" => metadata.sort_title.clone(),
        "plot" => metadata.overview.clone(),
        "release_date" => metadata.release_date.clone(),
        "runtime" => metadata.runtime_minutes.map(|value| value.to_string()),
        "tagline" => metadata.tagline.clone(),
        _ => None,
    }
}

fn hierarchy_from_nfo(root: XmlNode<'_, '_>) -> NfoHierarchy {
    let kind = match root.tag_name().name() {
        "movie" => Some(MediaKind::Movie),
        "tvshow" => Some(MediaKind::Series),
        "episodedetails" => Some(MediaKind::Episode),
        _ => None,
    };

    NfoHierarchy {
        kind,
        series_title: optional_child_text(root, "showtitle")
            .or_else(|| optional_child_text(root, "series")),
        season_number: optional_child_text(root, "season").and_then(|value| value.parse().ok()),
        episode_number: optional_child_text(root, "episode").and_then(|value| value.parse().ok()),
    }
}

fn required_child_text(node: XmlNode<'_, '_>, name: &str) -> Result<String> {
    optional_child_text(node, name).ok_or_else(|| TaruError::InvalidInput {
        message: format!("NFO is missing required <{name}> tag"),
    })
}

fn optional_child_text(node: XmlNode<'_, '_>, name: &str) -> Option<String> {
    child_texts(node, name).into_iter().next()
}

fn child_texts(node: XmlNode<'_, '_>, name: &str) -> Vec<String> {
    node.children()
        .filter(|child| child.is_element() && child.tag_name().name() == name)
        .filter_map(node_text)
        .collect()
}

fn node_text(node: XmlNode<'_, '_>) -> Option<String> {
    let value = node.text()?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn credits_from_nfo(root: XmlNode<'_, '_>) -> Vec<Credit> {
    let mut credits = Vec::new();

    for actor in root
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "actor")
    {
        let Some(name) = optional_child_text(actor, "name") else {
            continue;
        };
        credits.push(Credit {
            name,
            role: CreditRole::Actor,
            character: optional_child_text(actor, "role"),
            order: optional_child_text(actor, "order").and_then(|value| value.parse().ok()),
            external_ids: Vec::new(),
        });
    }

    for director in child_texts(root, "director") {
        credits.push(Credit {
            name: director,
            role: CreditRole::Director,
            character: None,
            order: None,
            external_ids: Vec::new(),
        });
    }

    for writer in child_texts(root, "writer") {
        credits.push(Credit {
            name: writer,
            role: CreditRole::Writer,
            character: None,
            order: None,
            external_ids: Vec::new(),
        });
    }

    credits
}

fn images_from_nfo(root: XmlNode<'_, '_>) -> Vec<ImageRef> {
    let mut images = Vec::new();

    for uri in child_texts(root, "poster") {
        push_nfo_image(&mut images, ImageKind::Poster, uri);
    }
    for uri in child_texts(root, "thumb") {
        push_nfo_image(&mut images, ImageKind::Thumbnail, uri);
    }
    for fanart in root
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "fanart")
    {
        let thumbs = child_texts(fanart, "thumb");
        if thumbs.is_empty() {
            if let Some(uri) = node_text(fanart) {
                push_nfo_image(&mut images, ImageKind::Backdrop, uri);
            }
        } else {
            for uri in thumbs {
                push_nfo_image(&mut images, ImageKind::Backdrop, uri);
            }
        }
    }

    images
}

fn push_nfo_image(images: &mut Vec<ImageRef>, kind: ImageKind, uri: String) {
    let uri = uri.trim();

    if uri.is_empty()
        || images
            .iter()
            .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri: uri.to_owned(),
        provider: ExternalProvider::Local,
        width: None,
        height: None,
        language: None,
    });
}

fn push_owned_tag(
    output: &mut String,
    updated_owned_fields: &mut Vec<String>,
    name: &str,
    value: Option<&str>,
) {
    if push_tag(output, name, value) {
        record_updated_field(updated_owned_fields, name);
    }
}

fn record_updated_field(updated_owned_fields: &mut Vec<String>, name: &str) {
    if !updated_owned_fields.iter().any(|existing| existing == name) {
        updated_owned_fields.push(name.to_owned());
    }
}

fn push_tag(output: &mut String, name: &str, value: Option<&str>) -> bool {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return false;
    };

    output.push_str("  <");
    output.push_str(name);
    output.push('>');
    output.push_str(&escape_xml(value));
    output.push_str("</");
    output.push_str(name);
    output.push_str(">\n");
    true
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
