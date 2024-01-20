use std::error::Error;
use std::fmt::format;
use std::rc::Rc;
use actix_web::{get, HttpResponse, post, Responder, web};
use crate::domain::models::User;
use crate::domain::repository::UserRepository;
use crate::application::account::buy_premium;
use crate::infra::state::AppState;


#[post("/buy_premium")]
async fn buypremium(
    req_body: String,
    app_state: web::Data<AppState>
) -> impl Responder {
    match buy_premium(&app_state.billing, &app_state.user_repo, 1).await {
        Ok(()) => HttpResponse::Ok().body("ok"),
        Err(err) => HttpResponse::NotFound().body(format!("err: {:?}", err))
    }
    // HttpResponse::Ok().body("ok")
}

#[get("/account/{id}")]
async fn get_account(
    app_state: web::Data<AppState>,
    id: web::Path<i64>
) -> impl Responder {
    let num = app_state.user_repo.sum().await;
    println!("row = {}", num);

    let user = app_state.user_repo.find_user(id.into_inner()).await;

    match user {
        Ok(None) => HttpResponse::NotFound().body("user not found"),
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().body(format!("err: {:?}", err))
    }
}

pub async fn manual_hello(
    // app_state: web::Data<AppState>
) -> impl Responder {

    // let results = app_state.room_repo.all().await;
    //
    // println!("Displaying {} posts", results.len());
    // for post in results {
    //     println!("{}", post.title);
    //     println!("-----------\n");
    //     println!("{}", post.body);
    //}

    // println!("total: {}", results.len());
    HttpResponse::Ok().body("Hey there! Room total")
}
