const REDACTED: &str = "<redacted>";

pub(crate) fn sanitize(input: &str, secrets: &[&str]) -> String {
    let mut sanitized = input.to_owned();
    for secret in secrets.iter().copied().filter(|secret| !secret.is_empty()) {
        sanitized = sanitized.replace(secret, REDACTED);
    }
    redact_bearer_tokens(&sanitized)
}

fn redact_bearer_tokens(input: &str) -> String {
    let mut output = Vec::new();
    let mut redact_next = false;
    for token in input.split_whitespace() {
        if redact_next {
            output.push(REDACTED);
            redact_next = false;
            continue;
        }
        output.push(token);
        if token.eq_ignore_ascii_case("bearer") {
            redact_next = true;
        }
    }
    if output.is_empty() {
        input.to_owned()
    } else {
        output.join(" ")
    }
}
