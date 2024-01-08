use std::error::Error;
use std::fmt;
use std::time::SystemTime;
use actix_web::error::{DispatchError, ErrorUnauthorized};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::repository;
use sqlx::{Pool, Postgres, Row};
use sqlx::Error::RowNotFound;
use crate::domain::models::{User};
use crate::domain::repository::Tx2pcID;

#[derive(Clone)]
pub struct PgUserRepository {
    // pub conn: &'a mut PgConnection,
    pub pool: Pool<Postgres>,
}

impl PgUserRepository {
    pub async fn sum(&self) -> i64 {
        let row: (i64, ) = sqlx::query_as("SELECT $1 + 100").bind(150_i64).fetch_one(&self.pool).await.unwrap();
        return row.0
    }
}

#[derive(Debug)]
struct UserNotFoundError {}
impl fmt::Display for UserNotFoundError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user not found")
    }
}


impl Error for UserNotFoundError{}

#[async_trait]
impl repository::UserRepository for PgUserRepository {
    async fn find_user(&self, id: i64) -> Result<User, Box<dyn Error>> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE id = $1",
        ).bind(id).fetch_one(&self.pool).await?;

        Ok(User {
            id: row.get("id"),
            name: row.get("name"),
            is_active: row.get("is_active"),
            premium_until: row.get("premium_until")
        })
    }

    async fn prepare_premium_until(&self, tx_id: Tx2pcID, user_id: i64, util: chrono::DateTime<Utc>) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        // TODO try lock

        let result = sqlx::query(
            "UPDATE user SET premium_util = $1 WHERE id = $2",
        )
            .bind(util)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            _ = tx.rollback().await?;
            return Err(Box::new(UserNotFoundError{}))
        }

        sqlx::query(
            &*format!("PREPARE TRANSACTION '{}';", tx_id),
        )
            .execute(&mut *tx)
            .await?;

        Ok(())
    }

    async fn commit_premium_until(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("COMMIT PREPARED '{}';", tx_id),
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn rollback_premium_until(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("ROLLBACK PREPARED '{}';", tx_id),
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}