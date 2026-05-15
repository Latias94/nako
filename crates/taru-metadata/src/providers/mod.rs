mod bangumi;
mod common;
mod douban;
mod tmdb;

pub use bangumi::{BangumiMetadataProvider, BangumiProviderConfig};
pub use douban::{DoubanMetadataProvider, DoubanProviderConfig};
pub use tmdb::{TmdbMetadataProvider, TmdbProviderConfig};

pub(crate) use bangumi::{BangumiSubject, bangumi_subject_to_metadata};
pub(crate) use common::{
    api_key_query, bearer_headers, first_non_empty, header_map_from_pairs, non_empty_string,
    now_utc_string, provider_parse_error, provider_request_error, push_provider_image_uri,
    release_year, retry_delay, tmdb_parse_error, truncate_message,
};
pub(crate) use douban::{DoubanSubject, douban_subject_to_metadata};
pub(crate) use tmdb::{
    TmdbMovieDetails, TmdbMovieSearchResult, tmdb_movie_details_to_metadata,
    tmdb_search_result_to_metadata,
};

pub(crate) const DEFAULT_TMDB_API_BASE_URL: &str = "https://api.themoviedb.org/3";
pub(crate) const DEFAULT_TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";
pub(crate) const DEFAULT_TMDB_LANGUAGE: &str = "en-US";
pub(crate) const DEFAULT_BANGUMI_API_BASE_URL: &str = "https://api.bgm.tv";
pub(crate) const DEFAULT_BANGUMI_IMAGE_BASE_URL: &str = "https://lain.bgm.tv";
pub(crate) const DEFAULT_DOUBAN_API_BASE_URL: &str = "https://api.douban.com/v2";
pub(crate) const TMDB_PROVIDER_NAME: &str = "tmdb";
pub(crate) const BANGUMI_PROVIDER_NAME: &str = "bangumi";
pub(crate) const DOUBAN_PROVIDER_NAME: &str = "douban";
