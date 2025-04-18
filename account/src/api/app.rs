use crate::api::routes::{google_auth, google_auth_callback, login, me, register};
use crate::infra::auth::jwt::JwtManager;
use crate::infra::repository::user::PgUserRepository;
use crate::service::account::AccountService;
use actix_web::web;
use actix_web::web::ServiceConfig;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub fn create_app(
    pool: Pool<Postgres>,
    secret_key: &'static str,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    let user_repo: Arc<PgUserRepository> = Arc::new(PgUserRepository::new(pool.clone()));

    Box::new(move |cfg: &mut ServiceConfig| {
        let jwt_manager = web::Data::new(JwtManager::new(secret_key.to_string()));

        let user_repo = Arc::clone(&user_repo);
        let account_service = web::Data::new(AccountService::new(Arc::clone(&user_repo) as Arc<_>));

        cfg.app_data(jwt_manager)
            .app_data(account_service)
            .service(register)
            .service(login)
            .service(google_auth)
            .service(google_auth_callback)
            .service(me);
    })
}
