-- Your SQL goes here

-- Add cascade rules to users_sessions table for user_id
alter table users_sessions drop constraint users_sessions_user_id_fkey;

alter table users_sessions add constraint users_sessions_user_id_fkey 
foreign key (user_id) references users(id) on delete cascade;


-- Add cascade rules to users_sessions table for session_id
alter table users_sessions drop constraint users_sessions_session_id_fkey;

alter table users_sessions add constraint users_sessions_session_id_fkey
foreign key (session_id) references sessions(id) on delete cascade;

-- Add cascade rules to words table for user_id
alter table words drop constraint words_user_id_fkey;

alter table words add constraint words_user_id_fkey
foreign key (user_id) references users(id) on delete cascade;

-- Add cascade rules to words table for session_id
alter table words drop constraint words_session_id_fkey;

alter table words add constraint words_session_id_fkey
foreign key (session_id) references sessions(id) on delete cascade;