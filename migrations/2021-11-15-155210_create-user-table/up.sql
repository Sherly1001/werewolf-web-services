-- Your SQL goes here
create table users (
    id bigint not null primary key,
    username text unique not null,
    hash_passwd text not null,
    email text,
    avatar_url text,
    win integer default 0,
    lose integer default 0
);
