use serde::{Deserialize, Serialize};

pub const STABLE_INTAKE_REQUIRED_OBSERVATIONS: u8 = 2;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakePlanInput {
    pub ready_candidates: u64,
    pub inspecting_candidates: u64,
    pub blocked_candidates: u64,
    pub recorded_candidates: u64,
    pub newly_ready_candidates: u64,
    pub suppressed_candidates: u64,
    pub active_suppressions: u64,
    pub failure_count: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakePlan {
    pub discover: WatchFolderIntakeDiscoverStep,
    pub suppression: WatchFolderIntakeSuppressionStep,
    pub enqueue: WatchFolderIntakeEnqueueDecision,
    pub summary: WatchFolderIntakeSummary,
}

impl WatchFolderIntakePlan {
    #[must_use]
    pub fn idle() -> Self {
        plan_watch_folder_intake(WatchFolderIntakePlanInput::default())
    }

    #[must_use]
    pub fn should_enqueue_scan(&self) -> bool {
        self.enqueue.action == WatchFolderIntakeEnqueueAction::EnqueueLibraryScan
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakeDiscoverStep {
    pub ready_candidates: u64,
    pub inspecting_candidates: u64,
    pub blocked_candidates: u64,
    pub recorded_candidates: u64,
    pub newly_ready_candidates: u64,
    pub failure_count: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakeSuppressionStep {
    pub suppressed_candidates: u64,
    pub active_suppressions: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakeEnqueueDecision {
    pub action: WatchFolderIntakeEnqueueAction,
    pub reason: WatchFolderIntakeEnqueueReason,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchFolderIntakeEnqueueAction {
    EnqueueLibraryScan,
    Skip,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchFolderIntakeEnqueueReason {
    NewStableCandidates,
    NoNewStableCandidates,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatchFolderIntakeSummary {
    pub observed_candidates: u64,
    pub suppressed_candidates: u64,
    pub blocked_candidates: u64,
    pub newly_ready_candidates: u64,
    pub failure_count: u64,
    pub enqueue_scan: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StableIntakeCandidateEvidence {
    pub observation_key: String,
    pub consecutive_stable_observations: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StableIntakeCandidateState {
    Inspecting,
    Stable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StableIntakeCandidateDecision {
    pub state: StableIntakeCandidateState,
    pub evidence: StableIntakeCandidateEvidence,
}

impl StableIntakeCandidateDecision {
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.state == StableIntakeCandidateState::Stable
    }
}

#[must_use]
pub fn observe_stable_intake_candidate(
    previous: Option<&StableIntakeCandidateEvidence>,
    observation_key: impl Into<String>,
) -> StableIntakeCandidateDecision {
    let observation_key = observation_key.into();
    let consecutive_stable_observations = previous
        .filter(|previous| previous.observation_key == observation_key)
        .map_or(1, |previous| {
            previous.consecutive_stable_observations.saturating_add(1)
        });
    let evidence = StableIntakeCandidateEvidence {
        observation_key,
        consecutive_stable_observations,
    };
    let state = if evidence.consecutive_stable_observations >= STABLE_INTAKE_REQUIRED_OBSERVATIONS {
        StableIntakeCandidateState::Stable
    } else {
        StableIntakeCandidateState::Inspecting
    };

    StableIntakeCandidateDecision { state, evidence }
}

#[must_use]
pub fn plan_watch_folder_intake(input: WatchFolderIntakePlanInput) -> WatchFolderIntakePlan {
    let enqueue = if input.newly_ready_candidates > 0 {
        WatchFolderIntakeEnqueueDecision {
            action: WatchFolderIntakeEnqueueAction::EnqueueLibraryScan,
            reason: WatchFolderIntakeEnqueueReason::NewStableCandidates,
        }
    } else {
        WatchFolderIntakeEnqueueDecision {
            action: WatchFolderIntakeEnqueueAction::Skip,
            reason: WatchFolderIntakeEnqueueReason::NoNewStableCandidates,
        }
    };

    WatchFolderIntakePlan {
        discover: WatchFolderIntakeDiscoverStep {
            ready_candidates: input.ready_candidates,
            inspecting_candidates: input.inspecting_candidates,
            blocked_candidates: input.blocked_candidates,
            recorded_candidates: input.recorded_candidates,
            newly_ready_candidates: input.newly_ready_candidates,
            failure_count: input.failure_count,
        },
        suppression: WatchFolderIntakeSuppressionStep {
            suppressed_candidates: input.suppressed_candidates,
            active_suppressions: input.active_suppressions,
        },
        enqueue,
        summary: WatchFolderIntakeSummary {
            observed_candidates: input
                .recorded_candidates
                .saturating_add(input.suppressed_candidates),
            suppressed_candidates: input.suppressed_candidates,
            blocked_candidates: input.blocked_candidates,
            newly_ready_candidates: input.newly_ready_candidates,
            failure_count: input.failure_count,
            enqueue_scan: enqueue.action == WatchFolderIntakeEnqueueAction::EnqueueLibraryScan,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_observation_requires_follow_up_before_candidate_is_stable() {
        let decision = observe_stable_intake_candidate(None, "sha256:first");

        assert_eq!(decision.state, StableIntakeCandidateState::Inspecting);
        assert_eq!(
            decision.evidence,
            StableIntakeCandidateEvidence {
                observation_key: "sha256:first".to_owned(),
                consecutive_stable_observations: 1,
            }
        );
        assert!(!decision.is_stable());
    }

    #[test]
    fn identical_follow_up_observation_marks_candidate_stable() {
        let first = observe_stable_intake_candidate(None, "sha256:stable");
        let second = observe_stable_intake_candidate(Some(&first.evidence), "sha256:stable");

        assert_eq!(second.state, StableIntakeCandidateState::Stable);
        assert_eq!(second.evidence.consecutive_stable_observations, 2);
        assert!(second.is_stable());
    }

    #[test]
    fn changed_observation_resets_stability_counter() {
        let stable = observe_stable_intake_candidate(
            Some(&StableIntakeCandidateEvidence {
                observation_key: "sha256:old".to_owned(),
                consecutive_stable_observations: 2,
            }),
            "sha256:new",
        );

        assert_eq!(stable.state, StableIntakeCandidateState::Inspecting);
        assert_eq!(stable.evidence.consecutive_stable_observations, 1);
        assert_eq!(stable.evidence.observation_key, "sha256:new");
    }

    #[test]
    fn watch_folder_intake_plan_enqueues_only_for_new_stable_candidates() {
        let inspecting = plan_watch_folder_intake(WatchFolderIntakePlanInput {
            inspecting_candidates: 1,
            recorded_candidates: 1,
            ..WatchFolderIntakePlanInput::default()
        });

        assert!(!inspecting.should_enqueue_scan());
        assert_eq!(
            inspecting.enqueue,
            WatchFolderIntakeEnqueueDecision {
                action: WatchFolderIntakeEnqueueAction::Skip,
                reason: WatchFolderIntakeEnqueueReason::NoNewStableCandidates,
            }
        );

        let ready = plan_watch_folder_intake(WatchFolderIntakePlanInput {
            ready_candidates: 1,
            recorded_candidates: 1,
            newly_ready_candidates: 1,
            ..WatchFolderIntakePlanInput::default()
        });

        assert!(ready.should_enqueue_scan());
        assert_eq!(
            ready.enqueue,
            WatchFolderIntakeEnqueueDecision {
                action: WatchFolderIntakeEnqueueAction::EnqueueLibraryScan,
                reason: WatchFolderIntakeEnqueueReason::NewStableCandidates,
            }
        );
    }

    #[test]
    fn watch_folder_intake_plan_keeps_only_redaction_safe_counts() {
        let plan = plan_watch_folder_intake(WatchFolderIntakePlanInput {
            ready_candidates: 2,
            inspecting_candidates: 1,
            blocked_candidates: 3,
            recorded_candidates: 6,
            newly_ready_candidates: 2,
            suppressed_candidates: 4,
            active_suppressions: 1,
            failure_count: 5,
        });

        assert_eq!(plan.discover.recorded_candidates, 6);
        assert_eq!(plan.suppression.suppressed_candidates, 4);
        assert_eq!(plan.suppression.active_suppressions, 1);
        assert_eq!(plan.summary.observed_candidates, 10);
        assert_eq!(plan.summary.blocked_candidates, 3);
        assert_eq!(plan.summary.failure_count, 5);
        assert!(plan.summary.enqueue_scan);
    }
}
