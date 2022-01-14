use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::{channels, chat_lines, user_channel_permissions};

#[derive(Debug, Serialize, Deserialize, Insertable, Queryable)]
pub struct Channel {
    pub id: i64,
    pub channel_name: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, Queryable)]
pub struct ChatLine {
    pub id: i64,
    pub user_id: i64,
    pub channel_id: i64,
    pub message: String,
    pub reply_to: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, Queryable)]
pub struct UserChannelPermission {
    pub id: i64,
    pub user_id: i64,
    pub channel_id: i64,
    pub readable: bool,
    pub sendable: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct UserChannelPermissionDisplay {
    pub user_id: i64,
    pub channel_id: i64,
    pub channel_name: String,
    pub readable: bool,
    pub sendable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelPermission {
    pub channel_name: String,
    pub readable: bool,
    pub sendable: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct ChatMsg {
    pub message_id: i64,
    pub user_id: i64,
    pub message: String,
    pub reply_to: Option<i64>,
}

impl ChatMsg {
    pub fn to_display_msg(&self) -> DispChatMsg {
        DispChatMsg {
            message_id: self.message_id.to_string(),
            user_id: self.user_id.to_string(),
            message: self.message.clone(),
            reply_to: self.reply_to.map(|id| id.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DispChatMsg {
    pub message_id: String,
    pub user_id: String,
    pub message: String,
    pub reply_to: Option<String>,
}
