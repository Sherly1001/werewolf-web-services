-- Your SQL goes here

create unique index user_channel_per on user_channel_permissions(user_id, channel_id);
