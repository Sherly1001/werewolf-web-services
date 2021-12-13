-- Your SQL goes here

alter table chat_lines add column reply_to bigint default null references chat_lines(id) on delete set default;
