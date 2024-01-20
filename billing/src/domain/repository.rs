use std::error::Error;
use async_trait::async_trait;
use uuid;
pub type Tx2pcID = String;

#[async_trait]
pub trait TxRepository {
    async fn balance(&self, user_id: i32) -> Result<i32, Box<dyn Error>>;
    async fn income(&self, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>>;

    // support 2pc commit
    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>>;
    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
    async fn rollback_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
}
// fn a (d: impl TxRepository) {
//
// }
//
// fn aa (d: dyn TxRepository) {
//
// }
