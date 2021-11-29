-- Your SQL goes here

create table channels (
    id bigint not null primary key,
    channel_name text not null
);

insert into channels values (0, 'rules'), (1, 'lobby');

create table chat_lines (
    id bigint not null primary key,
    user_id bigint not null references users(id) on delete no action,
    channel_id bigint not null references channels(id) on delete no action,
    message text not null
);

create table user_channel_permissions (
    id bigint not null primary key,
    user_id bigint not null references users(id) on delete cascade,
    channel_id bigint not null references channels(id) on delete cascade,
    readable boolean not null default false,
    sendable boolean not null default false
);
