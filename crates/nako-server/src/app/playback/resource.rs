use nako_transcode::{HardwareAcceleration, TranscodeExecutionPolicy};

use crate::config::NakoServerConfig;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PlaybackResourceClass {
    RemoteStream,
    RemoteStage,
    RemuxProcess,
    CpuTranscode,
    GpuTranscode,
    HlsArtifactIo,
}

impl PlaybackResourceClass {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::RemoteStream => "remote_stream",
            Self::RemoteStage => "remote_stage",
            Self::RemuxProcess => "remux_process",
            Self::CpuTranscode => "cpu_transcode",
            Self::GpuTranscode => "gpu_transcode",
            Self::HlsArtifactIo => "hls_artifact_io",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlaybackResourceWorkload {
    DirectStream,
    Remux,
    Hls,
}

impl PlaybackResourceWorkload {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DirectStream => "direct_stream",
            Self::Remux => "remux",
            Self::Hls => "hls",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlaybackResourceEnforcement {
    HostOwned,
    LowLevelRuntimeGuard,
    NotYetEnforced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackResourceRequirement {
    pub(crate) class: PlaybackResourceClass,
    pub(crate) units: usize,
    pub(crate) enforcement: PlaybackResourceEnforcement,
}

impl PlaybackResourceRequirement {
    #[must_use]
    pub(crate) const fn new(
        class: PlaybackResourceClass,
        units: usize,
        enforcement: PlaybackResourceEnforcement,
    ) -> Self {
        Self {
            class,
            units,
            enforcement,
        }
    }

    #[must_use]
    pub(crate) const fn host_owned(class: PlaybackResourceClass, units: usize) -> Self {
        Self::new(class, units, PlaybackResourceEnforcement::HostOwned)
    }

    #[must_use]
    pub(crate) const fn low_level_guard(class: PlaybackResourceClass, units: usize) -> Self {
        Self::new(
            class,
            units,
            PlaybackResourceEnforcement::LowLevelRuntimeGuard,
        )
    }

    #[must_use]
    pub(crate) const fn not_yet_enforced(class: PlaybackResourceClass, units: usize) -> Self {
        Self::new(class, units, PlaybackResourceEnforcement::NotYetEnforced)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackResourceDemand {
    pub(crate) workload: PlaybackResourceWorkload,
    requirements: Vec<PlaybackResourceRequirement>,
}

impl PlaybackResourceDemand {
    #[must_use]
    pub(crate) fn direct_stream(remote_input: bool) -> Self {
        let requirements = remote_input
            .then(|| {
                PlaybackResourceRequirement::host_owned(PlaybackResourceClass::RemoteStream, 1)
            })
            .into_iter()
            .collect();

        Self {
            workload: PlaybackResourceWorkload::DirectStream,
            requirements,
        }
    }

    #[must_use]
    pub(crate) fn remux(remote_input: bool) -> Self {
        let mut requirements = Vec::new();
        if remote_input {
            requirements.push(PlaybackResourceRequirement::host_owned(
                PlaybackResourceClass::RemoteStage,
                1,
            ));
        }
        requirements.push(PlaybackResourceRequirement::low_level_guard(
            PlaybackResourceClass::RemuxProcess,
            1,
        ));

        Self {
            workload: PlaybackResourceWorkload::Remux,
            requirements,
        }
    }

    #[must_use]
    pub(crate) fn hls(remote_input: bool, execution_policy: TranscodeExecutionPolicy) -> Self {
        let mut requirements = Vec::new();
        if remote_input {
            requirements.push(PlaybackResourceRequirement::host_owned(
                PlaybackResourceClass::RemoteStage,
                1,
            ));
        }

        let transcode_class = if execution_policy.acceleration.resource_acceleration()
            == HardwareAcceleration::None
        {
            PlaybackResourceClass::CpuTranscode
        } else {
            PlaybackResourceClass::GpuTranscode
        };
        requirements.push(PlaybackResourceRequirement::low_level_guard(
            transcode_class,
            1,
        ));
        requirements.push(PlaybackResourceRequirement::not_yet_enforced(
            PlaybackResourceClass::HlsArtifactIo,
            1,
        ));

        Self {
            workload: PlaybackResourceWorkload::Hls,
            requirements,
        }
    }

    #[must_use]
    pub(crate) fn requirements(&self) -> &[PlaybackResourceRequirement] {
        &self.requirements
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackResourceCapacity {
    pub(crate) remote_streams: usize,
    pub(crate) remote_stages: usize,
    pub(crate) remux_processes: usize,
    pub(crate) cpu_transcodes: usize,
    pub(crate) gpu_transcodes: usize,
}

impl PlaybackResourceCapacity {
    #[must_use]
    pub(crate) const fn from_config(config: &NakoServerConfig) -> Self {
        Self {
            remote_streams: config.playback.remote_stream_concurrency,
            remote_stages: config.playback.remote_stage_concurrency,
            remux_processes: config.remux_concurrency,
            cpu_transcodes: config.transcode.cpu_concurrency,
            gpu_transcodes: config.transcode.gpu_concurrency,
        }
    }

    #[must_use]
    pub(crate) const fn capacity_for(self, class: PlaybackResourceClass) -> Option<usize> {
        match class {
            PlaybackResourceClass::RemoteStream => Some(self.remote_streams),
            PlaybackResourceClass::RemoteStage => Some(self.remote_stages),
            PlaybackResourceClass::RemuxProcess => Some(self.remux_processes),
            PlaybackResourceClass::CpuTranscode => Some(self.cpu_transcodes),
            PlaybackResourceClass::GpuTranscode => Some(self.gpu_transcodes),
            PlaybackResourceClass::HlsArtifactIo => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlaybackResourceAdmissionStatus {
    Accepted,
    Rejected,
    NotYetEnforced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackResourceClassAdmission {
    pub(crate) class: PlaybackResourceClass,
    pub(crate) requested_units: usize,
    pub(crate) capacity_units: Option<usize>,
    pub(crate) status: PlaybackResourceAdmissionStatus,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackResourceAdmissionDecision {
    pub(crate) demand: PlaybackResourceDemand,
    classes: Vec<PlaybackResourceClassAdmission>,
}

impl PlaybackResourceAdmissionDecision {
    #[must_use]
    pub(crate) fn accepted(&self) -> bool {
        self.classes
            .iter()
            .all(|class| class.status != PlaybackResourceAdmissionStatus::Rejected)
    }

    #[must_use]
    pub(crate) fn has_not_yet_enforced_classes(&self) -> bool {
        self.classes
            .iter()
            .any(|class| class.status == PlaybackResourceAdmissionStatus::NotYetEnforced)
    }

    #[must_use]
    pub(crate) fn status_for(
        &self,
        class: PlaybackResourceClass,
    ) -> Option<PlaybackResourceAdmissionStatus> {
        self.classes
            .iter()
            .find(|admission| admission.class == class)
            .map(|admission| admission.status)
    }

    #[must_use]
    pub(crate) fn classes(&self) -> &[PlaybackResourceClassAdmission] {
        &self.classes
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlaybackRuntimeAdmission {
    capacity: PlaybackResourceCapacity,
}

impl PlaybackRuntimeAdmission {
    #[must_use]
    pub(crate) const fn new(capacity: PlaybackResourceCapacity) -> Self {
        Self { capacity }
    }

    #[must_use]
    pub(crate) const fn from_config(config: &NakoServerConfig) -> Self {
        Self::new(PlaybackResourceCapacity::from_config(config))
    }

    #[must_use]
    pub(crate) fn decide(
        &self,
        demand: PlaybackResourceDemand,
    ) -> PlaybackResourceAdmissionDecision {
        let classes = demand
            .requirements()
            .iter()
            .map(|requirement| self.decide_requirement(*requirement))
            .collect();

        PlaybackResourceAdmissionDecision { demand, classes }
    }

    fn decide_requirement(
        &self,
        requirement: PlaybackResourceRequirement,
    ) -> PlaybackResourceClassAdmission {
        let capacity = self.capacity.capacity_for(requirement.class);
        let (status, reason) = match requirement.enforcement {
            PlaybackResourceEnforcement::HostOwned => {
                if capacity.is_some_and(|capacity| capacity >= requirement.units) {
                    (
                        PlaybackResourceAdmissionStatus::Accepted,
                        "host-owned capacity is available",
                    )
                } else {
                    (
                        PlaybackResourceAdmissionStatus::Rejected,
                        "host-owned capacity is unavailable",
                    )
                }
            }
            PlaybackResourceEnforcement::LowLevelRuntimeGuard => (
                PlaybackResourceAdmissionStatus::NotYetEnforced,
                "currently guarded by the low-level runner",
            ),
            PlaybackResourceEnforcement::NotYetEnforced => (
                PlaybackResourceAdmissionStatus::NotYetEnforced,
                "host-owned admission is not implemented yet",
            ),
        };

        PlaybackResourceClassAdmission {
            class: requirement.class,
            requested_units: requirement.units,
            capacity_units: capacity,
            status,
            reason,
        }
    }
}
