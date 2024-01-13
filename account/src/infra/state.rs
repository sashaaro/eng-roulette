use crate::domain::repository::UserRepository;
use crate::infra::repository::{PgUserRepository};
use ahash::AHashMap;
use std::any::Any;
use std::any::TypeId;
use crate::domain::service::BillingService;

pub struct AppState {
    //pub room_repo: dyn RoomRepository,
    pub user_repo: Box<PgUserRepository>,
    pub billing: Box<dyn BillingService>,
    map: AHashMap<TypeId, Box<dyn Any>>,
}

impl AppState {
    pub fn new(user_repo: Box<PgUserRepository>, billing: Box<dyn BillingService>) -> AppState {
        AppState {
            user_repo,
            billing,
            map: AHashMap::new(),
        }
    }
}

#[async_trait::async_trait]
pub trait CreatedFromState: Clone {
    async fn create(state: &mut AppState) -> Self;
}