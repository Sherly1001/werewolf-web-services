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
        channel_id: String,
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
    GameEvent(GameEvent),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    UserJoin(String),
    UserLeave(String),
    UserStart(String),
    UserStop(String),
    UserNext(String),
    UserVote {
        user_id: String,
        vote_for: String,
    },
    PlayerDied(String),
    PlayerReborn(String),
    NewPhase {
        num_day: u16,
        is_day: bool,
    },
    StartGame,
    EndGame {
        winner: String,
    },
    StopGame,
}

impl Cmd {
    pub fn from_string(string: &str) -> serde_json::Result<Self> {
        from_str::<Self>(string)
    }

    pub fn to_string(&self) -> String {
        to_string(self).unwrap()
    }
}
