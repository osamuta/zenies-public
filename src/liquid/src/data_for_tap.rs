use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub channel: Option<String>,
    pub data: Option<T>,
    pub event: Option<String>,
}

pub type ResponseValue = Response<serde_json::Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub activity_timeout: u32,
    pub socket_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductReceiver {
    id: String,
    product_type: String,
    code: String,
    name: String,
    market_ask: String,
    market_bid: String,
    indicator: i32,
    currency: String,
    currency_pair_code: String,
    symbol: String,
    btc_minimum_withdraw: Option<u32>,
    fiat_minimum_withdraw: Option<u32>,
    pusher_channel: String,
    low_market_bid: String,
    high_market_ask: String,
    volume_24h: String,
    last_price_24h: String,
    last_traded_price: String,
    last_traded_quantity: String,
    average_price: String,
    quoted_currency: String,
    base_currency: String,
    tick_size: String,
    disabled: bool,
    margin_enabled: bool,
    cfd_enabled: bool,
    perpetual_enabled: bool,
    last_event_timestamp: String,
    timestamp: String,
    multiplier_up: f64,
    multiplier_down: f64,
    average_time_interval: u32,
    progressive_tier_eligible: bool,
}

#[derive(Clone, Debug)]
pub struct Product {
    pub id: u64,
    pub product_type: String,
    pub code: String,
    pub name: String,
    pub market_ask: i32,
    pub market_bid: i32,
    pub indicator: i32,
    pub currency: String,
    pub currency_pair_code: String,
    pub symbol: String,
    pub btc_minimum_withdraw: Option<u32>,
    pub fiat_minimum_withdraw: Option<u32>,
    pub pusher_channel: String,
    pub low_market_bid: i32,
    pub high_market_ask: i32,
    pub volume_24h: f64,
    pub last_price_24h: i32,
    pub last_traded_price: i32,
    pub last_traded_quantity: f64,
    pub average_price: f64,
    pub quoted_currency: String,
    pub base_currency: String,
    pub tick_size: f64,
    pub disabled: bool,
    pub margin_enabled: bool,
    pub cfd_enabled: bool,
    pub perpetual_enabled: bool,
    pub last_event_timestamp: f64,
    pub timestamp: f64,
    pub multiplier_up: f64,
    pub multiplier_down: f64,
    pub average_time_interval: u32,
    pub progressive_tier_eligible: bool,
}

