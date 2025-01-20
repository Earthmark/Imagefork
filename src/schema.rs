// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType, Clone, std::fmt::Debug)]
    #[diesel(postgres_type(name = "texture_kind"))]
    pub struct TextureKind;
}

diesel::table! {
    creators (id) {
        id -> Int8,
        creation_time -> Timestamp,
        email -> Text,
        email_hash -> Bytea,
        lockout -> Bool,
        moderator -> Bool,
        poster_limit -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TextureKind;

    poster_image (poster_id, kind) {
        poster_id -> Int8,
        kind -> TextureKind,
        url -> Text,
    }
}

diesel::table! {
    posters (id) {
        id -> Int8,
        creator -> Int8,
        creation_time -> Timestamp,
        stopped -> Bool,
        lockout -> Bool,
        servable -> Bool,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        data -> Bytea,
        expiry_date -> Timestamp,
    }
}

diesel::joinable!(poster_image -> posters (poster_id));
diesel::joinable!(posters -> creators (creator));

diesel::allow_tables_to_appear_in_same_query!(
    creators,
    poster_image,
    posters,
    sessions,
);
