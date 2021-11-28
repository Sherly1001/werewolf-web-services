use crypto::scrypt::scrypt_check;
use crypto::scrypt::{scrypt_simple, ScryptParams};
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::users;

use super::channel::set_pers;

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
struct NewUser<'a> {
    id: i64,
    username: &'a str,
    hash_passwd: &'a str,
    avatar_url: Option<&'a str>,
}

pub fn create(
    conn: &PgConnection,
    id: i64,
    rules_id: i64,
    lobby_id: i64,
    username: &str,
    passwd: &str,
    avatar_url: Option<&str>,
) -> QueryResult<User> {
    let hash_passwd = &scrypt_simple(passwd, &ScryptParams::new(14, 8, 1)).expect("hash error");

    let new_user = NewUser {
        id,
        username,
        hash_passwd,
        avatar_url,
    };

    let user = diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(conn)?;

    set_pers(conn, rules_id, user.id, 0, true, false)?;
    set_pers(conn, lobby_id, user.id, 1, true, true)?;

    Ok(user)
}

pub fn login(conn: &PgConnection, username: &str, passwd: &str) -> Result<User, &'static str> {
    let user = users::table
        .filter(users::username.eq(username))
        .get_result::<User>(conn)
        .map_err(|_| "login failed")?;

    scrypt_check(passwd, &user.hash_passwd).and_then(|rs| {
        if rs {
            Ok(user)
        } else {
            Err("login failed")
        }
    })
}

pub fn get_all(conn: &PgConnection) -> QueryResult<Vec<User>> {
    users::table.get_results::<User>(conn)
        .map(|mut users| {
            users.sort_by(|a, b| {
                a.username.cmp(&b.username)
            });
            users
        })
}

pub fn get_info(conn: &PgConnection, user_id: i64) -> QueryResult<User> {
    users::table.find(user_id)
        .get_result::<User>(conn)
}
