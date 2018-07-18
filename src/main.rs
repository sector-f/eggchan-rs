#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::Data;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::{ContentType, Status};
use rocket::response::Stream;
use rocket::response::status::Custom;

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

extern crate multipart;
use multipart::server::Multipart;
use multipart::server::save::Entries;
use multipart::server::save::SaveResult::*;

extern crate chrono;
use chrono::prelude::*;

extern crate failure;
// use failure::Error

extern crate futures;
use futures::future::Future;

// use std::path::PathBuf;
use std::process::exit;

use std::io::Read;
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

#[get("/boards")]
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

#[get("/board/<name>")]
fn list_threads(conn: DbConn, name: String) -> Result<Json<Vec<PostResponse>>, Custom<&'static str>> {
    use schema::boards;
    use schema::posts;

    match boards::table
        .left_join(posts::table.on(boards::columns::id.eq(posts::columns::board_id)))
        .select((
            posts::columns::post_num,
            posts::columns::reply_to,
            posts::columns::time,
            posts::columns::comment,
        ).nullable())
        .filter(boards::columns::name.eq(&name))
        .filter(posts::columns::reply_to.is_null())
        .get_results::<Option<PostResponse>>(&*conn)
        {
            Ok(maybe_threads) => {
                if let None = &maybe_threads.get(0) {
                    return Err(Custom(Status::NotFound, "Board not found"));
                }

                let threads = maybe_threads.into_iter().filter_map(|t| t).collect();
                return Ok(Json(threads));
            },
            Err(_) => Err(Custom(Status::InternalServerError, "Internal server error")),
        }
}

#[get("/board/<board>/<id>")]
fn show_thread(conn: DbConn, board: String, id: i32) -> Result<Json<Vec<PostResponse>>, Custom<&'static str>>{
    use schema::boards;
    use schema::posts;

    match posts::table
        .inner_join(boards::table.on(boards::columns::id.eq(posts::columns::board_id)))
        .select((
            posts::columns::post_num,
            posts::columns::reply_to,
            posts::columns::time,
            posts::columns::comment,
        ))
        .filter(boards::columns::name.eq(&board))
        .filter(posts::columns::post_num.eq(id).or(posts::columns::reply_to.eq(id)))
        .get_results::<PostResponse>(&*conn) {
            Ok(posts) => {
                match posts.len() {
                    0 => {
                        Err(Custom(Status::NotFound, "Thread not found"))
                    },
                    _ => {
                        Ok(Json(posts))
                    },
                }
            }
            Err(_) => Err(Custom(Status::InternalServerError, "Internal server error")),
        }
}

#[post("/board/<board>", format="multipart/form-data", data="<data>")]
fn post_thread<'a>(conn: DbConn, board: String, data: Data, cont_type: &ContentType) -> Result<&'a str, Custom<&'static str>> {
    use schema::boards;
    use schema::posts;
    use schema::posts::dsl::*;
    use diesel::insert_into;

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or_else(
        || Custom(
            Status::BadRequest,
            "`Content-Type: multipart/form-data` boundary param not provided".into()
        )
    )?;

    let mut multipart = Multipart::with_body(data.open(), boundary);
    let multipart_result = multipart.foreach_entry(|field| {
        let is_text = &field.is_text();
        let headers = field.headers;

        match &**headers.name {
            "comment" => {
                if ! is_text {
                    return;
                }
            },
            _ => {
                return;
            },
        }

        let mut data = field.data;
        let mut buf = Vec::new();
        let _ = data.read_to_end(&mut buf);
        let string = String::from_utf8_lossy(&buf);

        let id =
            boards::table
            .select(boards::columns::id)
            .filter(boards::columns::name.eq(&board))
            .first::<i32>(&*conn);

        if let Ok(id) = id {
            insert_into(posts).values((
                posts::columns::board_id.eq(id),
                posts::columns::comment.eq(string),
            )).execute(&*conn);
        }
    });

    if let Err(_) = multipart_result {
        return Err(Custom(
            Status::BadRequest,
            "Could not parse multipart data".into()
        ));
    }

    Ok("upload complete")
}

#[post("/board/<board>/<thread>", format="multipart/form-data", data="<data>")]
fn post_comment<'a>(conn: DbConn, board: String, thread: i32, data: Data, cont_type: &ContentType) -> Result<&'a str, Custom<&'static str>> {
    use schema::boards;
    use schema::posts;
    use schema::posts::dsl::*;
    use diesel::insert_into;

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or_else(
        || Custom(
            Status::BadRequest,
            "`Content-Type: multipart/form-data` boundary param not provided".into()
        )
    )?;

    let mut multipart = Multipart::with_body(data.open(), boundary);
    let multipart_result = multipart.foreach_entry(|field| {
        let is_text = &field.is_text();
        let headers = field.headers;

        match &**headers.name {
            "comment" => {
                if ! is_text {
                    return;
                }
            },
            _ => {
                return;
            },
        }

        let mut data = field.data;
        let mut buf = Vec::new();
        let _ = data.read_to_end(&mut buf);
        let string = String::from_utf8_lossy(&buf);

        let id =
            boards::table
            .select(boards::columns::id)
            .filter(boards::columns::name.eq(&board))
            .first::<i32>(&*conn);

        if let Ok(id) = id {
            insert_into(posts).values((
                posts::columns::board_id.eq(id),
                posts::columns::reply_to.eq(thread),
                posts::columns::comment.eq(string),
            )).execute(&*conn);
        }
    });

    if let Err(_) = multipart_result {
        return Err(Custom(
            Status::BadRequest,
            "Could not parse multipart data".into()
        ));
    }

    Ok("upload complete")
}

fn init_pool(url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::new(manager).expect("db pool")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![list_boards, list_threads, show_thread, post_thread, post_comment])
        .manage(init_pool("postgresql://127.0.0.1/eggchan"))
        .launch();
}
