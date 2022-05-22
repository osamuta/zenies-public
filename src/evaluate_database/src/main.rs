mod handler;

use chrono::prelude::*;
use chrono::DateTime;
use chrono::Duration;
use clap::{Arg, Command};
use common::*;
use database::*;
use futures::stream::{Stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("PERIOD")
                .value_name("PERIOD")
                .short("p")
                .long("period")
                .help("skipping period in seconds. It must be float number.")
        )
        .arg(
            Arg::new("URL")
                .required(true)
                .help("mongodb url")
        )
        .arg(
            Arg::new("TARGET")
                .required(true)
                .help("output file path. .csv and .json are supported.")
        )
        .arg(
            Arg::new("COLLECTION")
                .required(true)
                .help("mongodb collection. Alternatives are ticker, executions, order_book_buy and order_book_sell.\norder_book_buy and order_book_sell are not supported csv files!")
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

    if let (Some(url), Some(target), Some(collection), Some(from), Some(to)) = (
        matches.value_of("URL"),
        matches.value_of("TARGET"),
        matches.value_of("COLLECTION"),
        matches.value_of("FROM"),
        matches.value_of("TO"),
    ) {
        let database = match Database::new(url, common_constants::DATABASE_NAME).await {
            Ok(result) => result,
            Err(result) => {
                eprintln!("{}", result);
                return;
            }
        };

        let file_type = match target.split('.').last() {
            Some(result) => result,
            None => {
                eprintln!("target must has a extension! {}", target);
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

        match file_type {
            "csv" | "json" => {}
            _ => {
                eprintln!("{} is not supported!", file_type);
                return;
            }
        }

        let duration = match matches.value_of("PERIOD") {
            Some(result) => match result.parse::<i64>() {
                Ok(result) => Duration::seconds(result),
                Err(result) => {
                    eprintln!("PERIOD must be float number! {}", result);
                    return;
                }
            },
            None => Duration::seconds(300),
        };

        let (start_time, end_time) = tokio::join!(
            handler::from_handler(&database, collection, from),
            handler::to_handler(&database, collection, to)
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

        let options = match create_find_option_timeline(collection, 1) {
            Ok(content) => content,
            Err(result) => {
                eprintln!("failed to create query!\ndetails : {}", result);
                return;
            }
        };

        match collection {
            common_constants::DATABASE_COLLECTION_TICKER => {
                let mut ticker_stream = match stream::DatabaseStream::<data::Ticker>::request(
                    &database,
                    None,
                    query.clone(),
                    options.clone(),
                )
                .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        eprintln!("failed to request!\ndetails : {}", result);
                        return;
                    }
                };
                let bar = ProgressBar::new(ticker_stream.size_hint().0 as u64);
                bar.set_style(ProgressStyle::default_bar()
                    .template("Downloading {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",).progress_chars("#>-"));
                let mut ranges = Vec::new();
                let mut range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                loop {
                    tokio::select! {
                        Some(result) = ticker_stream.next() => {
                            bar.inc(1);
                            match result {
                                Ok(result) => {
                                    let time = Utc.timestamp(
                                        result.timestamp.trunc() as i64,
                                        (result.timestamp.fract() * 1_000_000_000.0) as u32,
                                    );
                                    match range.0 {
                                        Some(_) => {
                                            match range.1 {
                                                Some(content) => {
                                                    if time > (content + duration) {
                                                        ranges.push(range);
                                                        range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                                                        range.0 = Some(time);
                                                    } else {
                                                        range.1 = Some(time);
                                                    }
                                                }
                                                None => {
                                                    range.1 = Some(time);
                                                }
                                            }
                                        }
                                        None => {
                                            range.0 = Some(time);
                                        }
                                    }
                                }
                                Err(result) => {
                                    eprintln!("got malformed data!\ndetails : {}", result);
                                }
                            }
                        }
                        else => {
                            ranges.push(range);
                            break;
                        }
                    }
                }
                bar.finish();

                match file_type {
                    "csv" => {
                        let mut csv_writer = match csv::Writer::from_path(target) {
                            Ok(result) => result,
                            Err(_) => {
                                eprintln!("failed to create TARGET file.");
                                return;
                            }
                        };
                        for d in ranges {
                            if let Err(result) = csv_writer.serialize(d) {
                                eprintln!("failed to write! {:?}", result);
                                return;
                            }
                        }
                    }

                    "json" => {}

                    _ => {
                        eprintln!("{} is not supported!", file_type);
                        return;
                    }
                }
            }

            common_constants::DATABASE_COLLECTION_EXECUTIONS => {
                let mut execution_stream = match stream::DatabaseStream::<data::Execution>::request(
                    &database,
                    None,
                    query.clone(),
                    options.clone(),
                )
                .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        eprintln!("failed to request!\ndetails : {}", result);
                        return;
                    }
                };
                let bar = ProgressBar::new(execution_stream.size_hint().0 as u64);
                bar.set_style(ProgressStyle::default_bar()
                    .template("Downloading {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",).progress_chars("#>-"));
                let mut ranges = Vec::new();
                let mut range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                loop {
                    tokio::select! {
                        Some(result) = execution_stream.next() => {
                            bar.inc(1);
                            match result {
                                Ok(result) => {
                                    let time = Utc.timestamp(
                                        result.timestamp.trunc() as i64,
                                        (result.timestamp.fract() * 1_000_000_000.0) as u32,
                                    );
                                    match range.0 {
                                        Some(_) => {
                                            match range.1 {
                                                Some(content) => {
                                                    if time > (content + duration) {
                                                        ranges.push(range);
                                                        range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                                                        range.0 = Some(time);
                                                    } else {
                                                        range.1 = Some(time);
                                                    }
                                                }
                                                None => {
                                                    range.1 = Some(time);
                                                }
                                            }
                                        }
                                        None => {
                                            range.0 = Some(time);
                                        }
                                    }
                                }
                                Err(result) => {
                                    eprintln!("got malformed data!\ndetails : {}", result);
                                }
                            }
                        }
                        else => {
                            ranges.push(range);
                            break;
                        }
                    }
                }
                bar.finish();

                match file_type {
                    "csv" => {
                        let mut csv_writer = match csv::Writer::from_path(target) {
                            Ok(result) => result,
                            Err(_) => {
                                eprintln!("failed to create TARGET file.");
                                return;
                            }
                        };
                        for d in ranges {
                            if let Err(result) = csv_writer.serialize(d) {
                                eprintln!("failed to write! {:?}", result);
                                return;
                            }
                        }
                    }

                    "json" => {}
                    _ => {
                        eprintln!("{} is not supported!", file_type);
                        return;
                    }
                }
            }

            common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
            | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {
                let mut order_book_stream =
                    match stream::DatabaseStream::<data::OrderBook>::request(
                        &database,
                        Some(collection),
                        query.clone(),
                        options.clone(),
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to request!\ndetails : {}", result);
                            return;
                        }
                    };
                let bar = ProgressBar::new(order_book_stream.size_hint().0 as u64);
                bar.set_style(ProgressStyle::default_bar()
                    .template("Downloading {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",).progress_chars("#>-"));
                let mut ranges = Vec::new();
                let mut range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                loop {
                    tokio::select! {
                        Some(result) = order_book_stream.next() => {
                            bar.inc(1);
                            match result {
                                Ok(result) => {
                                    let time = Utc.timestamp(
                                        result.received_at.trunc() as i64,
                                        (result.received_at.fract() * 1_000_000_000.0) as u32,
                                    );
                                    match range.0 {
                                        Some(_) => {
                                            match range.1 {
                                                Some(content) => {
                                                    if time > (content + duration) {
                                                        ranges.push(range);
                                                        range = (Option::<DateTime<Utc>>::None, Option::<DateTime<Utc>>::None);
                                                        range.0 = Some(time);
                                                    } else {
                                                        range.1 = Some(time);
                                                    }
                                                }
                                                None => {
                                                    range.1 = Some(time);
                                                }
                                            }
                                        }
                                        None => {
                                            range.0 = Some(time);
                                        }
                                    }
                                }
                                Err(result) => {
                                    eprintln!("got malformed data!\ndetails : {}", result);
                                }
                            }
                        }
                        else => {
                            ranges.push(range);
                            break;
                        }
                    }
                }
                bar.finish();

                match file_type {
                    "csv" => {
                        let mut csv_writer = match csv::Writer::from_path(target) {
                            Ok(result) => result,
                            Err(_) => {
                                eprintln!("failed to create TARGET file.");
                                return;
                            }
                        };
                        for d in ranges {
                            if let Err(result) = csv_writer.serialize(d) {
                                eprintln!("failed to write! {:?}", result);
                                return;
                            }
                        }
                    }

                    "json" => {}

                    _ => {
                        eprintln!("{} is not supported!", file_type);
                        return;
                    }
                }
            }

            _ => {
                eprintln!("invalid collection!");
                return;
            }
        }
    }
}
