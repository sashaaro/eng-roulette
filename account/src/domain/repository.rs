use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::models::*;
use anyhow;

pub type Tx2pcID = Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, name: String, password: String) -> anyhow::Result<User>;
    async fn find(&self, id: i64) -> anyhow::Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
}

#[async_trait]
pub trait PremiumRepository {
    async fn prepare_premium_until(&self, tx2pc_id: Tx2pcID, user_id: i64, util: chrono::DateTime<Utc>) -> anyhow::Result<()>;
    async fn commit_premium_until(&self, tx2pc_id: Tx2pcID) -> anyhow::Result<()>;
    // async fn rollback_premium_until(&self, tx2pc_id: Tx2pcID) -> anyhow::Result<()>;
}
