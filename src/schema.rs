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
        repository_id -> Integer,
        number -> Integer,
        title -> Text,
        body -> Text,
        created_at -> Text,
        state -> Text,
        is_pull_request -> Bool,
    }
}
