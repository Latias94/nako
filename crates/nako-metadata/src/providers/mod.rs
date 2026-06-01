mod bangumi;
mod common;
mod douban;
mod tmdb;

pub use bangumi::{BangumiMetadataProvider, BangumiProviderConfig};
pub use douban::{DoubanMetadataProvider, DoubanProviderConfig};
pub use tmdb::{TmdbMetadataProvider, TmdbProviderConfig};

pub(crate) use bangumi::{BangumiEpisode, BangumiInfoBoxItem, BangumiSubject};
pub(crate) use common::{
    api_key_query, bearer_headers, first_non_empty, header_map_from_pairs, non_empty_string,
    now_utc_string, provider_parse_error, provider_request_error, push_provider_image_uri,
    release_year, retry_delay, tmdb_parse_error, truncate_message,
};
pub(crate) use douban::{DoubanPerson, DoubanSubject};
pub(crate) use tmdb::{
    TmdbCredits, TmdbEpisodeDetails, TmdbEpisodeSummary, TmdbImage, TmdbMovieDetails,
    TmdbMovieSearchResult, TmdbReleaseDates, TmdbSeasonDetails, TmdbSeasonSummary,
    TmdbSeriesDetails, result_original_title, result_release_date, result_title,
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
