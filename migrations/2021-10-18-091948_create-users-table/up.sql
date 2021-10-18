-- Your SQL goes here

create table users (
    username varchar(50) primary key not null,
    hash_passwd char(64) not null,
    email varchar(255),
    avatar_url varchar(255),
    win bigint default 0,
    lose bigint default 0
);
