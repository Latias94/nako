pub(crate) mod bangumi;
pub(crate) mod douban;
pub(crate) mod tmdb;

pub(crate) use bangumi::bangumi_subject_to_metadata;
pub(crate) use douban::douban_subject_to_metadata;
pub(crate) use tmdb::{tmdb_movie_details_to_metadata, tmdb_search_result_to_metadata};
