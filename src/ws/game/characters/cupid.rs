use super::player::{PlayerStatus, Player};


#[derive(Debug)]
pub struct Cupid {
    pub user_id: i64,
    pub personal_channel: i64,
    pub status: PlayerStatus,
}

impl Cupid {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            personal_channel: 0,
            status: PlayerStatus::Alive,
        }
    }
}

impl Player for Cupid {
    fn get_role_name(&self) -> &'static str {
        "Cupid"
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
