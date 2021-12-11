-- Your SQL goes here

create table games(
    id bigint not null primary key,
    is_stopped boolean not null default false
);

create table game_users(
    id bigint not null primary key,
    game_id bigint not null references games(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade
);

create table game_channels(
    id bigint not null primary key,
    game_id bigint not null references games(id) on delete cascade,
    channel_id bigint not null references channels(id) on delete cascade
);
