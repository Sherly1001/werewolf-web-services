-- This file should undo anything in `up.sql`

alter table chat_lines drop column reply_to;
