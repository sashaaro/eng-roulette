mod api;
mod domain;
mod infra;
mod service;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use api::app;
use env_logger::Builder;
use log::LevelFilter;
use std::env;
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
            .configure(app::create_app(pool.clone(), secret_key))
            .wrap(cors)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
