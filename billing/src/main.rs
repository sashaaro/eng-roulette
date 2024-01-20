#[macro_use] extern crate rocket;
extern crate core;

mod application;
mod domain;
mod infra;

use core::fmt;
use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Formatter};
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
async fn income(tx_repo: &State<Box<PgTxRepository>>, data: Json<Income>) -> String {
    let repo = tx_repo.inner();
    match application::commands::income(repo.clone(), data.user_id, data.amount).await {
        Ok(_) => "Ok".to_string(),
        Err(err) => format!("{:?}", err)
    }
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

fn print_number<T: std::fmt::Display>(x: T) {
    println!("{}", x);
}

fn print_number2(x: impl std::fmt::Display) {
    println!("{}", x);
}

fn mapp<U, T>(f: impl FnOnce(T)) -> Option<U> {
    return None
}
fn mappp<U, T, F>(f: F) -> Option<U> where F: FnOnce(T) -> U {
    return None
}

#[derive(Debug)]
struct ErrorOne;
impl Error for ErrorOne{}
impl fmt::Display for ErrorOne {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error one")
    }
}

#[derive(Debug)]
struct ErrorTwo;
impl Error for ErrorTwo{}
impl fmt::Display for ErrorTwo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error two")
    }
}

// fn return_error(input: u8) -> Result<String, impl Error> {
//     match input {
//         0 => Err(ErrorOne),
//         1 => Err(ErrorTwo),
//         _ => Ok("no error".to_string())
//     }
// }

// pub fn notify<T: Debug>(item: &T) {
//     println!("Breaking news! {}", item.summarize());
// }

#[launch]
async fn rocket() -> _ {
    let a = 5;
    let b = 10.0;
    print_number(a);
    print_number(b);
    print_number2(a);
    print_number2(b);
    // println!("error 1: {:?}", return_error(0));
    // println!("error 1: {:?}", return_error(1));
    // println!("error 1: {:?}", return_error(2));


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