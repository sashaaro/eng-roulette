use async_trait::async_trait;
use crate::domain::repository::Tx2pcID;


#[async_trait]
pub trait BillingService: Send + Sync {
    // async fn income(&self, user_id: i64, amount: i64) -> anyhow::Result<()>;
    // async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i64, amount: i64) -> anyhow::Result<()>;
    async fn commit_expense(&self, tx_id: Tx2pcID) -> anyhow::Result<()>;
}