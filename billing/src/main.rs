#[macro_use] extern crate rocket;

mod application;
mod domain;
mod infra;

use std::any::Any;
use rocket::State;
use crate::infra::repository::PgTxRepository;
use rocket::serde::json::Json;
use serde::Deserialize;
use crate::domain::repository::TxRepository;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
struct Income {
    user_id: i32,
    amount: i32,
}
#[post("/income", format = "json", data = "<data>")]
async fn income(tx_repo: &State<Box<PgTxRepository>>, data: Json<Income>) -> &'static str {
    let repo = tx_repo.inner();
    application::commands::income(repo.clone(), data.user_id, data.amount).await;
    return "OK";
}

#[launch]
async fn rocket() -> _ {
    println!("Hello, world!");
    let pool = infra::db::pg().await;
    let repo = PgTxRepository{
        // conn: connection,
        pool: pool//.clone()
    };

   //let d: Box<dyn TxRepository + Send> = Box::new(repo);

    //let res = application::commands::income(d, 1, 2).await;
    //application::commands::income(Box::new(repo), 1, 2);

    rocket::build()
        .manage(Box::new(repo.clone()))
        //.manage(d)
        .mount("/", routes![
            index,
            income
        ])
}