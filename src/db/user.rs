use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Serialize, Deserialize};

use crate::models::user::User;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name="users"]
struct NewUser<'a> {
    id: i64,
    username: &'a str,
    #[column_name="hash_passwd"]
    passwd: &'a str,
    avatar_url: Option<&'a str>,
}

pub fn create(
    conn: &PgConnection,
    id: i64,
    username: &str,
    passwd: &str,
    avatar_url: Option<&str>,
) -> Result<User, diesel::result::Error> {
    let new_user = NewUser {
        id,
        username,
        passwd,
        avatar_url,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .get_result(conn)
}
