use std::convert::TryFrom;

use taru_core::{
    CanonicalMetadata, ContentRating, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind,
};

use crate::providers::{DoubanPerson, DoubanSubject, push_provider_image_uri};

pub(crate) fn douban_subject_to_metadata(
    subject: DoubanSubject,
    image_base_url: Option<&str>,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url.unwrap_or_default(),
                ExternalProvider::Douban,
                None,
                None,
                None,
            );
        }
    }

    let mut credits = Vec::new();
    for person in subject.directors {
        push_douban_credit(&mut credits, person, CreditRole::Director);
    }
    for person in subject.writers {
        push_douban_credit(&mut credits, person, CreditRole::Writer);
    }
    for (order, person) in subject.casts.into_iter().enumerate() {
        let mut credit = douban_person_credit(person, CreditRole::Actor);
        credit.order = u32::try_from(order).ok();
        credits.push(credit);
    }

    let release_date = subject
        .year
        .as_ref()
        .filter(|year| year.len() == 4 && year.chars().all(|character| character.is_ascii_digit()))
        .map(|year| format!("{year}-01-01"));

    CanonicalMetadata {
        title: subject.title,
        original_title: subject.original_title.or(subject.alt_title),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date,
        genres: subject
            .genres
            .into_iter()
            .filter(|genre| !genre.trim().is_empty())
            .collect(),
        tags: subject
            .countries
            .into_iter()
            .filter(|country| !country.trim().is_empty())
            .collect(),
        ratings: subject
            .rating
            .and_then(|rating| rating.average)
            .map(|score| ContentRating {
                source: "Douban:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        credits,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Douban,
            value: subject.id,
        }],
        ..CanonicalMetadata::default()
    }
}

fn push_douban_credit(credits: &mut Vec<Credit>, person: DoubanPerson, role: CreditRole) {
    if person.name.trim().is_empty() {
        return;
    }
    credits.push(douban_person_credit(person, role));
}

fn douban_person_credit(person: DoubanPerson, role: CreditRole) -> Credit {
    Credit {
        name: person.name,
        role,
        character: None,
        order: None,
        external_ids: person
            .id
            .filter(|id| !id.trim().is_empty())
            .map(|id| ExternalId {
                provider: ExternalProvider::Douban,
                value: id,
            })
            .into_iter()
            .collect(),
    }
}
