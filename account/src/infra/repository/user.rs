use crate::domain::model::User;
use crate::domain::repository;
use async_trait::async_trait;
use sqlx::Error::RowNotFound;
use sqlx::{Pool, Postgres};

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
    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<User> {
        let result =
            sqlx::query("INSERT INTO users(username, password) VALUES ($1, $2) RETURNING *")
                .bind(username)
                .bind(password)
                .fetch_one(&self.pool)
                .await?;

        Ok(result.into())
    }

    async fn find(&self, id: i64) -> anyhow::Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.into()))
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
