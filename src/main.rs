#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;

extern crate rocket_contrib;
use rocket_contrib::Json;

#[macro_use] extern crate diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;

extern crate r2d2;
use r2d2::{Pool, PooledConnection};
extern crate r2d2_diesel;
use r2d2_diesel::ConnectionManager;

extern crate serde;
use serde::Serializer;

#[macro_use] extern crate serde_derive;
extern crate serde_json;

extern crate chrono;
use chrono::prelude::*;

extern crate failure;
// use failure::Error

extern crate futures;
use futures::future::Future;

// use std::path::PathBuf;
use std::process::exit;

use std::ops::Deref;

// pub mod db;
// use db::*;
pub mod models;
pub mod responses;
use responses::*;
// pub mod messages;
// use messages::*;
pub mod schema;

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<PgPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[get("/v1/boards")]
fn list_boards(conn: DbConn) -> Result<Json<Vec<BoardResponse>>, Status> {
    use schema::boards;

    match boards::table
        .select((
            boards::columns::name,
            boards::columns::description,
        )).get_results(&*conn) {
            Ok(boards) => {
                Ok(Json(boards))
            },
            Err(_) => {
                Err(Status::InternalServerError)
            },
    }
}

#[get("/v1/board/<name>")]
fn list_threads(conn: DbConn, name: String) -> Result<Json<Vec<PostResponse>>, Status> {
        use schema::boards;
        use schema::posts;

        // posts::table.inner_join(boards::table).filter(boards::name.eq("diy"))

        match boards::table
            .left_join(posts::table.on(boards::columns::id.eq(posts::columns::board_id)))
            .select((
                posts::columns::post_num.nullable(),
                posts::columns::reply_to.nullable(),
                posts::columns::time.nullable(),
                posts::columns::comment.nullable(),
            ))
            .filter(boards::columns::name.eq(&name))
            .filter(posts::columns::reply_to.is_null())
            .get_results::<PostResponse>(&*conn)
            {
                Ok(threads) => Ok(Json(threads)),
                Err(_) => Err(Status::NotFound),
            }
}

#[get("/v1/board/<name>")]
fn other_list_threads(conn: DbConn, name: String) -> Result<Json<Vec<PostResponse>>, Status> {
        use schema::boards;
        use schema::posts;

        // posts::table.inner_join(boards::table).filter(boards::name.eq("diy"))

        match posts::table
            .select((
                posts::columns::post_num,
                posts::columns::reply_to,
                posts::columns::time,
                posts::columns::comment,
            ))
            .inner_join(boards::table.on(posts::columns::board_id.eq(boards::columns::id)))
            .filter(boards::columns::name.eq(&name))
            .filter(posts::columns::reply_to.is_null())
            .get_results::<PostResponse>(&*conn)
            {
                Ok(threads) => Ok(Json(threads)),
                Err(_) => Err(Status::NotFound),
            }


        // let board_id: i32 =
        //     match boards::table
        //     .select(boards::columns::id)
        //     .filter(boards::columns::name.eq(&name))
        //     .first::<i32>(&*conn) {
        //         Ok(id) => id,
        //         Err(_) => return Err(Status::NotFound),
        //     };

        // match
        //     posts::table
        //     .select((
        //         posts::columns::post_num,
        //         posts::columns::reply_to,
        //         posts::columns::time,
        //         posts::columns::comment,
        //     ))
        //     .filter(posts::columns::board_id.eq(board_id))
        //     .filter(posts::columns::reply_to.is_null())
        //     .get_results::<PostResponse>(&*conn) {
        //         Ok(threads) => Ok(Json(threads)),
        //         Err(_) => Err(Status::NotFound),
        //     }
}

#[get("/v1/board/<board>/<id>")]
fn show_thread(conn: DbConn, board: String, id: i32) -> Result<Json<Vec<PostResponse>>, Status>{
        use schema::boards;
        use schema::posts;

        let board_id: i32 =
            match boards::table
            .select(boards::columns::id)
            .filter(boards::columns::name.eq(&board))
            .first::<i32>(&*conn) {
                Ok(id) => id,
                Err(_) => return Err(Status::NotFound),
            };

        // Get OP followed by replies
        let post =
            match
                posts::table
                .select((
                    posts::columns::post_num,
                    posts::columns::reply_to,
                    posts::columns::time,
                    posts::columns::comment,
                ))
                .filter(posts::columns::board_id.eq(board_id))
                .filter(posts::columns::reply_to.is_null())
                .get_results::<PostResponse>(&*conn) {
                    Ok(threads) => Ok(Json(threads)),
                    Err(_) => Err(Status::NotFound),
                };

        unimplemented!();
}

fn init_pool(url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::new(manager).expect("db pool")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![list_boards, list_threads, show_thread])
        .manage(init_pool("postgresql://127.0.0.1/eggchan"))
        .launch();
}
