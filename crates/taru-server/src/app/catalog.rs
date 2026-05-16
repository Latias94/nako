use taru_api::{
    GenreItemsResponse, GenreListResponse, ImagesResponse, ItemCreditsResponse, ItemDetailResponse,
    ItemsResponse, PeopleResponse, PersonItemsResponse, PersonResponse, SearchItemHit,
    SearchResponse, TagItemsResponse, TagsResponse, page_info_from_request,
};
use taru_core::{
    CatalogRepository, GenreId, MediaItemId, MediaProbeRepository, MediaRepository, MediaSourceId,
    PageRequest, PersonId, Result, TagId, TaruError,
};
use taru_db::SqliteStore;
use taru_search::{SearchIndex, SearchQuery};

#[derive(Clone, Debug)]
pub(crate) struct CatalogAppService {
    store: SqliteStore,
}

impl CatalogAppService {
    pub(crate) fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    pub async fn list_items(&self, page: PageRequest) -> Result<ItemsResponse> {
        let page = page.clamped();
        let items = self.store.list_media_items(page).await?;

        Ok(ItemsResponse {
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn get_item(&self, item_id: MediaItemId) -> Result<ItemDetailResponse> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        let sources = self
            .store
            .list_item_sources(item.id, PageRequest::first_page())
            .await?;
        let credits = self.store.list_item_credits(item.id).await?;
        let genres = self.store.list_item_genres(item.id).await?;
        let tags = self.store.list_item_tags(item.id).await?;
        let collections = self.store.list_item_collections(item.id).await?;
        let studios = self.store.list_item_studios(item.id).await?;
        let images = self.store.list_item_images(item.id).await?;

        Ok(ItemDetailResponse {
            item: item.into(),
            sources: sources.into_iter().map(Into::into).collect(),
            credits: credits.into_iter().map(Into::into).collect(),
            genres: genres.into_iter().map(Into::into).collect(),
            tags: tags.into_iter().map(Into::into).collect(),
            collections: collections.into_iter().map(Into::into).collect(),
            studios: studios.into_iter().map(Into::into).collect(),
            images: images.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_item_credits(&self, item_id: MediaItemId) -> Result<ItemCreditsResponse> {
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        let credits = self.store.list_item_credits(item.id).await?;
        let mut people = Vec::with_capacity(credits.len());

        for credit in &credits {
            if let Some(person) = self.store.get_person(credit.person_id).await? {
                people.push(person);
            }
        }

        Ok(ItemCreditsResponse {
            item_id: item.id,
            credits: credits.into_iter().map(Into::into).collect(),
            people: people.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_item_images(&self, item_id: MediaItemId) -> Result<ImagesResponse> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let images = self.store.list_item_images(item_id).await?;

        Ok(ImagesResponse {
            item_id,
            images: images.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_people(&self, page: PageRequest) -> Result<PeopleResponse> {
        let page = page.clamped();
        let people = self.store.list_people(page).await?;

        Ok(PeopleResponse {
            page: page_info_from_request(page, people.len()),
            people: people.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn get_person(&self, person_id: PersonId) -> Result<PersonResponse> {
        Ok(PersonResponse {
            person: self.get_person_record(person_id).await?.into(),
        })
    }

    async fn get_person_record(&self, person_id: PersonId) -> Result<taru_core::Person> {
        self.store
            .get_person(person_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "person",
                id: person_id.to_string(),
            })
    }

    pub async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<PersonItemsResponse> {
        let page = page.clamped();
        let person = self.get_person_record(person_id).await?;
        let items = self.store.list_person_items(person.id, page).await?;

        Ok(PersonItemsResponse {
            person: person.into(),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_tags(&self, page: PageRequest) -> Result<TagsResponse> {
        let page = page.clamped();
        let tags = self.store.list_tags(page).await?;

        Ok(TagsResponse {
            page: page_info_from_request(page, tags.len()),
            tags: tags.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_tag_items(
        &self,
        tag_id: TagId,
        page: PageRequest,
    ) -> Result<TagItemsResponse> {
        let page = page.clamped();
        let tag = self
            .store
            .get_tag(tag_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "tag",
                id: tag_id.to_string(),
            })?;
        let items = self.store.list_tag_items(tag.id, page).await?;

        Ok(TagItemsResponse {
            tag: tag.into(),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_genres(&self, page: PageRequest) -> Result<GenreListResponse> {
        let page = page.clamped();
        let genres = self.store.list_genres(page).await?;

        Ok(GenreListResponse {
            page: page_info_from_request(page, genres.len()),
            genres: genres.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<GenreItemsResponse> {
        let page = page.clamped();
        let genre = self
            .store
            .get_genre(genre_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "genre",
                id: genre_id.to_string(),
            })?;
        let items = self.store.list_genre_items(genre.id, page).await?;

        Ok(GenreItemsResponse {
            genre: genre.into(),
            page: page_info_from_request(page, items.len()),
            items: items.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn search_items(
        &self,
        query: String,
        facets: Vec<String>,
        page: PageRequest,
    ) -> Result<SearchResponse> {
        let page = page.clamped();
        let hits = self
            .store
            .search(SearchQuery {
                query,
                facets,
                limit: page.limit,
                offset: u32::try_from(page.offset).map_err(|err| TaruError::InvalidInput {
                    message: format!("search offset is too large: {err}"),
                })?,
            })
            .await?;
        let mut output_hits = Vec::with_capacity(hits.len());

        for hit in hits {
            let item = self
                .store
                .get_media_item(hit.item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: hit.item_id.to_string(),
                })?;
            output_hits.push(SearchItemHit {
                item: item.into(),
                score: hit.score,
            });
        }

        Ok(SearchResponse {
            page: page_info_from_request(page, output_hits.len()),
            hits: output_hits,
        })
    }

    pub async fn get_source_probe(
        &self,
        source_id: MediaSourceId,
    ) -> Result<taru_api::SourceProbeResponse> {
        let probe = self
            .store
            .get_media_probe(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source_probe",
                id: source_id.to_string(),
            })?;

        Ok(taru_api::SourceProbeResponse {
            source_id,
            probe: probe.into(),
        })
    }
}
