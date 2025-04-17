#[cfg(test)]
mod tests {
    use crate::domain::repository::MockUserRepository;
    use crate::service::account::AccountService;
    use anyhow::{bail, Result};
    use mockall::predicate::eq;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_service_login() -> Result<()> {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_username()
            .with(eq("alex"))
            .return_once(|_| {
                Box::pin(async {
                    Ok(Some(crate::domain::models::User {
                        id: 1,
                        username: "alex".to_string(),
                        password: "my_password".to_string(),
                        is_active: true,
                        premium_until: None,
                    }))
                })
            })
            .times(1);

        let account_service = AccountService::new(Arc::new(user_repo));
        let logged_user = account_service
            .login("alex".to_string(), "my_password".to_string())
            .await;

        assert_eq!(logged_user.is_ok(), true);
        Ok(())
    }
}
