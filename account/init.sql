create table "user"
(
    id            serial constraint user_pk primary key,
    username      varchar(255) unique,
    password      varchar(255) not null,
    is_active     boolean not null default true,
    premium_until timestamp
);