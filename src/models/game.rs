use serde::{Deserialize, Serialize};

use crate::schema::{game_channels, game_users, games};

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
pub struct Game {
    pub id: i64,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
pub struct GameUser {
    pub id: i64,
    pub game_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
pub struct GameChannel {
    pub id: i64,
    pub game_id: i64,
    pub channel_id: i64,
}
