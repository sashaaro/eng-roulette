use std::error::Error;
use async_trait::async_trait;
use crate::domain::repository::Tx2pcID;


#[async_trait]
pub trait BillingService {
    async fn income(&self, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>>;
    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>>;
    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
}