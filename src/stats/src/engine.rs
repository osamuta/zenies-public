use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use chrono::prelude::*;
use chrono::{DateTime, Duration};
use futures::stream::StreamExt;
use rayon::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use common::*;
use database::*;

#[derive(Debug)]
pub struct DataEngine<Z: TimeZone> {
    url: String,
    time_range: Range<DateTime<Z>>,
    _limited_size: usize,
    ticker_channel: Sender<data::Ticker>,
    executions_channel: Sender<data::Execution>,
    order_book_buy_channel: Sender<data::OrderBook>,
    order_book_sell_channel: Sender<data::OrderBook>,
}

#[derive(Debug)]
pub struct DataEngineChild<Z: TimeZone> {
    _time_range: Range<DateTime<Z>>,
    ticker_channel: Receiver<data::Ticker>,
    executions_channel: Receiver<data::Execution>,
    order_book_buy_channel: Receiver<data::OrderBook>,
    order_book_sell_channel: Receiver<data::OrderBook>,
    tickers: VecDeque<data::Ticker>,
    executions: VecDeque<data::Execution>,
    order_book_buy: VecDeque<data::OrderBook>,
    order_book_sell: VecDeque<data::OrderBook>,
}

impl<Z: TimeZone + std::fmt::Debug> DataEngineChild<Z> {
    pub async fn store<F: Fn()>(&mut self, f: F) -> Result<(), String> {
        loop {
            tokio::select! {
                Some(ticker) = self.ticker_channel.recv() => {
                    self.tickers.push_back(ticker);
                }
                Some(execution) = self.executions_channel.recv() => {
                    self.executions.push_back(execution);
                }
                Some(order_book_buy) = self.order_book_buy_channel.recv() => {
                    self.order_book_buy.push_back(order_book_buy);
                }
                Some(order_book_sell) = self.order_book_sell_channel.recv() => {
                    self.order_book_sell.push_back(order_book_sell);
                }
                else => break,
            }
            f();
        }
        Ok(())
    }

    pub fn get_limit_rayon(
        &self,
        date_time: &DateTime<Z>,
        limit: usize,
    ) -> (
        Option<Vec<data::Ticker>>,
        Option<Vec<data::Execution>>,
        Option<Vec<data::OrderBook>>,
        Option<Vec<data::OrderBook>>,
    )
    where
        <Z as chrono::TimeZone>::Offset: Sync,
    {
        let tickers =
            match self
                .tickers
                .par_iter()
                .enumerate()
                .skip(1)
                .position_any(|(index, current)| {
                    (self.tickers[index - 1].received_at..current.received_at)
                        .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
                        && current.received_at - self.tickers[index - 1].received_at
                            < Duration::minutes(5).num_seconds() as f64
                }) {
                Some(content) => Some(
                    self.tickers
                        .range(content..content + limit)
                        .map(|t| t.clone())
                        .collect(),
                ),
                None => None,
            };

        let executions =
            match self
                .executions
                .par_iter()
                .enumerate()
                .skip(1)
                .position_any(|(index, current)| {
                    (self.executions[index - 1].received_at..current.received_at)
                        .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
                        && current.received_at - self.executions[index - 1].received_at
                            < Duration::minutes(5).num_seconds() as f64
                }) {
                Some(content) => Some(
                    self.executions
                        .range(content..content + limit)
                        .map(|t| t.clone())
                        .collect(),
                ),
                None => None,
            };

        let order_book_buy = match self
            .order_book_buy
            .par_iter()
            .enumerate()
            .skip(1)
            .position_any(|(index, current)| {
                (self.order_book_buy[index - 1].received_at..current.received_at)
                    .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
                    && current.received_at - self.order_book_buy[index - 1].received_at
                        < Duration::minutes(5).num_seconds() as f64
            }) {
            Some(content) => Some(
                self.order_book_buy
                    .range(content..content + limit)
                    .map(|t| t.clone())
                    .collect(),
            ),
            None => None,
        };

        let order_book_sell = match self
            .order_book_sell
            .par_iter()
            .enumerate()
            .skip(1)
            .position_any(|(index, current)| {
                (self.order_book_sell[index - 1].received_at..current.received_at)
                    .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
                    && current.received_at - self.order_book_sell[index - 1].received_at
                        < Duration::minutes(5).num_seconds() as f64
            }) {
            Some(content) => Some(
                self.order_book_sell
                    .range(content..content + limit)
                    .map(|t| t.clone())
                    .collect(),
            ),
            None => None,
        };
        (tickers, executions, order_book_buy, order_book_sell)
    }

