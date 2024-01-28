use std::error::Error;
use crate::domain::repository::{Tx2pcID, TxRepository};


pub async fn income(tx_repo: Box<dyn TxRepository + Send>, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
    return tx_repo.income(user_id, amount).await
}

pub async fn prepare_expense(tx_repo: Box<dyn TxRepository + Send>, tx_id: Tx2pcID, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
    return tx_repo.prepare_expense(tx_id, user_id, amount).await
}

pub async fn commit_expense(tx_repo: Box<dyn TxRepository + Send>, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
    return tx_repo.commit_expense(tx_id).await
}