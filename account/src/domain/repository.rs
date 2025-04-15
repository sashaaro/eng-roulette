use crate::domain::models::*;
use anyhow;
use async_trait::async_trait;
use uuid::Uuid;

pub type Tx2pcID = Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, name: String, password: String) -> anyhow::Result<User>;
    async fn find(&self, id: i64) -> anyhow::Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
}
