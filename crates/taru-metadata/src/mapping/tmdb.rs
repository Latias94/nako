use taru_core::CanonicalMetadata;

use crate::providers::{TmdbMovieDetails, TmdbMovieSearchResult};

pub(crate) fn tmdb_movie_details_to_metadata(
    details: TmdbMovieDetails,
    image_base_url: &str,
) -> CanonicalMetadata {
    crate::providers::tmdb_movie_details_to_metadata(details, image_base_url)
}

pub(crate) fn tmdb_search_result_to_metadata(
    result: TmdbMovieSearchResult,
    image_base_url: &str,
) -> CanonicalMetadata {
    crate::providers::tmdb_search_result_to_metadata(result, image_base_url)
}
