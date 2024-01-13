use std::error::Error;
use chrono::{Utc, Duration};
use uuid::{Uuid};
use crate::domain::repository::UserRepository;
use crate::domain::service::BillingService;

// TODO generate from hostname?
const NODE: &[u8; 6] = &[1, 2, 3, 4, 5, 6];
const PREMIUM_COST: i32 = 1000;
pub async fn buy_premium(
    billing: Box<dyn BillingService>, // todo inject through ::new
    user_repo: Box<dyn UserRepository>,
    user_id: i32
) -> Result<(), Box<dyn Error>> {
    let txID = Uuid::now_v6(NODE);
    let until = Utc::now() + Duration::hours(2);

    user_repo.prepare_premium_until(txID, user_id, until);
    billing.prepare_expense(txID, user_id, PREMIUM_COST).await
}