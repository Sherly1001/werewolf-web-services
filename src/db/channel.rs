use std::collections::HashMap;

use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::channel::ChannelPermission;
use crate::models::channel::{Channel, ChatLine, ChatMsg, UserChannelPermission};
use crate::schema::{channels, chat_lines, user_channel_permissions as ucp};

pub fn get_pers(
    conn: &PgConnection,
    user_id: i64,
    channel_id: i64,
) -> QueryResult<UserChannelPermission> {
    ucp::table
        .filter(ucp::user_id.eq(user_id))
        .filter(ucp::channel_id.eq(channel_id))
        .get_result(conn)
}

pub fn get_all_pers(
    conn: &PgConnection,
    user_id: i64,
) -> QueryResult<HashMap<String, ChannelPermission>> {
    ucp::table
        .filter(ucp::user_id.eq(user_id))
        .get_results::<UserChannelPermission>(conn)
        .map(|pers| {
            let mut map = HashMap::new();
            for u in pers.iter() {
                map.insert(u.channel_id.to_string(), ChannelPermission {
                    readable: u.readable,
                    sendable: u.sendable,
                });
            }
            map
        })
}

pub fn set_pers(
    conn: &PgConnection,
    id: i64,
    user_id: i64,
    channel_id: i64,
    readable: bool,
    sendable: bool,
) -> QueryResult<usize> {
    match get_pers(conn, user_id, channel_id) {
        Ok(pers) => {
            let pers = ucp::table.find(pers.id);
            diesel::update(pers)
                .set((
                    ucp::readable.eq(readable),
                    ucp::sendable.eq(sendable),
                ))
                .execute(conn)
        }
        Err(_) => {
            diesel::insert_into(ucp::table)
                .values(UserChannelPermission {
                    id,
                    user_id,
                    channel_id,
                    readable,
                    sendable,
                })
                .execute(conn)
        }
    }
}

#[allow(dead_code)]
pub fn create_channel(
    conn: &PgConnection,
    id: i64,
    channel_name: String,
) -> QueryResult<Channel> {
    diesel::insert_into(channels::table)
        .values(Channel {
            id,
            channel_name,
        })
        .get_result(conn)
}

pub fn send_message(
    conn: &PgConnection,
    id: i64,
    user_id: i64,
    channel_id: i64,
    message: String,
) -> QueryResult<ChatLine> {
    diesel::insert_into(chat_lines::table)
        .values(ChatLine {
            id,
            user_id,
            channel_id,
            message,
        })
        .get_result(conn)
}

pub fn get_messages(
    conn: &PgConnection,
    channel_id: i64,
    offset: i64,
    limit: i64,
) -> QueryResult<Vec<ChatMsg>> {
    chat_lines::table
        .select((chat_lines::id, chat_lines::message))
        .filter(chat_lines::channel_id.eq(channel_id))
        .order(chat_lines::id.desc())
        .offset(offset)
        .limit(limit)
        .get_results(conn)
}
