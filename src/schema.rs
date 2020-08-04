table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        pass_hash -> Varchar,
        os_user -> Varchar,
    }
}
