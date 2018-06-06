use actix::prelude::*;
use failure::Error;
use responses::*;

pub struct ListBoards;

impl Message for ListBoards {
    type Result = Result<Vec<BoardResponse>, Error>;
}

pub struct ListThreads {
    pub board_name: String,
}

impl Message for ListThreads {
    type Result = Result<Vec<PostResponse>, Error>;
}

pub struct MakePost {
    pub board_name: String,
    pub reply_to: Option<i32>,
    pub image: Option<Vec<u8>>,
    pub comment: String,
}

impl Message for MakePost {
    type Result = Result<i32, Error>;
}