    pub fn get_limit(
        &self,
        date_time: &DateTime<Z>,
        limit: usize,
    ) -> (
        Option<Vec<data::Ticker>>,
        Option<Vec<data::Execution>>,
        Option<Vec<data::OrderBook>>,
        Option<Vec<data::OrderBook>>,
    )
    where
        <Z as chrono::TimeZone>::Offset: Sync,
    {
        let mut tickers = None;
        let mut executions = None;
        let mut order_book_buy = None;
        let mut order_book_sell = None;
        let mut previous = (0, &self.tickers[0]);
        for current in self.tickers.iter().enumerate().skip(1) {
            if (previous.1.received_at..current.1.received_at)
                .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
            {
                if current.1.received_at - previous.1.received_at
                    < Duration::minutes(5).num_seconds() as f64
                {
                    tickers = Some(
                        self.tickers
                            .range(previous.0..previous.0 + limit)
                            .map(|t| t.clone())
                            .collect(),
                    );
                }
                break;
            }
            previous = current;
        }

        let mut previous = (0, &self.executions[0]);
        for current in self.executions.iter().enumerate().skip(1) {
            if (previous.1.received_at..current.1.received_at)
                .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
            {
                if current.1.received_at - previous.1.received_at
                    < Duration::minutes(5).num_seconds() as f64
                {
                    executions = Some(
                        self.executions
                            .range(previous.0..previous.0 + limit)
                            .map(|t| t.clone())
                            .collect(),
                    );
                }
                break;
            }
            previous = current;
        }

        let mut previous = (0, &self.order_book_buy[0]);
        for current in self.order_book_buy.iter().enumerate().skip(1) {
            if (previous.1.received_at..current.1.received_at)
                .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
            {
                if current.1.received_at - previous.1.received_at
                    < Duration::minutes(5).num_seconds() as f64
                {
                    order_book_buy = Some(
                        self.order_book_buy
                            .range(previous.0..previous.0 + limit)
                            .map(|t| t.clone())
                            .collect(),
                    );
                }
                break;
            }
            previous = current;
        }

        let mut previous = (0, &self.order_book_sell[0]);
        for current in self.order_book_sell.iter().enumerate().skip(1) {
            if (previous.1.received_at..current.1.received_at)
                .contains(&(date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64))
            {
                if current.1.received_at - previous.1.received_at
                    < Duration::minutes(5).num_seconds() as f64
                {
                    order_book_sell = Some(
                        self.order_book_sell
                            .range(previous.0..previous.0 + limit)
                            .map(|t| t.clone())
                            .collect(),
                    );
                }
                break;
            }
            previous = current;
        }

        (tickers, executions, order_book_buy, order_book_sell)
    }

    pub fn clean(&mut self, date_time: DateTime<Z>) {
        self.tickers
            .retain(|t| t.received_at > date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64);
        self.executions
            .retain(|t| t.received_at > date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64);
        self.order_book_buy
            .retain(|t| t.received_at > date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64);
        self.order_book_sell
            .retain(|t| t.received_at > date_time.timestamp_nanos() as f64 / 1_000_000_000.0f64);
    }
}

