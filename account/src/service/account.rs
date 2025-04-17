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
        let user = self.user_repo.find_by_username(name.as_str()).await;
        match user {
            Err(err) => Err(err),
            Ok(Some(user)) if user.is_active => {
                if password == user.password {
                    Ok(user)
                } else {
                    Err(WrongPassword.into())
                }
            }
            Ok(None) => Err(AppError::NotFound.into()),
            _ => unreachable!(),
        }
    }

    pub async fn me(&self, id: i64) -> Result<User> {
        let user = self.user_repo.find(id).await;
        match user {
            Err(err) => Err(err),
            Ok(Some(user)) if user.is_active => Ok(user),
            Ok(None) => Err(AppError::NotFound.into()),
            _ => unreachable!(),
        }
    }
}
