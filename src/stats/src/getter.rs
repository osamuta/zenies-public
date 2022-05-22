use super::*;
use std::iter::once;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stats {
    pub start_time: f64,
    pub end_time: f64,
    pub size: f64,
    pub first: f64,
    pub last: f64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

impl Generater for Stats {
    type Output = Self;
    fn generate_from_document(doc: bson::Document) -> Result<Self, String>
    where
        Self: Sized,
    {
        match bson::from_document(doc) as bson::de::Result<Stats> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize!\ndetails : {:?}",
                result
            )),
        }
    }
}

impl Stats {
    pub fn field_iter(&self) -> impl Iterator<Item = f64> {
        once(self.start_time)
            .chain(once(self.end_time))
            .chain(once(self.size))
            .chain(once(self.first))
            .chain(once(self.last))
            .chain(once(self.sum))
            .chain(once(self.min))
            .chain(once(self.max))
            .chain(once(self.mean))
            .chain(once(self.std_dev))
    }

    pub fn name_iter(&self) -> impl Iterator<Item = &str> {
        once("start_time")
            .chain(once("end_time"))
            .chain(once("size"))
            .chain(once("first"))
            .chain(once("last"))
            .chain(once("sum"))
            .chain(once("min"))
            .chain(once("max"))
            .chain(once("mean"))
            .chain(once("std_dev"))
    }

    pub fn name_iter_with_index(&self, index: usize) -> impl Iterator<Item = String> {
        let index_str = format!("{}_", index);
        once(index_str.clone() + "start_time")
            .chain(once(index_str.clone() + "end_time"))
            .chain(once(index_str.clone() + "size"))
            .chain(once(index_str.clone() + "first"))
            .chain(once(index_str.clone() + "last"))
            .chain(once(index_str.clone() + "sum"))
            .chain(once(index_str.clone() + "min"))
            .chain(once(index_str.clone() + "max"))
            .chain(once(index_str.clone() + "mean"))
            .chain(once(index_str.clone() + "std_dev"))
    }
}

