use chrono::prelude::*;
use chrono::DateTime;
use chrono::Duration;
use mongodb::bson;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
#[macro_use]
extern crate ndarray;
use futures::stream::{Stream, StreamExt};

use ndarray::prelude::*;
use ndarray_linalg::*;

use rustfft::{num_complex::Complex, FftPlanner};

use std::ops::Range;

use common::*;
use database::*;

pub mod engine;
pub mod getter;
pub mod math;
pub mod plotter;

#[derive(Clone, Debug)]
pub struct DateTimeIter<Z: TimeZone> {
    range: Range<DateTime<Z>>,
    period: Duration,
}

impl<Z: TimeZone> DateTimeIter<Z> {
    pub fn new(range: Range<DateTime<Z>>, period: Duration) -> Self {
        Self { range, period }
    }
}

impl<Z: TimeZone> Iterator for DateTimeIter<Z> {
    type Item = DateTime<Z>;
    fn next(&mut self) -> Option<Self::Item> {
        self.range.start = self.range.start.clone() + self.period;
        if self.range.start < self.range.end {
            Some(self.range.start.clone())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketInfo {
    pub tickers: Vec<data::Ticker>,
    pub executions: Vec<data::Execution>,
    pub order_book_buy: Vec<data::OrderBook>,
    pub order_book_sell: Vec<data::OrderBook>,
}

impl MarketInfo {
    pub fn new() -> Self {
        Self {
            tickers: Vec::new(),
            executions: Vec::new(),
            order_book_buy: Vec::new(),
            order_book_sell: Vec::new(),
        }
    }

    pub fn vacuume(&mut self, period: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        self.tickers.retain(|e| e.received_at > now - period);
        self.executions.retain(|e| e.received_at > now - period);
        self.order_book_buy.retain(|e| e.received_at > now - period);
        self.order_book_sell
            .retain(|e| e.received_at > now - period);
    }
}

#[test]
fn check_iter() {
    for (i, t) in DateTimeIter::new(
        Utc.ymd(2021, 9, 13).and_hms(15, 58, 0)..Utc.ymd(2021, 9, 13).and_hms(16, 58, 0),
        Duration::minutes(1),
    )
    .enumerate()
    {
        println!("{:?}, {:?}", i, t);
    }
}
