use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner, Handler,
    Message, Recipient, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{Message as WsMessage, ProtocolError, WebsocketContext};

use crate::config::{AppState, DbPool};

use super::cmd_parser::GameEvent;
use super::game::cmds::UpdatePers;
use super::services::get_info;
use super::{
    cmd_parser::Cmd,
    game::{
        cmds::{BotMsg, GameMsg, StartGame, StopGame},
        Game,
    },
    message_handler::msg_handler,
    services,
};

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

pub struct ChatServer {
    pub clients: HashMap<i64, Recipient<Msg>>,
    pub users: HashMap<i64, Vec<i64>>,
    pub games: HashMap<i64, Addr<Game>>,
    pub current_game: Option<Addr<Game>>,
    pub app_state: AppState,
    pub db_pool: DbPool,
}

impl ChatServer {
    pub fn new(app_state: AppState, db_pool: DbPool) -> Self {
        Self {
            clients: HashMap::new(),
            users: HashMap::new(),
            games: HashMap::new(),
            current_game: None,
            app_state,
            db_pool,
        }
    }

    pub fn broadcast(&self, cmd: &Cmd, except: i64) {
        let uids = match &cmd {
            Cmd::BroadCastMsg { channel_id, .. } => {
                services::get_channel_users(self, channel_id.parse().unwrap_or(-1))
                    .iter()
                    .map(|u| u.id)
                    .collect()
            }
            _ => self.users.keys().cloned().collect::<Vec<i64>>(),
        };
        let allow_wsi = uids
            .iter()
            .map(|id| self.users.get(&id).unwrap_or(&vec![]).clone())
            .flatten()
            .collect::<Vec<i64>>();

        for (ws_id, client) in self.clients.iter() {
            if *ws_id != except && allow_wsi.contains(ws_id) {
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
        } else {
            self.broadcast(cmd, -1);
        }
    }

    pub fn send_to(&self, cmd: &Cmd, ws_id: i64) {
        if let Some(client) = self.clients.get(&ws_id) {
            client.do_send(Msg(cmd.to_string())).ok();
        }
    }

    pub fn send_to_user(&self, cmd: &Cmd, user_id: i64) {
        if let Some(ws) = self.users.get(&user_id) {
            for (ws_id, client) in self.clients.iter() {
                if ws.contains(ws_id) {
                    client.do_send(Msg(cmd.to_string())).ok();
                }
            }
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

    pub fn bot_send(&self, channel_id: i64, message: String, reply_to: Option<i64>) {
        let bot_id = self.app_state.bot_id;
        let chat = match services::send_msg(self, bot_id, channel_id, message, reply_to) {
            Ok(c) => c,
            Err(e) => return eprintln!("bot_send: {}", e),
        };
        let bc = Cmd::BroadCastMsg {
            user_id: bot_id.to_string(),
            message_id: chat.id.to_string(),
            channel_id: chat.channel_id.to_string(),
            message: chat.message,
            reply_to: reply_to.map(|id| id.to_string()),
        };
        self.broadcast(&bc, -1);
    }

    pub fn update_pers(&self, user_id: i64) {
        let pers = services::get_pers(self, user_id, None).unwrap_or(HashMap::new());
        self.send_to_user(&Cmd::GetPersRes(pers), user_id);
    }

    pub fn new_game(&mut self, ctx: &mut Context<Self>) -> Addr<Game> {
        let game_id = self
            .app_state
            .id_generatator
            .lock()
            .unwrap()
            .real_time_generate();

        let game = Game::new(
            game_id,
            ctx.address(),
            self.db_pool.clone(),
            self.app_state.id_generatator.clone(),
            self.app_state.bot_id,
            self.app_state.bot_prefix.clone(),
        )
        .start();
        self.games.insert(game_id, game.clone());

        game
    }

    pub fn get_user_game(&self, user_id: i64) -> Option<&Addr<Game>> {
        let game_id = services::get_game_from_user(self, user_id)?;
        self.games.get(&game_id)
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(game) = Game::load_from_db(
            ctx.address(),
            self.db_pool.clone(),
            self.app_state.id_generatator.clone(),
            self.app_state.bot_id,
            self.app_state.bot_prefix.clone(),
        ) {
            let id = game.id;
            let addr = game.start();
            self.games.insert(id, addr.clone());
        }
    }
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
        self.users.entry(msg.user_id).or_insert(vec![]).push(ws_id);

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
                self.users.remove(&msg.user_id);
                self.user_offline(msg.user_id);
            }
        }
    }
}

impl Handler<ClientMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMsg, ctx: &mut Self::Context) -> Self::Result {
        msg_handler(self, ctx, msg.ws_id, msg.user_id, msg.msg);
    }
}

impl Handler<BotMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BotMsg, _: &mut Self::Context) -> Self::Result {
        self.bot_send(msg.channel_id, msg.msg, msg.reply_to);
    }
}

impl Handler<GameMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: GameMsg, _ctx: &mut Self::Context) -> Self::Result {
        let uids = services::get_game_users(self, msg.game_id)
            .iter()
            .map(|u| u.id)
            .collect::<Vec<i64>>();

        let event = msg.event.clone();
        let cmd = &Cmd::GameEvent(msg.event);
        for (uid, ws) in self.users.iter() {
            if !uids.contains(uid) {
                continue;
            }
            for (wsi, client) in self.clients.iter() {
                if !ws.contains(wsi) {
                    continue;
                }
                client.do_send(Msg(cmd.to_string())).ok();
                if let GameEvent::EndGame { .. } = event {
                    if let Ok(user) = get_info(self, *uid) {
                        client
                            .do_send(Msg(Cmd::GetUserInfoRes(user).to_string()))
                            .ok();
                    }
                }
            }
        }
    }
}

impl Handler<StartGame> for ChatServer {
    type Result = ();

    fn handle(&mut self, _msg: StartGame, _ctx: &mut Self::Context) -> Self::Result {
        self.current_game = None;
    }
}

impl Handler<StopGame> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: StopGame, _: &mut Self::Context) -> Self::Result {
        self.games.remove(&msg.0);
        self.current_game = None;
    }
}

impl Handler<UpdatePers> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: UpdatePers, _: &mut Self::Context) -> Self::Result {
        self.update_pers(msg.0);
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
        self.addr.do_send(Disconnect {
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
