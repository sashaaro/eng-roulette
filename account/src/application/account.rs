use std::error::Error;
use std::ops::Add;
use std::time::SystemTime;
use time::Duration;
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
) -> Result<(), dyn Error> {
    let txID = Uuid::now_v6(NODE);
    let now = SystemTime::now();

    let until = chrono::DateTime::add();

    user_repo.prepare_premium_until(txID, user_id, until);
    billing.prepare_expense(txID, user_id, PREMIUM_COST).await
}