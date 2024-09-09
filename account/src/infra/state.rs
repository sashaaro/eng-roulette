use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use ahash::{RandomState};
use std::any::Any;
use std::any::TypeId;
use crate::domain::service::BillingService;
use std::collections::HashMap;
use crate::domain::repository::PremiumRepository;

pub struct AppState {
    //pub room_repo: dyn RoomRepository,
    pub user_repo: Box<PgUserRepository>,
    pub premium_repo: Box<PgPremiumRepository>,
    pub billing: Box<dyn BillingService>, // dyn BillingService
    map: HashMap<TypeId, Box<dyn Any>, RandomState>,
}

impl AppState {
    pub fn new(
        user_repo: Box<PgUserRepository>,
        premium_repo: Box<PgPremiumRepository>,
        billing: Box<dyn BillingService>,
    ) -> AppState {
        AppState {
            user_repo,
            premium_repo,
            billing,
            map: HashMap::default(),
        }
    }
}

#[async_trait::async_trait]
pub trait CreatedFromState: Clone {
    async fn create(state: &mut AppState) -> Self;
}