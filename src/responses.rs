use chrono::prelude::*;

#[derive(Queryable, Serialize)]
pub struct BoardResponse {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct PostResponse {
    post_num: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<i32>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // image: Option<String>,
    time: Option<DateTime<Utc>>,
    comment: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct ImageResponse {
    filepath: String,
}
