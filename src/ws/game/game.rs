use actix::{Actor, Addr, Context};
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use snowflake::SnowflakeIdGenerator;

use std::{collections::HashSet, sync::{Arc, Mutex}};

use crate::{config::DbPool, db};

use crate::ws::ChatServer;

pub struct Game {
    pub id: i64,
    pub channels: Vec<i64>,
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
    ) -> Self {
        let conn = get_conn(db_pool.clone());
        db::game::create(&conn, id).unwrap();
        Self {
            // game info
            id,
            channels: vec![],
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
        }
    }

    pub fn add_user(&mut self, user_id: i64) {
        let conn = get_conn(self.db_pool.clone());
        let id = self.id_gen.lock().unwrap().real_time_generate();
        db::game::add_user(&conn, id, self.id, user_id).unwrap();

        self.users.insert(user_id);
    }

    pub fn remove_user(&mut self, user_id: i64) {
        let conn = get_conn(self.db_pool.clone());
        db::game::remove_user(&conn, self.id, user_id).unwrap();

        self.users.remove(&user_id);
    }

    pub fn start(&mut self) {
        self.is_started = true;
    }

    pub fn stop(&mut self) {
        let conn = get_conn(self.db_pool.clone());
        db::game::set_stopped(&conn, self.id).unwrap();
        self.is_stopped = true;
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}
