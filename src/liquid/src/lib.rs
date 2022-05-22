//! # liquid
//! `liquid` is api to getting massive data derived from the liquid market easily.
//!

use futures::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_tungstenite::*;

use common::error_message;

pub use currency::*;

mod constants;
pub mod currency;
pub mod data;
pub mod data_for_tap;
pub mod liquid_tap;
mod private;
mod public;
mod test;
mod url_gen;

use constants::*;

///
/// https asynchronus liquid client
#[derive(Clone, Debug)]
pub struct LiquidClientAsync {
    _client: reqwest::Client,
    //_last_called: std::time::Instant,
}

///
/// https blocking liquid client
#[derive(Clone, Debug)]
pub struct LiquidClientBlocking {
    _client: reqwest::blocking::Client,
    // _last_called: std::time::Instant,
}

///
/// liquid api key
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LiquidApiKey {
    pub token_id: u32,
    pub secret_key: String,
}

///
/// order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_type: String,
    pub product_id: i32,
    pub side: String,
    pub quantity: f64,
    pub price: Option<i32>,
    pub price_range: Option<i32>,
    pub trailing_stop_type: Option<String>,
    pub trailing_stop_value: Option<f64>,
    pub client_order_id: Option<String>,
    pub leverage_level: Option<i32>,
    pub order_direction: Option<String>,
    pub take_profit: Option<i32>,
    pub stop_loss: Option<i32>,
}

pub enum TrailType {
    Fiat,
    Percentage,
}

pub enum OrderDirection {
    OneDirection,
    TwoDirection,
    NetOut,
}

///
/// liquid tap client
#[derive(Debug)]
pub struct LiquidTapClientBlocking {
    _socket: tungstenite::WebSocket<
        tokio_tungstenite::tungstenite::stream::MaybeTlsStream<std::net::TcpStream>,
    >,
    _activity_timeout: u32,
    _socket_id: f64,
}

///
/// async liquid tap client
//#[derive(Debug)]
pub struct LiquidTapClientAsync {
    _socket: tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    _activity_timeout: u32,
    _socket_id: f64,
}

const U24_MAX: usize = 0xFF_FFFF;

static COUNTER: Lazy<AtomicUsize> =
    Lazy::new(|| AtomicUsize::new(thread_rng().gen_range(0..=U24_MAX)));

fn authorizer(
    path: String,
    token_id: u32,
    secret_key: &str,
) -> std::result::Result<String, String> {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Payload {
        path: String,
        nonce: u128,
        token_id: u32,
    }
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        << 24
        | (counter % U24_MAX + 1) as u128;
    let payload = Payload {
        path,
        nonce,
        token_id,
    };

    match encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ) {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to encode a payload by HMAC-SHA256!\ndetails : {:?}",
            result
        )),
    }
}

fn authorizer_without_nonce(
    path: String,
    token_id: u32,
    secret_key: &str,
) -> std::result::Result<String, String> {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Payload {
        path: String,
        token_id: u32,
    }
    let payload = Payload { path, token_id };

    match encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ) {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to encode a payload by HMAC-SHA256!\ndetails : {:?}",
            result
        )),
    }
}
