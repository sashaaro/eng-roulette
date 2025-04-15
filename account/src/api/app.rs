use crate::api::routes::{login, me, register};
use crate::infra::auth::AuthManager;
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
        let auth_manager = web::Data::new(AuthManager::new(secret_key.to_string()));

        let user_repo = Arc::clone(&user_repo);
        let app = web::Data::new(AccountService::new(Arc::clone(&user_repo) as Arc<_>));

        cfg.app_data(auth_manager)
            .app_data(app)
            .service(register)
            .service(login)
            .service(me);
    })
}
