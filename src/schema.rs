// @generated automatically by Diesel CLI.

diesel::table! {
    rooms (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}
