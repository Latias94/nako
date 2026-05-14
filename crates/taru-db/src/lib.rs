use async_trait::async_trait;
use taru_core::Result;

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn migrate(&self) -> Result<()>;
}
