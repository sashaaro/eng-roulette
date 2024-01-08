use std::time::SystemTime;

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub is_active: bool, // or banned
    pub premium_until: Option<chrono::DateTime<chrono::Utc>>
}

