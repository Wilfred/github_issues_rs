// @generated automatically by Diesel CLI.

diesel::table! {
    repositories (id) {
        id -> Integer,
        user -> Text,
        name -> Text,
    }
}

diesel::table! {
    issues (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
    }
}
