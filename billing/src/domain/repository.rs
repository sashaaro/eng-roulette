use std::error::Error;
use async_trait::async_trait;
use uuid;
pub type Tx2pcID = String;

#[async_trait]
pub trait TxRepository: Send + Sync {
    async fn balance(&self, user_id: i64) -> Result<i64, Box<dyn Error>>;
    async fn income(&self, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>>;

    // support 2pc commit
    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>>;
    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
    async fn rollback_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
}