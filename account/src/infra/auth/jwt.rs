use actix_web::HttpRequest;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: i64,
    pub(crate) exp: i64,
}

pub struct JwtManager {
    secret_key: String,
    decoding_key: DecodingKey,
}

const JWT_TTL: i64 = 60 * 60;

impl JwtManager {
    pub fn new(secret_key: String) -> JwtManager {
        let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

        JwtManager {
            secret_key,
            decoding_key,
        }
    }

    pub fn gen_user_token(&self, user_id: i64) -> String {
        self.gen_token(Claims {
            sub: user_id,
            exp: chrono::Utc::now().timestamp() + JWT_TTL,
        })
    }

    pub fn gen_token(&self, claims: Claims) -> String {
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
