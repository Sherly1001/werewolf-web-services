use rocket::State;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::{Serialize, Deserialize};
use jsonwebtoken::{self as jwt, Header, EncodingKey, DecodingKey, Validation, Algorithm};

use crate::config::{self, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub exp: i64,
    pub username: String,
}

impl Auth {
    pub fn token(&self, secret: &[u8]) -> String {
        jwt::encode(&Header::new(Algorithm::HS256), self, &EncodingKey::from_secret(secret))
            .expect("jwt")
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let state = request.guard::<&State<AppState>>().await.unwrap();

        if let Some(auth) = extract_auth_from_request(request, &state.secret) {
            Outcome::Success(auth)
        } else {
            Outcome::Failure((Status::Forbidden, ()))
        }
    }
}

fn extract_auth_from_request(request: &Request, secret: &[u8]) -> Option<Auth> {
    request.headers()
        .get_one("authorization")
        .and_then(extract_token_from_header)
        .and_then(|token| decode_token(token, secret))
}

fn extract_token_from_header(header: &str) -> Option<&str> {
    if header.starts_with(config::TOKEN_PREFIX) {
        Some(&header[config::TOKEN_PREFIX.len()..])
    } else {
        None
    }
}

pub fn decode_token(token: &str, secret: &[u8]) -> Option<Auth> {
    jwt::decode(token, &DecodingKey::from_secret(secret), &Validation::new(Algorithm::HS256))
        .map_err(|err| {
            eprintln!("Auth decode error: {:?}", err);
        }).ok()
        .map(|token_data| token_data.claims)
}
