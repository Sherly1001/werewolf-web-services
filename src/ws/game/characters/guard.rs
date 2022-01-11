use actix::Addr;

use crate::ws::{ChatServer, game::{cmds::BotMsg, text_templates as ttp}};

use super::{player::{PlayerStatus, Player}, roles};


pub struct Guard {
    pub user_id: i64,
    pub personal_channel: i64,
    pub status: PlayerStatus,
    pub addr: Addr<ChatServer>,
    pub mana: bool,
}

impl Guard {
    pub fn new(user_id: i64, addr: Addr<ChatServer>) -> Self {
        Self {
            user_id,
            personal_channel: 0,
            status: PlayerStatus::Alive,
            addr,
            mana: false,
        }
    }
}

impl Player for Guard {
    fn get_role_name(&self) -> &'static str {
        roles::GUARD
    }

    fn get_status(&mut self) -> &mut PlayerStatus {
        &mut self.status
    }

    fn get_playerid(&mut self) -> &mut i64 {
        &mut self.user_id
    }

    fn get_channelid(&mut self) -> &mut i64 {
        &mut self.personal_channel
    }

    fn get_addr(&mut self) -> &mut Addr<ChatServer> {
        &mut self.addr
    }

    fn on_action(&self, bot_prefix: &str) {
        self.addr.do_send(BotMsg {
            channel_id: self.personal_channel,
            msg: ttp::guard_action(bot_prefix),
            reply_to: None,
        });
    }

    fn on_night(&mut self, _num_day: u16) {
        self.mana = true;
    }

    fn get_power(&mut self) -> bool {
        true
    }

    fn get_mana(&mut self) -> bool {
        self.mana
    }

    fn set_mana(&mut self, mana: bool) {
        self.mana = mana;
    }
}
