mod direct;

pub use direct::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_play_response_plan_handles_full_empty_and_partial_ranges() {
        let empty = plan_direct_play_response(0, "video/mp4", DirectPlayRangeRequest::None);
        assert_eq!(empty.status, DirectPlayResponseStatus::Ok);
        assert_eq!(empty.body_len, 0);
        assert_eq!(empty.seek_offset, 0);
        assert_eq!(empty.content_range, None);

        let requested = parse_http_range_header("bytes=2-5").unwrap();
        let partial =
            plan_direct_play_response(10, "video/mp4", DirectPlayRangeRequest::Range(requested));
        assert_eq!(partial.status, DirectPlayResponseStatus::PartialContent);
        assert_eq!(partial.body_len, 4);
        assert_eq!(partial.seek_offset, 2);
        assert_eq!(partial.content_range.as_deref(), Some("bytes 2-5/10"));
    }

    #[test]
    fn direct_play_response_plan_maps_invalid_ranges_to_416() {
        let out_of_bounds = RequestedByteRange {
            start: Some(20),
            end: Some(30),
        };
        let invalid = plan_direct_play_response(
            10,
            "video/mp4",
            DirectPlayRangeRequest::Range(out_of_bounds),
        );
        assert_eq!(
            invalid.status,
            DirectPlayResponseStatus::RangeNotSatisfiable
        );
        assert_eq!(invalid.body_len, 0);
        assert_eq!(invalid.content_range.as_deref(), Some("bytes */10"));

        let malformed = plan_direct_play_response(10, "video/mp4", DirectPlayRangeRequest::Invalid);
        assert_eq!(
            malformed.status,
            DirectPlayResponseStatus::RangeNotSatisfiable
        );
        assert_eq!(malformed.content_range.as_deref(), Some("bytes */10"));
    }

    #[test]
    fn range_parser_resolves_open_and_suffix_ranges() {
        let open = parse_http_range_header("bytes=2-").unwrap();
        let suffix = parse_http_range_header("bytes=-4").unwrap();

        assert_eq!(
            resolve_byte_range(Some(open), 10).unwrap(),
            Some(ResolvedByteRange { start: 2, end: 9 })
        );
        assert_eq!(
            resolve_byte_range(Some(suffix), 10).unwrap(),
            Some(ResolvedByteRange { start: 6, end: 9 })
        );
    }
}
