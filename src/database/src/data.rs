use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ticker {
    //#[serde(serialize_with = "mongodb::bson::serde_helpers::serialize_object_id_as_hex_string")]
    //pub _id: mongodb::bson::oid::ObjectId,
    pub received_at: f64,
    pub timestamp: f64,
    pub last_traded_price: i32,
    pub last_traded_quantity: f64,
    pub last_price_24h: i32,
    pub average_price_24h: f64,
    pub volume_24h: f64,
    pub market_ask: i32,
    pub market_bid: i32,
    pub low_market_price_24h: i32,
    pub high_market_price_24h: i32,
}

/*impl Serialize for Ticker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Ticker", 12)?;
        state.serialize_field("_id", &self._id.to_hex())?;
        state.serialize_field("received_at", &self.received_at)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("last_traded_price", &self.last_traded_price)?;
        state.serialize_field("last_traded_quantity", &self.last_traded_quantity)?;
        state.serialize_field("last_price_24h", &self.last_price_24h)?;
        state.serialize_field("average_price_24h", &self.average_price_24h)?;
        state.serialize_field("volume_24h", &self.volume_24h)?;
        state.serialize_field("market_ask", &self.market_ask)?;
        state.serialize_field("market_bid", &self.market_bid)?;
        state.serialize_field("low_market_price_24h", &self.low_market_price_24h)?;
        state.serialize_field("high_market_price_24h", &self.high_market_price_24h)?;
        state.end()
    }
}*/

impl Generater for Ticker {
    type Output = Self;
    fn generate_from_document(doc: bson::Document) -> Result<Self, String>
    where
        Self: Sized,
    {
        match bson::from_document(doc) as bson::de::Result<data::Ticker> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize!\ndetails : {:?}",
                result
            )),
        }
    }
}

pub trait Finder {
    fn find_price(&self, unix_time: f64) -> Option<data::Ticker>;
    fn find_price_by_rayon(&self, unix_time: f64) -> Option<data::Ticker>;
}

impl Finder for Vec<Ticker> {
    fn find_price(&self, unix_time: f64) -> Option<data::Ticker> {
        if unix_time < self[0].timestamp || unix_time > self[self.len() - 1].timestamp {
            return None;
        }
        for t in self {
            if t.timestamp > unix_time {
                return Some(t.clone());
            }
        }
        None
    }
    fn find_price_by_rayon(&self, unix_time: f64) -> Option<data::Ticker> {
        if unix_time < self[0].timestamp || unix_time > self[self.len() - 1].timestamp {
            return None;
        }
        self.par_iter()
            .find_first(|&t| t.timestamp > unix_time)
            .cloned()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Execution {
    //pub _id: mongodb::bson::oid::ObjectId,
    pub received_at: f64,
    pub timestamp: f64,
    pub created_at: i32,
    pub price: i32,
    pub quantity: f64,
    pub taker_side: String,
}

/*impl Serialize for Execution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Ticker", 7)?;
        state.serialize_field("_id", &self._id.to_hex())?;
        state.serialize_field("received_at", &self.received_at)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("price", &self.price)?;
        state.serialize_field("quantity", &self.quantity)?;
        state.serialize_field("taker_side", &self.taker_side)?;
        state.end()
    }
}*/

impl Generater for Execution {
    type Output = Self;
    fn generate_from_document(doc: bson::Document) -> Result<Self, String> {
        match bson::from_document(doc) as bson::de::Result<data::Execution> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize!\ndetails : {:?}",
                result
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderBook {
    //pub _id: mongodb::bson::oid::ObjectId,
    pub received_at: f64,
    pub orders: Vec<liquid::data::Order>,
}

impl Generater for OrderBook {
    type Output = Self;
    fn generate_from_document(doc: bson::Document) -> Result<Self, String> {
        match bson::from_document(doc) as bson::de::Result<data::OrderBook> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize!\ndetails : {:?}",
                result
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationExecution {
    pub timestamp: f64,
    pub price: i32,
    pub quantity: f64,
    pub order: String,
    pub result: Result<(), String>,
}
