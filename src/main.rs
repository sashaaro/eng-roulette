use std::rc::Rc;
use actix_web::{App, HttpServer, web};
use diesel::prelude::*;
mod db;
mod models;
mod schema;
mod infra;
mod domain;
mod routes;
mod state;

use infra::repository::PgRoomRepository;
use crate::db::pg;
use crate::domain::repository::RoomRepository;
use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let pool = pg().await;

    HttpServer::new(move || {
        let repo = PgRoomRepository{
            // conn: connection,
            pool: pool.clone()
        };

        let app_state = AppState::new(repo);

        App::new()
            .app_data(web::Data::new(app_state))
            .service(routes::hello)
            .service(routes::echo)
            .route("/hey", web::get().to(routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
