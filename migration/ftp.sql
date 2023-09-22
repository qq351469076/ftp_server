-- 需要先创建ftp数据库

CREATE EXTENSION pgcrypto;

create table if not exists public."user"
(
    id       uuid default gen_random_uuid() not null
    constraint user_pk
    primary key,
    account  varchar(20)                    not null,
    password varchar(20)                    not null
    );

