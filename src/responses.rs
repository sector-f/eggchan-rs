use chrono::prelude::*;

#[derive(Queryable, Serialize)]
pub struct BoardResponse {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct PostResponse {
    post_num: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<i32>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // image: Option<String>,
    time: DateTime<Utc>,
    comment: String,
}

#[derive(Queryable, Serialize)]
pub struct PostCreatedResponse {
    pub board: String,
    pub post_num: i32,
}

#[derive(Queryable, Serialize)]
pub struct ImageResponse {
    filepath: String,
}
