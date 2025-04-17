use crate::domain::models::User;
use crate::domain::repository;
use async_trait::async_trait;
use sqlx::Error::RowNotFound;
use sqlx::{Pool, Postgres, Row};

#[derive(Clone)]
pub struct PgUserRepository {
    pub pool: Pool<Postgres>,
}

impl PgUserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        PgUserRepository { pool }
    }
}

#[async_trait]
impl repository::UserRepository for PgUserRepository {
    async fn create_user(&self, username: String, password: String) -> anyhow::Result<User> {
        let result = sqlx::query(&*format!(
            "INSERT INTO users(username, password) VALUES ($1, $2) RETURNING id;"
        ))
        .bind(&username)
        .bind(&password)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: result.get("id"),
            is_active: true,
            username,
            password,
            premium_until: None,
        })
    }

    async fn find(&self, id: i64) -> anyhow::Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await;
        match row {
            Ok(row) => Ok(Some(row.into())),
            Err(RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await;
        match row {
            Ok(row) => Ok(Some(row.into())),
            Err(RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
