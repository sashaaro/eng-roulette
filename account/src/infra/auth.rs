use actix_web::HttpRequest;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i64,
    pub(crate) exp: i64,
}

pub struct AuthManager {
    secret_key: String,
    decoding_key: DecodingKey,
}

impl AuthManager {
    pub fn new(secret_key: String) -> AuthManager {
        let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

        AuthManager {
            secret_key,
            decoding_key,
        }
    }

    pub fn auth_header(&self, claims: Claims) -> String {
        jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
        .unwrap()
    }

    pub fn extract_claims_from_req(
        &self,
        req: &HttpRequest,
    ) -> Result<Claims, Box<dyn std::error::Error>> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let mut auth_header = req
            .headers()
            .get("Authorization")
            .unwrap()
            .to_str()?
            .to_string();
        auth_header = auth_header.trim_start_matches("Bearer").trim().to_string();

        jsonwebtoken::decode::<Claims>(auth_header.as_str(), &self.decoding_key, &validation)
            .map(|t| t.claims)
            .map_err(|e| e.into())
    }
}
