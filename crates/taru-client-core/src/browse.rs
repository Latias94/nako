#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorePageQuery {
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

impl CorePageQuery {
    #[must_use]
    pub const fn new(limit: Option<u32>, offset: Option<u64>) -> Self {
        Self { limit, offset }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreBrowsePagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreBrowseEntityRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreBrowseEntityPagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub id: String,
    pub page: Option<CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreSearchItemsRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub query: Option<String>,
    pub facets: Vec<String>,
    pub page: Option<CorePageQuery>,
}

#[must_use]
pub fn build_list_libraries_request(input: &CoreBrowsePagedRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.libraries",
        &input.base_url,
        &input.access_token,
        "/libraries",
        page_query(input.page),
    )
}

#[must_use]
pub fn build_get_library_request(input: &CoreBrowseEntityRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.library",
        &input.base_url,
        &input.access_token,
        &format!("/libraries/{}", crate::encode_path_segment(&input.id)),
        Vec::new(),
    )
}

#[must_use]
pub fn build_list_library_sources_request(
    input: &CoreBrowseEntityPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.library_sources",
        &input.base_url,
        &input.access_token,
        &format!(
            "/libraries/{}/sources",
            crate::encode_path_segment(&input.id)
        ),
        page_query(input.page),
    )
}

#[must_use]
pub fn build_list_items_request(input: &CoreBrowsePagedRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.items",
        &input.base_url,
        &input.access_token,
        "/items",
        page_query(input.page),
    )
}

#[must_use]
pub fn build_get_item_request(input: &CoreBrowseEntityRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.item",
        &input.base_url,
        &input.access_token,
        &format!("/items/{}", crate::encode_path_segment(&input.id)),
        Vec::new(),
    )
}

#[must_use]
pub fn build_list_item_images_request(
    input: &CoreBrowseEntityRequestInput,
) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.item_images",
        &input.base_url,
        &input.access_token,
        &format!("/items/{}/images", crate::encode_path_segment(&input.id)),
        Vec::new(),
    )
}

#[must_use]
pub fn build_get_person_request(input: &CoreBrowseEntityRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.person",
        &input.base_url,
        &input.access_token,
        &format!("/people/{}", crate::encode_path_segment(&input.id)),
        Vec::new(),
    )
}

#[must_use]
pub fn build_list_person_items_request(
    input: &CoreBrowseEntityPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.person_items",
        &input.base_url,
        &input.access_token,
        &format!("/people/{}/items", crate::encode_path_segment(&input.id)),
        page_query(input.page),
    )
}

#[must_use]
pub fn build_list_genres_request(input: &CoreBrowsePagedRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.genres",
        &input.base_url,
        &input.access_token,
        "/genres",
        page_query(input.page),
    )
}

#[must_use]
pub fn build_list_genre_items_request(
    input: &CoreBrowseEntityPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.genre_items",
        &input.base_url,
        &input.access_token,
        &format!("/genres/{}/items", crate::encode_path_segment(&input.id)),
        page_query(input.page),
    )
}

#[must_use]
pub fn build_list_tags_request(input: &CoreBrowsePagedRequestInput) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.tags",
        &input.base_url,
        &input.access_token,
        "/tags",
        page_query(input.page),
    )
}

#[must_use]
pub fn build_list_tag_items_request(
    input: &CoreBrowseEntityPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_browse_request(
        "browse.tag_items",
        &input.base_url,
        &input.access_token,
        &format!("/tags/{}/items", crate::encode_path_segment(&input.id)),
        page_query(input.page),
    )
}

#[must_use]
pub fn build_search_items_request(input: &CoreSearchItemsRequestInput) -> crate::CoreHttpRequest {
    let mut query = Vec::new();
    if let Some(search_query) = input
        .query
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
    {
        query.push(crate::CoreQueryParam::new("q", search_query));
    }
    if !input.facets.is_empty() {
        query.push(crate::CoreQueryParam::new("facet", input.facets.join(",")));
    }
    query.extend(page_query(input.page));
    build_browse_request(
        "browse.search",
        &input.base_url,
        &input.access_token,
        "/search",
        query,
    )
}

fn build_browse_request(
    request_id: &str,
    base_url: &str,
    access_token: &str,
    path: &str,
    query: Vec<crate::CoreQueryParam>,
) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(request_id, base_url, "GET", path)
            .query(query)
            .access_token(Some(access_token.to_owned())),
    )
}

fn page_query(page: Option<CorePageQuery>) -> Vec<crate::CoreQueryParam> {
    let Some(page) = page else {
        return Vec::new();
    };
    let mut query = Vec::new();
    if let Some(limit) = page.limit {
        query.push(crate::CoreQueryParam::new("limit", limit.to_string()));
    }
    if let Some(offset) = page.offset {
        query.push(crate::CoreQueryParam::new("offset", offset.to_string()));
    }
    query
}
