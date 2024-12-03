use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use crate::application::account::Application;
    use crate::infra;
    use crate::infra::auth::AuthManager;
    use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
    use crate::infra::service::InternalBillingService;
    use crate::infra::db;

    #[actix_web::test]
    async fn test_index_get() {
        let pool = db::pg().await;

        let user_repo = Arc::new(PgUserRepository::new(pool.clone()));

        let premium_repo = web::Data::new(PgPremiumRepository{
            // conn: connection,
            pool: pool.clone(),
        });

        let billing = web::Data::new(InternalBillingService{
            client: reqwest::ClientBuilder::new().build()
                .expect("fail to create request client")
        });

        let auth_manager = web::Data::new(AuthManager::new("secret".to_string()));
        let ua = Arc::clone(&user_repo);
        let pa = Arc::clone(&premium_repo);
        let ba = Arc::clone(&billing);

        let app = web::Data::new(Application::new(
            ua,pa,ba,
        ));

        let app = test::init_service(
            App::new()
                .app_data(auth_manager)
                .app_data(app)
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