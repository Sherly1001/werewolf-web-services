use diesel::{PgConnection, QueryResult};
use diesel::prelude::*;

use crate::models::channel::Channel;
use crate::models::game::{GameChannel, GameUser};
use crate::models::user::User;
use crate::schema::{game_channels, game_users, channels, users};
use crate::{models::game::Game, schema::games};

use super::channel;

pub fn create(
    conn: &PgConnection,
    id: i64,
) -> QueryResult<Game> {
    diesel::insert_into(games::table)
        .values(&Game {
            id,
            is_stopped: false,
        })
        .get_result(conn)
}

pub fn set_stopped(
    conn: &PgConnection,
    game_id: i64,
) -> QueryResult<Game> {
    let filter = games::table.find(game_id);
    diesel::update(filter)
        .set(games::is_stopped.eq(true))
        .get_result(conn)
}

pub fn add_channel(
    conn: &PgConnection,
    id: i64,
    game_id: i64,
    channel_id: i64,
    channel_name: String,
) -> QueryResult<GameChannel> {
    channel::create_channel(conn, channel_id, channel_name)?;
    diesel::insert_into(game_channels::table)
        .values(&GameChannel {
            id,
            game_id,
            channel_id
        })
        .get_result(conn)
}

pub fn add_user(
    conn: &PgConnection,
    id: i64,
    game_id: i64,
    user_id: i64,
) -> QueryResult<GameUser> {
    diesel::insert_into(game_users::table)
        .values(&GameUser {
            id,
            game_id,
            user_id,
        })
        .get_result(conn)
}

pub fn remove_user(
    conn: &PgConnection,
    game_id: i64,
    user_id: i64,
) -> QueryResult<usize> {
    let filter = game_users::table
        .filter(game_users::game_id.eq(game_id))
        .filter(game_users::user_id.eq(user_id));
    diesel::delete(filter)
        .execute(conn)
}

#[allow(dead_code)]
pub fn get_channels(
    conn: &PgConnection,
    game_id: i64,
) -> QueryResult<Vec<Channel>> {
    game_channels::table
        .filter(game_channels::game_id.eq(game_id))
        .inner_join(channels::table)
        .select(channels::all_columns)
        .get_results::<Channel>(conn)
}

#[allow(dead_code)]
pub fn get_users(
    conn: &PgConnection,
    game_id: i64,
) -> QueryResult<Vec<User>> {
    game_users::table
        .filter(game_users::game_id.eq(game_id))
        .inner_join(users::table)
        .select(users::all_columns)
        .get_results::<User>(conn)
}

#[allow(dead_code)]
pub fn get_from_user(
    conn: &PgConnection,
    user_id: i64,
) -> QueryResult<Game> {
    game_users::table
        .filter(game_users::user_id.eq(user_id))
        .filter(games::is_stopped.eq(false))
        .inner_join(games::table)
        .select(games::all_columns)
        .get_result(conn)
}

#[allow(dead_code)]
pub fn get_from_channel(
    conn: &PgConnection,
    channel_id: i64,
) -> QueryResult<Game> {
    game_channels::table
        .filter(game_channels::channel_id.eq(channel_id))
        .filter(games::is_stopped.eq(false))
        .inner_join(games::table)
        .select(games::all_columns)
        .get_result(conn)
}
