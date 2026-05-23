use nako_core::{NakoError, Result};
use nako_streaming::{
    PlaybackDecision, PlaybackExecutionPlan, PlaybackSelectionContext, PlaybackStorageContext,
};
use nako_transcode::{OutputContainer, RemuxContainer, TranscodePlan};
use nako_vfs::{StorageBackend, StorageCapabilities, StorageUri};

use super::super::storage::LibraryStorageBackend;
use super::direct::should_budget_remote_stream;

pub(super) async fn playback_selection_context(
    uri: &StorageUri,
    backend: &LibraryStorageBackend,
) -> PlaybackSelectionContext {
    let capabilities = backend
        .stat(uri)
        .await
        .ok()
        .map(|metadata| metadata.capabilities);

    PlaybackSelectionContext {
        storage: PlaybackStorageContext {
            remote: should_budget_remote_stream(uri),
            range_readable: capabilities
                .map(|capabilities| capabilities.contains(StorageCapabilities::RANGE_READABLE)),
        },
        preferences: Default::default(),
    }
}

pub(super) fn remux_output_container(decision: &PlaybackDecision) -> Result<RemuxContainer> {
    match &decision.execution {
        PlaybackExecutionPlan::Remux(plan) => Ok(plan.output_container),
        _ => Err(NakoError::Unsupported(
            "remux app service requires a remux playback decision",
        )),
    }
}

pub(super) fn hls_transcode_plan(decision: &PlaybackDecision) -> Result<&TranscodePlan> {
    match &decision.execution {
        PlaybackExecutionPlan::Transcode(plan) if plan.output_container == OutputContainer::Hls => {
            Ok(plan)
        }
        _ => Err(NakoError::Unsupported(
            "hls app service requires an hls transcode playback decision",
        )),
    }
}
