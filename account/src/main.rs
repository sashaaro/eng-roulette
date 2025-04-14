mod application;
mod domain;
mod infra;
mod test;

use crate::application::account::Application;
use crate::infra::auth::AuthManager;
use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use crate::infra::service::InternalBillingService;
use actix_cors::Cors;
use actix_web::web::ServiceConfig;
use actix_web::{web, App, HttpServer};
use env_logger::Builder;
use log::LevelFilter;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .filter(None, LevelFilter::Info)
        .filter(Some("sqlx::query"), LevelFilter::Warn)
        .filter(Some("actix_server::worker"), LevelFilter::Warn)
        .filter(Some("actix_server::builder"), LevelFilter::Warn)
        .filter(Some("audit"), LevelFilter::Warn)
        .init();

    let pool = infra::db::pg().await;

    let port = env::var_os("HTTP_PORT")
        .map(|val| {
            val.to_str()
                .expect("invalid port")
                .to_string()
                .parse::<u16>()
                .expect("invalid port")
        })
        .unwrap_or(8081);

    let secret_key = {
        let key = env::var_os("SECRET_KEY")
            .expect("Missing SECRET_KEY env variable")
            .to_str()
            .expect("SECRET_KEY contains invalid Unicode")
            .to_string();

        Box::leak(key.into_boxed_str())
    } as &'static str; // allow SECRET_KEY life endless

    HttpServer::new(move || {
        let cors = Cors::default()
            .send_wildcard()
            .allowed_origin_fn(|_, _| true) // TODO only for dev
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::AUTHORIZATION,
            ])
            .max_age(3600);

        App::new()
            .configure(config_app(pool.clone(), secret_key))
            .wrap(cors)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

fn config_app(pool: Pool<Postgres>, secret_key: &'static str) -> Box<dyn Fn(&mut ServiceConfig)> {
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));

    let premium_repo = web::Data::new(PgPremiumRepository {
        // conn: connection,
        pool: pool.clone(),
    });

    let billing = web::Data::new(InternalBillingService {
        client: reqwest::ClientBuilder::new()
            .build()
            .expect("fail to create request client"),
    });

    Box::new(move |cfg: &mut ServiceConfig| {
        let auth_manager = web::Data::new(AuthManager::new(secret_key.to_string()));
        let ua = Arc::clone(&user_repo);
        let pa = Arc::clone(&premium_repo);
        let ba = Arc::clone(&billing);

        let app = web::Data::new(Application::new(ua, pa, ba));

        cfg.app_data(auth_manager)
            .app_data(app)
            .service(infra::routes::buypremium)
            .service(infra::routes::register)
            .service(infra::routes::login)
            .service(infra::routes::me);
    })
}
