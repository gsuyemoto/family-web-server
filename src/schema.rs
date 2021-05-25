table! {
    users (id) {
        id -> Nullable<Integer>,
        fname -> Text,
        lname -> Nullable<Text>,
        is_admin -> Integer,
        num_bucks -> Nullable<Integer>,
        date_created -> Text,
    }
}
