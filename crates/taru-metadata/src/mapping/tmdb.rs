use taru_core::{
    CanonicalMetadata, CollectionRef, ContentRating, Credit, CreditRole, ExternalId,
    ExternalProvider, ImageKind, ImageRef, StudioRef,
};

use crate::providers::{
    TmdbCredits, TmdbImage, TmdbMovieDetails, TmdbMovieSearchResult, TmdbReleaseDates,
    push_provider_image_uri,
};

pub(crate) fn tmdb_search_result_to_metadata(
    result: TmdbMovieSearchResult,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    push_provider_image_uri(
        &mut images,
        ImageKind::Poster,
        result.poster_path.as_deref(),
        image_base_url,
        ExternalProvider::Tmdb,
        None,
        None,
        None,
    );
    push_provider_image_uri(
        &mut images,
        ImageKind::Backdrop,
        result.backdrop_path.as_deref(),
        image_base_url,
        ExternalProvider::Tmdb,
        None,
        None,
        None,
    );

    CanonicalMetadata {
        title: result.title,
        original_title: result.original_title,
        overview: result.overview,
        release_date: result.release_date,
        images,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: result.id.to_string(),
        }],
        ..CanonicalMetadata::default()
    }
}

pub(crate) fn tmdb_movie_details_to_metadata(
    details: TmdbMovieDetails,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut external_ids = vec![ExternalId {
        provider: ExternalProvider::Tmdb,
        value: details.id.to_string(),
    }];
    let imdb_id = details
        .external_ids
        .as_ref()
        .and_then(|ids| ids.imdb_id.as_ref())
        .or(details.imdb_id.as_ref())
        .filter(|value| !value.trim().is_empty());

    if let Some(imdb_id) = imdb_id {
        external_ids.push(ExternalId {
            provider: ExternalProvider::Imdb,
            value: imdb_id.clone(),
        });
    }

    let mut images = Vec::new();
    push_provider_image_uri(
        &mut images,
        ImageKind::Poster,
        details.poster_path.as_deref(),
        image_base_url,
        ExternalProvider::Tmdb,
        None,
        None,
        None,
    );
    push_provider_image_uri(
        &mut images,
        ImageKind::Backdrop,
        details.backdrop_path.as_deref(),
        image_base_url,
        ExternalProvider::Tmdb,
        None,
        None,
        None,
    );

    if let Some(collection) = details.belongs_to_collection.as_ref() {
        push_provider_image_uri(
            &mut images,
            ImageKind::Poster,
            collection.poster_path.as_deref(),
            image_base_url,
            ExternalProvider::Tmdb,
            None,
            None,
            None,
        );
        push_provider_image_uri(
            &mut images,
            ImageKind::Backdrop,
            collection.backdrop_path.as_deref(),
            image_base_url,
            ExternalProvider::Tmdb,
            None,
            None,
            None,
        );
    }

    if let Some(tmdb_images) = details.images.as_ref() {
        for image in &tmdb_images.posters {
            push_tmdb_image(&mut images, ImageKind::Poster, image, image_base_url);
        }
        for image in &tmdb_images.backdrops {
            push_tmdb_image(&mut images, ImageKind::Backdrop, image, image_base_url);
        }
        for image in &tmdb_images.logos {
            push_tmdb_image(&mut images, ImageKind::Logo, image, image_base_url);
        }
    }

    CanonicalMetadata {
        title: details.title,
        original_title: details.original_title,
        overview: details.overview,
        release_date: details.release_date,
        runtime_minutes: details.runtime,
        tagline: details.tagline,
        genres: details
            .genres
            .into_iter()
            .map(|genre| genre.name)
            .filter(|name| !name.trim().is_empty())
            .collect(),
        ratings: ratings_from_release_dates(details.release_dates.as_ref()),
        images,
        credits: credits_from_tmdb(details.credits.unwrap_or_default()),
        collections: details
            .belongs_to_collection
            .into_iter()
            .filter(|collection| !collection.name.trim().is_empty())
            .map(|collection| CollectionRef {
                name: collection.name,
                overview: None,
                sort_order: None,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: collection.id.to_string(),
                }],
            })
            .collect(),
        studios: details
            .production_companies
            .into_iter()
            .filter(|company| !company.name.trim().is_empty())
            .map(|company| StudioRef {
                name: company.name,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: company.id.to_string(),
                }],
            })
            .collect(),
        external_ids,
        ..CanonicalMetadata::default()
    }
}

fn ratings_from_release_dates(release_dates: Option<&TmdbReleaseDates>) -> Vec<ContentRating> {
    release_dates
        .into_iter()
        .flat_map(|dates| dates.results.iter())
        .filter_map(|country| {
            country
                .release_dates
                .iter()
                .find(|date| !date.certification.trim().is_empty())
                .map(|date| ContentRating {
                    source: format!("TMDB:{}", country.iso_3166_1),
                    value: date.certification.clone(),
                })
        })
        .collect()
}

fn credits_from_tmdb(credits: TmdbCredits) -> Vec<Credit> {
    let mut output = Vec::new();

    for member in credits.cast {
        output.push(Credit {
            name: member.name,
            role: CreditRole::Actor,
            character: member.character,
            order: member.order,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    for member in credits.crew {
        output.push(Credit {
            name: member.name,
            role: credit_role_from_tmdb_job(member.job.as_deref()),
            character: None,
            order: None,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    output
}

fn credit_role_from_tmdb_job(job: Option<&str>) -> CreditRole {
    match job.unwrap_or_default().to_ascii_lowercase().as_str() {
        "director" => CreditRole::Director,
        "writer" | "screenplay" | "story" => CreditRole::Writer,
        "producer" | "executive producer" => CreditRole::Producer,
        "creator" => CreditRole::Creator,
        value if value.is_empty() => CreditRole::Other("crew".to_owned()),
        value => CreditRole::Other(value.to_owned()),
    }
}

fn tmdb_person_external_ids(id: Option<u64>) -> Vec<ExternalId> {
    id.map(|id| ExternalId {
        provider: ExternalProvider::Tmdb,
        value: id.to_string(),
    })
    .into_iter()
    .collect()
}

fn push_tmdb_image(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    image: &TmdbImage,
    image_base_url: &str,
) {
    push_provider_image_uri(
        images,
        kind,
        Some(&image.file_path),
        image_base_url,
        ExternalProvider::Tmdb,
        image.width,
        image.height,
        image.iso_639_1.clone(),
    );
}
