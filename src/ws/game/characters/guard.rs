use super::{player::{PlayerStatus, Player}, roles};


#[derive(Debug)]
pub struct Guard {
    pub user_id: i64,
    pub personal_channel: i64,
    pub status: PlayerStatus,
}

impl Guard {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            personal_channel: 0,
            status: PlayerStatus::Alive,
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

    fn on_day(&mut self) {}

    fn on_night(&mut self) {}

    fn on_start_game(&mut self) {}

    fn on_end_game(&mut self) {}
}
