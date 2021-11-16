use crypto::scrypt::scrypt_check;
use crypto::scrypt::{scrypt_simple, ScryptParams};
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::users;

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
    username: &str,
    passwd: &str,
    avatar_url: Option<&str>,
) -> Result<User, diesel::result::Error> {
    let hash_passwd = &scrypt_simple(passwd, &ScryptParams::new(14, 8, 1)).expect("hash error");

    let new_user = NewUser {
        id,
        username,
        hash_passwd,
        avatar_url,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .get_result(conn)
}

pub fn login(conn: &PgConnection, username: &str, passwd: &str) -> Option<User> {
    let user = users::table
        .filter(users::username.eq(username))
        .get_result::<User>(conn)
        .map_err(|err| println!("login err: {}", err))
        .ok()?;

    if scrypt_check(passwd, &user.hash_passwd)
        .map_err(|err| eprint!("login_user: scrypt_check err: {}", err))
        .ok()?
    {
        Some(user)
    } else {
        eprintln!(
            "login attempt for '{}' failed: password doesn't match",
            username
        );
        None
    }
}

pub fn get_all(conn: &PgConnection) -> Result<Vec<User>, diesel::result::Error> {
    users::table.get_results(conn)
}