impl<Z: TimeZone + 'static> DataEngine<Z> {
    pub fn new(
        url: &str,
        time_range: Range<DateTime<Z>>,
        limited_size: usize,
    ) -> (Self, DataEngineChild<Z>) {
        let ticker_channel = channel(limited_size);
        let executions_channel = channel(limited_size);
        let order_book_buy_channel = channel(limited_size);
        let order_book_sell_channel = channel(limited_size);
        let engine = Self {
            url: String::from(url),
            time_range: time_range,
            _limited_size: limited_size,
            ticker_channel: ticker_channel.0,
            executions_channel: executions_channel.0,
            order_book_buy_channel: order_book_buy_channel.0,
            order_book_sell_channel: order_book_sell_channel.0,
        };

        let child = DataEngineChild {
            _time_range: engine.time_range.clone(),
            ticker_channel: ticker_channel.1,
            executions_channel: executions_channel.1,
            order_book_buy_channel: order_book_buy_channel.1,
            order_book_sell_channel: order_book_sell_channel.1,
            tickers: VecDeque::new(),
            executions: VecDeque::new(),
            order_book_buy: VecDeque::new(),
            order_book_sell: VecDeque::new(),
        };

        (engine, child)
    }

    pub async fn driven(self) -> Result<(), String>
    where
        <Z as chrono::TimeZone>::Offset: Sync,
        <Z as chrono::TimeZone>::Offset: std::marker::Send,
    {
        let self_pointer = Arc::new(self);
        let self_pointer_for_ticker = self_pointer.clone();
        let self_pointer_for_executions = self_pointer.clone();
        let self_pointer_for_order_book_buy = self_pointer.clone();
        let self_pointer_for_order_book_sell = self_pointer.clone();
        let ticker_task = tokio::spawn(async move {
            let database = match Database::new(
                &self_pointer_for_ticker.url,
                common_constants::DATABASE_NAME,
            )
            .await
            {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to connect query!\ndetails : {}",
                        result
                    ));
                }
            };
            let collection = common_constants::DATABASE_COLLECTION_TICKER;
            let query = match create_duration_query(
                collection,
                &self_pointer_for_ticker.time_range.start,
                &self_pointer_for_ticker.time_range.end,
            ) {
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

            loop {
                tokio::select! {
                    Some(result) = ticker_stream.next() => {
                        match result {
                            Ok(result) => {
                                if let Err(result) = self_pointer_for_ticker.ticker_channel.send(result).await {
                                    return Err(error_message!(
                                        "failed to send!\ndetails : {}",
                                        result
                                    ));
                                }
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

            Ok(())
        });

        let executions_task = tokio::spawn(async move {
            let database = match Database::new(
                &self_pointer_for_executions.url,
                common_constants::DATABASE_NAME,
            )
            .await
            {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to connect query!\ndetails : {}",
                        result
                    ));
                }
            };
            let collection = common_constants::DATABASE_COLLECTION_EXECUTIONS;
            let query = match create_duration_query(
                collection,
                &self_pointer_for_executions.time_range.start,
                &self_pointer_for_executions.time_range.end,
            ) {
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

            let mut execution_stream = match stream::DatabaseStream::<data::Execution>::request(
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

            loop {
                tokio::select! {
                    Some(result) = execution_stream.next() => {
                        match result {
                            Ok(result) => {
                                if let Err(result) = self_pointer_for_executions.executions_channel.send(result).await {
                                    return Err(error_message!(
                                        "failed to send!\ndetails : {}",
                                        result
                                    ));
                                }
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

            Ok(())
        });

        let order_book_buy_task = tokio::spawn(async move {
            let database = match Database::new(
                &self_pointer_for_order_book_buy.url,
                common_constants::DATABASE_NAME,
            )
            .await
            {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to connect query!\ndetails : {}",
                        result
                    ));
                }
            };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY;
            let query = match create_duration_query(
                collection,
                &self_pointer_for_order_book_buy.time_range.start,
                &self_pointer_for_order_book_buy.time_range.end,
            ) {
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
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => {
                                if let Err(result) = self_pointer_for_order_book_buy.order_book_buy_channel.send(result).await {
                                    return Err(error_message!(
                                        "failed to send!\ndetails : {}",
                                        result
                                    ));
                                }
                            }
                            Err(result) => {
                                return Err(error_message!(
                                    "failed to get!\ndetails : {}",
                                    result
                                ));
                            }
                        }
                    }
                    else => break Ok(()),
                }
            }
        });

        let order_book_sell_task = tokio::spawn(async move {
            let database = match Database::new(
                &self_pointer_for_order_book_sell.url,
                common_constants::DATABASE_NAME,
            )
            .await
            {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to connect query!\ndetails : {}",
                        result
                    ));
                }
            };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL;
            let query = match create_duration_query(
                collection,
                &self_pointer_for_order_book_sell.time_range.start,
                &self_pointer_for_order_book_sell.time_range.end,
            ) {
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
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => {
                                if let Err(result) = self_pointer_for_order_book_sell.order_book_sell_channel.send(result).await {
                                    return Err(error_message!(
                                        "failed to send!\ndetails : {}",
                                        result
                                    ));
                                }
                            }
                            Err(result) => {
                                return Err(error_message!(
                                    "failed to get!\ndetails : {}",
                                    result
                                ));
                            }
                        }
                    }
                    else => break Ok(()),
                }
            }
        });

        let _ = tokio::join!(
            ticker_task,
            executions_task,
            order_book_buy_task,
            order_book_sell_task
        );
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn check_engine() {
    use chrono::Duration;

    let start = Utc.ymd(2021, 9, 13).and_hms(15, 58, 0);
    let range = start..start + Duration::minutes(20);
    let (engine, mut child) = DataEngine::new("mongodb://viewer:isee@192.168.2.104:49457/?authSource=BtcJpy_In_Liquid&authMechanism=SCRAM-SHA-256&readPreference=primary&appname=MongoDB%20Compass&ssl=false", range, 1_000_000_000_000_000);
    let engine_task = tokio::spawn(async move {
        let _ = engine.driven().await;
        println!("driven done.");
    });
    let _ = tokio::join!(engine_task, async {
        let _ = child.store(|| {}).await;
        println!("storing done");
    });

    let result = child.get_limit(&(start + Duration::minutes(1)), 1);
    println!("None {:?}", result);
    let result = child.get_limit(&(start + Duration::minutes(10)), 1);
    println!("Some {:?}", result);
}

