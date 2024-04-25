use std::error::Error;
use async_trait::async_trait;
use rocket::futures::future::err;
use rocket::futures::FutureExt;
use crate::domain::repository;
use sqlx::{Executor, PgExecutor, PgPool, Pool, Postgres, Row};
use sqlx::pool::MaybePoolConnection::PoolConnection;
use crate::domain::models::{TxType};
use crate::domain::repository::Tx2pcID;

//#[derive(Clone)]
pub struct PgTxRepository {
    // pub conn: &'a mut PgConnection,
    pub pool: PgPool,
}

// unsafe impl Send for PgTxRepository {}
// unsafe impl Sync for PgTxRepository {}

impl PgTxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn _balance(&self, user_id: i64, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<i64, Box<dyn Error>> {
        let row = sqlx::query(
            "SELECT
    (SELECT coalesce(sum(amount), 0) as sum FROM transaction_history WHERE user_id = $1 and tx_type = 1) -
    (SELECT coalesce(sum(amount), 0) as sum FROM transaction_history WHERE user_id = $1 and tx_type = 0) AS balance",
        )
            .bind(user_id)
            .fetch_one(&mut **tx)
            .await?;

        let r: i64 = row.try_get("balance")?;
        Ok(r)
    }
}

#[async_trait]
impl repository::TxRepository for PgTxRepository {
    async fn balance(&self, user_id: i64) -> Result<i64, Box<dyn Error>> {
        let mut tx = self.pool.begin().await?; // todo stop explicit open transaction, pass Executor Trait?
        return self._balance(user_id, &mut tx).await;
    }

    async fn income(&self, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
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

    async fn prepare_expense(&self, tx_id: Tx2pcID, user_id: i64, amount: i64) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        //tx.execute();
        // TODO try lock

        sqlx::query(
            "INSERT INTO transaction_history(user_id, tx_type, amount) VALUES ($1, $2, $3)",
        )
            .bind(user_id)
            .bind(0)//TxType::Expense)
            .bind(amount)
            .execute(&mut *tx)
            .await?;

        {
            let balance = self._balance(user_id, &mut tx).await?;
            if balance < 0 {
                return Ok(tx.rollback().await?) // throw special negative balance error
            }
        }

        sqlx::query(
            &*format!("PREPARE TRANSACTION 'bill_{}';", tx_id),
        )
            .execute(&mut *tx)
            .await?;

        Ok(())
    }

    async fn commit_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("COMMIT PREPARED 'bill_{}';", tx_id),
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn rollback_expense(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("ROLLBACK PREPARED 'bill_{}';", tx_id),
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}