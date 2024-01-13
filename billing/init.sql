create table "transaction_history"
(
    id            serial
        constraint txh_pk
            primary key,
    user_id int,
    tx_type smallint,
    amount int
);