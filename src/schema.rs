// @generated automatically by Diesel CLI.

diesel::table! {
    following (id) {
        id -> Int4,
        followed_user_id -> Int4,
        #[max_length = 255]
        guild_id -> Varchar,
        #[max_length = 255]
        channel_id -> Varchar,
    }
}

diesel::table! {
    reviews (id) {
        id -> Int4,
        #[max_length = 255]
        place_name -> Varchar,
        review_text -> Text,
        review_original_text -> Nullable<Text>,
        stars -> Int4,
        user_id -> Int4,
        found_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        gmaps_id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::joinable!(following -> users (followed_user_id));
diesel::joinable!(reviews -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(following, reviews, users,);
