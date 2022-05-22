use chrono::prelude::*;
use mongodb::{
    bson, bson::doc, bson::Document, options, options::FindOptions, results, Client, Cursor,
};
use rayon::prelude::*;
use serde::Serialize;
use std::result::Result;

use common::*;

mod constants;
pub mod data;
pub mod list;
pub mod local;
pub mod simulation;
pub mod stream;
#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct Database {
    _client: Client,
    _database: mongodb::Database,
}

impl Database {
    pub async fn new(path: &str, database: &str) -> Result<Self, String> {
        let client = match Client::with_uri_str(path).await {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to connect a mongodb server!\n{:?}",
                    result
                ));
            }
        };

        let database = client.database(database);

        Ok(Database {
            _client: client,
            _database: database,
        })
    }

    pub async fn create<'a, D: Serialize + std::fmt::Debug + Sync>(
        &self,
        collection_name: &str,
        data: &[D],
        option: Option<options::InsertManyOptions>,
    ) -> Result<results::InsertManyResult, String> {
        let collection = self._database.collection::<Document>(collection_name);
        let mut bson_doc = Vec::with_capacity(data.len());
        for d in data {
            match bson::to_document(d) {
                Ok(result) => bson_doc.push(result),
                Err(result) => {
                    return Err(error_message!(
                "failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}",
                result,
                data
            ))
                }
            }
        }

        match collection.insert_many(bson_doc, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to insert posts!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn create_by_documents(
        &self,
        collection_name: &str,
        data: &[Document],
        option: Option<options::InsertManyOptions>,
    ) -> Result<results::InsertManyResult, String> {
        let collection = self._database.collection::<Document>(collection_name);
        match collection.insert_many(data, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to insert posts!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn request<D: Serialize + std::fmt::Debug>(
        &self,
        collection_name: &str,
        query: &D,
        option: Option<options::FindOptions>,
    ) -> Result<Cursor<bson::Document>, String> {
        let collection = self._database.collection::<Document>(collection_name);
        let bson_doc = match bson::to_document(query) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                "failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}",
                result,
                query
            ))
            }
        };

        match collection.find(bson_doc, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to request query!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn request_raw<D: Into<Option<bson::Document>>>(
        &self,
        collection_name: &str,
        query: D,
        option: Option<options::FindOptions>,
    ) -> Result<Cursor<bson::Document>, String> {
        let collection = self._database.collection::<Document>(collection_name);

        match collection.find(query, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to request query!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn update<D: Serialize + std::fmt::Debug>(
        &self,
        collection_name: &str,
        query: &D,
        update: &[D],
        option: Option<options::UpdateOptions>,
    ) -> Result<results::UpdateResult, String> {
        let collection = self._database.collection::<Document>(collection_name);
        let query_bson_doc = match bson::to_document(query) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                "failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}",
                result,
                query
            ))
            }
        };

        let mut update_bson_doc = Vec::with_capacity(update.len());
        for d in update {
            match bson::to_document(d) {
                Ok(result) => update_bson_doc.push(result),
                Err(result) => {
                    return Err(error_message!("failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}", result, update))
                },
            }
        }

        match collection
            .update_many(query_bson_doc, update_bson_doc, option)
            .await
        {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to insert posts!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn delete<D: Serialize + std::fmt::Debug>(
        &self,
        collection_name: &str,
        query: &D,
        option: Option<options::DeleteOptions>,
    ) -> Result<results::DeleteResult, String> {
        let collection = self._database.collection::<Document>(collection_name);
        let bson_doc = match bson::to_document(query) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                "failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}",
                result,
                query
            ))
            }
        };

        match collection.delete_many(bson_doc, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!("failed to delet!\ndetails : {:?}", result)),
        }
    }

    pub async fn count_documents<D: Into<Option<bson::Document>>>(
        &self,
        collection_name: &str,
        query: D,
        option: Option<options::CountOptions>,
    ) -> Result<u64, String> {
        let collection = self._database.collection::<Document>(collection_name);

        match collection.count_documents(query, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!("failed to count!\ndetails : {:?}", result)),
        }
    }

    pub async fn aggregate<D: IntoIterator<Item = bson::Document>>(
        &self,
        collection_name: &str,
        pipeline: D,
        option: Option<options::AggregateOptions>,
    ) -> Result<Cursor<bson::Document>, String> {
        let collection = self._database.collection::<Document>(collection_name);

        match collection.aggregate(pipeline, option).await {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to request query!\ndetails : {:?}",
                result
            )),
        }
    }
}

pub trait Generater {
    type Output;
    fn generate_from_document(doc: bson::Document) -> Result<Self::Output, String>;
}

pub fn generate_to_ticker(doc: bson::Document) -> Result<data::Ticker, String> {
    match bson::from_document(doc) as bson::de::Result<data::Ticker> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize!\ndetails : {:?}",
            result
        )),
    }
}

pub fn generate_to_execution(doc: bson::Document) -> Result<data::Execution, String> {
    match bson::from_document(doc) as bson::de::Result<data::Execution> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize!\ndetails : {:?}",
            result
        )),
    }
}

pub fn generate_to_order_book(doc: bson::Document) -> Result<data::OrderBook, String> {
    match bson::from_document(doc) as bson::de::Result<data::OrderBook> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize!\ndetails : {:?}",
            result
        )),
    }
}

pub fn create_duration_query<Z: TimeZone>(
    collection: &str,
    start_time: &DateTime<Z>,
    end_time: &DateTime<Z>,
) -> Result<Document, String> {
    match collection {
        common_constants::DATABASE_COLLECTION_TICKER
        | common_constants::DATABASE_COLLECTION_EXECUTIONS => Ok(doc! {
            "timestamp": {
             "$gte": start_time.timestamp_nanos() as f64 / 1_000_000_000.0,
             "$lte": end_time.timestamp_nanos() as f64 / 1_000_000_000.0,
            }
        }),
        common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
        | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => Ok(doc! {
            "received_at": {
             "$gte": start_time.timestamp_nanos() as f64 / 1_000_000_000.0,
             "$lte": end_time.timestamp_nanos() as f64 / 1_000_000_000.0,
            }
        }),
        _ => Err(error_message!("invalid collection!\n{}", collection)),
    }
}

pub fn create_find_option_timeline(
    collection: &str,
    order: i64,
    limit: Option<i64>,
) -> Result<FindOptions, String> {
    Ok(FindOptions::builder()
        .sort(match collection {
            common_constants::DATABASE_COLLECTION_TICKER
            | common_constants::DATABASE_COLLECTION_EXECUTIONS => {
                doc! { "timestamp": order }
            }
            common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
            | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {
                doc! { "received_at": order }
            }
            _ => {
                return Err(error_message!("invalid collection!"));
            }
        })
        .limit(limit)
        .build())
}

pub fn create_aggregate_timeline_pipeline(
    collection: &str,
    order: i64,
    num: i64,
) -> Result<Vec<Document>, String> {
    let mut pipeline = match collection {
        common_constants::DATABASE_COLLECTION_TICKER
        | common_constants::DATABASE_COLLECTION_EXECUTIONS => {
            vec![doc! {
                "$sort": {
                    "timestamp": order
                }
            }]
        }
        common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY
        | common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL => {
            vec![doc! {
                "$sort": {
                    "received_at": order
                }
            }]
        }
        _ => return Err(error_message!("invalid collection!\n{}", collection)),
    };
    pipeline.push(doc! {
        "$limit": num
    });
    Ok(pipeline)
}
