use std::rc::Rc;
use actix_web::{get, HttpResponse, post, Responder, web};
use crate::domain::repository::RoomRepository;
use crate::infra::state::AppState;


#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/")]
async fn hello(
    app_state: web::Data<AppState>
) -> impl Responder {
    let num = app_state.room_repo.sum().await;
    println!("row = {}", num);

    let user = app_state.room_repo.find_user(1).await;

    println!("user = {}", user.is_some());

    HttpResponse::Ok().body("Hello world!")
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
