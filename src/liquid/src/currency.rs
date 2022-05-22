use common::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum CurrencyPair {
    BtcUsd,
    BtcEur,
    BtcJpy,
    BtcSgd,
    EthJpy,
    Custom(i32),
}

impl CurrencyPair {
    pub fn generate_id(&self) -> i32 {
        match *self {
            CurrencyPair::BtcUsd => 1,
            CurrencyPair::BtcEur => 3,
            CurrencyPair::BtcJpy => 5,
            CurrencyPair::BtcSgd => 7,
            CurrencyPair::EthJpy => 29,
            CurrencyPair::Custom(pair) => pair,
        }
    }

    pub fn generate_pair_code(&self) -> &str {
        match *self {
            CurrencyPair::BtcUsd => "btcusd",
            CurrencyPair::BtcEur => "btceur",
            CurrencyPair::BtcJpy => "btcjpy",
            CurrencyPair::BtcSgd => "btcsgd",
            CurrencyPair::EthJpy => "ethjpy",
            CurrencyPair::Custom(_) => panic!(),
        }
    }

    pub fn generate_from_string(side: &str) -> Result<Self, String> {
        match side {
            "btcusd" => Ok(CurrencyPair::BtcUsd),
            "btceur" => Ok(CurrencyPair::BtcEur),
            "btcjpy" => Ok(CurrencyPair::BtcJpy),
            "btcsgd" => Ok(CurrencyPair::BtcSgd),
            "ethjpy" => Ok(CurrencyPair::EthJpy),
            _ => Err(error_message!("unknown pair\ndetails : {}", side)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Side {
    Buy,
    Sell,
}

impl Default for Side {
    fn default() -> Self {
        Side::Buy
    }
}

impl Side {
    pub fn generate_side_string(&self) -> &str {
        match *self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }

    pub fn generate_from_string(side: &str) -> Result<Self, String> {
        match side {
            "buy" => Ok(Side::Buy),
            "sell" => Ok(Side::Sell),
            "long" => Ok(Side::Buy),
            "short" => Ok(Side::Sell),
            _ => Err(error_message!("unknown side\ndetails : {}", side)),
        }
    }
}
