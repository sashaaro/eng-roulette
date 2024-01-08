#[macro_use] extern crate rocket;

mod application;
mod domain;
mod infra;

use std::any::Any;
use rocket::State;
use crate::infra::repository::PgTxRepository;
use rocket::serde::json::Json;
use serde::Deserialize;
use crate::domain::repository::{Tx2pcID, TxRepository};

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

#[derive(Deserialize)]
struct Expense {
    user_id: i32,
    amount: i32,
    tx_id: Tx2pcID,
}

#[post("/prepare_expense", format = "json", data = "<data>")]
async fn prepare_expense(tx_repo: &State<Box<PgTxRepository>>, data: Json<Expense>) -> &'static str {
    let repo = tx_repo.inner();
    let res = application::commands::prepare_expense(repo.clone(), data.tx_id.clone(), data.user_id, data.amount).await;
    if res.is_err() {
        println!("err: {}", res.err().unwrap());
        return "err"
    }
    return "OK";
}

#[post("/commit_expense", format = "json", data = "<data>")]
async fn commit_expense(tx_repo: &State<Box<PgTxRepository>>, data: Json<Expense>) -> &'static str {
    let repo = tx_repo.inner();
    let res = application::commands::commit_expense(repo.clone(), data.tx_id.clone()).await;
    if res.is_err() {
        println!("err: {}", res.err().unwrap());
        return "err"
    }
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
            income,
            prepare_expense,
            commit_expense
        ])
}