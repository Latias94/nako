use taru_core::CanonicalMetadata;

use crate::providers::DoubanSubject;

pub(crate) fn douban_subject_to_metadata(
    subject: DoubanSubject,
    image_base_url: Option<&str>,
) -> CanonicalMetadata {
    crate::providers::douban_subject_to_metadata(subject, image_base_url)
}
