extern crate actix;
use actix::prelude::*;

extern crate actix_web;
use actix_web::{http, middleware, server, App, AsyncResponder, HttpRequest, HttpResponse, Responder, FromRequest, Path};

#[macro_use] extern crate diesel;
// use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;

extern crate r2d2;
use r2d2::Pool;
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

pub mod db;
use db::*;
pub mod models;
pub mod responses;
pub mod messages;
use messages::*;
pub mod schema;

pub struct State {
    db: Addr<Syn, DbExecutor>,
}

fn make_post(req: HttpRequest<State>) -> impl Responder {
    // req.state()
    //     .db
    //     .send(CreateUser {
    //         name: name.to_owned(),
    //     })
    //     .from_err()
    //     .and_then(|res| match res {
    //         Ok(user) => Ok(HttpResponse::Ok().json(user)),
    //         Err(_) => Ok(HttpResponse::InternalServerError().into()),
    //     })
    // .responder()
    "unimplemented"
}

fn list_boards(req: HttpRequest<State>) -> impl Responder {
    req.state()
        .db
        .send(ListBoards)
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn list_threads((board_name, req): (Path<String>, HttpRequest<State>)) -> impl Responder {
    req.state()
        .db
        .send(ListThreads { board_name: board_name.to_string() })
        .and_then(|res| match res {
            Ok(threads) => Ok(HttpResponse::Ok().json(threads)),
            Err(e) => match e.cause().downcast_ref().unwrap() { // Is this unwrap() safe?
                DieselError::NotFound => Ok(HttpResponse::NotFound().into()),
                _ => Ok(HttpResponse::InternalServerError().into()),
            },
        })
    .responder()
}

fn show_thread(req: HttpRequest<State>) -> impl Responder {
    "unimplemented"
}

fn main() {
    let manager = ConnectionManager::new("postgresql://127.0.0.1/eggchan");
    let pool = Pool::new(manager).unwrap();

    let sys = actix::System::new("eggchan");
    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    server::new(move || {
        App::with_state(State { db: addr.clone() })
        .resource("/v1/boards", |r| {
            r.method(http::Method::GET).with(list_boards);
        })
        .resource("/v1/board/{name}", |r| {
            r.method(http::Method::GET).with(list_threads);
        })
        // .resource("/v1/{board}", |r| {
        //     r.method(http::Method::POST).with(make_post);
        // })
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    let _ = sys.run();
}
