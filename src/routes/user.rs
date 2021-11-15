use actix_web::http::StatusCode;
use actix_web::{post, web};
use serde::Deserialize;

use crate::config::{AppState, DbPool};
use crate::db::user;
use crate::error::{Res, ResBody, ResErr};

#[derive(Deserialize)]
pub struct NewUser {
    username: String,
    passwd: String,
    avatar_url: Option<String>,
}

#[post("/")]
pub async fn create(pool: web::Data<DbPool>, state: web::Data<AppState>, new_user: web::Json<NewUser>) -> Res {
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
    .map_err(|e| ResErr::new_err(StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

    ResBody::new("ok".to_string(), user)
}
