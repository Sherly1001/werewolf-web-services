use diesel::prelude::*;
use rocket::serde::Deserialize;
use diesel::result::{DatabaseErrorKind, Error};
use rocket_sync_db_pools::diesel::PgConnection;
use crypto::scrypt::{scrypt_check, scrypt_simple, ScryptParams};

use crate::models::user::User;
use crate::schema::users;

pub enum UserCreationError {
    DuplicatedUsername,
}

impl From<Error> for UserCreationError {
    fn from(err: Error) -> UserCreationError {
        if let Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) = &err {
            match info.constraint_name() {
                Some("users_pkey") => return UserCreationError::DuplicatedUsername,
                _ => {},
            }
        }
        panic!("Error creating user: {:?}", err)
    }
}

#[derive(Insertable, Deserialize)]
#[table_name="users"]
struct NewUser<'a> {
    username: &'a str,
    hash_passwd: &'a str,
    avatar_url: Option<&'a str>,
}

pub fn create(
    conn: &PgConnection,
    username: &str,
    passwd: &str,
    avatar_url: Option<&str>,
) -> Result<User, UserCreationError> {
    // see https://blog.filippo.io/the-scrypt-parameters
    let hash_passwd = &scrypt_simple(passwd, &ScryptParams::new(14, 8, 1))
        .expect("hash error");

    let user = &NewUser {
        username,
        avatar_url,
        hash_passwd,
    };

    diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
        .map_err(Into::into)
}

pub fn login(conn: &PgConnection, username: &str, passwd: &str) -> Option<User> {
    let user = users::table
        .filter(users::username.eq(username))
        .get_result::<User>(conn)
        .map_err(|err| eprintln!("login_user: {}", err))
        .ok()?;

    let passwd_matched = scrypt_check(passwd, &user.hash_passwd).map_err(|err| {
        eprintln!("login_user: scrypt_check: {}", err);
    }).ok()?;

    if passwd_matched {
        Some(user)
    } else {
        eprintln!("login attempt for '{}' failed: password doesn't match", username);
        None
    }
}

pub fn find(conn: &PgConnection, username: &str) -> Option<User> {
    users::table.filter(users::username.eq(username))
        .get_result::<User>(conn)
        .ok()
}

pub fn get_all(conn: &PgConnection) -> Vec<User> {
    users::table.get_results(conn)
        .unwrap_or_default()
}
