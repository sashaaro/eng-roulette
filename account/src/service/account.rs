use crate::domain::model::User;
use crate::domain::repository::UserRepository;
use crate::service::account::AppError::WrongPassword;
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::sync::Arc;
use thiserror;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("wrong password")]
    WrongPassword,
}

fn generate_password() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

pub struct AccountService {
    user_repo: Arc<dyn UserRepository>,
}

impl AccountService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> AccountService {
        AccountService { user_repo }
    }

    pub async fn create_user(&self, name: &str, password: &str) -> Result<User> {
        self.user_repo.create_user(name, password).await
    }

    pub async fn create_or_login(&self, email: &str) -> Result<User> {
        let user = self.user_repo.find_by_username(email).await?;

        match user {
            Some(user) => Ok(user),
            None => self.create_user(email, &generate_password()).await,
        }
    }

    pub async fn login(&self, name: String, password: String) -> Result<User> {
        let user = self.user_repo.find_by_username(name.as_str()).await?;
        match user {
            Some(user) => {
                if user.is_active && password == user.password {
                    Ok(user)
                } else {
                    Err(WrongPassword.into())
                }
            }
            None => Err(AppError::NotFound.into()),
        }
    }

    pub async fn me(&self, id: i64) -> Result<User> {
        self.user_repo
            .find(id)
            .await?
            .filter(|u| u.is_active)
            .ok_or_else(|| AppError::NotFound.into())
    }
}
