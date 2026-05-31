use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time,
};

use super::{
    HardwareAccelerationReport, HardwareEncoderDiscoveryStatus, TranscodeEngineAdapterKind,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeRuntimeInventoryStatus {
    Ready,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRuntimeInventory {
    pub engine: TranscodeEngineAdapterKind,
    pub probe_status: TranscodeRuntimeInventoryStatus,
    pub has_probe_error: bool,
    pub hardware_capability_count: u32,
    pub available_gpu_capabilities: u32,
}

impl TranscodeRuntimeInventory {
    #[must_use]
    pub fn ffmpeg_cli(report: &HardwareAccelerationReport) -> Self {
        let has_probe_error = report.capabilities.iter().any(|capability| {
            capability.encoder_discovery.status == HardwareEncoderDiscoveryStatus::ProbeError
        });
        let available_gpu_capabilities = report
            .capabilities
            .iter()
            .filter(|capability| capability.accelerator.is_gpu() && capability.available)
            .count();

        Self {
            engine: TranscodeEngineAdapterKind::FfmpegCli,
            probe_status: if has_probe_error {
                TranscodeRuntimeInventoryStatus::Degraded
            } else {
                TranscodeRuntimeInventoryStatus::Ready
            },
            has_probe_error,
            hardware_capability_count: usize_to_u32(report.capabilities.len()),
            available_gpu_capabilities: usize_to_u32(available_gpu_capabilities),
        }
    }
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeRuntimeLimits {
    pub max_concurrent_sessions: usize,
    pub timeout_ms: u64,
}

impl Default for TranscodeRuntimeLimits {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 1,
            timeout_ms: 30 * 60 * 1_000,
        }
    }
}

impl TranscodeRuntimeLimits {
    #[must_use]
    pub fn max_concurrent_sessions(self) -> usize {
        self.max_concurrent_sessions.max(1)
    }

    #[must_use]
    pub fn timeout(self) -> Duration {
        Duration::from_millis(self.timeout_ms.max(1))
    }
}

#[derive(Clone, Debug)]
pub struct TranscodeRuntimeGuard {
    semaphore: Option<Arc<Semaphore>>,
    timeout: Duration,
}

impl TranscodeRuntimeGuard {
    #[must_use]
    pub fn new(limits: TranscodeRuntimeLimits) -> Self {
        Self {
            semaphore: Some(Arc::new(Semaphore::new(limits.max_concurrent_sessions()))),
            timeout: limits.timeout(),
        }
    }

    #[must_use]
    pub fn timeout_only(timeout_ms: u64) -> Self {
        Self {
            semaphore: None,
            timeout: Duration::from_millis(timeout_ms.max(1)),
        }
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn acquire(&self) -> Result<TranscodeRuntimePermit> {
        let permit = match &self.semaphore {
            Some(semaphore) => Some(semaphore.clone().acquire_owned().await.map_err(|err| {
                NakoError::Provider {
                    provider: "ffmpeg".to_owned(),
                    message: format!("transcode runtime guard closed: {err}"),
                }
            })?),
            None => None,
        };

        Ok(TranscodeRuntimePermit { permit })
    }
}

#[derive(Debug)]
pub struct TranscodeRuntimePermit {
    #[allow(dead_code)]
    permit: Option<OwnedSemaphorePermit>,
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        loop {
            if self.is_cancelled() {
                return;
            }

            time::sleep(Duration::from_millis(10)).await;
        }
    }
}
