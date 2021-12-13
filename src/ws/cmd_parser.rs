use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::models::channel::{DispChatMsg, ChannelPermission};
use crate::models::user::UserDisplay;

#[derive(Debug, Serialize, Deserialize)]
pub enum Cmd {
    SendReq {
        channel_id: String,
        message: String,
        reply_to: Option<String>,
    },
    SendRes {
        message_id: String,
        reply_to: Option<String>,
    },
    BroadCastMsg {
        user_id: String,
        channel_id: String,
        message_id: String,
        message: String,
        reply_to: Option<String>,
    },
    GetMsg {
        channel_id: String,
        offset: Option<usize>,
        limit: Option<usize>,
    },
    GetMsgRes {
        channel_id: String,
        messages: Vec<DispChatMsg>,
    },
    GetUserInfo {
        user_id: Option<String>,
    },
    GetUserInfoRes(UserDisplay),
    GetUsers,
    GetUsersRes(Vec<UserDisplay>),
    GetPers {
        channel_id: Option<String>,
    },
    GetPersRes(HashMap<String, ChannelPermission>),
    UserOnline(UserDisplay),
    UserOffline(UserDisplay),
    Error(String),
}

impl Cmd {
    pub fn from_string(string: &str) -> serde_json::Result<Self> {
        from_str::<Self>(string)
    }

    pub fn to_string(&self) -> String {
        to_string(self).unwrap()
    }
}
