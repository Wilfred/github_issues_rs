use diesel::prelude::*;
use crate::schema::{repositories, issues};

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
}
