-- Your SQL goes here

alter table sessions add column admin_user_id varchar(36) references users(id) not null;