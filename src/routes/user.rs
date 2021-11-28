use actix_web::error::{BlockingError, ErrorUnauthorized, ErrorUnprocessableEntity};
use actix_web::{get, post, web};
use serde::Deserialize;

use crate::auth::Auth;
use crate::config::{AppState, DbPool, get_conn};
use crate::db::user;
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
    if new_user.username == "" {
        return Err(ErrorUnprocessableEntity("username is empty"));
    }

    if new_user.passwd == "" {
        return Err(ErrorUnprocessableEntity("password is empty"));
    }

    let id = state.id_generatator.lock().unwrap().real_time_generate();

    let user = web::block(move || {
        let conn = get_conn(pool);
        user::create(
            &conn,
            id,
            &new_user.username,
            &new_user.passwd,
            new_user.avatar_url.as_deref(),
        )
    })
    .await
    .map_err(|err| {
        if err.to_string().contains("users_username_key") {
            ErrorUnprocessableEntity("username existed")
        } else {
            ErrorUnprocessableEntity(err)
        }
    })?;

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
        let conn = get_conn(pool);
        user::login(&conn, &login_user.username, &login_user.passwd)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Error(err) => ErrorUnauthorized(err),
        BlockingError::Canceled => ErrorUnauthorized("login failed"),
    })?;

    ResBody::new(
        "ok".to_string(),
        user.to_user_auth(state.secret_key.as_bytes()),
    )
}

#[get("/")]
pub async fn get_all(_auth: Auth, pool: web::Data<DbPool>) -> Res {
    let users = web::block(move || {
        let conn = get_conn(pool);
        user::get_all(&conn).map(|users| {
            users
                .iter()
                .map(|u| u.to_display_user())
                .collect::<Vec<UserDisplay>>()
        })
    })
    .await?;

    ResBody::new("ok".to_string(), users)
}

#[get("/info/")]
pub async fn get_info(auth: Auth, pool: web::Data<DbPool>) -> Res {
    let user = web::block(move || {
        let conn = get_conn(pool);
        user::get_info(&conn, auth.user_id).map(|u| u.to_display_user())
    })
    .await?;

    ResBody::new("ok".to_string(), user)
}
