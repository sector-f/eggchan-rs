use actix::prelude::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use failure::Error;
use std::io;

use models::PostInsert;
use messages::*;
use responses::*;

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<ListBoards> for DbExecutor {
    type Result = Result<Vec<BoardResponse>, Error>;

    fn handle(&mut self, _msg: ListBoards, _: &mut Self::Context) -> Self::Result {
        use schema::boards;

        let pool = self.0.get()?;

        Ok(
            boards::table
            .select((
                boards::columns::name,
                boards::columns::description,
            ))
            .get_results(&*pool)?
        )
    }
}

impl Handler<ListThreads> for DbExecutor {
    type Result = Result<Vec<PostResponse>, Error>;

    fn handle(&mut self, msg: ListThreads, _: &mut Self::Context) -> Self::Result {
        use schema::boards;
        use schema::posts;

        let pool = self.0.get()?;

        let board_id: i32 =
            boards::table
            .select(boards::columns::id)
            .filter(boards::columns::name.eq(&msg.board_name))
            .first::<i32>(&*pool)?;

        Ok(
            posts::table
            .select((
                posts::columns::post_num,
                posts::columns::reply_to,
                posts::columns::time,
                posts::columns::comment,
            ))
            .filter(posts::columns::board_id.eq(board_id))
            .filter(posts::columns::reply_to.is_null())
            .get_results::<PostResponse>(&*pool)?
        )
    }
}

impl Handler<MakePost> for DbExecutor {
    type Result = Result<i32, Error>;

    fn handle(&mut self, msg: MakePost, _: &mut Self::Context) -> Self::Result {
        use schema::posts;
        use schema::boards;

        let pool = self.0.get()?;

        // SELECT id FROM boards WHERE name = msg.board_name;
        let board_id: i32 =
            boards::table
            .select(boards::columns::id)
            .filter(boards::columns::name.eq(&msg.board_name))
            .first(&*pool)?;

        let new_post = PostInsert {
            board_id: board_id,
            reply_to: msg.reply_to,
            image: None,
            comment: msg.comment,
        };

        // INSERT INTO posts (board_id, reply_to, comment) VALUES (board_id, msg.reply_to, msg.comment) RETURNING id;
        let post_num: i32 =
            diesel::insert_into(posts::table)
            .values(new_post)
            .returning(posts::columns::post_num)
            .get_results(&*pool)?[0];

        Ok(post_num)
    }
}
