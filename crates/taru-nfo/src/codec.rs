use serde::{Deserialize, Serialize};
use taru_core::{
    CanonicalMetadata, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind, ImageRef,
    MediaKind, Result, TaruError,
};

type XmlNode<'a, 'input> = roxmltree::Node<'a, 'input>;

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
        for tag in &metadata.tags {
            push_tag(&mut output, "tag", Some(tag));
        }
        for credit in &metadata.credits {
            match &credit.role {
                CreditRole::Actor => {
                    output.push_str("  <actor>\n");
                    push_tag(&mut output, "name", Some(&credit.name));
                    push_tag(&mut output, "role", credit.character.as_deref());
                    if let Some(order) = credit.order {
                        push_tag(&mut output, "order", Some(&order.to_string()));
                    }
                    output.push_str("  </actor>\n");
                }
                CreditRole::Director => push_tag(&mut output, "director", Some(&credit.name)),
                CreditRole::Writer => push_tag(&mut output, "writer", Some(&credit.name)),
                _ => {}
            }
        }
        for image in &metadata.images {
            match &image.kind {
                ImageKind::Poster => push_tag(&mut output, "poster", Some(&image.uri)),
                ImageKind::Backdrop => push_tag(&mut output, "fanart", Some(&image.uri)),
                ImageKind::Thumbnail => push_tag(&mut output, "thumb", Some(&image.uri)),
                _ => {}
            }
        }

        output.push_str("</movie>\n");
        Ok(output)
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
