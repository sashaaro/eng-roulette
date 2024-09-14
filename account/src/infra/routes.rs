use actix_web::{get, HttpRequest, HttpResponse, post, Responder, web};
use jsonwebtoken;
use crate::domain::repository::UserRepository;
use crate::application::account::{buy_premium, create_user};
use crate::infra::state::AppState;
use serde::{Deserialize, Serialize};
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
async fn register<'a>(
    req_body: String,
    app_state: web::Data<AppState<'a>>,
    auth_manager: web::Data<AuthManager>
) -> impl Responder {
    let body = serde_json::from_str::<RegisterBody>(req_body.as_str());
    if body.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", body.err()));
    }

    let b = body.unwrap();
    match create_user(&app_state.user_repo, b.name, b.password).await {
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

#[get("/me")]
async fn me(
    req: HttpRequest,
    auth_manager: web::Data<AuthManager>
) -> impl Responder {
    let token = auth_manager.fetch_claims_from_req(&req);

    if token.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", token.err()));
    }

     HttpResponse::Ok().body(token.unwrap().sub.to_string())
}



#[post("/buy_premium")]
async fn buypremium<'a>(
    app_state: web::Data<AppState<'a>>,
) -> impl Responder {
    match buy_premium(&app_state.billing, &app_state.user_repo, &app_state.premium_repo, 1).await {
        Ok(()) => HttpResponse::Ok().body("ok"),
        Err(err) => {
            HttpResponse::NotFound().body(format!("err: {:?}", err))
        }
    }
    // HttpResponse::Ok().body("ok")
}

#[get("/account/{id}")]
async fn get_account<'a>(
    app_state: web::Data<AppState<'a>>,
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