use actix::Addr;

use crate::ws::{ChatServer, game::{cmds::BotMsg, text_templates as ttp}};

use super::{player::{PlayerStatus, Player}, roles};


pub struct Witch {
    pub user_id: i64,
    pub personal_channel: i64,
    pub status: PlayerStatus,
    pub addr: Addr<ChatServer>,
    pub power: (bool, bool),
}

impl Witch {
    pub fn new(user_id: i64, addr: Addr<ChatServer>) -> Self {
        Self {
            user_id,
            personal_channel: 0,
            status: PlayerStatus::Alive,
            addr,
            power: (true, true),
        }
    }
}

impl Player for Witch {
    fn get_role_name(&self) -> &'static str {
        roles::WITCH
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
            msg: ttp::witch_action(bot_prefix),
            reply_to: None,
        });
    }

    fn get_power(&mut self) -> bool {
        self.power.0 || self.power.1
    }

    fn set_power(&mut self, power: bool) {
        self.power.0 = power
    }

    fn set_power2(&mut self, power: bool) {
        self.power.1 = power
    }

    fn on_use_power(&mut self) {}

    fn get_mana(&mut self) -> bool {
        true
    }
}
