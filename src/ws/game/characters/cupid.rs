use actix::Addr;

use crate::ws::{ChatServer, game::{cmds::BotMsg, text_templates as ttp}};

use super::{player::{PlayerStatus, Player}, roles};


pub struct Cupid {
    pub user_id: i64,
    pub personal_channel: i64,
    pub status: PlayerStatus,
    pub addr: Addr<ChatServer>,
    pub power: bool,
}

impl Cupid {
    pub fn new(user_id: i64, addr: Addr<ChatServer>) -> Self {
        Self {
            user_id,
            personal_channel: 0,
            status: PlayerStatus::Alive,
            addr,
            power: true,
        }
    }
}

impl Player for Cupid {
    fn get_role_name(&self) -> &'static str {
        roles::CUPID
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

    fn on_day(&mut self, num_day: u16) {
        if num_day > 0 && self.power {
            self.power = false;
            self.addr.do_send(BotMsg {
                channel_id: self.personal_channel,
                msg: ttp::cupid_out_of_power(),
                reply_to: None,
            });
        }
    }

    fn get_power(&mut self) -> bool {
        self.power
    }

    fn set_power(&mut self, power: bool) {
        self.power = power;
    }

    fn get_mana(&mut self) -> bool {
        true
    }
}
