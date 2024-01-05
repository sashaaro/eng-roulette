use actix_web::{get, HttpResponse, post, Responder, web};
use crate::domain::repository::RoomRepository;

#[derive(Clone)]
pub struct AppState {
    // pub room_repo: dyn RoomRepository,
    pub room_repo: dyn RoomRepository,
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn manual_hello(app_state: web::Data<AppState>) -> impl Responder {

    let results = app_state.room_repo.all().await;

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("-----------\n");
        println!("{}", post.body);
    }

    HttpResponse::Ok().body("Hey there! Room total" + results.len())
}
