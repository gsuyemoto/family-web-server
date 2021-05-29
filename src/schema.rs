table! {
    devices (id) {
        id -> Integer,
        username -> Text,
        addr_mac -> Text,
        addr_ip -> Nullable<Text>,
        name -> Nullable<Text>,
        is_watching -> Integer,
        watch_start -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        points -> Integer,
        is_admin -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    devices,
    users,
);
