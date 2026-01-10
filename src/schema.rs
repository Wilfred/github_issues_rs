// @generated automatically by Diesel CLI.

diesel::table! {
    repositories (id) {
        id -> Integer,
        user -> Text,
        name -> Text,
    }
}
