use actix_web::{App, HttpServer, web};
use diesel::prelude::*;
mod db;
mod models;
mod schema;
mod infra;
mod domain;
mod routes;

use infra::repository::PgRoomRepository;
use crate::db::pg;
use crate::domain::repository::RoomRepository;
use crate::routes::AppState;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    // let connection = &mut db::establish_connection();

    let pool = pg().await;
    let mut repo = PgRoomRepository{
        // conn: connection,
        pool: &pool
    };

    let num = repo.sum().await;
    println!("row = {}", num);

    let app_state = AppState{ room_repo: repo};
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(routes::hello)
            .service(routes::echo)
            .route("/hey", web::get().to(routes::manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