impl super::data::DataGenerater<ProductReceiver> for Product {
    fn generate_from_receiver(product_receiver: ProductReceiver) -> Product {
        Product {
            id: product_receiver.id.parse().expect("failed to parse"),
            product_type: product_receiver.product_type,
            code: product_receiver.code,
            name: product_receiver.name,
            market_ask: product_receiver
                .market_ask
                .parse::<f64>()
                .expect("failed to parse") as i32,
            market_bid: product_receiver
                .market_bid
                .parse::<f64>()
                .expect("failed to parse") as i32,
            indicator: product_receiver.indicator,
            currency: product_receiver.currency,
            currency_pair_code: product_receiver.currency_pair_code,
            symbol: product_receiver.symbol,
            btc_minimum_withdraw: product_receiver.btc_minimum_withdraw,
            fiat_minimum_withdraw: product_receiver.fiat_minimum_withdraw,
            pusher_channel: product_receiver.pusher_channel,
            low_market_bid: product_receiver
                .low_market_bid
                .parse::<f64>()
                .expect("failed to parse") as i32,
            high_market_ask: product_receiver
                .high_market_ask
                .parse::<f64>()
                .expect("failed to parse") as i32,
            volume_24h: product_receiver
                .volume_24h
                .parse()
                .expect("failed to parse"),
            last_price_24h: product_receiver
                .last_price_24h
                .parse::<f64>()
                .expect("failed to parse") as i32,
            last_traded_price: product_receiver
                .last_traded_price
                .parse::<f64>()
                .expect("failed to parse") as i32,
            last_traded_quantity: product_receiver
                .last_traded_quantity
                .parse()
                .expect("failed to parse"),
            average_price: product_receiver
                .average_price
                .parse()
                .expect("failed to parse"),
            quoted_currency: product_receiver.quoted_currency,
            base_currency: product_receiver.base_currency,
            tick_size: product_receiver.tick_size.parse().expect("failed to parse"),
            disabled: product_receiver.disabled,
            margin_enabled: product_receiver.margin_enabled,
            cfd_enabled: product_receiver.cfd_enabled,
            perpetual_enabled: product_receiver.perpetual_enabled,
            last_event_timestamp: product_receiver
                .last_event_timestamp
                .parse()
                .expect("failed to parse"),
            timestamp: product_receiver.timestamp.parse().expect("failed to parse"),
            multiplier_up: product_receiver.multiplier_up,
            multiplier_down: product_receiver.multiplier_down,
            average_time_interval: product_receiver.average_time_interval,
            progressive_tier_eligible: product_receiver.progressive_tier_eligible,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionDetailsReceiver {
    id: u64,
    quantity: String,
    price: f64,
    taker_side: String,
    buy_order_id: u64,
    sell_order_id: u64,
    created_at: u32,
    timestamp: String,
}

#[derive(Clone, Debug)]
pub struct ExecutionDetails {
    pub id: u64,
    pub quantity: f64,
    pub price: i32,
    pub taker_side: String,
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub created_at: u32,
    pub timestamp: f64,
}

impl super::data::DataGenerater<ExecutionDetailsReceiver> for ExecutionDetails {
    fn generate_from_receiver(receiver: ExecutionDetailsReceiver) -> ExecutionDetails {
        ExecutionDetails {
            id: receiver.id,
            quantity: receiver.quantity.parse().expect("failed to parse"),
            price: receiver.price as i32,
            taker_side: receiver.taker_side,
            buy_order_id: receiver.buy_order_id,
            sell_order_id: receiver.sell_order_id,
            created_at: receiver.created_at,
            timestamp: receiver.timestamp.parse().expect("failed to parse"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderStatus {
    pub average_price: f64,
    pub client_order_id: String,
    pub created_at: u32,
    pub crypto_account_id: Option<u32>,
    pub currency_pair_code: String,
    pub disc_quantity: f64,
    pub filled_quantity: f64,
    pub funding_currency: String,
    pub iceberg_total_quantity: f64,
    pub id: u64,
    pub leverage_level: u32,
    pub margin_interest: f64,
    pub margin_type: Option<String>,
    pub margin_used: f64,
    pub order_fee: f64,
    pub order_type: String,
    pub price: f64,
    pub product_code: String,
    pub product_id: u32,
    pub quantity: f64,
    pub side: String,
    pub source_action: String,
    pub source_exchange: u32,
    pub status: String,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub target: String,
    pub trade_id: Option<u64>,
    pub trading_type: String,
    pub unwound_trade_id: Option<u64>,
    pub unwound_trade_leverage_level: Option<u64>,
    pub updated_at: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradesPanelUpdate {
    pub close_price: String,
    pub id: u64,
    pub offset: u64,
    pub offset_time: String,
    pub open_pnl: String,
    pub order_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradesUpdate {
    pub claim_quantity: f64,
    pub close_fee: f64,
    pub close_pnl: f64,
    pub close_price: f64,
    pub close_quantity: f64,
    pub created_at: u32,
    pub currency_pair_code: String,
    pub funding_currency: String,
    pub id: u64,
    pub last_settlement_at: Option<String>,
    pub leverage_level: u32,
    pub liquidation_price: Option<String>,
    pub maintenance_margin: Option<String>,
    pub margin_type: String,
    pub margin_used: f64,
    pub open_pnl: f64,
    pub open_price: f64,
    pub open_quantity: f64,
    pub order_id: u64,
    pub original_open_price: Option<String>,
    pub pnl: f64,
    pub product_code: String,
    pub product_id: u32,
    pub quantity: f64,
    pub side: String,
    pub stop_loss: Option<String>,
    pub take_profit: Option<String>,
    pub total_fee: f64,
    pub total_interest: f64,
    pub trader_id: u64,
    pub trading_type: String,
    pub updated_at: u32,
}
