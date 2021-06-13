table! {
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

table! {
    users (user_id) {
        user_id -> Integer,
        name -> Text,
        points -> Integer,
        is_admin -> Integer,
    }
}

joinable!(devices -> users (user_id));

allow_tables_to_appear_in_same_query!(
    devices,
    users,
);
