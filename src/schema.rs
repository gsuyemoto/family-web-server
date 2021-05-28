table! {
    devices (id) {
        id -> Integer,
        username -> Nullable<Text>,
        mac -> Nullable<Text>,
        name -> Nullable<Text>,
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
