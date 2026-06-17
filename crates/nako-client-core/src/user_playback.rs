#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUserPlaybackPagedRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub page: Option<crate::CorePageQuery>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUserPlaybackItemRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUserPlaybackItemWriteRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub item_id: String,
    pub body_utf8: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUserPlaybackProfilePreferenceRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub body_utf8: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUserPlaybackProfileRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub profile_id: String,
    pub body_utf8: Option<String>,
}

#[must_use]
pub fn build_get_user_playback_profile_preference_request(
    input: &CoreUserPlaybackProfilePreferenceRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profile_preference",
        &input.base_url,
        &input.access_token,
        "GET",
        "/users/me/playback-profile",
        Vec::new(),
        None,
    )
}

#[must_use]
pub fn build_set_user_playback_profile_preference_request(
    input: &CoreUserPlaybackProfilePreferenceRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profile_preference.set",
        &input.base_url,
        &input.access_token,
        "PUT",
        "/users/me/playback-profile",
        Vec::new(),
        input.body_utf8.clone(),
    )
}

#[must_use]
pub fn build_delete_user_playback_profile_preference_request(
    input: &CoreUserPlaybackProfilePreferenceRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profile_preference.delete",
        &input.base_url,
        &input.access_token,
        "DELETE",
        "/users/me/playback-profile",
        Vec::new(),
        None,
    )
}

#[must_use]
pub fn build_list_user_playback_profiles_request(
    input: &CoreUserPlaybackPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profiles",
        &input.base_url,
        &input.access_token,
        "GET",
        "/users/me/playback-profiles",
        page_query(input.page),
        None,
    )
}

#[must_use]
pub fn build_create_user_playback_profile_request(
    input: &CoreUserPlaybackProfilePreferenceRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profiles.create",
        &input.base_url,
        &input.access_token,
        "POST",
        "/users/me/playback-profiles",
        Vec::new(),
        input.body_utf8.clone(),
    )
}

#[must_use]
pub fn build_get_user_playback_profile_request(
    input: &CoreUserPlaybackProfileRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profiles.get",
        &input.base_url,
        &input.access_token,
        "GET",
        &profile_path(&input.profile_id),
        Vec::new(),
        None,
    )
}

#[must_use]
pub fn build_update_user_playback_profile_request(
    input: &CoreUserPlaybackProfileRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profiles.update",
        &input.base_url,
        &input.access_token,
        "PUT",
        &profile_path(&input.profile_id),
        Vec::new(),
        input.body_utf8.clone(),
    )
}

#[must_use]
pub fn build_delete_user_playback_profile_request(
    input: &CoreUserPlaybackProfileRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.profiles.delete",
        &input.base_url,
        &input.access_token,
        "DELETE",
        &profile_path(&input.profile_id),
        Vec::new(),
        None,
    )
}

#[must_use]
pub fn build_get_user_playback_state_request(
    input: &CoreUserPlaybackItemRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.state",
        &input.base_url,
        &input.access_token,
        "GET",
        &item_path(&input.item_id),
        Vec::new(),
        None,
    )
}

#[must_use]
pub fn build_list_continue_watching_request(
    input: &CoreUserPlaybackPagedRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.continue_watching",
        &input.base_url,
        &input.access_token,
        "GET",
        "/users/me/playback-state/continue-watching",
        page_query(input.page),
        None,
    )
}

#[must_use]
pub fn build_update_user_playback_progress_request(
    input: &CoreUserPlaybackItemWriteRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.progress",
        &input.base_url,
        &input.access_token,
        "PUT",
        &format!("{}/progress", item_path(&input.item_id)),
        Vec::new(),
        Some(input.body_utf8.clone()),
    )
}

#[must_use]
pub fn build_set_user_watched_state_request(
    input: &CoreUserPlaybackItemWriteRequestInput,
) -> crate::CoreHttpRequest {
    build_user_playback_request(
        "user_playback.watched",
        &input.base_url,
        &input.access_token,
        "PUT",
        &format!("{}/watched", item_path(&input.item_id)),
        Vec::new(),
        Some(input.body_utf8.clone()),
    )
}

fn build_user_playback_request(
    request_id: &str,
    base_url: &str,
    access_token: &str,
    method: &str,
    path: &str,
    query: Vec<crate::CoreQueryParam>,
    body_utf8: Option<String>,
) -> crate::CoreHttpRequest {
    let headers = if body_utf8.is_some() {
        vec![crate::CoreHttpHeader::new(
            "Content-Type",
            "application/json",
        )]
    } else {
        Vec::new()
    };

    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(request_id, base_url, method, path)
            .query(query)
            .headers(headers)
            .access_token(Some(access_token.to_owned()))
            .body_utf8(body_utf8),
    )
}

fn item_path(item_id: &str) -> String {
    format!(
        "/users/me/playback-state/items/{}",
        crate::encode_path_segment(item_id)
    )
}

fn profile_path(profile_id: &str) -> String {
    format!(
        "/users/me/playback-profiles/{}",
        crate::encode_path_segment(profile_id)
    )
}

fn page_query(page: Option<crate::CorePageQuery>) -> Vec<crate::CoreQueryParam> {
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
