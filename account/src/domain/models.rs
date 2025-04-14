use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_active: bool, // or banned
    pub premium_until: Option<NaiveDateTime>,
}

impl From<PgRow> for User {
    fn from(row: PgRow) -> Self {
        let id: i32 = row.get(0);
        User {
            id: id.into(),
            username: row.get("username"),
            password: row.get("password"),
            is_active: row.get("is_active"),
            premium_until: row.get("premium_until"),
        }
    }
}
