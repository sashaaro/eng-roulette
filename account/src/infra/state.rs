use crate::domain::repository::UserRepository;
use crate::infra::repository::{PgUserRepository};
use ahash::AHashMap;
use std::any::Any;
use std::any::TypeId;

pub struct AppState {
    //pub room_repo: dyn RoomRepository,
    pub room_repo: PgUserRepository,
    map: AHashMap<TypeId, Box<dyn Any>>,
}

impl AppState {
    pub fn new(room_repo: PgUserRepository) -> AppState {
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