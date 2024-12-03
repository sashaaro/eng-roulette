use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use crate::application::account::Application;
    use crate::{config_app, infra};
    use crate::infra::auth::AuthManager;
    use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
    use crate::infra::service::InternalBillingService;
    use crate::infra::db;

    #[actix_web::test]
    async fn test_index_get() {
        let pool = db::pg().await;

        let app = test::init_service(
            App::new().configure(config_app(pool.clone()))
        )
            .await;

        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 404);
    }
}