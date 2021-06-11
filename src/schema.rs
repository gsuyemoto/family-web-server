table! {
    devices (id) {
        id -> Integer,
        user_id -> Integer,
        nickname -> Text,
        addr_mac -> Text,
        addr_ip -> Text,
        manufacturer_name -> Nullable<Text>,
        is_watching -> Integer,
        is_blocked -> Integer,
        is_tracked -> Integer,
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
