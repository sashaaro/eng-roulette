#[macro_use] extern crate rocket;

mod application;
mod domain;
mod infra;
mod lab;

use core::fmt;
use std::any::Any;
use std::env;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use rocket::http::Status;
use rocket::State;
use crate::infra::repository::{PgJournalRepository, PgTxRepository};
use rocket::serde::json::Json;
use serde::Deserialize;
use crate::domain::repository::{Tx2pcID, TxRepository};

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
struct Income {
    user_id: i64,
    amount: i64,
}

#[post("/income", format = "json", data = "<data>")]
async fn income(tx_repo: &State<Box<dyn TxRepository>>, data: Json<Income>) -> Status {
    let repo = tx_repo.inner();
    match application::commands::income(repo, data.user_id, data.amount).await {
        Ok(_) => Status::Ok,
        Err(err) => Status::BadRequest,
    }
}

#[derive(Deserialize)]
struct Expense {
    user_id: i64,
    amount: i64,
    tx_id: Tx2pcID,
}

#[post("/prepare_expense", format = "json", data = "<data>")]
async fn prepare_expense(tx_repo: &State<Box<dyn TxRepository>>, data: Json<Expense>) -> Status {
    let repo = tx_repo.inner();
    match application::commands::prepare_expense(repo, data.tx_id.clone(), data.user_id, data.amount).await {
        Ok(_) => Status::Ok,
        Err(err) => Status::BadRequest,
    }
}


#[derive(Deserialize)]
struct CommitReq {
    tx_id: Tx2pcID,
}

#[post("/commit_expense", format = "json", data = "<data>")]
async fn commit_expense(tx_repo: &State<Box<dyn TxRepository>>, data: Json<CommitReq>) -> Status {
    let repo = tx_repo.inner();
    match application::commands::commit_expense(repo, data.tx_id.clone()).await {
        Ok(_) => Status::Ok,
        Err(err) => Status::BadRequest,
    }
}


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let mut pool = infra::db::pg().await;
    let repo: Box<dyn TxRepository> = Box::new(PgTxRepository::new(pool.clone()));
    let journalRepo: PgJournalRepository = PgJournalRepository::new(pool.clone());

    journalRepo.log().await.unwrap();
    //let d: Box<dyn TxRepository + Send> = Box::new(repo);

    //let res = application::commands::income(d, 1, 2).await;
    //application::commands::income(Box::new(repo), 1, 2);

    rocket::build()
        // .manage(Box::new(repo.clone()))
        .manage(repo)
        //.manage(d)
        .mount("/", routes![
            index,
            income,
            prepare_expense,
            commit_expense
        ])
        .launch().await?;
    Ok(())
}