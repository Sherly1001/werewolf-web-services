use std::collections::HashSet;

use actix::{Message, Handler};

use crate::ws::cmd_parser::GameCmd;

use super::characters::roles;
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
pub struct Next {
    pub user_id: i64,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Vote {
    pub user_id: i64,
    pub vote_for: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Kill {
    pub user_id: i64,
    pub target: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Guard {
    pub user_id: i64,
    pub target: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Seer {
    pub user_id: i64,
    pub target: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Ship {
    pub user_id: i64,
    pub target1: Result<i64, u16>,
    pub target2: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Reborn {
    pub user_id: i64,
    pub target: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Curse {
    pub user_id: i64,
    pub target: Result<i64, u16>,
    pub msg_id: i64,
    pub channel_id: i64,
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
        if !self.info.lock().unwrap().users.contains(&user_id) {
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
        if self.info.lock().unwrap().users.contains(&msg.user_id) {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::aready_in_game(),
                reply_to: Some(msg.msg_id),
            });
        }

        if self.info.lock().unwrap().users.len() > 15 {
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
            msg: ttp::user_join(msg.user_id,
                self.info.lock().unwrap().users.len()),
            reply_to: Some(msg.msg_id),
        });
        self.addr.do_send(BotMsg {
            channel_id: *self.info.lock().unwrap()
                .channels.get(&GameChannel::GamePlay).unwrap(),
            msg: format!("Hi <@{}>.", msg.user_id),
            reply_to: None,
        });
    }
}

impl Handler<Leave> for Game {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        if self.info.lock().unwrap().is_started {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::leave_on_started(),
                reply_to: Some(msg.msg_id),
            })
        }

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
            msg: ttp::user_leave(msg.user_id,
                self.info.lock().unwrap().users.len()),
            reply_to: Some(msg.msg_id),
        });
        self.addr.do_send(BotMsg {
            channel_id: *self.info.lock().unwrap()
                .channels.get(&GameChannel::GamePlay).unwrap(),
            msg: format!("Bye <@{}>.", msg.user_id),
            reply_to: None,
        });
    }
}

impl Handler<Start> for Game {
    type Result = ();

    fn handle(&mut self, msg: Start, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        if self.info.lock().unwrap().is_started {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::game_is_started(),
                reply_to: Some(msg.msg_id),
            })
        }

        let num_users = self.info.lock().unwrap().users.len();
        if num_users < 4 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: ttp::not_enough_player(num_users),
                reply_to: Some(msg.msg_id),
            });
        }

        self.info.lock().unwrap().vote_starts.insert(msg.user_id);

        let numvote = self.info.lock().unwrap().vote_starts.len();
        let numplayer = self.info.lock().unwrap().users.len();
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
                channel_id: *self.info
                    .lock()
                    .unwrap()
                    .channels.get(&GameChannel::GamePlay)
                    .unwrap(),
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

        for &user in self.info.lock().unwrap().users.iter() {
            self.addr.do_send(UpdatePers(user));
        }
    }
}

impl Handler<Stop> for Game {
    type Result = ();

    fn handle(&mut self, msg: Stop, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.user_id, msg.msg_id) { return }

        if !self.info.lock().unwrap().is_ended {
            self.info.lock().unwrap().vote_stops.insert(msg.user_id);

            let numvote = self.info.lock().unwrap().vote_stops.len();
            let numplayer = self.info.lock().unwrap().users.len();
            if numvote * 3 < numplayer * 2 {
                return self.addr.do_send(BotMsg {
                    channel_id: 1,
                    msg: ttp::user_stop(msg.user_id, numvote, numplayer),
                    reply_to: Some(msg.msg_id),
                });
            }
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

        for &user in self.info.lock().unwrap().users.iter() {
            self.addr.do_send(UpdatePers(user));
        }
    }
}

impl Handler<Next> for Game {
    type Result = ();

