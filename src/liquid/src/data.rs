use serde::{Deserialize, Serialize};

pub trait DataGenerater<R> {
    fn generate_from_receiver(receiver: R) -> Self;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductReceiver {
    id: String,
    product_type: String,
    code: String,
    name: String,
    market_ask: f64,
    market_bid: f64,
    indicator: i32,
    currency: String,
    currency_pair_code: String,
    symbol: String,
    btc_minimum_withdraw: Option<u32>,
    fiat_minimum_withdraw: Option<u32>,
    pusher_channel: String,
    taker_fee: String,
    maker_fee: String,
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
    multiplier_up: String,
    multiplier_down: String,
    average_time_interval: u32,
    progressive_tier_eligible: bool,
    exchange_rate: u32,
}

#[derive(Clone, Debug)]
pub struct Product {
    pub id: u64,
    pub product_type: String,
    pub code: String,
    pub name: String,
    pub market_ask: f64,
    pub market_bid: f64,
    pub indicator: i32,
    pub currency: String,
    pub currency_pair_code: String,
    pub symbol: String,
    pub btc_minimum_withdraw: Option<u32>,
    pub fiat_minimum_withdraw: Option<u32>,
    pub pusher_channel: String,
    pub taker_fee: f64,
    pub maker_fee: f64,
    pub low_market_bid: f64,
    pub high_market_ask: f64,
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
    pub exchange_rate: u32,
}

impl DataGenerater<ProductReceiver> for Product {
    fn generate_from_receiver(product_receiver: ProductReceiver) -> Product {
        Product {
            id: product_receiver.id.parse().expect("failed to parse"),
            product_type: product_receiver.product_type,
            code: product_receiver.code,
            name: product_receiver.name,
            market_ask: product_receiver.market_ask,
            market_bid: product_receiver.market_bid,
            indicator: product_receiver.indicator,
            currency: product_receiver.currency,
            currency_pair_code: product_receiver.currency_pair_code,
            symbol: product_receiver.symbol,
            btc_minimum_withdraw: product_receiver.btc_minimum_withdraw,
            fiat_minimum_withdraw: product_receiver.fiat_minimum_withdraw,
            pusher_channel: product_receiver.pusher_channel,
            taker_fee: product_receiver.taker_fee.parse().expect("failed to parse"),
            maker_fee: product_receiver.maker_fee.parse().expect("failed to parse"),
            low_market_bid: product_receiver
                .low_market_bid
                .parse()
                .expect("failed to parse"),
            high_market_ask: product_receiver
                .high_market_ask
                .parse()
                .expect("failed to parse"),
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
            multiplier_up: product_receiver
                .multiplier_up
                .parse()
                .expect("failed to parse"),
            multiplier_down: product_receiver
                .multiplier_down
                .parse()
                .expect("failed to parse"),
            average_time_interval: product_receiver.average_time_interval,
            progressive_tier_eligible: product_receiver.progressive_tier_eligible,
            exchange_rate: product_receiver.exchange_rate,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerpetualProductReceiver {
    id: String,
    product_type: String,
    code: String,
    name: Option<u32>,
    market_ask: f64,
    market_bid: f64,
    indicator: Option<u32>,
    currency: String,
    currency_pair_code: String,
    symbol: String,
    btc_minimum_withdraw: Option<u32>,
    fiat_minimum_withdraw: Option<u32>,
    pusher_channel: String,
    taker_fee: String,
    maker_fee: String,
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
    multiplier_up: Option<String>,
    multiplier_down: String,
    average_time_interval: u32,
    progressive_tier_eligible: bool,
    index_price: String,
    mark_price: String,
    funding_rate: String,
    fair_price: String,
    average_funding_rate_8h: String,
}

#[derive(Clone, Debug)]
pub struct PerpetualProduct {
    pub id: u64,
    pub product_type: String,
    pub code: String,
    pub name: Option<u32>,
    pub market_ask: f64,
    pub market_bid: f64,
    pub indicator: Option<u32>,
    pub currency: String,
    pub currency_pair_code: String,
    pub symbol: String,
    pub btc_minimum_withdraw: Option<u32>,
    pub fiat_minimum_withdraw: Option<u32>,
    pub pusher_channel: String,
    pub taker_fee: f64,
    pub maker_fee: f64,
    pub low_market_bid: f64,
    pub high_market_ask: f64,
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
    pub multiplier_up: Option<f64>,
    pub multiplier_down: f64,
    pub average_time_interval: u32,
    pub progressive_tier_eligible: bool,
    pub index_price: f64,
    pub mark_price: f64,
    pub funding_rate: f64,
    pub fair_price: f64,
    pub average_funding_rate_8h: f64,
}

impl DataGenerater<PerpetualProductReceiver> for PerpetualProduct {
    fn generate_from_receiver(perpetual_receiver: PerpetualProductReceiver) -> PerpetualProduct {
        PerpetualProduct {
            id: perpetual_receiver.id.parse().expect("failed to parse"),
            product_type: perpetual_receiver.product_type,
            code: perpetual_receiver.code,
            name: perpetual_receiver.name,
            market_ask: perpetual_receiver.market_ask,
            market_bid: perpetual_receiver.market_bid,
            indicator: perpetual_receiver.indicator,
            currency: perpetual_receiver.currency,
            currency_pair_code: perpetual_receiver.currency_pair_code,
            symbol: perpetual_receiver.symbol,
            btc_minimum_withdraw: perpetual_receiver.btc_minimum_withdraw,
            fiat_minimum_withdraw: perpetual_receiver.fiat_minimum_withdraw,
            pusher_channel: perpetual_receiver.pusher_channel,
            taker_fee: perpetual_receiver
                .taker_fee
                .parse()
                .expect("failed to parse"),
            maker_fee: perpetual_receiver
                .maker_fee
                .parse()
                .expect("failed to parse"),
            low_market_bid: perpetual_receiver
                .low_market_bid
                .parse()
                .expect("failed to parse"),
            high_market_ask: perpetual_receiver
                .high_market_ask
                .parse()
                .expect("failed to parse"),
            volume_24h: perpetual_receiver
                .volume_24h
                .parse()
                .expect("failed to parse"),
            last_price_24h: perpetual_receiver
                .last_price_24h
                .parse::<f64>()
                .expect("failed to parse") as i32,
            last_traded_price: perpetual_receiver
                .last_traded_price
                .parse::<f64>()
                .expect("failed to parse") as i32,
            last_traded_quantity: perpetual_receiver
                .last_traded_quantity
                .parse()
                .expect("failed to parse"),
            average_price: perpetual_receiver
                .average_price
                .parse()
                .expect("failed to parse"),
            quoted_currency: perpetual_receiver.quoted_currency,
            base_currency: perpetual_receiver.base_currency,
            tick_size: perpetual_receiver
                .tick_size
                .parse()
                .expect("failed to parse"),
            disabled: perpetual_receiver.disabled,
            margin_enabled: perpetual_receiver.margin_enabled,
            cfd_enabled: perpetual_receiver.cfd_enabled,
            perpetual_enabled: perpetual_receiver.perpetual_enabled,
            last_event_timestamp: perpetual_receiver
                .last_event_timestamp
                .parse()
                .expect("failed to parse"),
            timestamp: perpetual_receiver
                .timestamp
                .parse()
                .expect("failed to parse"),
            multiplier_up: perpetual_receiver
                .multiplier_up
                .map(|string| string.parse().expect("failed to parse")),
            multiplier_down: perpetual_receiver
                .multiplier_down
                .parse()
                .expect("failed to parse"),
            average_time_interval: perpetual_receiver.average_time_interval,
            progressive_tier_eligible: perpetual_receiver.progressive_tier_eligible,
            index_price: perpetual_receiver
                .index_price
                .parse()
                .expect("failed to parse"),
            mark_price: perpetual_receiver
                .mark_price
                .parse()
                .expect("failed to parse"),
            funding_rate: perpetual_receiver
                .funding_rate
                .parse()
                .expect("failed to parse"),
            fair_price: perpetual_receiver
                .fair_price
                .parse()
                .expect("failed to parse"),
            average_funding_rate_8h: perpetual_receiver
                .average_funding_rate_8h
                .parse()
                .expect("failed to parse"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionReceiver {
    id: u64,
    quantity: f64,
    price: f64,
    taker_side: String,
    created_at: u32,
    timestamp: String,
}

#[derive(Clone, Debug)]
pub struct Execution {
    pub id: u64,
    pub quantity: f64,
    pub price: i32,
    pub taker_side: String,
    pub created_at: u32,
    pub timestamp: f64,
}

impl DataGenerater<ExecutionReceiver> for Execution {
    fn generate_from_receiver(receiver: ExecutionReceiver) -> Execution {
        Execution {
            id: receiver.id,
            quantity: receiver.quantity,
            price: receiver.price as i32,
            taker_side: receiver.taker_side,
            created_at: receiver.created_at,
            timestamp: receiver.timestamp.parse().expect("failed to parse"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderReceiver {
    price: String,
    amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub price: i32,
    pub amount: f64,
}

impl DataGenerater<OrderReceiver> for Order {
    fn generate_from_receiver(order_reciver: OrderReceiver) -> Order {
        Order {
            price: order_reciver.price.parse::<f64>().expect("failed to parse") as i32,
            amount: order_reciver.amount.parse().expect("failed to parse"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderBookReceiver {
    buy_price_levels: Vec<OrderReceiver>,
    sell_price_levels: Vec<OrderReceiver>,
    timestamp: String,
}

#[derive(Clone, Debug)]
pub struct OrderBook {
    pub buy_price_levels: Vec<Order>,
    pub sell_price_levels: Vec<Order>,
    pub timestamp: f64,
}

impl DataGenerater<OrderBookReceiver> for OrderBook {
    fn generate_from_receiver(order_book_receiver: OrderBookReceiver) -> OrderBook {
        let mut buy = Vec::new();
        for o in order_book_receiver.buy_price_levels {
            buy.push(Order::generate_from_receiver(o));
        }
        let mut sell = Vec::new();
        for o in order_book_receiver.sell_price_levels {
            sell.push(Order::generate_from_receiver(o));
        }
        OrderBook {
            buy_price_levels: buy,
            sell_price_levels: sell,
            timestamp: order_book_receiver
                .timestamp
                .parse()
                .expect("failed to parse"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyExecutionReceiver {
    id: u64,
    quantity: String,
    price: String,
    taker_side: String,
    my_side: String,
    created_at: u32,
    order_id: u64,
    client_order_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MyExecution {
    pub id: u64,
    pub quantity: f64,
    pub price: i32,
    pub taker_side: String,
    pub my_side: String,
    pub created_at: u32,
    pub order_id: u64,
    pub client_order_id: Option<String>,
}

impl DataGenerater<MyExecutionReceiver> for MyExecution {
    fn generate_from_receiver(receiver: MyExecutionReceiver) -> MyExecution {
        MyExecution {
            id: receiver.id,
            quantity: receiver.quantity.parse().expect("failed to parse"),
            price: receiver.price.parse::<f64>().expect("failed to parse") as i32,
            taker_side: receiver.taker_side,
            my_side: receiver.my_side,
            created_at: receiver.created_at,
            order_id: receiver.order_id,
            client_order_id: receiver.client_order_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pagination<T> {
    pub models: Vec<T>,
    pub current_page: u32,
    pub total_pages: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostOrderResponseReceiver {
    pub id: u64,
    pub order_type: String,
    pub quantity: String,
    pub disc_quantity: String,
    pub iceberg_total_quantity: String,
    pub side: String,
    pub filled_quantity: String,
    pub price: f64,
    pub created_at: u32,
    pub updated_at: u32,
    pub status: String,
    pub leverage_level: u32,
    pub source_exchange: String,
    pub product_id: u32,
    pub margin_type: Option<String>,
    pub take_profit: Option<String>,
    pub stop_loss: Option<String>,
    pub trading_type: String,
    pub product_code: String,
    pub funding_currency: String,
    pub crypto_account_id: Option<u32>,
    pub currency_pair_code: String,
    pub average_price: f64,
    pub target: String,
    pub order_fee: f64,
    pub source_action: String,
    pub unwound_trade_id: Option<u64>,
    pub trade_id: Option<String>,
    pub client_order_id: String,
}

#[derive(Clone, Debug)]
pub struct PostOrderResponse {
    pub id: u64,
    pub order_type: String,
    pub quantity: f64,
    pub disc_quantity: f64,
    pub iceberg_total_quantity: f64,
    pub side: String,
    pub filled_quantity: f64,
    pub price: f64,
    pub created_at: u32,
    pub updated_at: u32,
    pub status: String,
    pub leverage_level: u32,
    pub source_exchange: String,
    pub product_id: u32,
    pub margin_type: Option<String>,
    pub take_profit: Option<String>,
    pub stop_loss: Option<String>,
    pub trading_type: String,
    pub product_code: String,
    pub funding_currency: String,
    pub crypto_account_id: Option<u32>,
    pub currency_pair_code: String,
    pub average_price: f64,
    pub target: String,
    pub order_fee: f64,
    pub source_action: String,
    pub unwound_trade_id: Option<u64>,
    pub trade_id: Option<String>,
    pub client_order_id: String,
}

impl DataGenerater<PostOrderResponseReceiver> for PostOrderResponse {
    fn generate_from_receiver(receiver: PostOrderResponseReceiver) -> PostOrderResponse {
        PostOrderResponse {
            id: receiver.id,
            order_type: receiver.order_type.parse().expect("failed to parse"),
            quantity: receiver.quantity.parse().expect("failed to parse"),
            disc_quantity: receiver.disc_quantity.parse().expect("failed to parse"),
            iceberg_total_quantity: receiver
                .iceberg_total_quantity
                .parse()
                .expect("failed to parse"),
            side: receiver.side,
            filled_quantity: receiver.filled_quantity.parse().expect("failed to parse"),
            price: receiver.price,
            created_at: receiver.created_at,
            updated_at: receiver.updated_at,
            status: receiver.status,
            leverage_level: receiver.leverage_level,
            source_exchange: receiver.source_exchange,
            product_id: receiver.product_id,
            margin_type: receiver.margin_type,
            take_profit: receiver.take_profit,
            stop_loss: receiver.stop_loss,
            trading_type: receiver.trading_type,
            product_code: receiver.product_code,
            funding_currency: receiver.funding_currency,
            crypto_account_id: receiver.crypto_account_id,
            currency_pair_code: receiver.currency_pair_code,
            average_price: receiver.average_price,
            target: receiver.target,
            order_fee: receiver.order_fee,
            source_action: receiver.source_action,
            unwound_trade_id: receiver.unwound_trade_id,
            trade_id: receiver.trade_id,
            client_order_id: receiver.client_order_id,
        }
    }
}
