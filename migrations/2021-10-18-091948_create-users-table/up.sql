-- Your SQL goes here

create table users (
    username text primary key not null,
    hash_passwd text not null,
    email text,
    avatar_url text,
    win bigint default 0,
    lose bigint default 0
);
