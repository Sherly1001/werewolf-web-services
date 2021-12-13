use std::collections::HashMap;

use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::user::User;
use crate::models::channel::{
    Channel, ChatLine, ChatMsg, ChannelPermission,
    UserChannelPermission, UserChannelPermissionDisplay,
};
use crate::schema::{channels, chat_lines, users, user_channel_permissions as ucp};

pub fn get_pers(
    conn: &PgConnection,
    user_id: i64,
    channel_id: i64,
) -> QueryResult<UserChannelPermissionDisplay> {
    ucp::table
        .inner_join(channels::table)
        .select((
                ucp::user_id,
                ucp::channel_id,
                channels::channel_name,
                ucp::readable,
                ucp::sendable,
        ))
        .filter(ucp::user_id.eq(user_id))
        .filter(ucp::channel_id.eq(channel_id))
        .get_result(conn)
}

pub fn get_all_pers(
    conn: &PgConnection,
    user_id: i64,
) -> QueryResult<HashMap<String, ChannelPermission>> {
    ucp::table
        .inner_join(channels::table)
        .select((
                ucp::user_id,
                ucp::channel_id,
                channels::channel_name,
                ucp::readable,
                ucp::sendable,
        ))
        .filter(ucp::user_id.eq(user_id))
        .filter(ucp::readable.eq(true))
        .get_results::<UserChannelPermissionDisplay>(conn)
        .map(|pers| {
            let mut map = HashMap::new();
            for u in pers.iter() {
                map.insert(u.channel_id.to_string(), ChannelPermission {
                    channel_name: u.channel_name.clone(),
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
    diesel::insert_into(ucp::table)
        .values(&UserChannelPermission {
            id,
            user_id,
            channel_id,
            readable,
            sendable,
        })
        .on_conflict((ucp::user_id, ucp::channel_id))
        .do_update()
        .set((ucp::readable.eq(readable), ucp::sendable.eq(sendable)))
        .execute(conn)
}

#[allow(dead_code)]
pub fn create_channel(
    conn: &PgConnection,
    id: i64,
    channel_name: String,
) -> QueryResult<Channel> {
    diesel::insert_into(channels::table)
        .values(&Channel {
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
    reply_to: Option<i64>,
) -> QueryResult<ChatLine> {
    diesel::insert_into(chat_lines::table)
        .values(&ChatLine {
            id,
            user_id,
            channel_id,
            message,
            reply_to,
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
        .select((
                chat_lines::id,
                chat_lines::user_id,
                chat_lines::message,
                chat_lines::reply_to,
        ))
        .filter(chat_lines::channel_id.eq(channel_id))
        .order(chat_lines::id.desc())
        .offset(offset)
        .limit(limit)
        .get_results(conn)
}

pub fn get_users(
    conn: &PgConnection,
    channel_id: i64,
) -> QueryResult<Vec<User>> {
    ucp::table
        .filter(ucp::channel_id.eq(channel_id))
        .filter(ucp::readable.eq(true))
        .inner_join(users::table)
        .select(users::all_columns)
        .get_results(conn)
}
