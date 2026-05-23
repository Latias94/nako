mod evidence;
mod hierarchy;
mod plan;
mod source_records;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use hierarchy::resolve_local_inference_plan;
pub use plan::LocalInferenceEngine;
pub use types::{LocalInferencePlan, LocalInferenceRequest, ProvisionalAncestorPlan};
pub(crate) use types::{MediaItemResolution, ProvisionalItemPlan};
