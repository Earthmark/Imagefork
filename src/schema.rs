// @generated automatically by Diesel CLI.

diesel::table! {
    creator_sessions (creator, token) {
        creator -> Int8,
        token -> Varchar,
        creation_time -> Timestamp,
    }
}

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
    crsf_tokens (token) {
        token -> Varchar,
        crsf -> Varchar,
        creation_time -> Timestamp,
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

diesel::joinable!(creator_sessions -> creators (creator));
diesel::joinable!(posters -> creators (creator));

diesel::allow_tables_to_appear_in_same_query!(
    creator_sessions,
    creators,
    crsf_tokens,
    posters,
);
