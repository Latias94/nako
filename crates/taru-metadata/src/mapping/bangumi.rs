use taru_core::CanonicalMetadata;

use crate::providers::BangumiSubject;

pub(crate) fn bangumi_subject_to_metadata(
    subject: BangumiSubject,
    image_base_url: &str,
) -> CanonicalMetadata {
    crate::providers::bangumi_subject_to_metadata(subject, image_base_url)
}
