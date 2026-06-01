use nako_core::{MetadataCandidateGraph, MetadataCandidateReviewPlan};

#[must_use]
pub fn build_candidate_review_plan(graph: &MetadataCandidateGraph) -> MetadataCandidateReviewPlan {
    MetadataCandidateReviewPlan::from_graph(graph)
}
