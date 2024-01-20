use actix_web::{App, HttpServer, web};
use crate::infra::repository::PgUserRepository;
use crate::infra::service::InternalBillingService;

mod infra;
mod domain;
mod application;

use crate::infra::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let pool = infra::db::pg().await;

    HttpServer::new(move || {
        let user_repo = PgUserRepository{
            // conn: connection,
            pool: pool.clone()
        };

        let billing = Box::new(InternalBillingService{
            client: reqwest::ClientBuilder::new().build().expect("fail to create request client")
        });

        let app_state = AppState::new(Box::new(user_repo), billing);

        App::new()
            .app_data(web::Data::new(app_state))
            .service(infra::routes::get_account)
            .service(infra::routes::buypremium)
            .route("/hey", web::get().to(infra::routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
