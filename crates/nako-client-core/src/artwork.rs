#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreArtworkImageRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub image_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[must_use]
pub fn build_artwork_image_request(input: &CoreArtworkImageRequestInput) -> crate::CoreHttpRequest {
    let mut query = Vec::new();
    if let Some(width) = input.width {
        query.push(crate::CoreQueryParam::new("width", width.to_string()));
    }
    if let Some(height) = input.height {
        query.push(crate::CoreQueryParam::new("height", height.to_string()));
    }

    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            crate::ARTWORK_IMAGE_REQUEST_ID,
            &input.base_url,
            "GET",
            &format!("/images/{}", crate::encode_path_segment(&input.image_id)),
        )
        .query(query)
        .access_token(Some(input.access_token.clone())),
    )
}
