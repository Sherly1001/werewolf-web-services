use actix::{Actor, Addr, Context, Handler, Message};
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use snowflake::SnowflakeIdGenerator;

use std::{collections::HashSet, sync::{Arc, Mutex}};

use crate::{config::DbPool, db};

use super::{ChatServer, BotMsg, StopGame};

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

    pub fn must_in_game(&self, user_id: i64) -> bool {
        if !self.users.contains(&user_id) {
            self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: "Bạn đang không ở trong game.".to_string(),
            });
            return false;
        };
        return true;
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

impl Handler<Join> for Game {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Self::Context) -> Self::Result {
        if self.users.contains(&msg.0) {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: "Bạn đã tham gia trò chơi rồi, hãy đợi trò chơi bắt đầu.".to_string(),
            });
        }

        if self.users.len() > 15 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: "Đã đạt số lượng người chơi tối đa.".to_string(),
            });
        }

        self.add_user(msg.0);
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: format!("Người chơi <@{}> đã tham gia trò chơi, hiện có {}.",
                         msg.0, self.users.len()),
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
            msg: format!("Người chơi <@{}> đã rời khỏi trò chơi, hiện có {}.",
                         msg.0, self.users.len()),
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
                msg: format!("Chưa đủ người chơi, hiện có {}.", self.users.len()),
            });
        }

        self.vote_starts.insert(msg.0);

        let numvote = self.vote_starts.len() as i16;
        let numplayer = self.users.len() as i16;
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: format!("Người chơi <@{}> đã sằn sàng. {}/{}",
                             msg.0, numvote, numplayer),
            });
        }

        self.start();
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: format!("2/3 người chơi đã sẵn sàng, trò chơi chuẩn bị bắt đầu."),
        });
    }
}

impl Handler<Stop> for Game {
    type Result = ();

    fn handle(&mut self, msg: Stop, _: &mut Self::Context) -> Self::Result {
        if !self.must_in_game(msg.0) { return }

        self.vote_stops.insert(msg.0);

        let numvote = self.vote_stops.len() as i16;
        let numplayer = self.users.len() as i16;
        if numvote * 3 < numplayer * 2 {
            return self.addr.do_send(BotMsg {
                channel_id: 1,
                msg: format!("Người chơi <@{}> muốn dừng trò chơi. {}/{}",
                             msg.0, numvote, numplayer),
            });
        }

        self.stop();
        self.addr.do_send(StopGame(self.id));
        self.addr.do_send(BotMsg {
            channel_id: 1,
            msg: format!("Trò chơi đã kết thúc."),
        });
    }
}
