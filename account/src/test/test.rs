#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use crate::infra;
    use crate::infra::auth::AuthManager;
    use crate::infra::repository::PgUserRepository;
    use crate::infra::service::InternalBillingService;
    use crate::infra::state::AppState;
    use crate::infra::db;

    #[actix_web::test]
    async fn test_index_get() {
        let pool = db::pg().await;

        let user_repo = PgUserRepository::new(pool.clone());

        let billing = Box::new(InternalBillingService{
            client: reqwest::ClientBuilder::new().build()
                .expect("fail to create request client")
        });

        let auth_manager = AuthManager::new("secret".to_string());

        let app_state = AppState::new(
            Box::new(user_repo),
            billing,
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .app_data(web::Data::new(auth_manager))
                .service(infra::routes::get_account)
                .service(infra::routes::buypremium)
                .service(infra::routes::register)
                .service(infra::routes::me)
        )
            .await;

        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 404);
    }
}