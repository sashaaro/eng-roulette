use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::{HeaderMap, StatusCode};
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug)]
pub struct JWT(pub Claims);

pub enum JWTRejection {
    InvalidAuthorizationHeader,
    InvalidSignature,
}

impl From<Claims> for JWT {
    fn from(inner: Claims) -> Self {
        Self(inner)
    }
}

impl IntoResponse for JWTRejection {
    fn into_response(self) -> Response {
        match self {
            JWTRejection::InvalidAuthorizationHeader => (
                StatusCode::UNAUTHORIZED,
                "invalid authorization header".to_string(),
            )
                .into_response(),
            JWTRejection::InvalidSignature => {
                (StatusCode::UNAUTHORIZED, "invalid signature".to_string()).into_response()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i64,
    pub(crate) exp: i64,
}

impl<S> FromRequestParts<S> for JWT {
    type Rejection = JWTRejection;

    fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async { fetch_jwt(&parts.headers) }
    }
}

fn fetch_jwt(headers: &HeaderMap) -> Result<JWT, JWTRejection> {
    let secret = "secret".to_string();

    //let secret = std::env::var("SECRET_KEY").unwrap().as_ref();
    // TODO inject from config
    let decoding_key = &DecodingKey::from_secret(secret.as_ref());

    let header = headers
        .get("Authorization")
        .ok_or(JWTRejection::InvalidSignature)?
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| JWTRejection::InvalidAuthorizationHeader)?
        .clone();

    let header = header.trim_start_matches("Bearer").trim();

    jsonwebtoken::decode::<Claims>(header, decoding_key, &jsonwebtoken::Validation::default())
        .map(|t| t.claims.into())
        .map_err(|_| JWTRejection::InvalidSignature)
}

impl<S> FromRequest<S> for JWT
where
    S: Send + Sync,
{
    type Rejection = JWTRejection;
    async fn from_request(req: Request, _: &S) -> Result<Self, Self::Rejection> {
        fetch_jwt(req.headers())
    }
}
