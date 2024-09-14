use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use ahash::{RandomState};
use std::any::Any;
use std::any::TypeId;
use crate::domain::service::BillingService;
use std::collections::HashMap;
use crate::domain::repository::PremiumRepository;

pub struct AppState<'a> {
    //pub room_repo: dyn RoomRepository,
    pub user_repo: Box<PgUserRepository<'a>>,
    pub premium_repo: Box<PgPremiumRepository<'a>>,
    pub billing: Box<dyn BillingService>, // dyn BillingService
    map: HashMap<TypeId, Box<dyn Any>, RandomState>,
}

impl<'a> AppState<'a> {
    pub fn new(
        user_repo: Box<PgUserRepository<'a>>,
        premium_repo: Box<PgPremiumRepository<'a>>,
        billing: Box<dyn BillingService>,
    ) -> AppState<'a> {
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