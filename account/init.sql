create table "user"
(
    id            serial
        constraint user_pk
            primary key,
    name          varchar(255),
    is_active     boolean,
    premium_until timestamp
);