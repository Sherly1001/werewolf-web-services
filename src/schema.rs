table! {
    users (username) {
        username -> Varchar,
        hash_passwd -> Bpchar,
        email -> Nullable<Varchar>,
        avatar_url -> Nullable<Varchar>,
        win -> Nullable<Int8>,
        lose -> Nullable<Int8>,
    }
}
