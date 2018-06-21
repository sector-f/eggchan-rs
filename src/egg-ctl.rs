#[macro_use] extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

extern crate chrono;

extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

#[macro_use] extern crate diesel;
use diesel::prelude::*;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;

extern crate dotenv;

pub mod schema;
use schema::{boards, posts};
pub mod models;

use std::env;
use std::process::exit;

fn main() {
    let matches = App::new("egg-ctl")
        .arg(Arg::with_name("url")
             .short("u")
             .long("url")
             .help("Specify the PostgreSQL connection URL")
             .takes_value(true))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("list-boards")
                    .about("Lists boards in the database"))
        .subcommand(SubCommand::with_name("list-threads")
                    .about("Lists threads in a given board")
                    .arg(Arg::with_name("board")
                         .short("b")
                         .long("board")
                         .takes_value(true)
                         .required(true)))
        .get_matches();

    let _ = dotenv::dotenv();

    let arg_url = matches.value_of("url");
    let env_url = env::var("DATABASE_URL");

    let db_url = match arg_url {
        Some(url) => url.to_string(),
        None => {
            match env_url {
                Ok(url) => url,
                Err(_) => {
                    exit(1);
                },
            }
        },
    };

    let conn = match PgConnection::establish(&db_url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        },
    };

    match matches.subcommand() {
        ("list-boards", _) => {
            match boards::table.select(boards::all_columns).get_results::<models::Board>(&conn) {
                Ok(boards) => {
                    let mut table = Table::new();
                    table.add_row(row!["ID", "Name", "Description"]);
                    for board in boards {
                        let desc = if let Some(d) = board.description { d.to_string() } else { "".to_string() };
                        table.add_row(row![board.id.to_string(), board.name.to_string(), desc]);
                    }
                    table.printstd();
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                },
            }
        },
        ("list-threads", Some(thread_matches)) => {
            let board = thread_matches.value_of("board").unwrap();
        },
        _ => unreachable!(),
    }
}
