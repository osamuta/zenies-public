use super::*;
use futures::{Stream, StreamExt};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct DatabaseStream<'a, T>
where
    T: Generater,
{
    _database: &'a Database,
    _cursor: mongodb::Cursor<bson::Document>,
    _count: Option<usize>,
    _type: PhantomData<T>,
}

impl<'a, T> DatabaseStream<'a, T>
where
    T: Generater,
{
    pub async fn request<'b>(
        source_database: &'a Database,
        collection: Option<&str>,
        query: Document,
        option: options::FindOptions,
    ) -> Result<DatabaseStream<'a, T>, String> {
        let collection = match std::any::type_name::<T>() {
            "database::data::Ticker" => common_constants::DATABASE_COLLECTION_TICKER,
            "database::data::Execution" => common_constants::DATABASE_COLLECTION_EXECUTIONS,
            "database::data::OrderBook" => match collection {
                Some(content) => content,
                None => {
                    return Err(error_message!("unspecified collection!"));
                }
            },
            _ => match collection {
                Some(content) => content,
                None => {
                    return Err(error_message!("unspecified collection!"));
                }
            },
        };

        let (document_count, cursor) = tokio::join!(
            source_database.count_documents(collection, query.clone(), None),
            source_database.request_raw(collection, query, Some(option))
        );

        let document_count = match document_count {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!("failed to get documet number\n{:?}", result));
            }
        };

        let cursor = match cursor {
            Ok(result) => result,
            Err(result) => {
                eprintln!("{}", result);
                return Err(error_message!("failed to get mongodb cursor\n{:?}", result));
            }
        };

        Ok(Self {
            _database: source_database,
            _cursor: cursor,
            _count: Some(document_count as usize),
            _type: PhantomData,
        })
    }

    pub async fn aggregate<'b, D: IntoIterator<Item = bson::Document>>(
        source_database: &'a Database,
        collection: Option<&str>,
        pipeline: D,
        option: Option<options::AggregateOptions>,
    ) -> Result<DatabaseStream<'a, T>, String> {
        let collection = match std::any::type_name::<T>() {
            "database::data::Ticker" => common_constants::DATABASE_COLLECTION_TICKER,
            "database::data::Execution" => common_constants::DATABASE_COLLECTION_EXECUTIONS,
            "database::data::OrderBook" => match collection {
                Some(content) => content,
                None => {
                    return Err(error_message!("unspecified collection!"));
                }
            },
            _ => match collection {
                Some(content) => content,
                None => {
                    return Err(error_message!("unspecified collection!"));
                }
            },
        };

        let cursor = source_database
            .aggregate(collection, pipeline, option)
            .await;

        let cursor = match cursor {
            Ok(result) => result,
            Err(result) => {
                eprintln!("{}", result);
                return Err(error_message!("failed to get mongodb cursor\n{:?}", result));
            }
        };

        Ok(Self {
            _database: source_database,
            _cursor: cursor,
            _count: None,
            _type: PhantomData,
        })
    }
}

impl<T> Unpin for DatabaseStream<'_, T> where T: Generater {}

impl<T> Stream for DatabaseStream<'_, T>
where
    T: Generater<Output = T>,
{
    type Item = Result<T, String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let future = self._cursor.next();
        tokio::pin!(future);
        match future.poll(cx) {
            Poll::Ready(output) => match output {
                Some(content) => match content {
                    Ok(document) => {
                        self._count = self._count.map(|content| content - 1);
                        match T::generate_from_document(document) {
                            Ok(result) => Poll::Ready(Some(Ok(result))),
                            Err(result) => Poll::Ready(Some(Err(error_message!(
                                "failed to get ticker!\n{}",
                                result
                            )))),
                        }
                    }
                    Err(result) => Poll::Ready(Some(Err(error_message!(
                        "failed to get ticker!\n{}",
                        result
                    )))),
                },
                None => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self._count.unwrap_or(0), self._count)
    }
}
