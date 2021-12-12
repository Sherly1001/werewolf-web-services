use actix::{Message, Handler};

use super::Game;
use super::text_templates as ttp;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Join(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Leave(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Start(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Stop(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct BotMsg {
    pub channel_id: i64,
    pub msg: String,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct StopGame(pub i64);

impl Game {
    pub fn must_in_game(&self, user_id: i64) -> bool {
        if !self.users.contains(&user_id) {
            self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::not_in_game(),
            });
            return false;
        };
        return true;
    }
}

impl Handler<Join> for Game {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Self::Context) -> Self::Result {
        if self.users.contains(&msg.0) {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::aready_in_game(),
            });
        }

        if self.users.len() > 15 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::max_player(),
            });
        }

        self.add_user(msg.0);
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::user_join(msg.0, self.users.len()),
        });
    }
}

impl Handler<Leave> for Game {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.0) { return }

        self.remove_user(msg.0);

        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::user_leave(msg.0, self.users.len()),
        });
    }
}

impl Handler<Start> for Game {
    type Result = ();

    fn handle(&mut self, msg: Start, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.0) { return }

        if self.users.len() < 4 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::not_enough_player(self.users.len()),
            });
        }

        self.vote_starts.insert(msg.0);

        let numvote = self.vote_starts.len();
        let numplayer = self.users.len();
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::user_start(msg.0, numvote, numplayer),
            });
        }

        self.start();
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::start_game(),
        });
    }
}

impl Handler<Stop> for Game {
    type Result = ();

    fn handle(&mut self, msg: Stop, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.0) { return }

        self.vote_stops.insert(msg.0);

        let numvote = self.vote_stops.len();
        let numplayer = self.users.len();
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::user_stop(msg.0, numvote, numplayer),
            });
        }

        self.stop();
        self.addr.do_send(StopGame(self.id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::stop_game(),
        });
    }
}
