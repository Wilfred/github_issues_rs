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
    pub title: String,
    pub body: String,
}

#[derive(Insertable)]
#[diesel(table_name = issues)]
pub struct NewIssue {
    pub title: String,
    pub body: String,
}
