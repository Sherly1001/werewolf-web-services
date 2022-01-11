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
    fn on_action(&self, bot_prefix: &str) {}
    fn on_day(&mut self) {}
    fn on_night(&mut self) {}

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

    fn on_start_game(&mut self) {
        self.get_addr().clone().do_send(BotMsg {
            channel_id: *self.get_channelid(),
            msg: ttp::on_start_game(self.get_role_name()),
            reply_to: None,
        });
    }

    fn on_phase(&mut self, is_day: bool) {
        let stt = self.get_status();
        if *stt == PlayerStatus::Protected {
            *stt = PlayerStatus::Alive;
        }

        if is_day {
            self.on_day();
        } else {
            self.on_night();
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
