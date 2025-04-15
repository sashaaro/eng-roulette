use crate::webrtc::axum::SecretKey;
use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::{HeaderMap, StatusCode};
use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::future::Future;

// Экстрактор JWT для axum, который получает claims токена из Authorization заголовка
#[derive(Debug)]
pub struct JWT(pub Claims);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i64,
    pub(crate) exp: i64,
}

impl From<Claims> for JWT {
    fn from(inner: Claims) -> Self {
        Self(inner)
    }
}

impl<S> FromRequestParts<S> for JWT
where
    SecretKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = JWTRejection;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let query_jwt = parts.uri.query().and_then(|query| {
            serde_urlencoded::from_str::<HashMap<String, String>>(query)
                .ok()
                .and_then(|params| params.get("jwt").map(|s| s.to_owned()))
        });

        async { extract_token(&parts.headers, query_jwt, SecretKey::from_ref(state)) }
    }
}

fn extract_token(
    headers: &HeaderMap,
    query_jwt: Option<String>,
    secret_key: SecretKey,
) -> Result<JWT, JWTRejection> {
    let token = match query_jwt {
        Some(token) => token,
        None => {
            let header = headers
                .get("Authorization")
                .ok_or(JWTRejection::InvalidSignature)?
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| JWTRejection::InvalidAuthorizationHeader)?;

            let token = header.trim_start_matches("Bearer").trim();
            token.to_owned()
        }
    };

    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    jsonwebtoken::decode::<Claims>(&token, secret_key, &validation)
        .map(|t| t.claims.into())
        .map_err(|_| JWTRejection::InvalidSignature)
}

pub enum JWTRejection {
    InvalidAuthorizationHeader,
    InvalidSignature,
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
