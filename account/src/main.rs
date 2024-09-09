use actix_web::{App, HttpServer, web};
use crate::infra::repository::{PgPremiumRepository, PgUserRepository};
use crate::infra::service::InternalBillingService;

mod infra;
mod domain;
mod application;
use std::env;
use std::fmt::{Display};
use std::net::{ToSocketAddrs};
use std::ops::Add;

use crate::infra::state::AppState;
use rand::prelude::*;
use crate::infra::auth::AuthManager;

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
        let user_repo = PgUserRepository{
            // conn: connection,
            pool: &pool
        };

        let premium_repo = PgPremiumRepository{
            // conn: connection,
            pool: &pool
        };

        let billing = Box::new(InternalBillingService{
            client: reqwest::ClientBuilder::new().build()
                .expect("fail to create request client")
        });

        let auth_manager = AuthManager::new("secret".to_string());
        let app_state = AppState::new(
            Box::new(user_repo),
            Box::new(premium_repo),
            billing,
        );

        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(auth_manager))
            .service(infra::routes::get_account)
            .service(infra::routes::buypremium)
            .service(infra::routes::register)
            .service(infra::routes::me)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
