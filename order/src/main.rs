use std::rc::Rc;
use actix_web::{App, HttpServer, web};
use diesel::prelude::*;
mod infra;
mod domain;

use infra::repository::PgRoomRepository;
use crate::domain::repository::RoomRepository;
use crate::infra::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let pool = infra::db::pg().await;

    HttpServer::new(move || {
        let repo = PgRoomRepository{
            // conn: connection,
            pool: pool.clone()
        };

        let app_state = AppState::new(repo);

        App::new()
            .app_data(web::Data::new(app_state))
            .service(infra::routes::hello)
            .service(infra::routes::echo)
            .route("/hey", web::get().to(infra::routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
