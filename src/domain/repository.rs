use crate::models::*;

pub trait RoomRepository {
    async fn all(&mut self) -> Vec<Room>;
}