    fn handle(&mut self, msg: Next, _: &mut Self::Context) -> Self::Result {
        let gameplay = *self.info
            .lock()
            .unwrap()
            .channels
            .get(&GameChannel::GamePlay)
            .unwrap();
        if !self.asset_cmd_in(
            Some(gameplay),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        self.info.lock().unwrap().vote_nexts.insert(msg.user_id);

        let numvote = self.info.lock().unwrap().vote_nexts.len();
        let numplayer = self.info.lock().unwrap().users.len();
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: gameplay,
                msg: ttp::user_next(msg.user_id, numvote, numplayer),
                reply_to: Some(msg.msg_id),
            });
        }

        self.info.lock().unwrap().vote_nexts = HashSet::new();

        let next = self.info.lock().unwrap().next_flag.clone();
        next.wake();
    }
}

impl Handler<Vote> for Game {
    type Result = ();

    fn handle(&mut self, msg: Vote, _: &mut Self::Context) -> Self::Result {
        let gameplay = *self.info.lock().unwrap()
            .channels.get(&GameChannel::GamePlay).unwrap();
        if !self.asset_cmd_in(
            Some(gameplay),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        let user_list = self.info.lock().unwrap().get_alives();
        let vote_user = get_from_target(&user_list, msg.vote_for, Some(true));
        if let Err(err) = vote_user {
            return self.addr.do_send(BotMsg {
                channel_id: gameplay,
                msg: err,
                reply_to: Some(msg.msg_id),
            });
        }
        let vote_user = vote_user.unwrap();

        self.info.lock().unwrap().vote_kill.insert(msg.user_id, vote_user);
        self.addr.do_send(BotMsg {
            channel_id: gameplay,
            msg: ttp::vote_kill(msg.user_id, vote_user),
            reply_to: Some(msg.msg_id),
        });
    }
}

impl Handler<Kill> for Game {
    type Result = ();

    fn handle(&mut self, msg: Kill, _ctx: &mut Self::Context) -> Self::Result {
        let werewolf = *self.info.lock().unwrap()
            .channels.get(&GameChannel::WereWolf).unwrap();
        if !asset_cmd(
            self,
            &[roles::WEREWOLF, roles::SUPERWOLF],
            Some(werewolf),
            Some(false),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        let user_list = self.info.lock().unwrap().get_alives();
        let target = get_from_target(&user_list, msg.target, Some(true));
        if let Err(err) = target {
            return self.addr.do_send(BotMsg {
                channel_id: msg.channel_id,
                msg: err,
                reply_to: Some(msg.msg_id),
            });
        }
        let target = target.unwrap();

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }

        self.info.lock().unwrap().wolf_kill.insert(msg.user_id, target);
        self.addr.do_send(BotMsg {
            channel_id: werewolf,
            msg: ttp::wolf_kill(msg.user_id, target),
            reply_to: Some(msg.msg_id),
        });
    }
}

impl Handler<Guard> for Game {
    type Result = ();

    fn handle(&mut self, msg: Guard, _ctx: &mut Self::Context) -> Self::Result {
        if !asset_cmd(
            self,
            &[roles::GUARD],
            None,
            Some(false),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        let user_list = self.info.lock().unwrap().get_alives();
        let target = get_from_target(&user_list, msg.target, Some(true));
        if let Err(err) = target {
            return self.addr.do_send(BotMsg {
                channel_id: msg.channel_id,
                msg: err,
                reply_to: Some(msg.msg_id),
            });
        }
        let target = target.unwrap();

        let personal_channel = *self.info.lock().unwrap()
            .players.get_mut(&msg.user_id).unwrap()
            .get_channelid();

        if let Some((old, _)) = self.info.lock().unwrap().guard_yesterday_target {
            if old == target {
                return self.addr.do_send(BotMsg {
                    channel_id: personal_channel,
                    msg: ttp::guard_yesterday_target(),
                    reply_to: Some(msg.msg_id),
                });
            }
        }

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }

        let mut info_lock = self.info.lock().unwrap();
        info_lock.players.get_mut(&target).unwrap().get_protected();
        info_lock.guard_yesterday_target = Some((target, info_lock.num_day));
        self.addr.do_send(BotMsg {
            channel_id: personal_channel,
            msg: ttp::guard_success(target),
            reply_to: Some(msg.msg_id),
        });
    }
}

impl Handler<Seer> for Game {
    type Result = ();

    fn handle(&mut self, msg: Seer, _ctx: &mut Self::Context) -> Self::Result {
        if !asset_cmd(
            self,
            &[roles::SEER],
            None,
            Some(false),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }
    }
}

impl Handler<Ship> for Game {
    type Result = ();

