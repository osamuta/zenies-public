use std::result::Result;
//use std::sync::{Arc, Mutex};

use clap::{Arg, Command};
use libsystemd::*;

use common::{common_constants, error_message, error_message_colored, *};
//use database::*;
use liquid::*;

mod channel_handler;
mod daemon_main;
mod event_handler;
mod handler;
mod initialize;
mod misc;

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(Command::new("status").about("show the status."))
        .subcommand(
            Command::new("start")
                .about("start the program.")
                .arg(Arg::new("CONFIG").required(true).help("config file path.")),
        )
        .subcommand(Command::new("shutdown").about("shutdown the program."))
        .get_matches();

    if let Some(_matched) = matches.subcommand_matches("status") {
        handler::status_handler().await;
    } else if let Some(matched) = matches.subcommand_matches("start") {
        if let Some(config_path) = matched.value_of("CONFIG") {
            handler::start_handler(config_path).await;
        }
    } else if let Some(_matched) = matches.subcommand_matches("shutdown") {
        handler::shutdown_handler().await;
    } else {
    }
}
