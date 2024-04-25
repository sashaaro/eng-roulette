use actix_web::HttpRequest;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i32,
    pub(crate) exp: i64,
}

pub struct AuthManager {
    decoding_key:  DecodingKey,
    encoding_key: EncodingKey
}

impl AuthManager {
    pub fn new(secret_key: String) -> AuthManager {
        AuthManager{
            decoding_key: DecodingKey::from_secret(secret_key.as_ref()),
            encoding_key: EncodingKey::from_secret(secret_key.as_ref()),
        }
    }

    pub fn auth_header(&self, claims: Claims) -> String {
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &self.encoding_key);
        return token.unwrap()  // TODO stop use unwrap
    }
    pub fn fetch_claims_from_req(&self, req: &HttpRequest) -> Result<Claims, Box<dyn std::error::Error>> {
        let mut authHeader = req.headers().get("Authorization").unwrap().to_str()?.to_string();
        authHeader = authHeader.trim_start_matches("Bearer").trim().to_string();
        let token = jsonwebtoken::decode::<Claims>(authHeader.as_str(),
                                                   &self.decoding_key,
                                                   &jsonwebtoken::Validation::default()
        );
        return token.map(|t| t.claims ).map_err(|e| e.into())
    }
}