    fn handle(&mut self, msg: Ship, _ctx: &mut Self::Context) -> Self::Result {
        if !asset_cmd(
            self,
            &[roles::CUPID],
            None,
            None,
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }
    }
}

impl Handler<Reborn> for Game {
    type Result = ();

    fn handle(&mut self, msg: Reborn, _ctx: &mut Self::Context) -> Self::Result {
        if !asset_cmd(
            self,
            &[roles::WITCH],
            None,
            Some(false),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }

        self.info.lock().unwrap()
            .players.get_mut(&msg.user_id).unwrap()
            .set_power(false);
    }
}

impl Handler<Curse> for Game {
    type Result = ();

    fn handle(&mut self, msg: Curse, _ctx: &mut Self::Context) -> Self::Result {
        if !asset_cmd(
            self,
            &[roles::WITCH],
            None,
            Some(false),
            msg.user_id,
            msg.msg_id,
            msg.channel_id,
        ) { return }

        if !assert_use_skill(self, msg.user_id, msg.msg_id, msg.channel_id) {
            return;
        }

        self.info.lock().unwrap()
            .players.get_mut(&msg.user_id).unwrap()
            .set_power2(false);
    }
}

// must Some(true) if alive Some(false) if died
fn get_from_target(
    (alive, died): &(Vec<i64>, Vec<i64>),
    target: Result<i64, u16>,
    must: Option<bool>,
) -> Result<i64, String> {
    if let Err(idx) = target {
        let idx = idx as usize;
        if idx < 1 || idx > alive.len() {
            return Err(ttp::invalid_index(1, alive.len()));
        }
        return Ok(alive[idx - 1]);
    }

    let target = target.unwrap();

    if !alive.contains(&target) && !died.contains(&target) {
        return Err(ttp::player_not_in_game(target));
    }

    if Some(true) == must && died.contains(&target) {
        return Err(ttp::player_died());
    }

    if Some(false) == must && alive.contains(&target) {
        return Err(ttp::player_still_alive(target));
    }

    Ok(target)
}

fn asset_cmd(
    game: &Game,
    roles: &[&'static str],
    channel_id: Option<i64>,
    phase: Option<bool>,
    user_id: i64,
    msg_id: i64,
    msg_channel_id: i64,
) -> bool {
    if !game.asset_cmd_in(
        channel_id,
        user_id,
        msg_id,
        msg_channel_id,
    ) { return false }

    if roles.len() > 0 && roles.iter()
        .map(|r| game.assert_role(r, user_id)).all(|v| !v) {
        game.addr.do_send(BotMsg {
            channel_id: msg_channel_id,
            msg: ttp::invalid_author(),
            reply_to: Some(msg_id),
        });
        return false;
    }

    let is_day = game.info.lock().unwrap().is_day;
    if let Some(phase) = phase {
        if is_day != phase {
            game.addr.do_send(BotMsg {
                channel_id: msg_channel_id,
                msg: if phase {
                    ttp::invalid_daytime()
                } else {
                    ttp::invalid_nighttime()
                },
                reply_to: Some(msg_id),
            });
            return false;
        }
    }

    let mut info_lock = game.info.lock().unwrap();
    let player = info_lock.players.get_mut(&user_id).unwrap();

    if !player.is_alive() {
        game.addr.do_send(BotMsg {
            channel_id: msg_channel_id,
            msg: ttp::must_alive(),
            reply_to: Some(msg_id),
        });
        return false;
    }

    true
}

fn assert_use_skill(
    game: &Game,
    user_id: i64,
    msg_id: i64,
    msg_channel_id: i64,
) -> bool {
    let mut info_lock = game.info.lock().unwrap();
    let player = info_lock.players.get_mut(&user_id).unwrap();

    if !player.get_power() {
        game.addr.do_send(BotMsg {
            channel_id: msg_channel_id,
            msg: ttp::out_of_power(),
            reply_to: Some(msg_id),
        });
        return false;
    }
    player.on_use_power();

    if !player.get_mana() {
        game.addr.do_send(BotMsg {
            channel_id: msg_channel_id,
            msg: ttp::out_of_mana(),
            reply_to: Some(msg_id),
        });
        return false;
    }
    player.on_use_mana();

    true
}
