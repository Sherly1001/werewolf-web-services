use actix_web::{
    dev::Payload, error::ErrorUnauthorized, http::header, web::Data, Error, FromRequest,
    HttpRequest,
};
use jsonwebtoken::{self as jwt, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use qstring::QString;
use serde::{Deserialize, Serialize};

use std::future::Future;
use std::pin::Pin;

use crate::config::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub exp: i64,
    pub user_id: i64,
}

impl Auth {
    #[allow(dead_code)]
    pub fn token(&self, secret: &[u8]) -> String {
        jwt::encode(
            &Header::new(Algorithm::HS256),
            self,
            &EncodingKey::from_secret(secret),
        )
        .expect("jwt")
    }

    #[allow(dead_code)]
    pub fn decode(token: &str, secret: &[u8]) -> Option<Self> {
        jwt::decode(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|err| {
            eprintln!("Auth decode error: {:?}", err);
        })
        .ok()
        .map(|token_data| token_data.claims)
    }
}

impl FromRequest for Auth {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req_cl = req.clone();
        let state = req.clone().app_data::<Data<AppState>>().unwrap().clone();
        let qs = QString::from(req.clone().query_string());

        Box::pin(async move {
            if let Some(token) = req_cl
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|token| token.to_str().ok())
                .map_or(qs.get("token"), |token| Some(token))
            {
                Auth::decode(token, state.secret_key.as_bytes())
                    .ok_or(ErrorUnauthorized("unauthorized"))
            } else {
                Err(ErrorUnauthorized("unauthorized"))
            }
        })
    }
}
