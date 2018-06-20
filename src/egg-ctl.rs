extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    let matches = App::new("egg-ctl")
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

    match matches.subcommand() {
        ("list-boards", _) => {

        },
        ("list-threads", Some(thread_matches)) => {
            let board = thread_matches.value_of("board").unwrap();
        },
        _ => unreachable!(),
    }
}
