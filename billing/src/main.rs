#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[launch]
fn rocket() -> _ {
    println!("Hello, world!");
    rocket::build().mount("/", routes![index])
}