#[must_use]
pub fn encode_path_segment(value: &str) -> String {
    percent_encode(value)
}

pub(crate) fn path_with_query(path: &str, query: &[crate::CoreQueryParam]) -> String {
    if query.is_empty() {
        return path.to_owned();
    }
    let mut path_and_query = path.to_owned();
    path_and_query.push('?');
    for (index, param) in query.iter().enumerate() {
        if index > 0 {
            path_and_query.push('&');
        }
        path_and_query.push_str(&percent_encode(&param.name));
        path_and_query.push('=');
        path_and_query.push_str(&percent_encode(&param.value));
    }
    path_and_query
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
        }
    }
    encoded
}

#[must_use]
pub fn url_on(base_url: &str, path_and_query: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path_and_query)
}
