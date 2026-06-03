use std::{sync::Arc, time::Duration};

use nako_core::{NakoError, Result};
use nako_transcode::{HardwareAcceleration, TranscodeExecutionPolicy};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError},
    time::{Instant, sleep},
};

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
    AdmissionPermit,
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
    pub(crate) const fn admission_permit(class: PlaybackResourceClass, units: usize) -> Self {
        Self::new(class, units, PlaybackResourceEnforcement::AdmissionPermit)
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
        requirements.push(PlaybackResourceRequirement::admission_permit(
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
        requirements.push(PlaybackResourceRequirement::admission_permit(
            transcode_class,
            1,
        ));
        requirements.push(PlaybackResourceRequirement::admission_permit(
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
    pub(crate) hls_artifact_io: usize,
}

impl PlaybackResourceCapacity {
    #[must_use]
    pub(crate) fn from_config(config: &NakoServerConfig) -> Self {
        let transcode = config.transcode.resource_budget();
        Self {
            remote_streams: config.playback.remote_stream_concurrency,
            remote_stages: config.playback.remote_stage_concurrency,
            remux_processes: config.remux_concurrency,
            cpu_transcodes: config.transcode.cpu_concurrency,
            gpu_transcodes: config.transcode.gpu_concurrency,
            hls_artifact_io: transcode
                .cpu_slots
                .saturating_add(transcode.gpu_slots)
                .max(1),
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
            PlaybackResourceClass::HlsArtifactIo => Some(self.hls_artifact_io),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlaybackResourceAdmissionStatus {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlaybackResourceAdmissionPolicy {
    Immediate,
    HlsSupersede,
}

impl PlaybackResourceAdmissionPolicy {
    #[must_use]
    fn wait_policy(self) -> Option<PlaybackResourceWaitPolicy> {
        match self {
            Self::Immediate => None,
            Self::HlsSupersede => Some(PlaybackResourceWaitPolicy {
                operation: "hls supersede",
                timeout: HLS_SUPERSEDE_ADMISSION_WAIT,
                retry_interval: HLS_SUPERSEDE_ADMISSION_RETRY_INTERVAL,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlaybackResourceWaitPolicy {
    operation: &'static str,
    timeout: Duration,
    retry_interval: Duration,
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
    remux_processes: Arc<Semaphore>,
    cpu_transcodes: Arc<Semaphore>,
    gpu_transcodes: Arc<Semaphore>,
    hls_artifact_io: Arc<Semaphore>,
}

impl PlaybackRuntimeAdmission {
    #[must_use]
    pub(crate) fn new(capacity: PlaybackResourceCapacity) -> Self {
        Self {
            capacity,
            remux_processes: Arc::new(Semaphore::new(capacity.remux_processes)),
            cpu_transcodes: Arc::new(Semaphore::new(capacity.cpu_transcodes)),
            gpu_transcodes: Arc::new(Semaphore::new(capacity.gpu_transcodes)),
            hls_artifact_io: Arc::new(Semaphore::new(capacity.hls_artifact_io)),
        }
    }

    #[must_use]
    pub(crate) fn from_config(config: &NakoServerConfig) -> Self {
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

    pub(crate) fn try_acquire(
        &self,
        demand: &PlaybackResourceDemand,
    ) -> Result<PlaybackResourcePermitSet> {
        let mut permits = Vec::new();
        for requirement in demand.requirements().iter().filter(|requirement| {
            requirement.enforcement == PlaybackResourceEnforcement::AdmissionPermit
        }) {
            permits.push(self.try_acquire_requirement(*requirement)?);
        }

        Ok(PlaybackResourcePermitSet { _permits: permits })
    }

    pub(crate) async fn try_acquire_until(
        &self,
        demand: &PlaybackResourceDemand,
        timeout: Duration,
        retry_interval: Duration,
    ) -> Result<PlaybackResourcePermitSet> {
        let deadline = Instant::now() + timeout;

        loop {
            match self.try_acquire(demand) {
                Ok(permit) => return Ok(permit),
                Err(error @ NakoError::Conflict { .. }) if Instant::now() < deadline => {
                    sleep(retry_interval).await;
                    if Instant::now() >= deadline {
                        return Err(error);
                    }
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(crate) fn ensure_capacity_for_policy(
        &self,
        demand: &PlaybackResourceDemand,
        policy: PlaybackResourceAdmissionPolicy,
    ) -> Result<()> {
        let Some(wait_policy) = policy.wait_policy() else {
            return Ok(());
        };

        self.ensure_configured_capacity(demand, wait_policy.operation)
    }

    pub(crate) async fn acquire_for_policy(
        &self,
        demand: &PlaybackResourceDemand,
        policy: PlaybackResourceAdmissionPolicy,
    ) -> Result<PlaybackResourcePermitSet> {
        let Some(wait_policy) = policy.wait_policy() else {
            return self.try_acquire(demand);
        };

        self.ensure_configured_capacity(demand, wait_policy.operation)?;
        self.try_acquire_until(demand, wait_policy.timeout, wait_policy.retry_interval)
            .await
    }

    pub(crate) fn ensure_configured_capacity(
        &self,
        demand: &PlaybackResourceDemand,
        operation: &str,
    ) -> Result<()> {
        let decision = self.decide(demand.clone());
        if decision.accepted() {
            return Ok(());
        }

        let blocked = decision.classes().iter().find(|class| {
            class
                .capacity_units
                .is_none_or(|capacity| capacity < class.requested_units)
        });
        let Some(blocked) = blocked else {
            return Err(NakoError::Conflict {
                message: format!("{operation} resource admission was rejected"),
            });
        };

        Err(NakoError::Conflict {
            message: format!(
                "playback resource {} is unavailable for {operation}: {}",
                blocked.class.as_str(),
                blocked.reason
            ),
        })
    }

    #[must_use]
    pub(crate) fn resource_pressure(&self) -> PlaybackRuntimeResourcePressure {
        PlaybackRuntimeResourcePressure {
            classes: vec![
                self.class_pressure(
                    PlaybackResourceClass::RemoteStream,
                    PlaybackResourceEnforcement::HostOwned,
                    None,
                ),
                self.class_pressure(
                    PlaybackResourceClass::RemoteStage,
                    PlaybackResourceEnforcement::HostOwned,
                    None,
                ),
                self.class_pressure(
                    PlaybackResourceClass::RemuxProcess,
                    PlaybackResourceEnforcement::AdmissionPermit,
                    Some(self.remux_processes.available_permits()),
                ),
                self.class_pressure(
                    PlaybackResourceClass::CpuTranscode,
                    PlaybackResourceEnforcement::AdmissionPermit,
                    Some(self.cpu_transcodes.available_permits()),
                ),
                self.class_pressure(
                    PlaybackResourceClass::GpuTranscode,
                    PlaybackResourceEnforcement::AdmissionPermit,
                    Some(self.gpu_transcodes.available_permits()),
                ),
                self.class_pressure(
                    PlaybackResourceClass::HlsArtifactIo,
                    PlaybackResourceEnforcement::AdmissionPermit,
                    Some(self.hls_artifact_io.available_permits()),
                ),
            ],
        }
    }

    fn class_pressure(
        &self,
        class: PlaybackResourceClass,
        enforcement: PlaybackResourceEnforcement,
        available_permits: Option<usize>,
    ) -> PlaybackRuntimeResourceClassPressure {
        let configured_capacity = self.capacity.capacity_for(class);
        let in_use_permits = configured_capacity
            .zip(available_permits)
            .map(|(configured, available)| configured.saturating_sub(available));

        PlaybackRuntimeResourceClassPressure {
            class,
            enforcement,
            configured_capacity,
            available_permits,
            in_use_permits,
        }
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
            PlaybackResourceEnforcement::AdmissionPermit => {
                if capacity.is_some_and(|capacity| capacity >= requirement.units) {
                    (
                        PlaybackResourceAdmissionStatus::Accepted,
                        "host admission capacity is available",
                    )
                } else {
                    (
                        PlaybackResourceAdmissionStatus::Rejected,
                        "host admission capacity is unavailable",
                    )
                }
            }
        };

        PlaybackResourceClassAdmission {
            class: requirement.class,
            requested_units: requirement.units,
            capacity_units: capacity,
            status,
            reason,
        }
    }

    fn try_acquire_requirement(
        &self,
        requirement: PlaybackResourceRequirement,
    ) -> Result<PlaybackResourcePermit> {
        let semaphore =
            self.semaphore_for_class(requirement.class)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: format!(
                        "playback resource {} does not have an admission semaphore",
                        requirement.class.as_str()
                    ),
                })?;
        let units = u32::try_from(requirement.units).map_err(|_| NakoError::InvalidInput {
            message: format!(
                "playback resource {} requested too many units",
                requirement.class.as_str()
            ),
        })?;
        let permit = semaphore
            .try_acquire_many_owned(units)
            .map_err(|error| match error {
                TryAcquireError::NoPermits => NakoError::Conflict {
                    message: format!("playback resource {} is busy", requirement.class.as_str()),
                },
                TryAcquireError::Closed => NakoError::Provider {
                    provider: "playback_resource_admission".to_owned(),
                    message: format!(
                        "playback resource {} admission semaphore was closed",
                        requirement.class.as_str()
                    ),
                },
            })?;

        Ok(PlaybackResourcePermit {
            class: requirement.class,
            _permit: permit,
        })
    }

    fn semaphore_for_class(&self, class: PlaybackResourceClass) -> Option<Arc<Semaphore>> {
        match class {
            PlaybackResourceClass::RemuxProcess => Some(self.remux_processes.clone()),
            PlaybackResourceClass::CpuTranscode => Some(self.cpu_transcodes.clone()),
            PlaybackResourceClass::GpuTranscode => Some(self.gpu_transcodes.clone()),
            PlaybackResourceClass::HlsArtifactIo => Some(self.hls_artifact_io.clone()),
            PlaybackResourceClass::RemoteStream | PlaybackResourceClass::RemoteStage => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PlaybackResourcePermitSet {
    _permits: Vec<PlaybackResourcePermit>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackRuntimeResourcePressure {
    pub(crate) classes: Vec<PlaybackRuntimeResourceClassPressure>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackRuntimeResourceClassPressure {
    pub(crate) class: PlaybackResourceClass,
    pub(crate) enforcement: PlaybackResourceEnforcement,
    pub(crate) configured_capacity: Option<usize>,
    pub(crate) available_permits: Option<usize>,
    pub(crate) in_use_permits: Option<usize>,
}

#[derive(Debug)]
struct PlaybackResourcePermit {
    #[allow(dead_code)]
    class: PlaybackResourceClass,
    _permit: OwnedSemaphorePermit,
}

const HLS_SUPERSEDE_ADMISSION_WAIT: Duration = Duration::from_secs(5);
const HLS_SUPERSEDE_ADMISSION_RETRY_INTERVAL: Duration = Duration::from_millis(50);

#[cfg(test)]
mod tests {
    use super::*;
    use nako_transcode::{
        TranscodeAccelerationPlan, TranscodeExecutionPolicy, TranscodeOutputConstraints,
        TranscodeTrackSelection,
    };

    fn hls_demand() -> PlaybackResourceDemand {
        PlaybackResourceDemand::hls(
            false,
            TranscodeExecutionPolicy::hls_single_variant(
                TranscodeAccelerationPlan::software(),
                TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
            ),
        )
    }

    #[tokio::test]
    async fn playback_resource_admission_immediate_policy_uses_non_waiting_acquire_path() {
        let admission = PlaybackRuntimeAdmission::new(PlaybackResourceCapacity {
            remote_streams: 1,
            remote_stages: 1,
            remux_processes: 1,
            cpu_transcodes: 1,
            gpu_transcodes: 1,
            hls_artifact_io: 1,
        });

        let permit = admission
            .acquire_for_policy(
                &PlaybackResourceDemand::remux(false),
                PlaybackResourceAdmissionPolicy::Immediate,
            )
            .await
            .unwrap();

        drop(permit);
    }

    #[tokio::test]
    async fn playback_resource_admission_hls_supersede_policy_rejects_unconfigured_capacity_before_waiting()
     {
        let admission = PlaybackRuntimeAdmission::new(PlaybackResourceCapacity {
            remote_streams: 1,
            remote_stages: 1,
            remux_processes: 1,
            cpu_transcodes: 0,
            gpu_transcodes: 1,
            hls_artifact_io: 1,
        });

        let err = admission
            .acquire_for_policy(&hls_demand(), PlaybackResourceAdmissionPolicy::HlsSupersede)
            .await
            .unwrap_err();
        let NakoError::Conflict { message } = err else {
            panic!("expected conflict");
        };
        assert!(message.contains("hls supersede"));
        assert!(message.contains("unavailable"));
    }

    #[tokio::test]
    async fn playback_resource_admission_hls_supersede_policy_waits_for_permit_release() {
        let admission = PlaybackRuntimeAdmission::new(PlaybackResourceCapacity {
            remote_streams: 1,
            remote_stages: 1,
            remux_processes: 1,
            cpu_transcodes: 1,
            gpu_transcodes: 1,
            hls_artifact_io: 1,
        });
        let demand = hls_demand();
        let first_permit = admission
            .acquire_for_policy(&demand, PlaybackResourceAdmissionPolicy::HlsSupersede)
            .await
            .unwrap();

        let release = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;
            drop(first_permit);
        });

        let started = std::time::Instant::now();
        let second_permit = admission
            .acquire_for_policy(&demand, PlaybackResourceAdmissionPolicy::HlsSupersede)
            .await
            .unwrap();

        assert!(started.elapsed() >= Duration::from_millis(20));
        drop(second_permit);
        release.await.unwrap();
    }
}
