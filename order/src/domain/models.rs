pub struct User {
    pub id: i32,
    pub name: String,
    pub available_rooms: i32
}

pub struct Room {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub user: User
}

