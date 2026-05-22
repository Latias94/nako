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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxRuntimeLimits {
    pub max_concurrent_sessions: usize,
    pub timeout_ms: u64,
}

impl Default for RemuxRuntimeLimits {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 1,
            timeout_ms: 30 * 60 * 1_000,
        }
    }
}

impl RemuxRuntimeLimits {
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
pub struct RemuxRuntimeGuard {
    semaphore: Arc<Semaphore>,
    timeout: Duration,
}

impl RemuxRuntimeGuard {
    #[must_use]
    pub fn new(limits: RemuxRuntimeLimits) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(limits.max_concurrent_sessions())),
            timeout: limits.timeout(),
        }
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn acquire(&self) -> Result<RemuxRuntimePermit> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| NakoError::Provider {
                provider: "ffmpeg".to_owned(),
                message: format!("remux runtime guard closed: {err}"),
            })?;

        Ok(RemuxRuntimePermit { permit })
    }
}

#[derive(Debug)]
pub struct RemuxRuntimePermit {
    #[allow(dead_code)]
    permit: OwnedSemaphorePermit,
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
