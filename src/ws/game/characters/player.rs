use actix::Addr;

use crate::ws::{ChatServer, game::{cmds::BotMsg, text_templates as ttp}};

use super::roles;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PlayerStatus {
    Alive,
    Killed,
    Protected,
}

pub trait Player {
    fn get_role_name(&self) -> &'static str;
    fn get_status(&mut self) -> &mut PlayerStatus;
    fn get_playerid(&mut self) -> &mut i64;
    fn get_channelid(&mut self) -> &mut i64;
    fn get_addr(&mut self) -> &mut Addr<ChatServer>;

    #[allow(unused_variables)]
    fn set_power(&mut self, power: bool) {}
    #[allow(unused_variables)]
    fn set_power2(&mut self, power: bool) {}
    #[allow(unused_variables)]
    fn set_mana(&mut self, mana: bool) {}

    fn get_power(&mut self) -> bool {
        false
    }

    fn get_power2(&mut self) -> bool {
        false
    }

    fn on_use_power(&mut self) {
        self.set_power(false);
    }

    fn on_use_power2(&mut self) {
        self.set_power2(false);
    }

    fn get_mana(&mut self) -> bool {
        false
    }

    fn on_use_mana(&mut self) {
        self.set_mana(false);
    }

    #[allow(unused_variables)]
    fn on_action(&self, bot_prefix: &str) {}
    #[allow(unused_variables)]
    fn on_day(&mut self, num_day: u16) {}
    #[allow(unused_variables)]
    fn on_night(&mut self, num_day: u16) {}

    // Some(true) if werewolf, Some(false) if fox, None if otherwise
    fn on_seer(&self) -> Option<bool> {
        match self.get_role_name() {
            roles::WEREWOLF => Some(true),
            roles::FOX => Some(false),
            _ => None,
        }
    }

    fn is_alive(&self) -> bool {
        unsafe {
            let ptr = self as *const Self;
            let ptr = ptr as *mut Self;
            let ptr = &mut *ptr;
            *ptr.get_status() != PlayerStatus::Killed
        }
    }

    fn on_end_game(&mut self) {}

    #[allow(unused_variables)]
    fn on_start_game(&mut self, bot_prefix: &str) {
        self.get_addr().clone().do_send(BotMsg {
            channel_id: *self.get_channelid(),
            msg: ttp::on_start_game(self.get_role_name()),
            reply_to: None,
        });
    }

    fn on_phase(&mut self, num_day: u16, is_day: bool) {
        let stt = self.get_status();
        if *stt == PlayerStatus::Protected {
            *stt = PlayerStatus::Alive;
        }

        if is_day {
            self.on_day(num_day);
        } else {
            self.on_night(num_day);
        }
    }

    fn get_killed(&mut self) -> bool {
        let stt = self.get_status();
        if *stt == PlayerStatus::Protected { false }
        else {
            *stt = PlayerStatus::Killed;
            true
        }
    }

    fn get_protected(&mut self) {
        *self.get_status() = PlayerStatus::Protected;
    }
}
