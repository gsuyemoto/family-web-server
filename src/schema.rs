table! {
    devices (id) {
        id -> Integer,
        user_id -> Integer,
        nickname -> Text,
        addr_mac -> Text,
        addr_ip -> Nullable<Text>,
        manufacture_name -> Nullable<Text>,
        is_watching -> Integer,
        is_blocked -> Integer,
        is_tracked -> Integer,
        watch_start -> Nullable<Integer>,
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

joinable!(users -> devices (user_id));
allow_tables_to_appear_in_same_query!(
    devices,
    users,
);
