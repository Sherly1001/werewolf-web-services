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
