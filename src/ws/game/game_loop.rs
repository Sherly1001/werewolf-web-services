use std::collections::{HashMap, HashSet};
use std::time::Duration;

use actix::Arbiter;

use crate::db;

use super::characters::player::PlayerStatus;
use super::characters::roles;
use super::cmds::{BotMsg, UpdatePers};
use super::game::GameChannel;
use super::{text_templates as ttp, Game};

use super::game::get_conn;

pub struct GameLoop {
    game: Game,
}

#[allow(unused)]
struct CurrentState {
    is_day: bool,
    num_day: u16,
    alive: Vec<i64>,
    died: Vec<i64>,
    gameplay: i64,
    werewolf: i64,
    cemetery: i64,
}

impl std::ops::Deref for GameLoop {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl GameLoop {
    pub async fn new(game: Game) {
        let game = Self {
            game,
        };
        game.run().await;
    }

    pub async fn run(&self) {
        let next = self.info.lock().unwrap().next_flag.clone();

        let gameplay = *self.info.lock().unwrap()
            .channels.get(&GameChannel::GamePlay).unwrap();
        let werewolf = *self.info.lock().unwrap()
            .channels.get(&GameChannel::WereWolf).unwrap();
        let cemetery = *self.info.lock().unwrap()
            .channels.get(&GameChannel::Cemetery).unwrap();

        let bot_prefix = self.bot_prefix.clone();

        let (alive, _died) = self.info.lock().unwrap().get_alives();
        for (&uid, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_start_game(&bot_prefix);
            let role = player.get_role_name();
            if role == roles::WEREWOLF || role == roles::SUPERWOLF {
                self.addr.do_send(BotMsg {
                    channel_id: werewolf,
                    msg: ttp::new_wolf(uid),
                    reply_to: None,
                });
            } else if role == roles::CUPID {
                self.addr.do_send(BotMsg {
                    channel_id: *player.get_channelid(),
                    msg: ttp::cupid_action(&bot_prefix),
                    reply_to: None,
                });
                self.addr.do_send(BotMsg {
                    channel_id: *player.get_channelid(),
                    msg: ttp::player_list(&alive, true),
                    reply_to: None,
                });
            }
        }

        while !self.info.lock().unwrap().is_ended {
            let is_day = self.info.lock().unwrap().is_day;
            let num_day = self.info.lock().unwrap().num_day;
            let (alive, died) = self.info.lock().unwrap().get_alives();

            let state = CurrentState {
                is_day,
                num_day,
                alive,
                died,
                gameplay,
                werewolf,
                cemetery,
            };

            println!("start");

            self.addr.do_send(BotMsg {
                channel_id: gameplay,
                msg: ttp::new_phase(&bot_prefix, num_day, is_day),
                reply_to: None,
            });

            if is_day {
                self.do_start_day(&state);
            } else {
                self.do_start_night(&state);
            }

            for (_, player) in self.info.lock().unwrap().players.iter_mut() {
                player.on_phase(num_day, is_day);
            }

            self.start_timmer();
            next.wait().await;

            if is_day {
                self.do_end_day(&state);
            } else {
                self.do_end_night(&state);
            }

            println!("stop");

            if !is_day { self.info.lock().unwrap().num_day += 1; }
            self.info.lock().unwrap().is_day = !is_day;
        }

        for (_, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_end_game();
        }

        self.info.lock().unwrap().is_ended = true;
    }

    fn do_start_day(&self, state: &CurrentState) {
        println!("alive: {:?}", state.alive);
        self.addr.do_send(BotMsg {
            channel_id: state.gameplay,
            msg: ttp::player_list(&state.alive, true),
            reply_to: None,
        });

        for &user_id in state.alive.iter() {
            self.set_pers(user_id, state.gameplay, true, true);
        }
    }

    fn do_end_day(&self, state: &CurrentState) {
        let top_vote = get_top_vote(&mut self.info.lock().unwrap().vote_kill);

        if let Some((uid, _)) = top_vote {
            let mut info_lock = self.info.lock().unwrap();
            let player = info_lock.players.get_mut(&uid).unwrap();
            if player.get_killed() {
                if player.get_role_name() == roles::WEREWOLF
                    || player.get_role_name() == roles::SUPERWOLF {
                    self.set_pers(uid, state.werewolf, false, false);
                }
                self.set_pers(uid, state.gameplay, true, false);
                self.set_pers(uid, state.cemetery, true, true);
                self.addr.do_send(BotMsg {
                    channel_id: state.cemetery,
                    msg: ttp::after_death(uid),
                    reply_to: None,
                });
            }
        }

        for &user_id in state.alive.iter() {
            self.set_pers(user_id, state.gameplay, true, false);
        }

        self.addr.do_send(BotMsg {
            channel_id: state.gameplay,
            msg: ttp::execution(top_vote),
            reply_to: None,
        });
    }

