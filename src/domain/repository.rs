use crate::models::*;

pub trait RoomRepository {
    fn all(&mut self) -> Vec<Room>;
}
