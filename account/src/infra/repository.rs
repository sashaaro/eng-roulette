use std::any::Any;
use std::error::Error;
use std::fmt;
use std::time::SystemTime;
use actix_web::error::{DispatchError, ErrorUnauthorized};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::domain::repository;
use sqlx::{Executor, Pool, Postgres, Row};
use sqlx::Error::RowNotFound;
use sqlx::postgres::PgRow;
use crate::domain::models::{User};
use crate::domain::repository::Tx2pcID;

#[derive(Clone)]
pub struct PgUserRepository<'a> {
    // pub conn: &'a mut PgConnection,
    pub pool: &'a Pool<Postgres>,
}

impl<'a> PgUserRepository<'a> {
    pub async fn sum(&self) -> i64 {
        let row: (i64, ) = sqlx::query_as("SELECT $1 + 100").bind(150_i64).fetch_one(self.pool).await.unwrap();
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
impl <'a>repository::UserRepository for PgUserRepository<'a> {
    async fn create_user(&self, username: String, password: String) -> Result<User, Box<dyn Error>> {
        let result = sqlx::query(
            &*format!("INSERT INTO \"user\"(username, password) VALUES ($1, $2) RETURNING id;"),
        )
            .bind(&username)
            .bind(&password)
            .fetch_one(self.pool)
            .await?;

        Ok(User{
            id: result.get("id"),
            is_active: true,
            username,
            password,
            premium_until: None,
        })
    }

    async fn find_user(&self, id: i64) -> Result<Option<User>, Box<dyn Error>> {
        let row = sqlx::query(
            "SELECT * FROM \"user\" WHERE id = $1",
        ).bind(id).fetch_one(self.pool).await;
        match row {
            Ok(row) => {
                let u = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    password: "".to_string(),
                    is_active: row.get("is_active"),
                    premium_until: row.get("premium_until"),
                };
                Ok(Some(u))
            },
            Err(err) => {
                match err {
                    RowNotFound => Ok(None),
                    _ => Err(Box::try_from(err).unwrap())
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PgPremiumRepository<'a> {
    // pub conn: &'a mut PgConnection,
    pub pool: &'a Pool<Postgres>,
}

#[async_trait]
impl<'a> repository::PremiumRepository for PgPremiumRepository<'a> {
    async fn prepare_premium_until(&self, tx_id: Tx2pcID, user_id: i64, until: chrono::DateTime<Utc>) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        // TODO try lock

        let result = sqlx::query!(
            "UPDATE \"user\" SET premium_until = $1 WHERE id = $2",
        until.naive_local(), i32::try_from(user_id).expect(""))
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            _ = tx.rollback().await?;
            return Err(Box::new(UserNotFoundError{}))
        }

        sqlx::query(
            &*format!("PREPARE TRANSACTION 'acc_{}';", tx_id),
        )
            .execute(&mut *tx)
            .await?;

        Ok(())
    }

    async fn commit_premium_until(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("COMMIT PREPARED 'acc_{}';", tx_id),
        )
            .execute(self.pool)
            .await?;

        Ok(())
    }

    async fn rollback_premium_until(&self, tx_id: Tx2pcID) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            &*format!("ROLLBACK PREPARED 'acc_{}';", tx_id),
        )
            .execute(self.pool)
            .await?;

        Ok(())
    }
}
