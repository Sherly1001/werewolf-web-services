use rocket::State;
use rocket::serde::Deserialize;
use rocket::serde::json::{Json, Value, serde_json::json};

use crate::errors;
use crate::models;
use crate::auth::Auth;
use crate::config::AppState;
use crate::db::{self, users::UserCreationError};

#[derive(Deserialize)]
pub struct NewUser {
    user: NewUserData,
}

#[derive(Deserialize)]
struct NewUserData {
    username: String,
    passwd: String,
    avatar_url: Option<String>,
}

#[post("/", format = "json", data = "<new_user>")]
pub async fn create(
    new_user: Json<NewUser>,
    conn: db::Conn,
    state: &State<AppState>,
) -> Result<Value, errors::Error> {
    let new_user = new_user.into_inner().user;

    conn.run(move |c| {
        db::users::create(c, &new_user.username, &new_user.passwd, new_user.avatar_url.as_deref())
    }).await.map(|user| json!({ "user": user.to_auth_user(&state.secret) }))
    .map_err(|err| {
        let err = match err {
            UserCreationError::DuplicatedUsername => "username has already been taken",
        };

        errors::Error::new(err)
    })
}

#[get("/")]
pub async fn get_users(_auth: Auth, conn: db::Conn) -> Value {
    let users = conn.run(move |c| {
        db::users::get_all(c)
    }).await;

    json!({
        "users": users.iter().map(|u| u.to_display_user())
            .collect::<Vec<models::user::DisplayUser>>()
    })
}


#[derive(Deserialize)]
pub struct LoginUser {
    user: LoginUserData,
}

#[derive(Deserialize)]
struct LoginUserData {
    username: String,
    passwd: String,
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login(
    user: Json<LoginUser>,
    conn: db::Conn,
    state: &State<AppState>,
) -> Result<Value, errors::Error> {
    let user = user.into_inner().user;

    conn.run(move |c| {
        db::users::login(c, &user.username, &user.passwd)
    }).await.map(|user| json!({ "user": user.to_auth_user(&state.secret) }))
    .ok_or_else(|| errors::Error::new("email or password is invalid"))
}
