use super::*;

pub async fn from_handler(
    database: &Database,
    collection: &str,
    from: &str,
) -> Result<DateTime<Utc>, ()> {
    match from {
        "oldest" => {
            let pipeline = match create_aggregate_timeline_pipeline(collection, 1, 1) {
                Ok(result) => result,
                Err(result) => {
                    eprintln!("failed to create pipeline\ndetails : {}", result);
                    return Err(());
                }
            };
            let start_time_timestamp = match collection {
                common_constants::DATABASE_COLLECTION_TICKER => {
                    let mut cursor = match stream::DatabaseStream::<data::Ticker>::aggregate(
                        database, None, pipeline, None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").timestamp,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }

                common_constants::DATABASE_COLLECTION_EXECUTIONS => {
                    let mut cursor = match stream::DatabaseStream::<data::Execution>::aggregate(
                        database, None, pipeline, None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").timestamp,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }
                common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
                | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {
                    let mut cursor = match stream::DatabaseStream::<data::OrderBook>::aggregate(
                        database,
                        Some(collection),
                        pipeline,
                        None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").received_at,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }
                _ => {
                    eprintln!("invalid collection!");
                    return Err(());
                }
            };
            Ok(Utc.timestamp(
                start_time_timestamp.trunc() as i64,
                (start_time_timestamp.fract() * 1_000_000_000.0) as u32,
            ))
        }

        _ => match DateTime::parse_from_rfc3339(from) {
            Ok(result) => Ok(result.with_timezone(&Utc)),
            Err(result) => {
                eprintln!("<FROM> : {}, {:?}", from, result);
                Err(())
            }
        },
    }
}

pub async fn to_handler(
    database: &Database,
    collection: &str,
    to: &str,
) -> Result<DateTime<Utc>, ()> {
    match to {
        "newest" => {
            let pipeline = match create_aggregate_timeline_pipeline(collection, -1, 1) {
                Ok(result) => result,
                Err(result) => {
                    eprintln!("failed to create pipeline\ndetails : {}", result);
                    return Err(());
                }
            };
            let end_time_timestamp = match collection {
                common_constants::DATABASE_COLLECTION_TICKER => {
                    let mut cursor = match stream::DatabaseStream::<data::Ticker>::aggregate(
                        database, None, pipeline, None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").timestamp,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }

                common_constants::DATABASE_COLLECTION_EXECUTIONS => {
                    let mut cursor = match stream::DatabaseStream::<data::Execution>::aggregate(
                        database, None, pipeline, None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").timestamp,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }
                common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
                | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {
                    let mut cursor = match stream::DatabaseStream::<data::OrderBook>::aggregate(
                        database,
                        Some(collection),
                        pipeline,
                        None,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(result) => {
                            eprintln!("failed to get min timestamp\ndetails : {}", result);
                            return Err(());
                        }
                    };
                    match cursor.next().await {
                        Some(content) => content.expect("failed to get!").received_at,
                        None => {
                            eprintln!("failed to get min timestamp");
                            return Err(());
                        }
                    }
                }
                _ => {
                    eprintln!("invalid collection!");
                    return Err(());
                }
            };
            Ok(Utc.timestamp(
                end_time_timestamp.trunc() as i64,
                (end_time_timestamp.fract() * 1_000_000_000.0) as u32,
            ))
        }

        _ => match DateTime::parse_from_rfc3339(to) {
            Ok(result) => Ok(result.with_timezone(&Utc)),
            Err(result) => {
                eprintln!("<FROM> : {}, {:?}", to, result);
                Err(())
            }
        },
    }
}
