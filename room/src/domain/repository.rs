use async_trait::async_trait;
use crate::domain::models::*;
#[async_trait]
pub trait RoomRepository {
    async fn all(&self) -> Vec<Room>;
}
