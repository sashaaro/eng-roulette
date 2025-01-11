use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_active: bool, // or banned
    pub premium_until: Option<NaiveDateTime>
}

