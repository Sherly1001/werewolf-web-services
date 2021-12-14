use std::collections::HashMap;

pub fn not_in_game() -> String {
    format!("Bạn đang không ở trong game.")
}

pub fn in_other_game() -> String {
    format!("Bạn đang trong trò chơi khác.")
}

pub fn game_is_started() -> String {
    format!("Trò chơi đã bắt đầu rồi.")
}

pub fn aready_in_game() -> String {
    format!("Bạn đã tham gia trò chơi rồi, hãy đợi trò chơi bắt đầu.")
}

pub fn max_player() -> String {
    format!("Đã đạt số lượng người chơi tối đa.")
}

pub fn must_in_channel(channel_id: i64) -> String {
    format!("Hãy sử dụng lệnh trong <#{}>.", channel_id)
}

pub fn not_enough_player(numplayer: usize) -> String {
    format!("Chưa đủ người chơi, hiện có {}.", numplayer)
}

pub fn start_game() -> String {
    format!("2/3 người chơi đã sẵn sàng, trò chơi chuẩn bị bắt đầu.")
}

pub fn roles_list(roles: &HashMap<String, usize>) -> String {
    format!("Danh sách nhân vật trong game: {:?}.", roles)
}

pub fn stop_game() -> String {
    format!("Trò chơi đã kết thúc.")
}

pub fn user_join(user_id: i64, numplayer: usize) -> String {
    format!("Người chơi <@{}> đã tham gia trò chơi, hiện có {}.",
            user_id, numplayer)
}

pub fn user_leave(user_id: i64, numplayer: usize) -> String {
    format!("Người chơi <@{}> đã rời khỏi trò chơi, hiện có {}.",
            user_id, numplayer)
}

pub fn user_start(user_id: i64, numvote: usize, numplayer: usize) -> String {
    format!("Người chơi <@{}> đã sằn sàng. {}/{}",
            user_id, numvote, numplayer)
}

pub fn user_stop(user_id: i64, numvote: usize, numplayer: usize) -> String {
    format!("Người chơi <@{}> muốn dừng trò chơi. {}/{}",
            user_id, numvote, numplayer)
}
