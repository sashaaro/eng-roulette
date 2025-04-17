use crate::infra::auth::{AuthManager, Claims};
use crate::service::account::AccountService;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct RegisterBody {
    name: String,
    password: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct RegisterResponse {
    pub token: String,
}

const JWT_TTL: i64 = 60 * 60;

#[post("/register")]
async fn register(
    req_body: String,
    app: web::Data<AccountService>,
    auth_manager: web::Data<AuthManager>,
) -> impl Responder {
    let body = match serde_json::from_str::<RegisterBody>(req_body.as_str()) {
        Ok(body) => body,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("err: {:?}", err));
        }
    };

    let user = match app.create_user(body.name, body.password).await {
        Ok(user) => user,
        Err(err) => return HttpResponse::NotFound().body(format!("err: {:?}", err)),
    };

    let token = auth_manager.auth_header(Claims {
        sub: user.id as i64,
        exp: chrono::Utc::now().timestamp() + JWT_TTL,
    });

    HttpResponse::Ok().json(&RegisterResponse { token })
}

#[post("/login")]
async fn login(
    req_body: String,
    _: HttpRequest,
    auth_manager: web::Data<AuthManager>,
    app: web::Data<AccountService>,
) -> impl Responder {
    let body = match serde_json::from_str::<RegisterBody>(req_body.as_str()) {
        Ok(b) => b,
        Err(err) => return HttpResponse::BadRequest().body(format!("err: {:?}", err)),
    };
    let user = app.login(body.name.clone(), body.password.clone()).await;

    let user = match user {
        Ok(user) => user,
        Err(err) => {
            log::info!(username:? = body.name; "Failed login attempt");
            return HttpResponse::NotFound().body(format!("err: {:?}", err));
        }
    };

    let token = auth_manager.auth_header(Claims {
        sub: user.id as i64,
        exp: chrono::Utc::now().timestamp() + JWT_TTL,
    });

    log::info!(user:? = user.username; "User authenticated");

    HttpResponse::Ok().json(&RegisterResponse { token })
}

#[get("/me")]
async fn me(
    req: HttpRequest,
    auth_manager: web::Data<AuthManager>,
    app: web::Data<AccountService>,
) -> impl Responder {
    let token = match auth_manager.extract_claims_from_req(&req) {
        Ok(token) => token,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("err: {:?}", err));
        }
    };

    let user = match app.me(token.sub).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::InternalServerError().body("err");
        }
    };

    HttpResponse::Ok().json(user)
}
