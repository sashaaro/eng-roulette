use axum::extract::{FromRef, FromRequestParts};
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::{HeaderMap, StatusCode};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

// Экстрактор JWT для Axum, который извлекает claims токена из заголовка Authorization или query-параметра `jwt`.
//
// # Описание
// Этот экстрактор позволяет получать claims JWT в ваших обработчиках Axum. Он ищет JWT-токен в заголовке `Authorization: Bearer <token>`
// или в query-параметре `jwt`. Токен валидируется с помощью переданного секретного ключа, и если он валиден, claims становятся доступны в обработчике.
//
// Тип состояния приложения (`S`) должен реализовывать трейт `FromRef<S>` для `SecretKey`, чтобы экстрактор мог получить ключ из состояния.
//
// # Пример
// ```rust
// use axum::{routing::get, Router, extract::State};
// use room::extract::jwt::{JWT, Claims};
// use jsonwebtoken::DecodingKey;
// use axum::extract::FromRef;
//
// struct AppState {
//     secret: DecodingKey,
// }
//
// impl FromRef<AppState> for &'static DecodingKey {
//     fn from_ref(state: &AppState) -> Self {
//         &state.secret
//     }
// }
//
// async fn handler(JWT(claims): JWT) -> String {
//     let user_id = claims.sub;
//     // ...
// }
//
// #[tokio::main]
// async fn main() {
//     let secret = DecodingKey::from_secret(b"mysecret");
//     let state = AppState { secret };
//     let app = Router::new()
//         .route("/", get(handler))
//         .with_state(state);
//     // ...
// }
// ```
//
// # Ошибки
// Возвращает 401 Unauthorized, если токен отсутствует, невалиден или имеет неверную подпись.
#[derive(Debug)]
pub struct JWT(pub Claims);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i64,
    pub(crate) exp: i64,
}

pub type SecretKey = &'static DecodingKey;

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
                .ok_or(JWTRejection::InvalidAuthorizationHeader)?
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| JWTRejection::InvalidAuthorizationHeader)?;

            let header = header.trim();
            if !header.starts_with("Bearer ") {
                return Err(JWTRejection::InvalidAuthorizationHeader);
            }
            let token = header.trim_start_matches("Bearer ").trim();
            token.to_owned()
        }
    };

    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    jsonwebtoken::decode::<Claims>(&token, secret_key, &validation)
        .map(|t| JWT(t.claims))
        .map_err(|_| JWTRejection::InvalidSignature)
}

#[derive(Debug)]
pub enum JWTRejection {
    InvalidAuthorizationHeader,
    InvalidSignature,
}

impl IntoResponse for JWTRejection {
    fn into_response(self) -> Response {
        match self {
            JWTRejection::InvalidAuthorizationHeader => {
                (StatusCode::UNAUTHORIZED, "invalid authorization header")
            }
            JWTRejection::InvalidSignature => (StatusCode::UNAUTHORIZED, "invalid signature"),
        }
        .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header};
    use std::sync::OnceLock;

    static SECRET: OnceLock<String> = OnceLock::new();
    static DECODING_KEY: OnceLock<DecodingKey> = OnceLock::new();

    fn get_secret() -> &'static str {
        SECRET.get_or_init(|| "test_secret".to_string())
    }

    fn get_decoding_key() -> &'static DecodingKey {
        DECODING_KEY.get_or_init(|| DecodingKey::from_secret(get_secret().as_bytes()))
    }

    fn create_token(sub: i64, exp: i64) -> String {
        let claims = Claims { sub, exp };
        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(get_secret().as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn valid_token_in_header() {
        let token = create_token(42, 9999999999);
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        let result = extract_token(&headers, None, get_decoding_key());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.sub, 42);
    }

    #[test]
    fn valid_token_in_query() {
        let token = create_token(100, 9999999999);
        let headers = HeaderMap::new();
        let result = extract_token(&headers, Some(token.clone()), get_decoding_key());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.sub, 100);
    }

    #[test]
    fn missing_token() {
        let headers = HeaderMap::new();
        let result = extract_token(&headers, None, get_decoding_key());
        assert!(matches!(
            result,
            Err(JWTRejection::InvalidAuthorizationHeader)
        ));
    }

    #[test]
    fn invalid_signature() {
        let claims = Claims {
            sub: 1,
            exp: 9999999999,
        };
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"wrong_secret"),
        )
        .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        let result = extract_token(&headers, None, get_decoding_key());
        assert!(matches!(result, Err(JWTRejection::InvalidSignature)));
    }

    #[test]
    fn malformed_header() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "NotBearerToken".parse().unwrap());
        let result = extract_token(&headers, None, get_decoding_key());
        assert!(matches!(
            result,
            Err(JWTRejection::InvalidAuthorizationHeader)
        ));
    }
}
