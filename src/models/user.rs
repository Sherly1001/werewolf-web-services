use rocket::serde::{Serialize, Deserialize};
use chrono::{Duration, Utc};

use crate::auth::Auth;

#[derive(Queryable, Serialize)]
pub struct User {
    pub username: String,
    pub hash_passwd: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub win: Option<i64>,
    pub lose: Option<i64>,
}

#[derive(Serialize)]
pub struct UserAuth<'a> {
    pub username: &'a str,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayUser<'a> {
    pub username: &'a str,
    pub avatar_url: Option<&'a str>,
    pub win: Option<i64>,
    pub lose: Option<i64>,
}

impl User {
    pub fn to_auth_user(&self, secret: &[u8]) -> UserAuth {
        let exp = Utc::now() + Duration::days(60);
        let token = Auth {
            username: self.username.clone(),
            exp: exp.timestamp(),
        }.token(secret);

        UserAuth {
            username: &self.username,
            token,
        }
    }

    pub fn to_display_user(&self) -> DisplayUser {
        DisplayUser {
            username: &self.username,
            avatar_url: self.avatar_url.as_deref(),
            win: self.win,
            lose: self.lose,
        }
    }
}
