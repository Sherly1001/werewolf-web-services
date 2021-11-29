use diesel::{PgConnection, r2d2::{ConnectionManager, PooledConnection}};

use crate::{config::DbPool, db::channel::get_messages};
use crate::db::channel::{get_pers, send_message};
use crate::models::channel::{ChatLine, DispChatMsg};

use super::ChatServer;


pub fn send_msg(
    srv: &mut ChatServer,
    user_id: i64,
    channel_id: i64,
    message: String,
) -> Result<ChatLine, String> {
    let conn = get_conn(srv.db_pool.clone());

    let pers = get_pers(&conn, user_id, channel_id)
        .map_err(|err| err.to_string())?;

    if !pers.readable || !pers.sendable {
        return Err(
            "don't have permission to send message to this channel".to_string()
        );
    }

    let id = srv.app_state.id_generatator.lock().unwrap().real_time_generate();
    send_message(&conn, id, user_id, channel_id, message)
        .map_err(|err| err.to_string())
}

pub fn get_msg(
    srv: &mut ChatServer,
    channel_id: i64,
    offset: i64,
    limit: i64,
) -> Result<Vec<DispChatMsg>, String> {
    let conn = get_conn(srv.db_pool.clone());
    get_messages(&conn, channel_id, offset, limit)
        .map(|msg| msg.iter().map(|m| m.to_display_msg()).collect())
        .map_err(|err| err.to_string())
}



fn get_conn(pool: DbPool) -> PooledConnection<ConnectionManager<PgConnection>> {
    loop {
        match pool.get_timeout(std::time::Duration::from_secs(3)) {
            Ok(conn) => break conn,
            _ => continue,
        }
    }
}
