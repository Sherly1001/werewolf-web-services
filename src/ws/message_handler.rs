use actix::Context;

use crate::ws::game::{cmds as game_cmds, text_templates as ttp};

use super::{cmd_parser::Cmd, services, ChatServer};

pub fn msg_handler(
    srv: &mut ChatServer,
    ctx: &mut Context<ChatServer>,
    ws_id: i64,
    user_id: i64,
    msg: String,
) {
    Cmd::from_string(&msg)
        .map_err(|err| err.to_string())
        .and_then(|cmd| cmd_handler(srv, ctx, ws_id, user_id, cmd))
        .map_err(|err| srv.send_to(&Cmd::Error(err), ws_id))
        .ok();
}

pub fn cmd_handler(
    srv: &mut ChatServer,
    ctx: &mut Context<ChatServer>,
    ws_id: i64,
    user_id: i64,
    cmd: Cmd,
) -> Result<(), String> {
    match cmd {
        Cmd::SendReq {
            channel_id,
            message,
            reply_to,
        } => {
            let channel_id = channel_id.parse::<i64>()
                .map_err(|err| err.to_string())?;
            let reply_to = reply_to
                .map(|id| id.parse::<i64>().map_err(|err| err.to_string()))
                .transpose()?;
            let chat = services::send_msg(
                srv, user_id, channel_id, message.clone(), reply_to)?;
            let bc = Cmd::BroadCastMsg {
                user_id: user_id.to_string(),
                channel_id: channel_id.to_string(),
                message_id: chat.id.to_string(),
                message: chat.message.clone(),
                reply_to: reply_to.map(|id| id.to_string()),
            };
            let rs = Cmd::SendRes {
                channel_id: channel_id.to_string(),
                message_id: chat.id.to_string(),
                reply_to: reply_to.map(|id| id.to_string()),
            };

            srv.broadcast(&bc, ws_id);
            srv.send_to(&rs, ws_id);

            if message.starts_with(srv.app_state.bot_prefix.as_str()) {
                game_commands(
                    srv, ctx, ws_id, user_id, channel_id, message, chat.id)
                    .map_err(|err| srv.bot_send(channel_id, err, Some(chat.id)))
                    .ok();
            }
        }
        Cmd::GetMsg {
            channel_id,
            offset,
            limit,
        } => {
            let messages = services::get_msg(
                srv,
                channel_id.parse::<i64>().map_err(|err| err.to_string())?,
                offset.unwrap_or(0) as i64,
                limit.unwrap_or(50) as i64,
            )?;
            let rs = Cmd::GetMsgRes {
                channel_id,
                messages,
            };

            srv.send_to(&rs, ws_id);
        }
        Cmd::GetUserInfo { user_id: uid } => {
            let uid = uid
                .map(|id| id.parse::<i64>().map_err(|err| err.to_string()))
                .transpose()?
                .unwrap_or(user_id);
            let user = services::get_info(srv, uid)?;

            srv.send_to(&Cmd::GetUserInfoRes(user), ws_id);
        }
        Cmd::GetUsers => {
            let mut users = services::get_users(srv)?;

            for u in users.iter_mut() {
                u.is_online = Some(
                    srv.users.contains_key(&u.id.parse().unwrap())
                    || u.id == srv.app_state.bot_id.to_string());
            }

            srv.send_to(&Cmd::GetUsersRes(users), ws_id);
        }
        Cmd::GetPers { channel_id } => {
            let channel_id = channel_id
                .map(|id| id.parse::<i64>().map_err(|err| err.to_string()))
                .transpose()?;
            let pers = services::get_pers(srv, user_id, channel_id)?;

            srv.send_to(&Cmd::GetPersRes(pers), ws_id);
        }
        _ => {}
    };
    Ok(())
}

fn game_commands(
    srv: &mut ChatServer,
    ctx: &mut Context<ChatServer>,
    _ws_id: i64,
    user_id: i64,
    channel_id: i64,
    message: String,
    msg_id: i64,
) -> Result<(), String> {
    let cmds = message
        .strip_prefix(srv.app_state.bot_prefix.as_str())
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();

    println!("game: {} - {} - {}: {:?}", user_id, channel_id, msg_id, cmds);

    match cmds[0] {
        "join" => {
            must_in_channel(1, channel_id)?;
            if let (Some(game), Some(cur)) =
                (srv.get_user_game(user_id), srv.current_game.as_ref()) {
                if game != cur { return Err(ttp::in_other_game()) }
            }
            match srv.current_game.as_ref() {
                Some(game) => {
                    game.do_send(game_cmds::Join { user_id, msg_id });
                }
                None => {
                    let game = srv.new_game(ctx);
                    srv.current_game = Some(game.clone());
                    game.do_send(game_cmds::Join { user_id, msg_id });
                }
            }
        }
        "leave" => {
            must_in_channel(1, channel_id)?;
            if let Some(game) = srv.get_user_game(user_id) {
                game.do_send(game_cmds::Leave { user_id, msg_id });
            } else {
                srv.bot_send(channel_id, ttp::not_in_game(), Some(msg_id));
            }
        }
        "start" => {
            must_in_channel(1, channel_id)?;
            if let Some(game) = srv.get_user_game(user_id) {
                game.do_send(game_cmds::Start { user_id, msg_id });
                return Ok(());
            }
            match srv.current_game.as_ref() {
                Some(game) => {
                    game.do_send(game_cmds::Start { user_id, msg_id });
                }
                None => {
                    srv.bot_send(channel_id, ttp::not_in_game(), Some(msg_id));
                }
            }
        }
        "stop" => {
            must_in_channel(1, channel_id)?;
            if let Some(game) = srv.get_user_game(user_id) {
                game.do_send(game_cmds::Stop { user_id, msg_id });
                return Ok(());
            }
            match srv.current_game.as_ref() {
                Some(game) => {
                    game.do_send(game_cmds::Stop { user_id, msg_id });
                }
                None => {
                    srv.bot_send(channel_id, ttp::not_in_game(), Some(msg_id));
                }
            }
        }
        "next" => {
            if let Some(game) = srv.get_user_game(user_id) {
                game.do_send(game_cmds::Next { user_id, msg_id, channel_id });
                return Ok(());
            }
            match srv.current_game.as_ref() {
                Some(game) => {
                    game.do_send(game_cmds::Next { user_id, msg_id, channel_id });
                }
                None => {
                    srv.bot_send(channel_id, ttp::not_in_game(), Some(msg_id));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn must_in_channel(
    channel_id: i64,
    current_channel_id: i64,
) -> Result<(), String> {
    if channel_id == current_channel_id { Ok(()) }
    else { Err(ttp::must_in_channel(channel_id)) }
}
