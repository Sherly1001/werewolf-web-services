use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::Auth;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub hash_passwd: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub win: Option<i32>,
    pub lose: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuth {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDisplay {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub win: Option<i32>,
    pub lose: Option<i32>,
    pub is_online: Option<bool>,
}

impl User {
    pub fn to_user_auth(&self, secret: &[u8]) -> UserAuth {
        let exp = Utc::now() + Duration::days(60);
        let token = Auth {
            exp: exp.timestamp(),
            user_id: self.id,
        }
        .token(secret);

        UserAuth { token }
    }

    pub fn to_display_user(&self) -> UserDisplay {
        UserDisplay {
            id: self.id.to_string(),
            username: self.username.clone(),
            avatar_url: self.avatar_url.clone(),
            win: self.win,
            lose: self.lose,
            is_online: None,
        }
    }
}
