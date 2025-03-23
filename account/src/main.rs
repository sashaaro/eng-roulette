use actix_web::{App, HttpServer, web};
use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use crate::infra::service::InternalBillingService;

mod infra;
mod domain;
mod application;
mod test;

use std::env;
use std::fmt::{Display};
use std::net::{ToSocketAddrs};
use std::ops::Add;
use std::sync::Arc;
use rand::prelude::*;
use crate::application::account::Application;
use crate::infra::auth::AuthManager;
use sqlx::{Pool, Postgres};
use actix_web::web::{Data, ServiceConfig};
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = infra::db::pg().await;

    let port = env::var_os("HTTP_PORT")
        .map(|val| val.to_str()
            .expect("invalid port")
            .to_string().parse::<u16>()
            .expect("invalid port"))
        .unwrap_or(8080);

    println!("Start server on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new().configure(config_app(pool.clone())).wrap(cors)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

fn config_app(pool: Pool<Postgres>) -> Box<dyn Fn(&mut ServiceConfig)> {
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));

    let premium_repo = web::Data::new(PgPremiumRepository {
        // conn: connection,
        pool: pool.clone(),
    });

    let billing = web::Data::new(InternalBillingService {
        client: reqwest::ClientBuilder::new().build()
            .expect("fail to create request client")
    });

    Box::new(move |cfg: &mut ServiceConfig| {
        let auth_manager = web::Data::new(AuthManager::new("secret".to_string()));
        let ua = Arc::clone(&user_repo);
        let pa = Arc::clone(&premium_repo);
        let ba = Arc::clone(&billing);

        let app = web::Data::new(Application::new(
            ua, pa, ba,
        ));

        cfg.app_data(auth_manager)
            .app_data(app)
            .service(infra::routes::get_account)
            .service(infra::routes::buypremium)
            .service(infra::routes::register)
            .service(infra::routes::login)
            .service(infra::routes::me);
    })
}
