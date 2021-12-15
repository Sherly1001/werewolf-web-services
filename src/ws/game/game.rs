use actix::{Actor, Addr, Context};
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use snowflake::SnowflakeIdGenerator;

use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crate::{config::DbPool, db};

use crate::ws::ChatServer;

use super::characters::{player::Player, self};

pub struct Game {
    pub id: i64,
    pub channels: HashMap<GameChannel, i64>,
    pub users: HashSet<i64>,
    pub players: HashMap<i64, Box<dyn Player>>,
    pub is_started: bool,
    pub is_stopped: bool,
    pub is_day: bool,
    pub num_day: i16,

    pub vote_starts: HashSet<i64>,
    pub vote_stops: HashSet<i64>,
    pub addr: Addr<ChatServer>,
    pub db_pool: DbPool,
    pub id_gen: Arc<Mutex<SnowflakeIdGenerator>>,
    pub bot_id: i64,
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

        let mut s = Self {
            // game info
            id,
            channels: HashMap::new(),
            users: HashSet::new(),
            players: HashMap::new(),
            is_started: false,
            is_stopped: false,
            is_day: true,
            num_day: 0,

            // other info
            vote_starts: HashSet::new(),
            vote_stops: HashSet::new(),
            addr,
            db_pool,
            id_gen,
            bot_id,
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

        Some(Self {
            // game info
            id,
            channels,
            users,
            players: HashMap::new(),
            is_started: false,
            is_stopped: false,
            is_day: true,
            num_day: 0,

            // other info
            vote_starts: HashSet::new(),
            vote_stops: HashSet::new(),
            addr,
            db_pool,
            id_gen,
            bot_id,
        })
    }

    pub fn add_user(&mut self, user_id: i64) -> Result<(), String> {
        let &gameplay = self.channels.get(&GameChannel::GamePlay)
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

        self.users.insert(user_id);
        Ok(())
    }

    pub fn remove_user(&mut self, user_id: i64) -> Result<(), String> {
        let conn = get_conn(self.db_pool.clone());
        db::game::remove_user(&conn, self.id, user_id)
            .map_err(|err| err.to_string())?;

        let mut id_lock = self.id_gen.lock().unwrap();
        for (_, &channel_id) in self.channels.iter() {
            db::channel::set_pers(
                &conn, id_lock.real_time_generate(),
                user_id, channel_id, false, false,
            ).map_err(|err| err.to_string())?;
        }

        self.users.remove(&user_id);
        self.vote_starts.remove(&user_id);
        self.vote_stops.remove(&user_id);

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
        self.channels.insert(channel, channel_id);
        db::channel::set_pers(&conn, new_id2, self.bot_id, channel_id, true, true)
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<HashMap<String, usize>, String> {
        self.players = characters::rand_roles(
            &self.users.iter().collect::<Vec<&i64>>())?;

        let conn = get_conn(self.db_pool.clone());
        let mut id_lock = self.id_gen.lock().unwrap();
        let mut roles = HashMap::new();

        for (_, player) in self.players.iter_mut() {
            let role_name = player.get_role_name().to_string();
            *roles.entry(role_name).or_default() += 1;

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

            self.channels.insert(
                GameChannel::Personal(*player.get_playerid()), channel_id);
            *player.get_channelid() = channel_id;
        }

        self.is_started = true;
        Ok(roles)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.is_stopped { return Ok(()) }
        let conn = get_conn(self.db_pool.clone());
        db::game::delete(&conn, self.id)
            .map_err(|err| err.to_string())?;
        self.is_stopped = true;
        Ok(())
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl std::ops::Drop for Game {
    fn drop(&mut self) {
        if self.is_started {
            self.stop().ok();
        }
    }
}
