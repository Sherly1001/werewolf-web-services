table! {
    users (id) {
        id -> Int8,
        username -> Text,
        hash_passwd -> Text,
        email -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
        win -> Nullable<Int8>,
        lose -> Nullable<Int8>,
    }
}
