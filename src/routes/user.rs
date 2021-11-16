use actix_web::error::{ErrorUnauthorized, ErrorUnprocessableEntity};
use actix_web::{get, post, web};
use serde::Deserialize;

use crate::config::{AppState, DbPool};
use crate::db::user;
use crate::auth::Auth;
use crate::error::{Res, ResBody};
use crate::models::user::UserDisplay;

#[derive(Deserialize)]
pub struct NewUser {
    username: String,
    passwd: String,
    avatar_url: Option<String>,
}

#[post("/")]
pub async fn create(
    pool: web::Data<DbPool>,
    state: web::Data<AppState>,
    new_user: web::Json<NewUser>,
) -> Res {
    let mut id_generator = state.id_generatator;

    let user = web::block(move || {
        let conn = pool.get().unwrap();
        user::create(
            &conn,
            id_generator.real_time_generate(),
            &new_user.username,
            &new_user.passwd,
            new_user.avatar_url.as_deref(),
        )
    })
    .await
    .map_err(|e| ErrorUnprocessableEntity(e.to_string()))?;

    ResBody::new(
        "ok".to_string(),
        user.to_user_auth(state.secret_key.as_bytes()),
    )
}

#[derive(Deserialize)]
pub struct LoginUser {
    username: String,
    passwd: String,
}

#[post("/login/")]
pub async fn login(
    pool: web::Data<DbPool>,
    state: web::Data<AppState>,
    login_user: web::Json<LoginUser>,
) -> Res {
    let user = web::block(move || {
        let conn = pool.get().unwrap();
        user::login(&conn, &login_user.username, &login_user.passwd)
            .ok_or("email or password is invalid")
    })
    .await
    .map_err(|e| ErrorUnauthorized(e))?;

    ResBody::new(
        "ok".to_string(),
        user.to_user_auth(state.secret_key.as_bytes()),
    )
}

#[get("/")]
pub async fn get_all(_auth: Auth, pool: web::Data<DbPool>) -> Res {
    let users = web::block(move || {
        let conn = pool.get().unwrap();
        user::get_all(&conn).map(|users| users.iter().map(|u| u.to_display_user()).collect::<Vec<UserDisplay>>())
    })
    .await?;

    ResBody::new("ok".to_string(), users)
}
