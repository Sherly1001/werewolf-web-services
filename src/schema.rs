table! {
    channels (id) {
        id -> Int8,
        channel_name -> Text,
    }
}

table! {
    chat_lines (id) {
        id -> Int8,
        user_id -> Int8,
        channel_id -> Int8,
        message -> Text,
        reply_to -> Nullable<Int8>,
    }
}

table! {
    game_channels (id) {
        id -> Int8,
        game_id -> Int8,
        channel_id -> Int8,
    }
}

table! {
    game_users (id) {
        id -> Int8,
        game_id -> Int8,
        user_id -> Int8,
    }
}

table! {
    games (id) {
        id -> Int8,
        is_stopped -> Bool,
    }
}

table! {
    user_channel_permissions (id) {
        id -> Int8,
        user_id -> Int8,
        channel_id -> Int8,
        readable -> Bool,
        sendable -> Bool,
    }
}

table! {
    users (id) {
        id -> Int8,
        username -> Text,
        hash_passwd -> Text,
        email -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
        win -> Nullable<Int4>,
        lose -> Nullable<Int4>,
    }
}

joinable!(chat_lines -> channels (channel_id));
joinable!(chat_lines -> users (user_id));
joinable!(game_channels -> channels (channel_id));
joinable!(game_channels -> games (game_id));
joinable!(game_users -> games (game_id));
joinable!(game_users -> users (user_id));
joinable!(user_channel_permissions -> channels (channel_id));
joinable!(user_channel_permissions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    channels,
    chat_lines,
    game_channels,
    game_users,
    games,
    user_channel_permissions,
    users,
);
