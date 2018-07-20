use chrono::prelude::*;
use schema::*;

#[derive(Queryable, Insertable)]
#[table_name = "categories"]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Insertable)]
pub struct Board {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<i32>
}

#[derive(Queryable, Insertable)]
pub struct Image {
    pub id: i32,
    pub filepath: String,
    pub thumbpath: String,
}

#[derive(Queryable)]
pub struct Post {
    pub board_id: i32,
    pub post_num: i32,
    pub reply_to: Option<i32>,
    pub image: Option<i32>,
    pub time: DateTime<Utc>,
    pub comment: String,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct PostInsert {
    pub board_id: i32,
    pub reply_to: Option<i32>,
    pub image: Option<i32>,
    pub comment: String,
}
