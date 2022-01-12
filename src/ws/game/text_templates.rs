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

pub fn game_is_not_started() -> String {
    format!("Trò chơi chưa bắt đầu.")
}

pub fn aready_in_game() -> String {
    format!("Bạn đã tham gia trò chơi rồi, hãy đợi trò chơi bắt đầu.")
}

pub fn leave_on_started() -> String {
    format!("Trò chơi đã bắt đầu, hãy đợi trò chơi kết thúc!")
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

pub fn new_wolf(user_id: i64) -> String {
    format!("Chào sói <@{}>.", user_id)
}

pub fn on_start_game(role: &'static str) -> String {
    format!("Chào mừng, vai của bạn là {}.", role)
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

pub fn user_next(user_id: i64, numvote: usize, numplayer: usize) -> String {
    format!("Người chơi <@{}> muốn chuyển sang phase tiếp theo. {}/{}",
            user_id, numvote, numplayer)
}

pub fn new_phase(bot_prefix: &str, num_day: u16, is_day: bool) -> String {
    match is_day {
        true => new_day(bot_prefix, num_day),
        false => new_night(),
    }
}

pub fn timeout(mut count: u64) -> String {
    let h = count / 3600;
    count -= h * 3600;
    let m = count / 60;
    count -= m * 60;

    let mut s = String::from("Còn");
    if h > 0 {
        s += format!(" {} giờ", h).as_str();
    }
    if m > 0 {
        s += format!(" {} phút", m).as_str();
    }
    if count > 0 {
        s += format!(" {} giây", count).as_str();
    }
    s += ".";

    s
}

pub fn vote_kill(user_id: i64, vote_for: i64) -> String {
    format!("Người chơi <@{}> đã biểu quyết loại <@{}> ra khỏi làng.",
            user_id, vote_for)
}

pub fn wrong_cmd_format(prefix: &str, s: &str) -> String {
    format!("Không đúng định dạnh lệnh, `{}{}`", prefix, s)
}

pub fn player_not_in_game(user_id: i64) -> String {
    format!("Người chơi <@{}> không ở trong game này.", user_id)
}

pub fn player_died() -> String {
    format!("Người ta đã hẹo rồi, đừng có vote nữa. Vote người nào còn sống thôi :3")
}

pub fn player_still_alive(user_id: i64) -> String {
    format!("<@{}> còn sống mà bạn!", user_id)
}

pub fn invalid_index(from: usize, to: usize) -> String {
    format!("Giá trị không hợp lệ, chọn từ {} đến {}.", from, to)
}

pub fn player_list(list: &Vec<i64>, is_alive: bool) -> String {
    let mut s = format!("Danh sách những người chơi {}:\n",
        if is_alive { "còn sống" } else { "đã chết" });
    s += list.iter()
        .enumerate()
        .map(|(idx, id)| format!("{}: <@{}>", idx + 1, id))
        .collect::<Vec<String>>()
        .join("\n")
        .as_str();
    s
}

pub fn execution(top_vote: Option<(i64, u16)>) -> String {
    match top_vote {
        None => format!(
            "Không có ai bị hành hình. Trò chơi sẽ tiếp tục. Hãy cẩn thân để sống sót!
==========================================================================="),
        Some((uid, votes)) => format!(
            "Thời gian quyết định đã hết.
Người chơi <@{}> đã bị đưa lên máy chém với số phiếu bầu là {}.
Hy vọng tình thế của làng có thể thay đổi sau quyết định này.
===========================================================================", uid, votes)
    }
}

pub fn new_day(bot_prefix: &str, num_day: u16) -> String {
    format!(
        "Một ngày mới bắt đầu, mọi người thức giấc. Báo cáo tình hình ngày {}:
- Hãy nhập `{}vote <player>` để bỏ phiếu cho người bạn nghi là Sói!",
        num_day, bot_prefix)
}

pub fn new_night() -> String {
    format!("Đêm đã tới. Cảnh vật hóa tĩnh lặng, mọi người an giấc. Liệu đêm nay có xảy ra chuyện gì không?")
}

pub fn after_death(user_id: i64) -> String {
    format!("Chào mừng <@{}> đến với nghĩa trang vui vẻ ^^.", user_id)
}

pub fn seer_action(bot_prefix: &str) -> String {
    format!(
        "Tiên tri muốn thấy gì, từ ai?
- Hãy làm phép bằng cách nhập `{}seer <player>` để xem người chơi đó là ai.", bot_prefix)
}

pub fn guard_action(bot_prefix: &str) -> String {
    format!(
        "Bảo vệ muốn ai sống qua đêm nay, hãy nhập `{}guard <player>` để người đó qua đêm an bình. Ví dụ: `{}guard 2`
- Bạn chỉ sử dụng kỹ năng được 1 lần mỗi đêm. Hãy cẩn trọng!", bot_prefix, bot_prefix)
}

pub fn witch_action(bot_prefix: &str) -> String {
    format!(
        "Bạn có thể cứu 1 người và giết 1 người. Bạn chỉ được dùng mỗi kỹ năng 1 lần.
- Nhập `{}reborn <player>` để cứu người.
- Nhập `{}curse <player>` để nguyền rủa 1 người.", bot_prefix, bot_prefix)
}

pub fn cupid_action(bot_prefix: &str) -> String {
    format!(
        "Cupid muốn cho cặp đôi nào được đồng sinh cộng tử.
- Hãy làm phép bằng cách nhập `{}ship <player 1> <player 2>` để ghép đôi.", bot_prefix)
}

pub fn before_wolf_action(bot_prefix: &str) -> String {
    format!(
        "Đêm nay, Sói muốn lấy mạng ai? Hãy nhập `{}kill <player>` để lặng lẽ xử lý nạn nhân. Ví dụ: `{}kill 2`", bot_prefix, bot_prefix)
}

pub fn list_killed(list: &Vec<i64>) -> String {
    let mut s = String::new();

    if list.is_empty() {
        s += "Đêm qua, mọi người đều bình an.\n";
    } else {
        s += "Đêm qua, ";
        s += list.iter()
            .map(|id| format!("<@{}>", id))
            .collect::<Vec<String>>()
            .join(", ")
            .as_str();
        if list.len() == 1 { s += " " }
        s += "đã bị mất tích một cách bí ẩn.\n";
    }

    s += "===========================================================================";

    s
}

pub fn must_alive() -> String {
    format!("Bạn phải còn sống để sủ dụng kỹ năng này!")
}

pub fn invalid_author() -> String {
    format!("Hiện tại bạn không được phép dùng kỹ năng này!")
}

pub fn invalid_daytime() -> String {
    format!("Hãy sử dụng vào ban ngày!")
}

pub fn invalid_nighttime() -> String {
    format!("Hãy đợi tới đêm để sử dụng kỹ năng!")
}

pub fn out_of_mana() -> String {
    format!("Bạn chỉ sử dụng kỹ năng được 1 lần mỗi đêm!")
}

pub fn out_of_power() -> String {
    format!("Bạn chỉ sử dụng kỹ năng được 1 lần duy nhất!")
}

pub fn cupid_out_of_power() -> String {
    format!("Ngày đầu tiên kết thúc, kỹ năng đã hết hiệu lực!")
}

pub fn wolf_kill(wolf_id: i64, target_id: i64) -> String {
    format!("Sói <@{}> muốn xử lý <@{}> trong đêm nay.", wolf_id, target_id)
}

pub fn guard_success(target_id: i64) -> String {
    format!("Đã bảo vệ thành công <@{}>.", target_id)
}

pub fn guard_yesterday_target() -> String {
    format!("Hôm qua bạn đã bảo vệ người này. Hãy đổi mục tiêu khác!")
}

pub fn seer_use_skill(target_id: i64, is_wolf: bool) -> String {
    format!("Ồ, <@{}> {}là Sói. Pháp lực đã hết, tiên tri cần đi ngủ để hồi phục năng lượng.", target_id, if is_wolf { "" } else { "không phải " })
}

pub fn shipped_with(target_id: i64, role: &str) -> String {
    format!("Bạn và <@{}> - {} đã được thần tình yêu chọn làm cặp đôi đồng sinh cộng tử.",
        target_id, role)
}

pub fn ship_success(target1: i64, target2: i64) -> String {
    format!("Bạn đã ghép đôi thành công <@{}> và <@{}>.", target1, target2)
}
