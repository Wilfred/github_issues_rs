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
        author -> Nullable<Text>,
        last_synced_at -> Nullable<Text>,
    }
}

diesel::table! {
    labels (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    issue_labels (id) {
        id -> Integer,
        issue_id -> Integer,
        label_id -> Integer,
    }
}

diesel::table! {
    issue_reactions (id) {
        id -> Integer,
        issue_id -> Integer,
        reaction_type -> Text,
        count -> Integer,
    }
}

diesel::joinable!(issue_labels -> issues (issue_id));
diesel::joinable!(issue_labels -> labels (label_id));
diesel::joinable!(issue_reactions -> issues (issue_id));

diesel::allow_tables_to_appear_in_same_query!(
    repositories,
    issues,
    labels,
    issue_labels,
    issue_reactions,
);