pub async fn get_stats_ticker(
    database: &Database,
    name: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Option<Stats>, String> {
    let name = String::from("$") + name;
    let pipeline = vec![
        doc! {"$match": {
          "received_at": {
          "$gte": start_time.timestamp(),
          "$lt": end_time.timestamp()
         }
        }},
        doc! {"$sort": {
          "received_at": 1
        }},
        doc! {"$group": {
          "_id": null,
          "size": { "$sum": 1 },
          "first": { "$first": &name },
          "last": { "$last": &name },
          "sum": { "$sum": &name },
          "min": {
            "$min": &name
          },
          "max": {
            "$max": &name
          },
          "mean": {
            "$avg": &name
          },
          "std_dev": {
            "$stdDevPop": &name
          },
        }},
        doc! {"$addFields": {
            "start_time": start_time.timestamp(),
            "end_time": end_time.timestamp(),
        }},
    ];
    let mut stats_stream = match stream::DatabaseStream::<Stats>::aggregate(
        &database,
        Some(common_constants::DATABASE_COLLECTION_TICKER),
        pipeline,
        None,
    )
    .await
    {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to send a query!\ndetails : {}",
                result
            ));
        }
    };
    loop {
        tokio::select! {
            Some(result) = stats_stream.next() => {
                match result {
                    Ok(result) => {
                        return Ok(Some(result));
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => return Ok(None),
        }
    }
}

pub async fn get_stats_order_book_buy(
    database: &Database,
    name: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Stats, String> {
    let name = String::from("$orders.") + name;
    let pipeline = vec![
        doc! {
            "$match": {
                "received_at": {
                    "$gte": start_time.timestamp(),
                    "$lt": end_time.timestamp()
                }
            }
        },
        doc! {"$sort": {
          "received_at": 1
        }},
        doc! {
            "$unwind": {
                "path": "$orders",
            }
        },
        doc! {
            "$group": {
                "_id": null,
                "size": {
                    "$sum": 1
                },
                "first": { "$first": &name },
                "last": { "$last": &name },
                "sum": { "$sum": &name },
                "min": {
                    "$min": &name
                },
                "max": {
                    "$max": &name
                },
                "mean": {
                    "$avg": &name
                },
                "std_dev": {
                    "$stdDevPop": &name
                },
            }
        },
    ];
    let mut stats_stream = match stream::DatabaseStream::<Stats>::aggregate(
        &database,
        Some(common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY),
        pipeline,
        None,
    )
    .await
    {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to send a query!\ndetails : {}",
                result
            ));
        }
    };
    loop {
        tokio::select! {
            Some(result) = stats_stream.next() => {
                match result {
                    Ok(result) => {
                        return Ok(result);
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => return Err(error_message!(
                "nothing!"
            )),
        }
    }
}

pub async fn get_stats_order_book_sell(
    database: &Database,
    name: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Stats, String> {
    let name = String::from("$orders.") + name;
    let pipeline = vec![
        doc! {
            "$match": {
                "received_at": {
                    "$gte": start_time.timestamp(),
                    "$lt": end_time.timestamp()
                }
            }
        },
        doc! {"$sort": {
          "received_at": 1
        }},
        doc! {
            "$unwind": {
                "path": "$orders",
            }
        },
        doc! {
            "$group": {
                "_id": null,
                "size": {
                    "$sum": 1
                },
                "first": { "$first": &name },
                "last": { "$last": &name },
                "sum": { "$sum": &name },
                "min": {
                    "$min": &name
                },
                "max": {
                    "$max": &name
                },
                "mean": {
                    "$avg": &name
                },
                "std_dev": {
                    "$stdDevPop": &name
                },
            }
        },
    ];
    let mut stats_stream = match stream::DatabaseStream::<Stats>::aggregate(
        &database,
        Some(common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL),
        pipeline,
        None,
    )
    .await
    {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to send a query!\ndetails : {}",
                result
            ));
        }
    };
    loop {
        tokio::select! {
            Some(result) = stats_stream.next() => {
                match result {
                    Ok(result) => {
                        return Ok(result);
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => return Err(error_message!(
                "nothing!"
            )),
        }
    }
}

pub async fn get_tickers(
    database: Arc<Database>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<data::Ticker>, String> {
    let collection = common_constants::DATABASE_COLLECTION_TICKER;
    let query = match create_duration_query(collection, &start_time, &end_time) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let options = match create_find_option_timeline(collection, 1, None) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let mut ticker_stream = match stream::DatabaseStream::<data::Ticker>::request(
        &database, None, query, options,
    )
    .await
    {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to send a query!\ndetails : {}",
                result
            ));
        }
    };
    let mut data = Vec::with_capacity(ticker_stream.size_hint().0);
    loop {
        tokio::select! {
            Some(result) = ticker_stream.next() => {
                match result {
                    Ok(result) => {
                        data.push(result);
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => break,
        }
    }
    Ok(data)
}

pub async fn get_executions(
    database: Arc<Database>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<data::Execution>, String> {
    let collection = common_constants::DATABASE_COLLECTION_EXECUTIONS;
    let query = match create_duration_query(collection, &start_time, &end_time) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let options = match create_find_option_timeline(collection, 1, None) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let mut ticker_stream =
        match stream::DatabaseStream::<data::Execution>::request(&database, None, query, options)
            .await
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to send a query!\ndetails : {}",
                    result
                ));
            }
        };
    let mut data = Vec::with_capacity(ticker_stream.size_hint().0);
    loop {
        tokio::select! {
            Some(result) = ticker_stream.next() => {
                match result {
                    Ok(result) => {
                        data.push(result);
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => break,
        }
    }
    Ok(data)
}

pub async fn get_order_book(
    database: Arc<Database>,
    collection: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<data::OrderBook>, String> {
    let query = match create_duration_query(collection, &start_time, &end_time) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let options = match create_find_option_timeline(collection, 1, None) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to create query!\ndetails : {}",
                result
            ));
        }
    };

    let mut ticker_stream = match stream::DatabaseStream::<data::OrderBook>::request(
        &database,
        Some(collection),
        query,
        options,
    )
    .await
    {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to send a query!\ndetails : {}",
                result
            ));
        }
    };
    let mut data = Vec::with_capacity(ticker_stream.size_hint().0);
    loop {
        tokio::select! {
            Some(result) = ticker_stream.next() => {
                match result {
                    Ok(result) => {
                        data.push(result);
                    }
                    Err(result) => {
                        return Err(error_message!(
                            "failed to get!\ndetails : {}",
                            result
                        ));
                    }
                }
            }
            else => break,
        }
    }
    Ok(data)
}
