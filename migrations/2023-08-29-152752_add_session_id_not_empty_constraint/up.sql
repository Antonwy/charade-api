-- Your SQL goes here
alter table sessions add constraint session_id_not_empty check (length(id) > 0);