use std::time::{Duration, Instant};
use std::collections::HashMap;

use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext,
    Context, ContextFutureSpawner, Handler,
    Message, Recipient, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{Message as WsMessage, ProtocolError, WebsocketContext};

use crate::config::{AppState, DbPool};

use super::{message_handler::msg_handler, cmd_parser::Cmd, services};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Msg(pub String);

#[derive(Message, Debug)]
#[rtype(result = "i64")]
pub struct Connect {
    user_id: i64,
    addr: Recipient<Msg>,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    ws_id: i64,
    user_id: i64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ClientMsg {
    ws_id: i64,
    user_id: i64,
    msg: String,
}

#[derive(Clone)]
pub struct ChatServer {
    pub clients: HashMap<i64, Recipient<Msg>>,
    pub users: HashMap<i64, Vec<i64>>,
    pub app_state: AppState,
    pub db_pool: DbPool,
}

impl ChatServer {
    pub fn new(app_state: AppState, db_pool: DbPool) -> Self {
        Self {
            clients: HashMap::new(),
            users: HashMap::new(),
            app_state,
            db_pool,
        }
    }

    pub fn broadcast(&self, cmd: &Cmd, except: i64) {
        for (&ws_id, client) in self.clients.iter() {
            if ws_id != except {
                client.do_send(Msg(cmd.to_string())).ok();
            }
        }
    }

    pub fn broadcast_user(&self, cmd: &Cmd, except: i64) {
        if let Some(excepts) = self.users.get(&except) {
            for (ws_id, client) in self.clients.iter() {
                if !excepts.contains(ws_id) {
                    client.do_send(Msg(cmd.to_string())).ok();
                }
            }
        }
    }

    pub fn send_to(&self, cmd: &Cmd, ws_id: i64) {
        if let Some(client) = self.clients.get(&ws_id) {
            client.do_send(Msg(cmd.to_string())).ok();
        }
    }

    pub fn user_online(&self, user_id: i64) {
        if let Ok(u) = services::get_info(self, user_id) {
            self.broadcast_user(&Cmd::UserOnline(u), user_id);
        }
    }

    pub fn user_offline(&self, user_id: i64) {
        if let Ok(u) = services::get_info(self, user_id) {
            self.broadcast_user(&Cmd::UserOffline(u), user_id);
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = i64;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        let ws_id = self
            .app_state
            .id_generatator
            .lock()
            .unwrap()
            .real_time_generate();

        self.clients.insert(ws_id, msg.addr);
        self.users
            .entry(msg.user_id)
            .or_insert(vec![])
            .push(ws_id);

        if let Some(ws) = self.users.get(&msg.user_id) {
            if ws.len() == 1 {
                self.user_online(msg.user_id);
            }
        }

        ws_id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.ws_id);
        if let Some(ws) = self.users.get_mut(&msg.user_id) {
            ws.retain(|&id| id != msg.ws_id);
            if ws.is_empty() {
                self.user_offline(msg.user_id);
            }
        }
    }
}

impl Handler<ClientMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMsg, _ctx: &mut Self::Context) -> Self::Result {
        msg_handler(self, msg.ws_id, msg.user_id, msg.msg);
    }
}

pub struct WsClient {
    pub id: i64,
    pub user_id: i64,
    pub hb: Instant,
    pub hb_interval: Duration,
    pub hb_timeout: Duration,
    pub addr: Addr<ChatServer>,
}

impl WsClient {
    pub fn new(user_id: i64, addr: Addr<ChatServer>) -> Self {
        Self {
            id: 0,
            user_id,
            hb: Instant::now(),
            hb_interval: Duration::from_secs(5),
            hb_timeout: Duration::from_secs(10),
            addr,
        }
    }

    pub fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(self.hb_interval, |atx, ctx| {
            if Instant::now().duration_since(atx.hb) > atx.hb_timeout {
                println!("Websocket Client heartbeat failed, disconnecting!");
                atx.addr.do_send(Disconnect {
                    ws_id: atx.id,
                    user_id: atx.user_id,
                });
                ctx.stop();
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsClient {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address().recipient();
        let user_id = self.user_id;
        self.addr
            .send(Connect { user_id, addr })
            .into_actor(self)
            .then(|res, atx, ctx| {
                match res {
                    Ok(id) => atx.id = id,
                    _ => ctx.stop(),
                }
                actix::fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        self.addr
            .do_send(Disconnect {
                ws_id: self.id,
                user_id: self.user_id,
            });
        actix::Running::Stop
    }
}

impl Handler<Msg> for WsClient {
    type Result = ();

    fn handle(&mut self, msg: Msg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<WsMessage, ProtocolError>> for WsClient {
    fn handle(&mut self, item: Result<WsMessage, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        match msg {
            WsMessage::Ping(msg) => {
                self.hb = Instant::now();
                WsMessage::Pong(msg);
            }
            WsMessage::Pong(_) => {
                self.hb = Instant::now();
            }
            WsMessage::Text(msg) => {
                self.addr.do_send(ClientMsg {
                    ws_id: self.id,
                    user_id: self.user_id,
                    msg,
                });
            }
            WsMessage::Binary(_) => {}
            WsMessage::Nop => (),
            WsMessage::Continuation(_) => {
                ctx.stop();
            }
            WsMessage::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
        }
    }
}
