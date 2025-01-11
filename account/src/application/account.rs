use std::error::Error;
use std::sync::Arc;
use actix_web::web::Data;
use chrono::{Utc, Duration};
use uuid::{Uuid};
use crate::domain::models::User;
use crate::domain::repository::{PremiumRepository, UserRepository};
use crate::domain::service::BillingService;
use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use crate::infra::service::InternalBillingService;

// TODO generate from hostname?
const NODE: &[u8; 6] = &[1, 2, 3, 4, 5, 6];
const PREMIUM_COST: i64 = 1000;

pub struct Application {
    user_repo: Arc<dyn UserRepository>,
    premium_repo: Arc<dyn PremiumRepository>,
    billing: Arc<InternalBillingService>
}

impl Application {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        premium_repo: Arc<dyn PremiumRepository>,
        billing: Arc<InternalBillingService>,
    ) -> Application {
        Application {
            user_repo,
            premium_repo,
            billing,
        }
    }

    pub async fn buy_premium(&self, user_id: i64) -> Result<(), Box<dyn Error>> {
        let tx_id = Uuid::now_v6(NODE);
        let until = Utc::now() + Duration::hours(2);

        let user = self.user_repo.find_user(user_id).await?;

        if user.is_none() {
            ()
            //return Err(Box::new(Error::fmt("user not found")));
        }
        self.premium_repo.prepare_premium_until(tx_id, user_id, until).await?;
        //billing.prepare_expense(txID, user_id, PREMIUM_COST).await?;
        self.premium_repo.commit_premium_until(tx_id).await?;
        self.billing.commit_expense(tx_id).await;
        Ok(())
    }

    pub async fn create_user(&self, name: String, password: String) -> Result<User, Box<dyn Error>> {
        self.user_repo.create_user(name, password).await
    }

    pub async fn login(&self, name: String, password: String) -> Result<User, Box<dyn Error>> {
        let user = self.user_repo.find_username(&name).await?;
        if user.is_some() {
            let user = user.unwrap();
            if user.password == password {
                return Ok(user);
            }
        }

        Err(Box::new(sqlx::Error::RowNotFound))
    }
}