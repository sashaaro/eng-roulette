use crate::domain::repository;
use crate::domain::repository::RoomRepository;
use crate::models::Room;
use diesel::pg::PgConnection;
use crate::schema::rooms::dsl::rooms;
use diesel::prelude::*;

pub struct PgRoomRepository<'a> {
    pub conn: &'a mut PgConnection
}

impl repository::RoomRepository for PgRoomRepository<'_> {
    fn all(&mut self) -> Vec<Room> {
        rooms
            // .filter(published.eq(true))
            .limit(5)
            .select(Room::as_select())
            .load(self.conn)
            .expect("Error loading posts");

        return Vec::new()
    }
}
