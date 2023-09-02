-- Your SQL goes here

create table words (
    word varchar(255) not null,
    created_at timestamp not null default now(),
    session_id varchar(20) references sessions(id),
    user_id varchar(36) not null references users(id),
    primary key (word, session_id)
);