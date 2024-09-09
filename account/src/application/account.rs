use std::error::Error;
use chrono::{Utc, Duration};
use uuid::{Uuid};
use crate::domain::models::User;
use crate::domain::repository::{PremiumRepository, UserRepository};
use crate::domain::service::BillingService;

// TODO generate from hostname?
const NODE: &[u8; 6] = &[1, 2, 3, 4, 5, 6];
const PREMIUM_COST: i64 = 1000;
pub async fn buy_premium<'a, T: BillingService+ 'a + ?Sized, A: UserRepository+ 'a, B: PremiumRepository+ 'a>(
    billing: &Box< T >, // todo inject through ::new
    user_repo: &Box< A >,
    premium_repo: &Box< B >,
    user_id: i64
) -> Result<(), Box<dyn Error>> {
    let txID = Uuid::now_v6(NODE);
    let until = Utc::now() + Duration::hours(2);

    let user = user_repo.find_user(user_id).await?;

    if user.is_none() {
        ()
        //return Err(Box::new(Error::fmt("user not found")));
    }
    premium_repo.prepare_premium_until(txID, user_id, until).await?;
    //billing.prepare_expense(txID, user_id, PREMIUM_COST).await?;
    premium_repo.commit_premium_until(txID).await?;
    billing.commit_expense(txID).await;
    Ok(())
}

pub async fn create_user<A: UserRepository+ 'static>(
    user_repo: &Box< A >,
    name: String,
    password: String,
) -> Result<User, Box<dyn Error>> {
    user_repo.create_user(name, password).await
}