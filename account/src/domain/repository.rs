use std::error::Error;
use std::time::SystemTime;
use async_trait::async_trait;
use chrono::Utc;
use time::{Date, Time};
use uuid::Uuid;
use crate::domain::models::*;

pub type Tx2pcID = Uuid;

#[async_trait]
pub trait UserRepository {
    async fn find_user(&self, id: i64) -> Result<Option<User>, Box<dyn Error>>;
    async fn prepare_premium_until(&self, tx2pc_id: Tx2pcID, user_id: i32, util: chrono::DateTime<Utc>) -> Result<(), Box<dyn Error>>;
    async fn commit_premium_until(&self, tx2pc_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
    async fn rollback_premium_until(&self, tx2pc_id: Tx2pcID) -> Result<(), Box<dyn Error>>;
}
