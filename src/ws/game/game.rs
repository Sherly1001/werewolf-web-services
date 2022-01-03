use actix::{Actor, Addr, Context};
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use snowflake::SnowflakeIdGenerator;

use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};
use std::pin::Pin;

use std::future::Future;
use std::task::{self, Poll, Waker};


#[derive(Clone, Debug)]
pub struct NextFut {
    next: Arc<Mutex<bool>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl NextFut {
    pub fn new() -> Self {
        Self {
            next: Arc::new(Mutex::new(false)),
            waker: Arc::new(Mutex::new(None)),
        }
    }

    pub fn wait(&self) -> NextFut {
        NextFut {
            next: self.next.clone(),
            waker: self.waker.clone(),
        }
    }

    pub fn wake(&self) {
        *self.next.lock().unwrap() = true;
        let waker = self.waker.lock().unwrap().clone();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl Future for NextFut {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if *self.next.lock().unwrap() {
            *self.next.lock().unwrap() = false;
            *self.waker.lock().unwrap() = None;
            Poll::Ready(())
        } else {
            *self.waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

use crate::{config::DbPool, db};
use crate::ws::{ChatServer, game::{cmds::BotMsg, text_templates as ttp}};

use super::characters::{player::Player, self, roles};

pub struct GameInfo {
    pub channels: HashMap<GameChannel, i64>,
    pub users: HashSet<i64>,
    pub players: HashMap<i64, Box<dyn Player>>,
    pub is_started: bool,
    pub is_ended: bool,
    pub is_stopped: bool,
    pub is_day: bool,
    pub num_day: i16,

    pub vote_starts: HashSet<i64>,
    pub vote_stops: HashSet<i64>,
    pub vote_nexts: HashSet<i64>,

    pub next_flag: NextFut,
}

impl GameInfo {
    pub fn new(
        channels: HashMap<GameChannel, i64>,
        users: HashSet<i64>,
    ) -> Self {
        Self {
            channels,
            users,
            players: HashMap::new(),
            is_started: false,
            is_ended: false,
            is_stopped: false,
            is_day: true,
            num_day: 0,

            vote_starts: HashSet::new(),
            vote_stops: HashSet::new(),
            vote_nexts: HashSet::new(),

            next_flag: NextFut::new(),
        }
    }
}

pub struct Game {
    pub id: i64,
    pub addr: Addr<ChatServer>,
    pub db_pool: DbPool,
    pub id_gen: Arc<Mutex<SnowflakeIdGenerator>>,
    pub bot_id: i64,
    pub info: Arc<Mutex<GameInfo>>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum GameChannel {
    GamePlay,
    WereWolf,
    Cemetery,
    Personal(i64),
}

fn get_conn(pool: DbPool) -> PooledConnection<ConnectionManager<PgConnection>> {
    loop {
        match pool.get_timeout(std::time::Duration::from_secs(3)) {
            Ok(conn) => break conn,
            _ => continue,
        }
    }
}

impl Game {
    pub fn new(
        id: i64,
        addr: Addr<ChatServer>,
        db_pool: DbPool,
        id_gen: Arc<Mutex<SnowflakeIdGenerator>>,
        bot_id: i64,
    ) -> Self {
        let conn = get_conn(db_pool.clone());
        db::game::create(&conn, id).unwrap();

        let info = Arc::new(Mutex::new(GameInfo::new(
            HashMap::new(),
            HashSet::new(),
        )));

        let mut s = Self {
            id,
            addr,
            db_pool,
            id_gen,
            bot_id,
            info,
        };

        s.add_channel(GameChannel::GamePlay, "gameplay".to_string()).unwrap();
        s.add_channel(GameChannel::WereWolf, "werewolf".to_string()).unwrap();
        s.add_channel(GameChannel::Cemetery, "cemetery".to_string()).unwrap();

        s
    }

    pub fn load_from_db(
        addr: Addr<ChatServer>,
        db_pool: DbPool,
        id_gen: Arc<Mutex<SnowflakeIdGenerator>>,
        bot_id: i64,
    ) -> Option<Self> {
        let conn = get_conn(db_pool.clone());
        let id = db::game::get(&conn)?.id;
        let channels = db::game::get_channels(&conn, id).ok()?;
        let users = db::game::get_users(&conn, id).ok()?;

        let users = users.iter().map(|u| u.id).collect();
        let channels = channels.iter()
            .map(|cl| {
                match cl.channel_name.as_str() {
                    "gameplay" => Some((GameChannel::GamePlay, cl.id)),
                    "werewolf" => Some((GameChannel::WereWolf, cl.id)),
                    "cemetery" => Some((GameChannel::Cemetery, cl.id)),
                    _ => return None,
                }
            })
            .collect::<Option<HashMap<GameChannel, i64>>>()?;


        let info = Arc::new(Mutex::new(GameInfo::new(
            channels,
            users,
        )));

        Some(Self {
            id,
            addr,
            db_pool,
            id_gen,
            bot_id,
            info,
        })
    }

    pub fn add_user(&mut self, user_id: i64) -> Result<(), String> {
        let mut info = self.info.lock().unwrap();
        let &gameplay = info.channels.get(&GameChannel::GamePlay)
            .ok_or("can't get gameplay".to_string())?;
        let new_id1;
        let new_id2;

        {
            let mut id_lock = self.id_gen.lock().unwrap();
            new_id1 = id_lock.real_time_generate();
            new_id2 = id_lock.real_time_generate();
        }

        let conn = get_conn(self.db_pool.clone());
        db::game::add_user(&conn, new_id1, self.id, user_id).unwrap();
        db::channel::set_pers(&conn, new_id2, user_id, gameplay, true, true)
            .unwrap();

        info.users.insert(user_id);
        Ok(())
    }

    pub fn remove_user(&mut self, user_id: i64) -> Result<(), String> {
        let mut info = self.info.lock().unwrap();

        let conn = get_conn(self.db_pool.clone());
        db::game::remove_user(&conn, self.id, user_id)
            .map_err(|err| err.to_string())?;

        let mut id_lock = self.id_gen.lock().unwrap();
        for (_, &channel_id) in info.channels.iter() {
            db::channel::set_pers(
                &conn, id_lock.real_time_generate(),
                user_id, channel_id, false, false,
            ).map_err(|err| err.to_string())?;
        }

        info.users.remove(&user_id);
        info.vote_starts.remove(&user_id);
        info.vote_stops.remove(&user_id);

        Ok(())
    }

    pub fn add_channel(
        &mut self,
        channel: GameChannel,
        channel_name: String,
    ) -> Result<(), String> {
        let conn = get_conn(self.db_pool.clone());

        let channel_id;
        let new_id1;
        let new_id2;
        {
            let mut id_lock = self.id_gen.lock().unwrap();
            channel_id = id_lock.real_time_generate();
            new_id1 = id_lock.real_time_generate();
            new_id2 = id_lock.real_time_generate();
        }

        db::game::add_channel(&conn, new_id1, self.id, channel_id, channel_name)
            .map_err(|err| err.to_string())?;
        self.info.lock().unwrap().channels.insert(channel, channel_id);
        db::channel::set_pers(&conn, new_id2, self.bot_id, channel_id, true, true)
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn start(
        &mut self,
    ) -> Result<HashMap<String, usize>, String> {
        let mut info = self.info.lock().unwrap();

        let mut players = characters::rand_roles(
            &info.users.iter().collect::<Vec<&i64>>(),
            self.addr.clone(),
        )?;

        let conn = get_conn(self.db_pool.clone());
        let mut id_lock = self.id_gen.lock().unwrap();
        let mut roles = HashMap::new();

        for (_, player) in players.iter_mut() {
            let role_name = player.get_role_name();
            *roles.entry(role_name.to_string()).or_default() += 1;

            let new_id1 = id_lock.real_time_generate();
            let new_id2 = id_lock.real_time_generate();
            let new_id3 = id_lock.real_time_generate();
            let channel_id = id_lock.real_time_generate();

            db::game::add_channel(&conn, new_id1, self.id,
                channel_id, "personal channel".to_string())
                .map_err(|err| err.to_string())?;
            db::channel::set_pers(&conn, new_id2, *player.get_playerid(),
                channel_id, true, true)
                .map_err(|err| err.to_string())?;
            db::channel::set_pers(&conn, new_id3, self.bot_id,
                channel_id, true, true)
                .map_err(|err| err.to_string())?;

            info.channels.insert(
                GameChannel::Personal(*player.get_playerid()), channel_id);
            *player.get_channelid() = channel_id;

            if role_name == roles::WEREWOLF || role_name == roles::SUPERWOLF {
                let new_id1 = id_lock.real_time_generate();
                let werewolf = info.channels.get(&GameChannel::WereWolf)
                    .ok_or("not found werewolf channel".to_string())?;

                db::channel::set_pers(&conn, new_id1, *player.get_playerid(),
                    *werewolf, true, true)
                    .map_err(|err| err.to_string())?;
            }
        }

        info.players = players;

        let game_loop = GameLoop::new(self.info.clone(), self.addr.clone());
        actix::Arbiter::spawn(game_loop);

        info.is_started = true;
        Ok(roles)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let mut info = self.info.lock().unwrap();

        if info.is_stopped { return Ok(()) }
        let conn = get_conn(self.db_pool.clone());
        db::game::delete(&conn, self.id)
            .map_err(|err| err.to_string())?;
        info.is_stopped = true;
        Ok(())
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl std::ops::Drop for Game {
    fn drop(&mut self) {
        if self.info.lock().unwrap().is_started {
            self.stop().ok();
        }
    }
}


struct GameLoop {
    info: Arc<Mutex<GameInfo>>,
    addr: Addr<ChatServer>,
}

impl GameLoop {
    async fn new(info: Arc<Mutex<GameInfo>>, addr: Addr<ChatServer>) {
        let game = Self {
            info,
            addr,
        };
        game.run().await;
    }

    async fn run(&self) {
        for (_, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_start_game();
        }

        let next = self.info.lock().unwrap().next_flag.clone();

        while !self.info.lock().unwrap().is_ended {
            let is_day = self.info.lock().unwrap().is_day;
            let num_day = self.info.lock().unwrap().num_day;

            let gameplay = *self.info
                .lock()
                .unwrap()
                .channels
                .get(&GameChannel::GamePlay)
                .unwrap();

            println!("start");

            self.addr.do_send(BotMsg {
                channel_id: gameplay,
                msg: ttp::new_pharse(num_day, is_day),
                reply_to: None,
            });

            for (_, player) in self.info.lock().unwrap().players.iter_mut() {
                player.on_phase(is_day);
            }

            next.wait().await;

            println!("stop");

            if !is_day { self.info.lock().unwrap().num_day += 1; }
            self.info.lock().unwrap().is_day = !is_day;
            if self.info.lock().unwrap().num_day > 5 {
                self.info.lock().unwrap().is_ended = true;
            }
        }

        for (_, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_end_game();
        }

        self.info.lock().unwrap().is_ended = true;
    }
}
