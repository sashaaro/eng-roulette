use std::collections::HashMap;
use crate::api::goauth2::goauth2;
use crate::infra::auth::{AuthManager, Claims};
use crate::service::account::AccountService;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use oauth2::{CsrfToken, PkceCodeChallenge, RedirectUrl, Scope};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct RegisterBody {
    name: String,
    password: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct RegisterResponse {
    pub token: String,
}

const JWT_TTL: i64 = 60 * 60;

#[post("/register")]
async fn register(
    req_body: String,
    app: web::Data<AccountService>,
    auth_manager: web::Data<AuthManager>,
) -> impl Responder {
    let body = serde_json::from_str::<RegisterBody>(req_body.as_str());
    if body.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", body.err()));
    }

    let b = body.unwrap();
    match app.create_user(b.name, b.password).await {
        Ok(user) => {
            let token = auth_manager.auth_header(Claims {
                sub: user.id as i64,
                exp: chrono::Utc::now().timestamp() + JWT_TTL,
            });

            HttpResponse::Ok().json(&RegisterResponse { token: token })
        }
        Err(err) => HttpResponse::NotFound().body(format!("err: {:?}", err)),
    }
}

#[post("/login")]
async fn login(
    req_body: String,
    _: HttpRequest,
    auth_manager: web::Data<AuthManager>,
    app: web::Data<AccountService>,
) -> impl Responder {
    let body = serde_json::from_str::<RegisterBody>(req_body.as_str());
    if body.is_err() {
        return HttpResponse::BadRequest().body(format!("err: {:?}", body.err()));
    }

    let b = body.unwrap();

    let user = app.login(b.name.clone(), b.password.clone()).await;

    if user.is_err() {
        log::info!(username:? = b.name; "Failed login attempt");

        return HttpResponse::NotFound().body(format!("err: {:?}", user.err()));
    }
    let user = user.unwrap();

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
    let token = auth_manager.extract_claims_from_req(&req);

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

#[get("/auth/google")]
async fn google_auth(req: HttpRequest) -> impl Responder {
    let client = goauth2();

    #[derive(Debug, Deserialize)]
    pub struct Params {
        redirect_url: String,
    }


    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let client = client.set_redirect_uri(
        RedirectUrl::new(params.redirect_url.to_string()).unwrap()
    );

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    authorize_url.to_string()
}
