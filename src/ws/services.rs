use std::collections::HashMap;

use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};

use crate::config::DbPool;
use crate::db;
use crate::models::channel::{ChannelPermission, ChatLine, DispChatMsg};
use crate::models::user::{User, UserDisplay};

use super::ChatServer;

pub fn send_msg(
    srv: &ChatServer,
    user_id: i64,
    channel_id: i64,
    message: String,
    reply_to: Option<i64>,
) -> Result<ChatLine, String> {
    let conn = get_conn(srv.db_pool.clone());

    let pers = db::channel::get_pers(&conn, user_id, channel_id)
        .map_err(|err| err.to_string())?;

    if !pers.readable || !pers.sendable {
        return Err(
            "don't have permission to send message to this channel".to_string()
        );
    }

    let id = srv
        .app_state
        .id_generatator
        .lock()
        .unwrap()
        .real_time_generate();
    db::channel::send_message(&conn, id, user_id, channel_id, message, reply_to)
        .map_err(|err| err.to_string())
}

pub fn get_msg(
    srv: &ChatServer,
    channel_id: i64,
    offset: i64,
    limit: i64,
) -> Result<Vec<DispChatMsg>, String> {
    let conn = get_conn(srv.db_pool.clone());
    db::channel::get_messages(&conn, channel_id, offset, limit)
        .map(|msg| msg.iter().map(|m| m.to_display_msg()).collect())
        .map_err(|err| err.to_string())
}

pub fn get_info(srv: &ChatServer, user_id: i64) -> Result<UserDisplay, String> {
    let conn = get_conn(srv.db_pool.clone());
    db::user::get_info(&conn, user_id)
        .map(|u| u.to_display_user())
        .map_err(|err| err.to_string())
}

pub fn get_users(srv: &ChatServer) -> Result<Vec<UserDisplay>, String> {
    let conn = get_conn(srv.db_pool.clone());
    db::user::get_all(&conn)
        .map(|u| u.iter().map(|u| u.to_display_user()).collect())
        .map_err(|err| err.to_string())
}

pub fn get_pers(
    srv: &ChatServer,
    user_id: i64,
    channel_id: Option<i64>,
) -> Result<HashMap<String, ChannelPermission>, String> {
    let conn = get_conn(srv.db_pool.clone());

    if let None = channel_id {
        return db::channel::get_all_pers(&conn, user_id)
            .map_err(|err| err.to_string());
    }

    let channel_id = channel_id.unwrap();
    db::channel::get_pers(&conn, user_id, channel_id)
        .map_err(|err| err.to_string())
        .map(|per| {
            let mut hash = HashMap::new();
            hash.insert(
                per.channel_id.to_string(),
                ChannelPermission {
                    channel_name: per.channel_name,
                    readable: per.readable,
                    sendable: per.sendable,
                },
            );
            hash
        })
}

pub fn get_channel_users(srv: &ChatServer, channel_id: i64) -> Vec<User> {
    let conn = get_conn(srv.db_pool.clone());
    db::channel::get_users(&conn, channel_id).unwrap_or(vec![])
}

pub fn get_game_users(srv: &ChatServer, game_id: i64) -> Vec<User> {
    let conn = get_conn(srv.db_pool.clone());
    db::game::get_users(&conn, game_id).unwrap_or(vec![])
}

pub fn get_game_from_user(srv: &ChatServer, user_id: i64) -> Option<i64> {
    let conn = get_conn(srv.db_pool.clone());
    db::game::get_from_user(&conn, user_id).map(|g| g.id).ok()
}

fn get_conn(pool: DbPool) -> PooledConnection<ConnectionManager<PgConnection>> {
    loop {
        match pool.get_timeout(std::time::Duration::from_secs(3)) {
            Ok(conn) => break conn,
            _ => continue,
        }
    }
}
