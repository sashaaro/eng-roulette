use crate::domain::model::User;
use crate::domain::repository::UserRepository;
use crate::service::account::AppError::WrongPassword;
use anyhow::Result;
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

pub struct AccountService {
    user_repo: Arc<dyn UserRepository>,
}

impl AccountService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> AccountService {
        AccountService { user_repo }
    }

    pub async fn create_user(&self, name: String, password: String) -> Result<User> {
        self.user_repo.create_user(name, password).await
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
