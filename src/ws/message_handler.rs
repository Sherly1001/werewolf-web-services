use super::{cmd_parser::Cmd, services, ChatServer};

pub fn msg_handler(srv: &mut ChatServer, ws_id: i64, user_id: i64, msg: String) {
    Cmd::from_string(&msg)
        .map_err(|err| err.to_string())
        .and_then(|cmd| cmd_handler(srv, ws_id, user_id, cmd))
        .map_err(|err| srv.send_to(&Cmd::Error(err).to_string(), ws_id))
        .ok();
}

pub fn cmd_handler(
    srv: &mut ChatServer,
    ws_id: i64,
    user_id: i64,
    cmd: Cmd,
) -> Result<(), String> {
    match cmd {
        Cmd::SendReq {
            channel_id,
            message,
        } => {
            let channel_id = channel_id.parse::<i64>()
                .map_err(|err| err.to_string())?;
            let chat = services::send_msg(srv, user_id, channel_id, message)?;
            let bc = Cmd::BroadCastMsg {
                user_id: user_id.to_string(),
                channel_id: channel_id.to_string(),
                message_id: chat.id.to_string(),
                message: chat.message,
            }
            .to_string();
            let rs = Cmd::SendRes {
                message_id: chat.id.to_string(),
            }
            .to_string();

            srv.broadcast(&bc, ws_id);
            srv.send_to(&rs, ws_id);
            Ok(())
        }
        Cmd::GetMsg {
            channel_id,
            offset,
            limit,
        } => {
            let messages = services::get_msg(
                srv,
                channel_id.parse::<i64>().map_err(|err| err.to_string())?,
                offset as i64,
                limit as i64,
            )?;
            let rs = Cmd::GetMsgRes {
                channel_id,
                messages,
            }
            .to_string();

            srv.send_to(&rs, ws_id);
            Ok(())
        }
        Cmd::GetUserInfo { user_id: uid } => {
            let uid = match uid {
                Some(id) => id.parse::<i64>()
                    .map_err(|err| err.to_string())?,
                None => user_id,
            };
            let user = services::get_info(srv, uid)?;

            srv.send_to(&Cmd::GetUserInfoRes(user).to_string(), ws_id);
            Ok(())
        }
        Cmd::GetUsers => {
            let users = services::get_users(srv)?;

            srv.send_to(&Cmd::GetUsersRes(users).to_string(), ws_id);
            Ok(())
        }
        _ => Ok(()),
    }
}