#[derive(Debug, Clone)]
pub struct EasyEngine {
    url: String,
}

impl EasyEngine {
    pub async fn new(url: &str) -> Result<Self, String> {
        let _ = match Database::new(url, common_constants::DATABASE_NAME).await {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to connect to a specified database!\ndetails : {}",
                    result
                ));
            }
        };

        Ok(Self {
            url: String::from(url),
        })
    }

    pub async fn get_limit(
        &self,
        time: &DateTime<Utc>,
        limit: i64,
    ) -> Result<
        (
            Vec<data::Ticker>,
            Vec<data::Execution>,
            Vec<data::OrderBook>,
            Vec<data::OrderBook>,
        ),
        String,
    > {
        let self_pointer = Arc::new(self.clone());
        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = limit;
        let ticker_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_TICKER;
            let query = match create_duration_query(
                collection,
                &time_for_other,
                &(time_for_other + Duration::minutes(5)),
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let options = match create_find_option_timeline(collection, 1, Some(limit_for_other)) {
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = ticker_stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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

            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = limit;
        let executions_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_EXECUTIONS;
            let query = match create_duration_query(
                collection,
                &time_for_other,
                &(time_for_other + Duration::minutes(5)),
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let options = match create_find_option_timeline(collection, 1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let mut execution_stream = match stream::DatabaseStream::<data::Execution>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = execution_stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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

            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = limit;
        let order_book_buy_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY;
            let query = match create_duration_query(
                collection,
                &time_for_other,
                &(time_for_other + Duration::minutes(5)),
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let options = match create_find_option_timeline(collection, 1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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
            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = limit;
        let order_book_sell_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL;
            let query = match create_duration_query(
                collection,
                &time_for_other,
                &(time_for_other + Duration::minutes(5)),
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let options = match create_find_option_timeline(collection, 1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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
            Ok(buf)
        });

        let (ticker_task, executions_task, order_book_buy_task, order_book_sell_task) = tokio::join!(
            ticker_task,
            executions_task,
            order_book_buy_task,
            order_book_sell_task
        );
        let ticker = match ticker_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let executions = match executions_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let order_book_buy = match order_book_buy_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let order_book_sell = match order_book_sell_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        Ok((ticker, executions, order_book_buy, order_book_sell))
    }

    pub async fn get_one_precicely(
        &self,
        time: &DateTime<Utc>,
    ) -> Result<
        (
            Vec<data::Ticker>,
            Vec<data::Execution>,
            Vec<data::OrderBook>,
            Vec<data::OrderBook>,
        ),
        String,
    > {
        let self_pointer = Arc::new(self.clone());
        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = 1;
        let ticker_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_TICKER;
            let query = match create_duration_query(
                collection,
                &(time_for_other - Duration::minutes(5)),
                &time_for_other,
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let options = match create_find_option_timeline(collection, -1, Some(limit_for_other)) {
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = ticker_stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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

            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = 1;
        let executions_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_EXECUTIONS;
            let query = match create_duration_query(
                collection,
                &(time_for_other - Duration::minutes(5)),
                &time_for_other,
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let options = match create_find_option_timeline(collection, -1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };

            let mut execution_stream = match stream::DatabaseStream::<data::Execution>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = execution_stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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

            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = 1;
        let order_book_buy_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY;
            let query = match create_duration_query(
                collection,
                &(time_for_other - Duration::minutes(5)),
                &time_for_other,
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let options = match create_find_option_timeline(collection, -1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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
            Ok(buf)
        });

        let self_pointer_for_other = self_pointer.clone();
        let time_for_other = time.clone();
        let limit_for_other = 1;
        let order_book_sell_task = tokio::spawn(async move {
            let database =
                match Database::new(&self_pointer_for_other.url, common_constants::DATABASE_NAME)
                    .await
                {
                    Ok(result) => result,
                    Err(result) => {
                        return Err(error_message!(
                            "failed to connect a database!\ndetails : {}",
                            result
                        ));
                    }
                };
            let collection = common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL;
            let query = match create_duration_query(
                collection,
                &(time_for_other - Duration::minutes(5)),
                &time_for_other,
            ) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let options = match create_find_option_timeline(collection, -1, Some(limit_for_other)) {
                Ok(content) => content,
                Err(result) => {
                    return Err(error_message!(
                        "failed to create query!\ndetails : {}",
                        result
                    ));
                }
            };
            let mut stream = match stream::DatabaseStream::<data::OrderBook>::request(
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

            let mut buf = Vec::new();
            loop {
                tokio::select! {
                    Some(result) = stream.next() => {
                        match result {
                            Ok(result) => buf.push(result),
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
            Ok(buf)
        });

        let (ticker_task, executions_task, order_book_buy_task, order_book_sell_task) = tokio::join!(
            ticker_task,
            executions_task,
            order_book_buy_task,
            order_book_sell_task
        );
        let ticker = match ticker_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let executions = match executions_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let order_book_buy = match order_book_buy_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        let order_book_sell = match order_book_sell_task.expect("failed to run tasks!") {
            Ok(result) => result,
            Err(result) => return Err(result),
        };
        Ok((ticker, executions, order_book_buy, order_book_sell))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn check_easy_engine() {
    use chrono::Duration;

    let start = Utc.ymd(2021, 9, 13).and_hms(15, 58, 0);
    let engine = match EasyEngine::new("mongodb://viewer:isee@192.168.2.104:49457/?authSource=BtcJpy_In_Liquid&authMechanism=SCRAM-SHA-256&readPreference=primary&appname=MongoDB%20Compass&ssl=false").await {
        Ok(result) => result,
        Err(result) => {
            panic!();
        }
    };

    let result = engine.get_limit(&(start + Duration::minutes(1)), 1).await;
    println!("None {:?}", result);
    let result = engine.get_limit(&(start + Duration::minutes(10)), 1).await;
    println!("Some {:?}", result);
}
