use serde::{Deserialize, Serialize};

pub const STABLE_INTAKE_REQUIRED_OBSERVATIONS: u8 = 2;

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
}
