use crate::domain::repository::UserRepository;
use crate::infra::repository::{PgUserRepository};
use ahash::{AHashMap, RandomState};
use std::any::Any;
use std::any::TypeId;
use crate::domain::service::BillingService;
use crate::infra::service::InternalBillingService;
use std::collections::HashMap;

pub struct AppState {
    //pub room_repo: dyn RoomRepository,
    pub user_repo: Box<PgUserRepository>,
    pub billing: Box<dyn BillingService>, // dyn BillingService
    map: HashMap<TypeId, Box<dyn Any>, RandomState>,
}

impl AppState {
    pub fn new(user_repo: Box<PgUserRepository>, billing: Box<dyn BillingService>) -> AppState {
        AppState {
            user_repo,
            billing,
            map: HashMap::default(),
        }
    }
}

#[async_trait::async_trait]
pub trait CreatedFromState: Clone {
    async fn create(state: &mut AppState) -> Self;
}