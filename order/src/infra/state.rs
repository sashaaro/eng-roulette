use crate::domain::repository::RoomRepository;
use crate::infra::repository::PgRoomRepository;
use ahash::AHashMap;
use std::any::Any;
use std::any::TypeId;

pub struct AppState {
    //pub room_repo: dyn RoomRepository,
    pub room_repo: PgRoomRepository,
    map: AHashMap<TypeId, Box<dyn Any>>,
}

impl AppState {
    pub fn new(room_repo: PgRoomRepository) -> AppState {
        AppState {
            room_repo,
            map: AHashMap::new(),
        }
    }
}

#[async_trait::async_trait]
pub trait CreatedFromState: Clone {
    async fn create(state: &mut AppState) -> Self;
}