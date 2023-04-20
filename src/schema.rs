// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Integer,
        user_id -> Integer,
        nickname -> Text,
        addr_mac -> Text,
        addr_ip -> Text,
        is_watching -> Integer,
        is_blocked -> Integer,
        is_tracked -> Integer,
        last_checked -> Integer,
        last_last_checked -> Integer,
        manufacturer_name -> Nullable<Text>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Integer,
        name -> Text,
        points -> Integer,
        is_admin -> Integer,
    }
}

diesel::joinable!(devices -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    users,
);
