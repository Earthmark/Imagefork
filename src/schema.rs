// @generated automatically by Diesel CLI.

diesel::table! {
    creators (id) {
        id -> Int8,
        creation_time -> Timestamp,
        email -> Text,
        lockout -> Bool,
        moderator -> Bool,
        poster_limit -> Int4,
    }
}

diesel::table! {
    posters (id) {
        id -> Int8,
        creator -> Int8,
        creation_time -> Timestamp,
        url -> Text,
        stopped -> Bool,
        lockout -> Bool,
        servable -> Bool,
    }
}

diesel::joinable!(posters -> creators (creator));

diesel::allow_tables_to_appear_in_same_query!(
    creators,
    posters,
);
