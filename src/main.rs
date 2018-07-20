#![feature(plugin, custom_attribute)]
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
use std::convert::From;
use std::io::Read;
use std::ops::Deref;

pub mod models;
pub mod responses;
use responses::*;
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

#[derive(Serialize)]
struct ApiError {
    code: u16,
    message: String,
}

impl<'a> From<Custom<&'a str>> for ApiError {
    fn from(custom: Custom<&'a str>) -> ApiError {
        ApiError {
            code: custom.0.code,
            message: custom.1.to_string(),
        }
    }
}

#[get("/boards")]
fn list_boards(conn: DbConn) -> Result<Json<Vec<BoardResponse>>, Status> {
    use schema::boards;
    use schema::categories;

    match boards::table.left_join(categories::table)
        .select((
            boards::columns::name,
            boards::columns::description,
            categories::columns::name.nullable(),
        )).get_results::<BoardResponse>(&*conn) {
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

    // Prevent individual posts from being listed
    // Maybe this should be allowed though?
    if let Ok(posts) =
        posts::table
        .select(posts::columns::post_num)
        .filter(posts::columns::post_num.eq(id))
        .filter(posts::columns::reply_to.is_not_null())
        .first::<i32>(&*conn) {
            return Err(Custom(Status::NotFound, "Thread not found"));
        }

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
fn post_thread<'a>(conn: DbConn, board: String, data: Data, cont_type: &ContentType) -> Result<Json<PostCreatedResponse>, Custom<&'static str>> {
    use schema::boards;
    use schema::posts;
    use schema::posts::dsl::*;
    use diesel::insert_into;
    use models::Post;

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or_else(
        || Custom(
            Status::BadRequest,
            "`Content-Type: multipart/form-data` boundary param not provided".into()
        )
    )?;

    let mut comment_text = String::new();

    let mut multipart = Multipart::with_body(data.open(), boundary);
    let multipart_result = multipart.foreach_entry(|field| {
        let is_text = &field.is_text();
        let headers = field.headers;

        match &**headers.name {
            "comment" => {
                if ! is_text {
                    return;
                }

                let mut data = field.data;
                let _ = data.read_to_string(&mut comment_text);
            },
            _ => {
                return;
            },
        }

    });

    if let Err(_) = multipart_result {
        return Err(Custom(
            Status::BadRequest,
            "Could not parse multipart data",
        ));
    }

    if comment_text.is_empty() {
        return Err(Custom(Status::BadRequest, "Comment must be non-empty UTF8"));
    }

    let id = match boards::table
        .select(boards::columns::id)
        .filter(boards::columns::name.eq(&board))
        .first::<i32>(&*conn) {
            Ok(id) => id,
            Err(_) => {
                return Err(Custom(Status::Forbidden, "Board does not exist"));
            },
        };

    match insert_into(posts).values((
        posts::columns::board_id.eq(id),
        posts::columns::comment.eq(&comment_text),
    )).get_result::<Post>(&*conn) {
        Ok(post) => {
            Ok(Json(PostCreatedResponse { board: board, post_num: post.post_num }))
        },
        Err(_) => {
            Err(Custom(Status::InternalServerError, "Internal server error"))
        },
    }
}

#[post("/board/<board>/<thread>", format="multipart/form-data", data="<data>")]
fn post_comment<'a>(conn: DbConn, board: String, thread: i32, data: Data, cont_type: &ContentType) -> Result<Json<PostCreatedResponse>, Custom<&'static str>> {
    use schema::boards;
    use schema::posts;
    use schema::posts::dsl::*;
    use diesel::insert_into;
    use models::Post;

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or_else(
        || Custom(
            Status::BadRequest,
            "`Content-Type: multipart/form-data` boundary param not provided".into()
        )
    )?;

    let mut comment_text = String::new();

    let mut multipart = Multipart::with_body(data.open(), boundary);
    let multipart_result = multipart.foreach_entry(|field| {
        let is_text = &field.is_text();
        let headers = field.headers;

        match &**headers.name {
            "comment" => {
                if ! is_text {
                    return;
                }

                let mut data = field.data;
                let _ = data.read_to_string(&mut comment_text);
            },
            _ => {
                return;
            },
        }

    });

    if let Err(_) = multipart_result {
        return Err(Custom(
            Status::BadRequest,
            "Could not parse multipart data",
        ));
    }

    if comment_text.is_empty() {
        return Err(Custom(Status::BadRequest, "Comment must be non-empty UTF8"));
    }

    let id = match boards::table
        .select(boards::columns::id)
        .filter(boards::columns::name.eq(&board))
        .first::<i32>(&*conn) {
            Ok(id) => id,
            Err(_) => {
                return Err(Custom(Status::Forbidden, "Board does not exist"));
            },
        };

    // Make sure the post we're replying to is an OP
    match posts::table
        .select(posts::columns::post_num)
        .filter(posts::columns::post_num.eq(thread))
        .filter(posts::columns::reply_to.is_null())
        .get_results::<i32>(&*conn) {
            Ok(results) => {
                if results.len() == 0 {
                    return Err(Custom(Status::Forbidden, "Replies must be to the first post in a thread"));
                }
            },
            Err(_) => {
                return Err(Custom(Status::InternalServerError, "Internal server error"));
            }
        }

    match insert_into(posts).values((
        posts::columns::board_id.eq(id),
        posts::columns::reply_to.eq(thread),
        posts::columns::comment.eq(&comment_text),
    )).get_result::<Post>(&*conn) {
        Ok(post) => {
            Ok(Json(PostCreatedResponse { board: board, post_num: post.post_num }))
        },
        Err(_) => {
            Err(Custom(Status::InternalServerError, "Internal server error"))
        },
    }
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
