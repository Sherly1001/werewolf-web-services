use actix::{Message, Handler};

use crate::ws::cmd_parser::GameCmd;

use super::{Game, game::GameChannel};
use super::text_templates as ttp;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Join {
    pub user_id: i64,
    pub msg_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Leave {
    pub user_id: i64,
    pub msg_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Start {
    pub user_id: i64,
    pub msg_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Stop {
    pub user_id: i64,
    pub msg_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct BotMsg {
    pub channel_id: i64,
    pub msg: String,
    pub reply_to: Option<i64>,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct GameMsg {
    pub game_id: i64,
    pub cmd: GameCmd,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct StopGame(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct StartGame(pub i64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct UpdatePers(pub i64);

impl Game {
    pub fn must_in_game(&self, user_id: i64, msg_id: i64) -> bool {
        if !self.users.contains(&user_id) {
            self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::not_in_game(),
                reply_to: Some(msg_id),
            });
            return false;
        };
        return true;
    }
}

impl Handler<Join> for Game {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Self::Context) -> Self::Result {
        if self.users.contains(&msg.user_id) {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::aready_in_game(),
                reply_to: Some(msg.msg_id),
            });
        }

        if self.users.len() > 15 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::max_player(),
                reply_to: Some(msg.msg_id),
            });
        }

        if let Err(err) = self.add_user(msg.user_id) {
            self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: err,
                reply_to: Some(msg.msg_id),
            })
        }

        self.addr.do_send(UpdatePers(msg.user_id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::user_join(msg.user_id, self.users.len()),
            reply_to: Some(msg.msg_id),
        });
    }
}

impl Handler<Leave> for Game {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        if let Err(err) = self.remove_user(msg.user_id) {
            self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: err,
                reply_to: Some(msg.msg_id),
            })
        }

        self.addr.do_send(UpdatePers(msg.user_id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::user_leave(msg.user_id, self.users.len()),
            reply_to: Some(msg.msg_id),
        });
    }
}

impl Handler<Start> for Game {
    type Result = ();

    fn handle(&mut self, msg: Start, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        if self.is_started {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::game_is_started(),
                reply_to: Some(msg.msg_id),
            })
        }

        if self.users.len() < 4 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::not_enough_player(self.users.len()),
                reply_to: Some(msg.msg_id),
            });
        }

        self.vote_starts.insert(msg.user_id);

        let numvote = self.vote_starts.len();
        let numplayer = self.users.len();
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::user_start(msg.user_id, numvote, numplayer),
                reply_to: Some(msg.msg_id),
            });
        }

        match self.start() {
            Err(err) => return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: err,
                reply_to: Some(msg.msg_id),
            }),
            Ok(roles) => self.addr.do_send(BotMsg {
                channel_id: *self.channels.get(&GameChannel::GamePlay).unwrap(),
                msg: ttp::roles_list(&roles),
                reply_to: None,
            })
        }

        self.addr.do_send(StartGame(self.id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::start_game(),
            reply_to: Some(msg.msg_id),
        });

        for &user in self.users.iter() {
            self.addr.do_send(UpdatePers(user));
        }
    }
}

impl Handler<Stop> for Game {
    type Result = ();

    fn handle(&mut self, msg: Stop, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        self.vote_stops.insert(msg.user_id);

        let numvote = self.vote_stops.len();
        let numplayer = self.users.len();
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::user_stop(msg.user_id, numvote, numplayer),
                reply_to: Some(msg.msg_id),
            });
        }

        if let Err(err) = self.stop() {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: err,
                reply_to: Some(msg.msg_id),
            });
        }

        self.addr.do_send(StopGame(self.id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: ttp::stop_game(),
            reply_to: Some(msg.msg_id),
        });

        for &user in self.users.iter() {
            self.addr.do_send(UpdatePers(user));
        }
    }
}
