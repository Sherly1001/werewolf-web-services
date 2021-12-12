-- Your SQL goes here

alter table chat_lines drop constraint chat_lines_channel_id_fkey;
alter table chat_lines add constraint chat_lines_channel_id_fkey foreign key(channel_id) references channels(id) on delete cascade;
