use crate::infra::auth::g_oauth::create_google_oauth_client;
use crate::infra::auth::jwt::JwtManager;
use crate::service::account::AccountService;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, ResponseError};
use anyhow::anyhow;
use oauth2::{
    AuthorizationCode, CsrfToken, CurlHttpClient, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Debug)]
struct RegisterBody {
    name: String,
    password: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct RegisterResponse {
    pub token: String,
}

#[post("/register")]
async fn register(
    req_body: String,
    account_service: web::Data<AccountService>,
    jwt_manager: web::Data<JwtManager>,
) -> impl Responder {
    let body = match serde_json::from_str::<RegisterBody>(req_body.as_str()) {
        Ok(body) => body,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("err: {:?}", err));
        }
    };

    let user = match account_service
        .create_user(&body.name, &body.password)
        .await
    {
        Ok(user) => user,
        Err(err) => return HttpResponse::NotFound().body(format!("err: {:?}", err)),
    };

    HttpResponse::Ok().json(&RegisterResponse {
        token: jwt_manager.gen_user_token(user.id as _),
    })
}

#[post("/login")]
async fn login(
    req_body: String,
    _: HttpRequest,
    jwt_manager: web::Data<JwtManager>,
    account_service: web::Data<AccountService>,
) -> impl Responder {
    let body = match serde_json::from_str::<RegisterBody>(req_body.as_str()) {
        Ok(b) => b,
        Err(err) => return HttpResponse::BadRequest().body(format!("err: {:?}", err)),
    };
    let user = account_service
        .login(body.name.clone(), body.password.clone())
        .await;

    let user = match user {
        Ok(user) => user,
        Err(err) => {
            log::info!(username:? = body.name; "Failed login attempt");
            return HttpResponse::NotFound().body(format!("err: {:?}", err));
        }
    };

    log::info!(user:? = user.username; "User authenticated");

    HttpResponse::Ok().json(&RegisterResponse {
        token: jwt_manager.gen_user_token(user.id as _),
    })
}

#[get("/me")]
async fn me(
    req: HttpRequest,
    jwt_manager: web::Data<JwtManager>,
    account_service: web::Data<AccountService>,
) -> impl Responder {
    let token = match jwt_manager.extract_claims_from_req(&req) {
        Ok(token) => token,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("err: {:?}", err));
        }
    };

    let user = match account_service.me(token.sub).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::InternalServerError().body("err");
        }
    };

    HttpResponse::Ok().json(user)
}

#[get("/auth/google")]
async fn google_auth(req: HttpRequest) -> Result<impl Responder, AppError> {
    #[derive(Debug, Deserialize)]
    pub struct Params {
        redirect_url: String,
    }

    let params = web::Query::<Params>::from_query(req.query_string())
        .map_err(|_err| AppError(anyhow!("internal error")))?;

    let google_client = create_google_oauth_client();
    let google_client = google_client.set_redirect_uri(
        RedirectUrl::new(params.redirect_url.to_string())
            .map_err(|_err| AppError(anyhow!("internal error")))?,
    );

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, _) = google_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    #[derive(Debug, Serialize)]
    struct Response {
        authorize_url: String,
        pkce_code_verifier: String,
    }

    Ok(HttpResponse::Ok().json(Response {
        authorize_url: authorize_url.to_string(),
        pkce_code_verifier: pkce_code_verifier.secret().to_string(),
    }))
}

#[get("/auth/google/callback")]
async fn google_auth_callback(
    req: HttpRequest,
    jwt_manager: web::Data<JwtManager>,
    account_service: web::Data<AccountService>,
) -> anyhow::Result<impl Responder, AppError> {
    #[derive(Debug, Deserialize)]
    pub struct Params {
        code: String,
        // state: String,
        pkce_code_verifier: String,
        redirect_url: String,
    }

    let params = web::Query::<Params>::from_query(req.query_string())
        .map_err(|_err| AppError(anyhow!("internal error")))?;
    let google_client = create_google_oauth_client();
    let google_client = google_client.set_redirect_uri(
        RedirectUrl::new(params.redirect_url.to_string())
            .map_err(|_err| AppError(anyhow!("internal error")))?,
    );

    let access_token = google_client
        .exchange_code(AuthorizationCode::new(params.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(params.pkce_code_verifier.clone()))
        .request(&CurlHttpClient {})
        .map_err(|err| {
            println!("{:?}", err);
            AppError(anyhow!(err))
        })?
        .access_token()
        .secret()
        .to_string();

    let client = reqwest::Client::new();
    let user_info_res = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|_err| AppError(anyhow!("internal error")))?
        .error_for_status()
        .map_err(|_err| AppError(anyhow!("internal error")))?;

    #[derive(Debug, Deserialize)]
    struct GoogleUserInfo {
        pub email: String,
    }

    let user_info: GoogleUserInfo = user_info_res.json().await.map_err(|err| {
        println!("{:?} {:?}", err, &access_token);
        AppError(anyhow!(err))
    })?;

    let user = account_service.create_or_login(&user_info.email).await?;

    Ok(HttpResponse::Ok().json(&RegisterResponse {
        token: jwt_manager.gen_user_token(user.id as _),
    }))
}

// Для поддержки Result в actix контроллерах нужен этот тип
#[derive(Debug)]
struct AppError(anyhow::Error);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body(format!("Internal error: {}", self.0))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}