    fn do_start_night(&self, state: &CurrentState) {
        self.addr.do_send(BotMsg {
            channel_id: state.werewolf,
            msg: ttp::before_wolf_action(&self.bot_prefix),
            reply_to: None,
        });
        self.addr.do_send(BotMsg {
            channel_id: state.werewolf,
            msg: ttp::player_list(&state.alive, true),
            reply_to: None,
        });

        let mut info_lock = self.info.lock().unwrap();
        if let Some((_, day)) = info_lock.guard_yesterday_target {
            if day != info_lock.num_day - 1 {
                info_lock.guard_yesterday_target = None;
            }
        }

        for (_uid, player) in info_lock.players.iter_mut() {
            player.on_action(&self.bot_prefix);
            if [roles::GUARD, roles::SEER, roles::WITCH]
                .contains(&player.get_role_name()) {
                let &mut personal_channel = player.get_channelid();
                self.addr.do_send(BotMsg {
                    channel_id: personal_channel,
                    msg: ttp::player_list(&state.alive, true),
                    reply_to: None,
                });
                if roles::WITCH == player.get_role_name() {
                    self.addr.do_send(BotMsg {
                        channel_id: personal_channel,
                        msg: ttp::player_list(&state.died, false),
                        reply_to: None,
                    });
                }
            }
        }
    }

    fn do_end_night(&self, state: &CurrentState) {
        let mut info_lock = self.info.lock().unwrap();
        if let Some((uid, _)) = get_top_vote(&mut info_lock.wolf_kill) {
            info_lock.night_pending_kill.insert(uid);
        }

        let mut killed = vec![];
        for user_id in info_lock.night_pending_kill.clone() {
            let player = info_lock.players.get_mut(&user_id).unwrap();
            if player.get_killed() {
                killed.push(user_id);
            }
        }
        info_lock.night_pending_kill = HashSet::new();

        self.addr.do_send(BotMsg {
            channel_id: state.gameplay,
            msg: ttp::list_killed(&killed),
            reply_to: None,
        });

        for uid in killed {
            let player = info_lock.players.get_mut(&uid).unwrap();
            let is_wolf = player.get_role_name() == roles::WEREWOLF
                || player.get_role_name() == roles::SUPERWOLF;

            self.set_pers(uid, state.gameplay, true, false);
            self.set_pers(uid, state.cemetery, true, true);
            if is_wolf { self.set_pers(uid, state.werewolf, false, false); }
            self.addr.do_send(BotMsg {
                channel_id: state.cemetery,
                msg: ttp::after_death(uid),
                reply_to: None,
            });
        }

        if let Some(uid) = info_lock.witch_reborn {
            info_lock.witch_reborn = None;
            let player = info_lock.players.get_mut(&uid).unwrap();
            *player.get_status() = PlayerStatus::Alive;
            let is_wolf = player.get_role_name() == roles::WEREWOLF
                || player.get_role_name() == roles::SUPERWOLF;

            self.set_pers(uid, state.cemetery, false, false);
            self.set_pers(uid, state.gameplay, true, false);
            if is_wolf { self.set_pers(uid, state.werewolf, true, true); }

            self.addr.do_send(BotMsg {
                channel_id: state.gameplay,
                msg: ttp::reborned(uid),
                reply_to: None,
            });
        }
    }

    fn start_timmer(&self) {
        let addr = self.addr.clone();
        let info = self.info.clone();

        let is_day = info.lock().unwrap().is_day;
        let num_day = info.lock().unwrap().num_day;
        let (daytime, nighttime, preiod) = info.lock().unwrap().timmer;
        let next = info.lock().unwrap().next_flag.clone();

        let gameplay = *info
            .lock()
            .unwrap()
            .channels
            .get(&GameChannel::GamePlay)
            .unwrap();

        let timecount = if is_day { daytime } else { nighttime };

        let fut = async move {
            for count in (1..timecount + 1).rev() {
                {
                    let lock = info.lock().unwrap();
                    if lock.is_ended || lock.is_stopped ||
                        lock.is_day != is_day || lock.num_day != num_day {
                        return;
                    }
                }

                if count % preiod == 0 || count <= 5 {
                    addr.do_send(BotMsg {
                        channel_id: gameplay,
                        msg: ttp::timeout(count),
                        reply_to: None,
                    });
                }

                actix::clock::delay_for(Duration::from_secs(1)).await;
            }

            next.wake();
        };

        Arbiter::spawn(fut);
    }

    fn set_pers(&self,
        user_id: i64,
        channel_id: i64,
        readable: bool,
        sendable: bool,
    ) {
        let conn = get_conn(self.db_pool.clone());
        let id = self.id_gen.lock().unwrap().real_time_generate();
        db::channel::set_pers(&conn, id, user_id,
            channel_id, readable, sendable).ok();
        self.addr.do_send(UpdatePers(user_id));
    }
}

fn get_top_vote(vote_list: &mut HashMap<i64, i64>) -> Option<(i64, u16)> {
    let mut h = HashMap::new();

    for (_, &uid) in vote_list.iter() {
        *h.entry(uid).or_insert(0) += 1;
    }
    *vote_list = HashMap::new();

    let mut vec = h.into_iter().collect::<Vec<(i64, u16)>>();
    vec.sort_by(|a, b| b.1.cmp(&a.1));

    if vec.len() == 1 || (vec.len() >= 2 && vec[0].1 > vec[1].1) {
        return Some(vec[0]);
    }

    return None;
}
