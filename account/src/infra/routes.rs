use std::ops::Deref;
use actix_web::{get, HttpRequest, HttpResponse, post, Responder, web};
use jsonwebtoken;
use crate::domain::repository::UserRepository;
use serde::{Deserialize, Serialize};
use crate::application::account::AppError::NotFound;
use crate::application::account::Application;
use crate::infra::auth::{AuthManager, Claims};


#[derive(Deserialize, Serialize, Debug)]
struct RegisterBody {
    name: String,
    password: String
}

#[derive(Deserialize, Serialize, Debug)]
struct RegisterResp{
    token: String,
}

const JWT_TTL: i64= 60;

#[post("/register")]
async fn register(
    req_body: String,
    app: web::Data<Application>,
    auth_manager: web::Data<AuthManager>
) -> impl Responder {
    let body = serde_json::from_str::<RegisterBody>(req_body.as_str());
    if body.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", body.err()));
    }

    let b = body.unwrap();
    match app.create_user(b.name, b.password).await {
        Ok(user) => {
            let token = auth_manager.auth_header(Claims {
                sub: user.id,
                exp: chrono::Utc::now().timestamp() + JWT_TTL,
            });

            HttpResponse::Ok().json(&RegisterResp{
                token: token
            })
        },
        Err(err) => {
            HttpResponse::NotFound().body(format!("err: {:?}", err))
        }
    }
}

#[post("/login")]
async fn login(
    req_body: String,
    auth_manager: web::Data<AuthManager>,
    app: web::Data<Application>,
) -> impl Responder {
    let body = serde_json::from_str::<RegisterBody>(req_body.as_str());
    if body.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", body.err()));
    }

    let b = body.unwrap();
    match app.login(b.name, b.password).await {
        Ok(user) => {
            let token = auth_manager.auth_header(Claims {
                sub: user.id,
                exp: chrono::Utc::now().timestamp() + JWT_TTL,
            });

            HttpResponse::Ok().json(&RegisterResp{
                token: token
            })
        },
        Err(err) => {
            HttpResponse::NotFound().body(format!("err: {:?}", err))
            // HttpResponse::InternalServerError().body(format!("err: {:?}", err))
        }
    }
}

#[get("/me")]
async fn me(
    req: HttpRequest,
    auth_manager: web::Data<AuthManager>,
    app: web::Data<Application>,
) -> impl Responder {
    let token = auth_manager.fetch_claims_from_req(&req);

    if token.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", token.err()));
    }

    let user = app.me(token.unwrap().sub).await;

    if user.is_err() {
        return HttpResponse::InternalServerError().body("err");
    }
    let user = user.unwrap();

    HttpResponse::Ok().json(user)
}



#[post("/buy_premium")]
async fn buypremium(
    app: web::Data<Application>,
) -> impl Responder {
    match app.buy_premium(1).await {
        Ok(()) => HttpResponse::Ok().body("ok"),
        Err(err) => {
            HttpResponse::NotFound().body(format!("err: {:?}", err))
        }
    }
    // HttpResponse::Ok().body("ok")
}

#[get("/account/{id}")]
async fn get_account(
    user_repo: web::Data<dyn UserRepository>,
    id: web::Path<i64>
) -> impl Responder {
    let user = user_repo.find(id.into_inner()).await;

    match user {
        Ok(None) => HttpResponse::NotFound().body("user not found"),
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().body(format!("err: {:?}", err))
    }
}