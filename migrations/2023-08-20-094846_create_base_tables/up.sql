create table users (
  id varchar(36) primary key,
  name varchar(50),
  created_at timestamp default now() not null
);

create table sessions (
  id varchar(20) primary key,
  public boolean default false not null,
  created_at timestamp default now() not null
);

create table users_sessions (
  user_id varchar(36) references users(id),
  session_id varchar(20) references sessions(id),
  primary key (user_id, session_id)
);