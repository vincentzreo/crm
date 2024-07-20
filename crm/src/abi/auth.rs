use std::ops::Deref;

use chrono::{DateTime, Utc};
use jwt_simple::prelude::*;
use tonic::service::Interceptor;

const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DecodingKey(Ed25519PublicKey);

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }
    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let options = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..Default::default()
        };

        let claims = self.0.verify_token(token, Some(options))?;
        Ok(claims.custom)
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Interceptor for DecodingKey {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| tonic::Status::unauthenticated("No token"))?;
        let token = token
            .strip_prefix("Bearer ")
            .ok_or_else(|| tonic::Status::unauthenticated("Invalid token"))?;
        let user = self
            .verify(token)
            .map_err(|e| tonic::Status::unauthenticated(e.to_string()))?;
        req.extensions_mut().insert(user);
        Ok(req)
    }
}
