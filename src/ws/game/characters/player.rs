#[derive(PartialEq, Eq, Debug)]
pub enum PlayerStatus {
    Alive,
    Killed,
    Protected,
}

pub trait Player: std::fmt::Debug {
    fn get_role_name(&self) -> &'static str;
    fn get_status(&mut self) -> &mut PlayerStatus;
    fn get_playerid(&mut self) -> &mut i64;
    fn get_channelid(&mut self) -> &mut i64;
    fn on_day(&mut self);
    fn on_night(&mut self);
    fn on_start_game(&mut self);
    fn on_end_game(&mut self);

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
