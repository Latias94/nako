use async_trait::async_trait;

use crate::Result;

#[async_trait]
pub trait DatabaseLifecycle: Send + Sync {
    async fn migrate(&self) -> Result<()>;
}
