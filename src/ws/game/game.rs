use actix::{Actor, Addr, Context};
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use snowflake::SnowflakeIdGenerator;

use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crate::{config::DbPool, db};

use crate::ws::ChatServer;

pub struct Game {
    pub id: i64,
    pub channels: HashMap<GameChannel, i64>,
    pub users: HashSet<i64>,
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

#[derive(PartialEq, Eq, Hash)]
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

        s.add_channel(GameChannel::GamePlay, "gameplay".to_string());
        s.add_channel(GameChannel::WereWolf, "werewolf".to_string());
        s.add_channel(GameChannel::Cemetery, "cemetery".to_string());

        s
    }

    pub fn add_user(&mut self, user_id: i64) {
        let &gameplay = self.channels.get(&GameChannel::GamePlay).unwrap();
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
    }

    pub fn remove_user(&mut self, user_id: i64) {
        let conn = get_conn(self.db_pool.clone());
        db::game::remove_user(&conn, self.id, user_id).unwrap();

        let mut id_lock = self.id_gen.lock().unwrap();
        for (_, &channel_id) in self.channels.iter() {
            db::channel::set_pers(
                &conn, id_lock.real_time_generate(),
                user_id, channel_id, false, false,
            ).unwrap();
        }

        self.users.remove(&user_id);
    }

    pub fn add_channel(&mut self, channel: GameChannel, channel_name: String) {
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
            .unwrap();
        self.channels.insert(channel, channel_id);
        db::channel::set_pers(&conn, new_id2, self.bot_id, channel_id, true, true)
            .unwrap();
    }

    pub fn start(&mut self) {
        self.is_started = true;
    }

    pub fn stop(&mut self) {
        if self.is_stopped { return }
        let conn = get_conn(self.db_pool.clone());
        db::game::delete(&conn, self.id).unwrap();
        self.is_stopped = true;
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl std::ops::Drop for Game {
    fn drop(&mut self) {
        self.stop();
    }
}
