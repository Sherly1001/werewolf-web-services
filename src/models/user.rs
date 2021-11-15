use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    id: i64,
    username: String,
    hash_passwd: String,
    email: Option<String>,
    avatar_url: Option<String>,
    win: Option<i64>,
    lose: Option<i64>,
}
