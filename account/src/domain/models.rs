use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub is_active: bool, // or banned
    pub premium_until: Option<NaiveDateTime>
}

