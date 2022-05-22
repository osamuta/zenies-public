mod handler;
mod transferer;

use chrono::prelude::*;
use chrono::DateTime;
use clap::{Arg, Command};
use common::*;
use database::*;
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use mongodb::bson::Document;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("SOURCE")
                .required(true)
                .help("source mongodb url")
        )
        .arg(
            Arg::new("DESTINATION")
                .required(true)
                .help("destination mongodb url")
        )
        .arg(
            Arg::new("COLLECTION")
                .required(true)
                .help("mongodb collection. ticker or executions.")
        )
        .arg(
            Arg::new("FROM")
                .required(true)
                .help("Start time. Time format must be followed by ISO 8601. Such as 1996-12-19T16:39:57-08:00. Also you can use the \"oldest\" keyword."),
        )
        .arg(
            Arg::new("TO")
                .required(true)
                .help("end time. Time format must be followed by ISO 8601. Such as 1996-12-19T16:39:57-08:00. Also you can use the \"newest\" keyword."),
        )
        .get_matches();

    if let (Some(source), Some(destination), Some(collection), Some(from), Some(to)) = (
        matches.value_of("SOURCE"),
        matches.value_of("DESTINATION"),
        matches.value_of("COLLECTION"),
        matches.value_of("FROM"),
        matches.value_of("TO"),
    ) {
        let source_database = match Database::new(source, common_constants::DATABASE_NAME).await {
            Ok(result) => result,
            Err(result) => {
                eprintln!("{}", result);
                return;
            }
        };
        let destination_database =
            match Database::new(destination, common_constants::DATABASE_NAME).await {
                Ok(result) => result,
                Err(result) => {
                    eprintln!("{}", result);
                    return;
                }
            };

        match collection {
            common_constants::DATABASE_COLLECTION_TICKER
            | common_constants::DATABASE_COLLECTION_EXECUTIONS
            | common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
            | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {}
            _ => {
                eprintln!("{} is a invalid collection", collection);
                return;
            }
        }

        let (start_time, end_time) = tokio::join!(
            handler::from_handler(&source_database, collection, from),
            handler::to_handler(&source_database, collection, to)
        );
        let start_time = match start_time {
            Ok(result) => result,
            Err(_) => return,
        };
        let end_time = match end_time {
            Ok(result) => result,
            Err(_) => return,
        };

        if start_time > end_time {
            eprintln!("FROM must be lesser than TO!");
            return;
        }

        let query = match create_duration_query(collection, &start_time, &end_time) {
            Ok(content) => content,
            Err(result) => {
                eprintln!("failed to create query!\ndetails : {}", result);
                return;
            }
        };

        let options = match create_find_option_timeline(collection, 1, None) {
            Ok(content) => content,
            Err(result) => {
                eprintln!("failed to create query!\ndetails : {}", result);
                return;
            }
        };

        let document_count = match source_database
            .count_documents(collection, query.clone(), None)
            .await
        {
            Ok(result) => result,
            Err(result) => {
                println!("{}", result);
                return;
            }
        };

        let mut cursor = match source_database
            .request_raw(collection, query, Some(options))
            .await
        {
            Ok(result) => result,
            Err(result) => {
                eprintln!("{}", result);
                return;
            }
        };

        let bar = ProgressBar::new(document_count as u64);
        bar.set_style(
        ProgressStyle::default_bar()
            .template(
                    "Transfering {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
        )
        .progress_chars("#>-"));

        let destination_database_arc = std::sync::Arc::new(destination_database);
        let collection_arc = std::sync::Arc::new(String::from(collection));
        let mut executors = Vec::new();
        let mut counter = 0;
        let cpu_cores = num_cpus::get();
        for _ in 0..cpu_cores {
            let (transmitter, receiver) = mpsc::unbounded_channel::<tokio::task::JoinHandle<()>>();
            executors.push((
                tokio::task::spawn(transferer::executor(receiver)),
                transmitter,
            ));
        }

        loop {
            tokio::task::yield_now().await;
            tokio::select! {
                Some(result) = cursor.next() => {
                    match result {
                        Ok(document) => {
                            bar.inc(1);
                            let destination_database_ref = destination_database_arc.clone();
                            let collection_ref = collection_arc.clone();
                            executors[counter % cpu_cores].1.send(tokio::task::spawn(transferer::transferer(
                                destination_database_ref,
                                document,
                                collection_ref.to_string(),
                            ))).expect("failed to send");
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        },
                    }
                }
                else => break,
            };
            counter += 1;
        }
        for v in executors {
            drop(v.1);
            v.0.await.expect("failed to ran tasks!");
        }
        bar.finish();
    }
}
