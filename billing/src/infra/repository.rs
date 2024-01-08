use std::error::Error;
use async_trait::async_trait;
use rocket::futures::future::err;
use crate::domain::repository;
use sqlx::{Executor, Pool, Postgres, Row};
use crate::domain::models::{TxType};
use crate::domain::repository::Tx2pcID;

#[derive(Clone)]
pub struct PgTxRepository {
    // pub conn: &'a mut PgConnection,
    pub pool: Pool<Postgres>,
}

impl PgTxRepository {
    pub async fn sum(&self) -> i64 {
        let row: (i64, ) = sqlx::query_as("SELECT $1 + 100").bind(150_i64).fetch_one(&self.pool).await.unwrap();
        return row.0
    }
}
#[async_trait]
impl repository::TxRepository for PgTxRepository {
    async fn balance(&self, user_id: i32) -> Result<i32, Box<dyn Error>> {
        let row = sqlx::query(
            "SELECT sum(amount) FROM transaction_history WHERE user_id = $1",
        )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await;
        if row.is_ok() {
            return Ok(row.ok().expect("panic!]").get("sum"))
        }

        Err(Box::new(row.err().unwrap()))
    }

    async fn income(&self, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "INSERT INTO transaction_history(user_id, tx_type, amount) VALUES ($1, $2, $3)",
        )
            .bind(user_id)
            .bind(1)//TxType::Income)
            .bind(amount)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i32, amount: i32) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO transaction_history(user_id, tx_type, amount) VALUES ($1, $2, $3)",
        )
            .bind(user_id)
            .bind(0)//TxType::Expense)
            .bind(amount)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "PREPARE TRANSACTION $1",
        )
            .bind(tx_id)
            .execute(&mut *tx)
            .await?;

        Ok(())
    }

    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "COMMIT PREPARED $1",
        )
            .bind(tx_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn rollback_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "ROLLBACK PREPARED $1",
        )
            .bind(tx_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}