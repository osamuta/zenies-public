use clap::{Arg, Command};

mod channel_handler;
mod daemon_main;
mod event_handler;
mod handler;
mod initialize;
mod misc;
mod order;
mod trader;

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("status")
                .about("show the status.")
                .arg(Arg::new("IDENT").required(true).help("identifier")),
        )
        .subcommand(
            Command::new("start").about("start the program.").arg(
                Arg::new("IDENT")
                    .required(true)
                    .help("config file path. it will load [IDENT].toml."),
            ),
        )
        .subcommand(
            Command::new("shutdown")
                .about("shutdown the program.")
                .arg(Arg::new("IDENT").required(true).help("identifier")),
        )
        .get_matches();

    if let Some(matched) = matches.subcommand_matches("status") {
        if let Some(ident) = matched.value_of("IDENT") {
            handler::status_handler(ident).await;
        }
    } else if let Some(matched) = matches.subcommand_matches("start") {
        if let Some(ident) = matched.value_of("IDENT") {
            handler::start_handler(ident).await;
        }
    } else if let Some(matched) = matches.subcommand_matches("shutdown") {
        if let Some(ident) = matched.value_of("IDENT") {
            handler::shutdown_handler(ident).await;
        }
    } else {
    }
}
