use std::convert::TryInto;

use super::{cmd_parser::Cmd, services, ChatServer};

pub fn msg_handler(srv: &mut ChatServer, ws_id: i64, user_id: i64, msg: String) {
    let cmd = Cmd::from_string(&msg);
    if let Err(err) = cmd {
        return srv.send_to(&Cmd::Error(err.to_string()).to_string(), ws_id);
    }

    let cmd = cmd.ok().unwrap();
    match cmd {
        Cmd::SendReq {
            channel_id,
            message,
        } => {
            let channel_id = channel_id.parse().unwrap();
            match services::send_msg(srv, user_id, channel_id, message) {
                Ok(chat) => {
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
                }
                Err(e) => {
                    srv.send_to(&Cmd::Error(e).to_string(), ws_id);
                }
            }
        }
        Cmd::GetMsg {
            channel_id,
            offset,
            limit,
        } => {
            match services::get_msg(
                srv,
                channel_id.parse().unwrap(),
                offset.try_into().unwrap(),
                limit.try_into().unwrap(),
            ) {
                Ok(messages) => {
                    let rs = Cmd::GetMsgRes {
                        channel_id,
                        messages,
                    }
                    .to_string();

                    srv.send_to(&rs, ws_id);
                }
                Err(e) => {
                    srv.send_to(&Cmd::Error(e).to_string(), ws_id);
                }
            }
        }
        _ => {}
    }
}
