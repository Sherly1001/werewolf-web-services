use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::models::channel::DispChatMsg;

#[derive(Debug, Serialize, Deserialize)]
pub enum Cmd {
    SendReq {
        channel_id: String,
        message: String,
    },
    SendRes {
        message_id: String,
    },
    BroadCastMsg {
        user_id: String,
        channel_id: String,
        message_id: String,
        message: String,
    },
    GetMsg {
        channel_id: String,
        offset: usize,
        limit: usize,
    },
    GetMsgRes {
        channel_id: String,
        messages: Vec<DispChatMsg>,
    },
}

impl Cmd {
    pub fn from_string(string: &str) -> serde_json::Result<Self> {
        from_str::<Self>(string)
    }

    pub fn to_string(&self) -> String {
        to_string(self).unwrap()
    }
}
