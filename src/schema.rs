table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        pass_hash -> Varchar,
        volumes -> Nullable<Array<Text>>,
    }
}
