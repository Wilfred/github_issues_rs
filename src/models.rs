use diesel::prelude::*;
use crate::schema::{repositories, issues, labels, issue_labels, issue_reactions};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = repositories)]
pub struct Repository {
    #[allow(dead_code)]
    pub id: i32,
    pub user: String,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = repositories)]
pub struct NewRepository {
    pub user: String,
    pub name: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = issues)]
pub struct Issue {
    #[allow(dead_code)]
    pub id: i32,
    pub repository_id: i32,
    pub number: i32,
    pub title: String,
    #[allow(dead_code)]
    pub body: String,
    pub created_at: String,
    pub state: String,
    pub is_pull_request: bool,
    pub author: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = issues)]
pub struct NewIssue {
    pub repository_id: i32,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub state: String,
    pub is_pull_request: bool,
    pub author: Option<String>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = labels)]
pub struct Label {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = labels)]
pub struct NewLabel {
    pub name: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = issue_labels)]
#[allow(dead_code)]
pub struct IssueLabel {
    pub id: i32,
    pub issue_id: i32,
    pub label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = issue_labels)]
pub struct NewIssueLabel {
    pub issue_id: i32,
    pub label_id: i32,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = issue_reactions)]
pub struct IssueReaction {
    #[allow(dead_code)]
    pub id: i32,
    #[allow(dead_code)]
    pub issue_id: i32,
    pub reaction_type: String,
    pub count: i32,
}

#[derive(Insertable)]
#[diesel(table_name = issue_reactions)]
pub struct NewIssueReaction {
    pub issue_id: i32,
    pub reaction_type: String,
    pub count: i32,
}
