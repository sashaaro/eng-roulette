use async_trait::async_trait;
use crate::domain::repository;
use sqlx::{Pool, Postgres, Row};
use sqlx::Error::RowNotFound;
use crate::domain::models::{Room, User};

#[derive(Clone)]
pub struct PgRoomRepository {
    // pub conn: &'a mut PgConnection,
    pub pool: Pool<Postgres>,
}

impl PgRoomRepository {
    pub async fn sum(&self) -> i64 {
        let row: (i64, ) = sqlx::query_as("SELECT $1 + 100").bind(150_i64).fetch_one(&self.pool).await.unwrap();
        return row.0
    }
}
#[async_trait]
impl repository::RoomRepository for PgRoomRepository {
    async fn all(&self) -> Vec<Room> {
        // let list = sqlx::query_as!(Room, "SELECT * FROM rooms").fetch_all(self.pool).await.unwrap();
        // return list

        let rows = sqlx::query(
            "SELECT * FROM rooms INNER JOIN users u ON u.id = rooms.id",
        )
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();
        if rows.is_empty() {
            return Vec::new();
        }
        let mut list: Vec<Room> = Vec::with_capacity(rows.len());
        for row in rows {
            list.push(Room {
                //id: row.get("rooms.id"),
                id: 0,
                title: row.get("title"),
                body: row.get("body"),
                published: row.get("published"),
                user: User {
                    //id: row.get("u.id"),
                    id: 0,
                    name: row.get("name"),
                    available_rooms: row.get("available_rooms"),
                },
            });
        }
        return list
    }
}