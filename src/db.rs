use rocket_sync_db_pools::{database, diesel::PgConnection};

#[database("postgres")]
pub struct Conn(PgConnection);


pub mod users;
