use chrono::prelude::*;
use schema::*;

#[derive(Queryable, Insertable)]
pub struct Board {
    id: i32,
    name: String,
    description: Option<String>,
}

#[derive(Queryable, Insertable)]
pub struct Image {
    id: i32,
    filepath: String,
    thumbpath: String,
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
