use statrs::distribution::{ContinuousCDF, FisherSnedecor};
use statrs::statistics::Statistics;
// use std::cmp::Ordering;
use ndarray::prelude::*;
//use rayon::prelude::*;
use chrono::prelude::*;
use itertools::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::fs::*;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Notify, RwLock};
use tokio::time;

use common::*;
use database::data;
use liquid::*;
use misc::{Config, State};
use stats::*;

use crate::misc;
use crate::order;

#[derive(Clone, Debug, PartialEq)]
pub enum PositionStatus {
    Live,
    Filled,
    Cancelled,
    Open,
    Closed,
}

impl Default for PositionStatus {
    fn default() -> Self {
        PositionStatus::Live
    }
}

impl PositionStatus {
    pub fn generate_from_string(status: &str) -> Result<Self, String> {
        match status {
            "live" => Ok(PositionStatus::Live),
            "filled" => Ok(PositionStatus::Filled),
            "cancelled" => Ok(PositionStatus::Cancelled),
            _ => return Err(error_message!("unknown status!\ndetails : {}", status)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Position {
    pub order_id: u64,
    pub trade_id: Option<u64>,
    pub price: i32,
    pub quantity: f64,
    pub side: Side,
    pub status: PositionStatus,
    pub profit: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct PositionUpdate {
    pub order_id: u64,
    pub trade_id: u64,
    pub profit: f64,
}

#[derive(Clone, Debug)]
pub enum TraderEvent {
    Booted,
    Shutdown,
    Ticker(data::Ticker),
    Execution(data::Execution),
    OrderBookBuy(data::OrderBook),
    OrderBookSell(data::OrderBook),
    MarketInfoUpdated,
    Position(Position),
    PositionUpdate(PositionUpdate),
}

pub fn new_trader_event_channel() -> (TraderEventSender, TraderEventReceiver) {
    let channel = mpsc::unbounded_channel();
    (
        TraderEventSender::new(channel.0),
        TraderEventReceiver::new(channel.1),
    )
}

#[derive(Clone, Debug)]
pub struct TraderEventSender {
    sender: mpsc::UnboundedSender<TraderEvent>,
}

impl TraderEventSender {
    pub fn new(sender: mpsc::UnboundedSender<TraderEvent>) -> Self {
        Self { sender }
    }

    #[allow(dead_code)]
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    pub async fn post_booted(&self) -> Result<(), String> {
        match self.sender.send(TraderEvent::Booted) {
            Ok(()) => Ok(()),
            Err(result) => Err(error_message!("failed to send!\ndetails : {}", result)),
        }
    }

    pub async fn post_shudown(&self) -> Result<(), String> {
        match self.sender.send(TraderEvent::Shutdown) {
            Ok(()) => Ok(()),
            Err(result) => Err(error_message!("failed to send!\ndetails : {}", result)),
        }
    }

    pub async fn post_ticker(&self, ticker: data::Ticker) -> Result<(), String> {
        match self.sender.send(TraderEvent::Ticker(ticker)) {
            Ok(()) => {}
            Err(result) => return Err(error_message!("failed to send!\ndetails : {}", result)),
        };
        self.post_market_info_updated().await
    }

    pub async fn post_execution(&self, execution: data::Execution) -> Result<(), String> {
        match self.sender.send(TraderEvent::Execution(execution)) {
            Ok(()) => {}
            Err(result) => return Err(error_message!("failed to send!\ndetails : {}", result)),
        };
        self.post_market_info_updated().await
    }

    pub async fn post_order_book_buy(&self, order_book: data::OrderBook) -> Result<(), String> {
        match self.sender.send(TraderEvent::OrderBookBuy(order_book)) {
            Ok(()) => {}
            Err(result) => return Err(error_message!("failed to send!\ndetails : {}", result)),
        };
        self.post_market_info_updated().await
    }

    pub async fn post_order_book_sell(&self, order_book: data::OrderBook) -> Result<(), String> {
        match self.sender.send(TraderEvent::OrderBookSell(order_book)) {
            Ok(()) => {}
            Err(result) => return Err(error_message!("failed to send!\ndetails : {}", result)),
        };
        self.post_market_info_updated().await
    }

    pub async fn post_market_info_updated(&self) -> Result<(), String> {
        match self.sender.send(TraderEvent::MarketInfoUpdated) {
            Ok(()) => Ok(()),
            Err(result) => Err(error_message!("failed to send!\ndetails : {}", result)),
        }
    }

    pub async fn post_postion(&self, postion: Position) -> Result<(), String> {
        match self.sender.send(TraderEvent::Position(postion)) {
            Ok(()) => Ok(()),
            Err(result) => Err(error_message!("failed to send!\ndetails : {}", result)),
        }
    }

    pub async fn post_postion_from_order(
        &self,
        order: data_for_tap::OrderStatus,
    ) -> Result<(), String> {
        let status = match PositionStatus::generate_from_string(&order.status) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to parse status!\ndetails : {}",
                    result
                ))
            }
        };
        let side = match currency::Side::generate_from_string(&order.side) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to parse side!\ndetails : {}",
                    result
                ))
            }
        };
        let position = Position {
            order_id: order.id,
            trade_id: order.trade_id,
            price: order.price as i32,
            quantity: order.quantity,
            side: side,
            status: status,
            profit: None,
        };

        self.post_postion(position).await?;
        Ok(())
    }

    pub async fn post_postion_from_trade(
        &self,
        trade: data_for_tap::TradesUpdate,
    ) -> Result<(), String> {
        let status = if trade.open_quantity >= trade.close_quantity {
            PositionStatus::Open
        } else {
            PositionStatus::Closed
        };
        let side = match currency::Side::generate_from_string(&trade.side) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to parse side!\ndetails : {}",
                    result
                ))
            }
        };
        let position = Position {
            order_id: trade.order_id,
            trade_id: Some(trade.id),
            price: trade.open_price as i32,
            quantity: trade.quantity,
            side: side,
            status: status,
            profit: Some(trade.close_pnl),
        };

        self.post_postion(position).await?;
        Ok(())
    }

    pub async fn post_postion_from_panel(
        &self,
        update: data_for_tap::TradesPanelUpdate,
    ) -> Result<(), String> {
        let position = PositionUpdate {
            order_id: update.order_id,
            trade_id: update.id,
            profit: update.open_pnl.parse::<f64>().expect("failed to parse!"),
        };
        match self.sender.send(TraderEvent::PositionUpdate(position)) {
            Ok(()) => Ok(()),
            Err(result) => Err(error_message!("failed to send!\ndetails : {}", result)),
        }
    }
}

#[derive(Debug)]
pub struct TraderEventReceiver {
    orders: Arc<RwLock<HashMap<u64, Position>>>,
    notifier: Arc<Notify>,
    receiver: mpsc::UnboundedReceiver<TraderEvent>,
}

impl TraderEventReceiver {
    pub fn new(receiver: mpsc::UnboundedReceiver<TraderEvent>) -> Self {
        Self {
            orders: Arc::new(RwLock::new(HashMap::new())),
            notifier: Arc::new(Notify::new()),
            receiver,
        }
    }

    pub async fn recv(&mut self) -> Result<TraderEvent, String> {
        loop {
            match self.receiver.recv().await {
                Some(event) => {
                    let mut orders = self.orders.write().await;
                    match &event {
                        TraderEvent::Position(position) => {
                            orders.insert(position.order_id, position.clone());
                            self.notifier.notify_waiters();
                            return Ok(event);
                        }
                        TraderEvent::PositionUpdate(position) => {
                            match orders.get_mut(&position.order_id) {
                                Some(content) => {
                                    content.profit = Some(position.profit);
                                    self.notifier.notify_waiters();
                                }
                                None => {}
                            };
                            return Ok(event);
                        }
                        _ => return Ok(event),
                    };
                }
                None => return Err(error_message!("channel closed!")),
            };
        }
    }

    pub fn new_order_checker(&self) -> PositionsChecker {
        PositionsChecker {
            orders: self.orders.clone(),
            notifier: self.notifier.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PositionsChecker {
    orders: Arc<RwLock<HashMap<u64, Position>>>,
    notifier: Arc<Notify>,
}

impl PositionsChecker {
    #[allow(dead_code)]
    pub async fn get_raw(&self, id: u64) -> Option<Position> {
        let orders = self.orders.read().await;
        match orders.get(&id).cloned() {
            Some(content) => Some(content),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub async fn get(&self, res: &liquid::data::PostOrderResponse) -> Option<Position> {
        self.get_raw(res.id).await
    }

    pub async fn check_raw(&self, id: u64) -> Option<PositionStatus> {
        let orders = self.orders.read().await;
        match orders.get(&id).cloned() {
            Some(content) => Some(content.status),
            None => None,
        }
    }

    pub async fn check(&self, res: &liquid::data::PostOrderResponse) -> Option<PositionStatus> {
        self.check_raw(res.id).await
    }

    pub async fn wait_until(
        &self,
        res: &liquid::data::PostOrderResponse,
        status: Option<PositionStatus>,
    ) -> Result<(), String> {
        loop {
            if self.check(res).await == status {
                return Ok(());
            }
            tokio::task::yield_now().await;
            self.notifier.notified().await;
        }
    }

    #[allow(dead_code)]
    pub async fn wait_until_with_timeout(
        &self,
        res: &liquid::data::PostOrderResponse,
        status: Option<PositionStatus>,
        duration: std::time::Duration,
    ) -> Result<(), String> {
        match time::timeout(duration, self.wait_until(res, status)).await {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "exeeded limited time!\ndetails : {:?}",
                result
            )),
        }
    }
}

#[derive(Clone, Debug)]
enum Mode {
    Await,
    PostOrder,
    PostingOrder,
}

pub async fn trader(
    mut trader_recv: TraderEventReceiver,
    config: Arc<misc::Config>,
    env: Arc<Enviornment>,
) {
    let mut market_info = MarketInfo::new();
    let (positions_sender, mut positions_receiver) = mpsc::unbounded_channel::<
        Result<
            (
                Result<liquid::data::PostOrderResponse, liquid::data::PostOrderResponse>,
                Result<liquid::data::PostOrderResponse, liquid::data::PostOrderResponse>,
            ),
            (),
        >,
    >();
    let positions_checker = trader_recv.new_order_checker();
    let mut mode = Mode::Await;
    let mut state = match misc::State::load(
        &(env.general.etc_directory_path.clone()
            + env!("CARGO_PKG_NAME")
            + "_"
            + &config.identifier
            + ".state"),
    )
    .await
    {
        Ok(content) => content,
        Err(_) => {
            return;
        }
    };
    let mut stats = Stats::new(config.clone(), &state);
    loop {
        tokio::select! {
            Ok(event) = trader_recv.recv() => {
                match event {
                    TraderEvent::Booted => {
                        mode = Mode::PostOrder;
                        match mode {
                            Mode::Await => {}
                            Mode::PostOrder => {
                                if let Ok((buy_order, sell_order)) = build_strategy(config.clone(), &stats).await {
                                    if !config.dry_trade {
                                        tokio::spawn(order::post_detailed_order(buy_order, sell_order, config.clone(), positions_checker.clone(), positions_sender.clone()));
                                        mode = Mode::PostingOrder;
                                    }
                                }
                            }
                            _ => {}
                        };
                    }
                    TraderEvent::Shutdown => break,
                    TraderEvent::Ticker(ticker) => {
                        market_info.tickers.push(ticker);

                    }
                    TraderEvent::Execution(execution) => {
                        market_info.executions.push(execution);
                        market_info.vacuume(config.evaluation_time);
                    }
                    TraderEvent::OrderBookBuy(order_book_buy) => {
                        market_info.order_book_buy.push(order_book_buy);

                    }
                    TraderEvent::OrderBookSell(order_book_sell) => {
                        market_info.order_book_sell.push(order_book_sell);
                    }
                    TraderEvent::MarketInfoUpdated => {
                        market_info.vacuume(config.evaluation_time);
                        stats.vacuume(config.evaluation_time);
                        let _ = stats.calculate_stats(&market_info);
                        match mode {
                            Mode::Await => {}
                            Mode::PostOrder => {
                                if let Ok((buy_order, sell_order)) = build_strategy(config.clone(), &stats).await {
                                    if !config.dry_trade {
                                        tokio::spawn(order::post_detailed_order(buy_order, sell_order, config.clone(), positions_checker.clone(), positions_sender.clone()));
                                        mode = Mode::PostingOrder;
                                    }
                                }
                            }
                            _ => {}
                        };
                    }
                    TraderEvent::Position(_position) => {
                    }
                    TraderEvent::PositionUpdate(_position) => {
                    }
                };
            }
            Some(content) = positions_receiver.recv() => {
                match content {
                    Ok(result) => {
                        mode = Mode::PostOrder;
                        match result {
                            (Ok(_), Ok(_)) => {
                                stats.succeed();
                            }
                            (Ok(_), Err(_)) => {
                                log::error!("failed to make market!");
                                stats.fail();
                                stats.decrease_offset();
                            }
                            (Err(_), Ok(_)) => {
                                log::error!("failed to make market!");
                                stats.fail();
                                stats.increase_offset();
                            }
                            (Err(_), Err(_)) => {
                                log::error!("failed to make market!");

                            }
                        }
                    }
                    Err(_) => {
                        log::error!("retry!");
                        mode = Mode::PostOrder;
                    }
                }
            }
            else => break,
        };
    }
    if let Err(_) = state
        .save(
            &(env.general.etc_directory_path.clone()
                + env!("CARGO_PKG_NAME")
                + "_"
                + &config.identifier
                + ".state"),
            stats.offset,
            stats.succeeded_trade,
            stats.whole_trade,
        )
        .await
    {}
}

#[allow(dead_code)]
async fn record(num: &mut u32, env: Arc<Enviornment>, config: Arc<misc::Config>, profit: f64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(
            &(env.general.log_directory_path.clone()
                + env!("CARGO_PKG_NAME")
                + "_"
                + &config.identifier
                + ".csv"),
        )
        .await
    {
        Ok(mut result) => {
            if let Err(result) = result
                .write_all(format!("{},{},{}\n", num, now, profit).as_bytes())
                .await
            {
                log::warn!("failed to write!\n-->\ndetails : {:?}\n<--", result);
            }
            *num += 1;
        }
        Err(result) => log::warn!("failed to open csv file!\n-->\ndetails : {:?}\n<--", result),
    };
}

#[allow(dead_code)]
async fn check_market(market_info: &MarketInfo) -> bool {
    let length = market_info.tickers.len();
    let n_1 = market_info.tickers[..length / 2].len() as f64;
    let n_2 = market_info.tickers[length / 2..].len() as f64;
    let v_1 = market_info.tickers[..length / 2]
        .iter()
        .map(|e| e.last_traded_price as f64)
        .variance();
    let v_2 = market_info.tickers[length / 2..]
        .iter()
        .map(|e| e.last_traded_price as f64)
        .variance();
    let m_1 = market_info.tickers[..length / 2]
        .iter()
        .map(|e| e.last_traded_price as f64)
        .mean();
    let m_2 = market_info.tickers[length / 2..]
        .iter()
        .map(|e| e.last_traded_price as f64)
        .mean();
    let f = if v_1 > v_2 { v_1 / v_2 } else { v_2 / v_1 };
    let f_dist = match FisherSnedecor::new(n_1 - 1.0, n_2 - 1.0) {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to check market!\n-->\ndetails : {:?}\n<--", result);
            return false;
        }
    };
    let p = f_dist.cdf(f);

    let t = (m_1 - m_2)
        / ((n_1 * v_1 + n_2 * v_2) / (n_1 + n_2 - 2.0) * (1.0 / n_1 + 1.0 / n_2)).sqrt();

    if p > 0.95 || t.abs() > 1.96 {
        false
    } else {
        log::info!("range market! p : {}, t : {}", p, t.abs());
        true
    }
}

async fn build_strategy(config: Arc<misc::Config>, stats: &Stats) -> Result<(Order, Order), ()> {
    let cross_point_price = match stats.cross_point_price.back() {
        Some(content) => content,
        None => {
            log::error!("must contain!");
            return Err(());
        }
    };
    let (buy_spread_mean, sell_spread_mean, trend, r_mean) = (
        stats.buy_spread_sum / stats.buy_spread.len() as f64,
        stats.sell_spread_sum / stats.sell_spread.len() as f64,
        (stats.buy_angle_sum + stats.sell_angle_sum)
            / (stats.buy_angle.len() + stats.sell_angle.len()) as f64,
        (stats.buy_r_sum + stats.sell_r_sum) / (stats.buy_r.len() + stats.sell_r.len()) as f64,
    );
    let rate = stats.succeeded_trade as f64 / stats.whole_trade as f64;

    log::info!(
        "[ STRATEGY ] cross_point_price : {}, buy_spread : {}, sell_spread : {}, offset : {}, succeeded_trade : {}, whole_trade : {}, success_rate : {}, trend : {}, R : {}",
        cross_point_price,
        buy_spread_mean,
        sell_spread_mean,
        stats.offset,
        stats.succeeded_trade,
        stats.whole_trade,
        rate,
        trend,
        r_mean,
    );
    let lower_price =
        (cross_point_price + stats.offset as f64 + buy_spread_mean * rate + trend).round() as i32;
    let higher_price =
        (cross_point_price + stats.offset as f64 + sell_spread_mean * rate + trend).round() as i32;

    let buy_order = Order::limit(
        config.currency_pair.generate_id(),
        Side::Buy,
        config.quantity,
        lower_price,
    );
    let sell_order = Order::limit(
        config.currency_pair.generate_id(),
        Side::Sell,
        config.quantity,
        higher_price,
    );

    Ok((buy_order, sell_order))
}

fn calculate_r(order_book: &Vec<(f64, f64)>, model: &Array<f64, Ix1>) -> f64 {
    let model_buy = |q: f64| model[0] * q + model[1];
    let top_sum = order_book
        .iter()
        .map(|e| (e.1 - model_buy(e.0)).powi(2))
        .sum::<f64>();
    let mean = order_book.iter().map(|e| e.1).mean();
    let bottom_sum = order_book.iter().map(|e| (e.1 - mean).powi(2)).sum::<f64>();
    1.0 - top_sum / bottom_sum
}

fn convert_order_book_to_order_book_acumulated(order_book: &data::OrderBook) -> Vec<(f64, f64)> {
    let mut sum = 0.0f64;
    order_book
        .orders
        .iter()
        .map(|e| {
            sum += e.amount;
            (sum, e.price as f64)
        })
        .collect::<Vec<(f64, f64)>>()
}

fn calculate_line(data: &Vec<(f64, f64)>) -> Result<Array<f64, Ix1>, String> {
    math::linear_regression(&data)
}

fn get_cross_point(
    buy_line: &Array<f64, Ix1>,
    sell_line: &Array<f64, Ix1>,
) -> Result<Array<f64, Ix1>, String> {
    math::get_cross_point(&buy_line, &sell_line)
}

#[derive(Clone, Debug)]
struct Stats {
    pub offset: i32,
    pub offset_unit: i32,
    pub succeeded_trade: usize,
    pub whole_trade: usize,
    pub timestamps: VecDeque<f64>,
    pub cross_point_price: VecDeque<f64>,
    pub buy_spread: VecDeque<f64>,
    pub sell_spread: VecDeque<f64>,
    pub buy_angle: VecDeque<f64>,
    pub sell_angle: VecDeque<f64>,
    pub buy_r: VecDeque<f64>,
    pub sell_r: VecDeque<f64>,
    pub buy_spread_sum: f64,
    pub sell_spread_sum: f64,
    pub buy_angle_sum: f64,
    pub sell_angle_sum: f64,
    pub buy_r_sum: f64,
    pub sell_r_sum: f64,
}

impl Stats {
    pub fn new(config: Arc<Config>, state: &State) -> Self {
        Self {
            offset: state.offset,
            offset_unit: config.offset_unit,
            succeeded_trade: state.succeeded_trade,
            whole_trade: state.whole_trade,
            timestamps: VecDeque::new(),
            cross_point_price: VecDeque::new(),
            buy_spread: VecDeque::new(),
            sell_spread: VecDeque::new(),
            buy_angle: VecDeque::new(),
            sell_angle: VecDeque::new(),
            buy_r: VecDeque::new(),
            sell_r: VecDeque::new(),
            buy_spread_sum: 0.0,
            sell_spread_sum: 0.0,
            buy_angle_sum: 0.0,
            sell_angle_sum: 0.0,
            buy_r_sum: 0.0,
            sell_r_sum: 0.0,
        }
    }

    pub fn increase_offset(&mut self) {
        self.offset += self.offset_unit;
    }
    pub fn decrease_offset(&mut self) {
        self.offset -= self.offset_unit;
    }
    pub fn succeed(&mut self) {
        self.succeeded_trade += 1;
        self.whole_trade += 1;
    }

    pub fn fail(&mut self) {
        self.whole_trade += 1;
    }

    pub fn vacuume(&mut self, period: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let index = match self.timestamps.iter().position(|&e| e > now - period) {
            Some(content) => content,
            None => return,
        };
        self.timestamps.drain(0..index);
        self.cross_point_price.drain(0..index);
        self.buy_spread_sum -= self.buy_spread.drain(0..index).sum::<f64>();
        self.sell_spread_sum -= self.sell_spread.drain(0..index).sum::<f64>();
        self.buy_angle_sum -= self.buy_angle.drain(0..index).sum::<f64>();
        self.sell_angle_sum -= self.sell_angle.drain(0..index).sum::<f64>();
        self.buy_r_sum -= self.buy_r.drain(0..index).sum::<f64>();
        self.sell_r_sum -= self.sell_r.drain(0..index).sum::<f64>();
    }

    pub fn calculate_stats(&mut self, market_info: &MarketInfo) {
        let ticker = match market_info.tickers.last() {
            Some(content) => content,
            None => {
                return;
            }
        };
        let execution = match market_info.executions.last() {
            Some(content) => content,
            None => {
                return;
            }
        };
        let order_book_buy = match market_info.order_book_buy.last() {
            Some(content) => content,
            None => {
                return;
            }
        };
        let order_book_sell = match market_info.order_book_sell.last() {
            Some(content) => content,
            None => {
                return;
            }
        };

        let (order_book_buy_acumulated, order_book_sell_acumulated) = (
            convert_order_book_to_order_book_acumulated(&order_book_buy),
            convert_order_book_to_order_book_acumulated(&order_book_sell),
        );

        let buy_line = match calculate_line(&order_book_buy_acumulated) {
            Ok(result) => Arc::new(result),
            Err(result) => {
                log::error!(
                    "failed to linear regression!\n-->\ndetails : {:?}\n<--",
                    result
                );
                return;
            }
        };
        let sell_line = match calculate_line(&order_book_sell_acumulated) {
            Ok(result) => Arc::new(result),
            Err(result) => {
                log::error!(
                    "failed to linear regression!\n-->\ndetails : {:?}\n<--",
                    result
                );
                return;
            }
        };

        let r_buy = calculate_r(&order_book_buy_acumulated, &buy_line);
        let r_sell = calculate_r(&order_book_sell_acumulated, &sell_line);
        let cross_point = match get_cross_point(&buy_line, &sell_line) {
            Ok(result) => result,
            Err(result) => {
                log::error!(
                    "failed to get cross point!\n-->\ndetails : {:?}\n<--",
                    result
                );
                return;
            }
        };

        self.timestamps.push_back(
            [
                ticker.received_at,
                execution.received_at,
                order_book_buy.received_at,
                order_book_sell.received_at,
            ]
            .max(),
        );
        self.cross_point_price.push_back(cross_point[1]);
        self.buy_spread
            .push_back(order_book_buy.orders[0].price as f64 - cross_point[1]);
        self.sell_spread
            .push_back(order_book_sell.orders[0].price as f64 - cross_point[1]);
        self.buy_angle.push_back(buy_line[0]);
        self.sell_angle.push_back(sell_line[0]);
        self.buy_r.push_back(r_buy);
        self.sell_r.push_back(r_sell);

        self.buy_spread_sum += order_book_buy.orders[0].price as f64 - cross_point[1];
        self.sell_spread_sum += order_book_sell.orders[0].price as f64 - cross_point[1];
        self.buy_angle_sum += buy_line[0];
        self.sell_angle_sum += sell_line[0];
        self.buy_r_sum += r_buy;
        self.sell_r_sum += r_sell;
    }
}

pub async fn get_old_market(
    database: Arc<database::Database>,
    trader_trans: TraderEventSender,
    config: Arc<misc::Config>,
) -> Result<(), ()> {
    let end_time = Utc::now();
    let start_time =
        end_time - chrono::Duration::nanoseconds((config.evaluation_time * 1_000_000_000.0) as i64);
    match tokio::join!(
        tokio::spawn(getter::get_tickers(
            database.clone(),
            start_time.clone(),
            end_time.clone()
        )),
        tokio::spawn(getter::get_executions(
            database.clone(),
            start_time.clone(),
            end_time.clone()
        )),
        tokio::spawn(getter::get_order_book(
            database.clone(),
            common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY,
            start_time.clone(),
            end_time.clone(),
        )),
        tokio::spawn(getter::get_order_book(
            database.clone(),
            common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL,
            start_time.clone(),
            end_time.clone(),
        ))
    ) {
        (Ok(Ok(tickers)), Ok(Ok(executions)), Ok(Ok(order_book_buy)), Ok(Ok(order_book_sell))) => {
            let mut tickers_iter = tickers.into_iter();
            let mut executions_iter = executions.into_iter();
            let mut order_book_buy_iter = order_book_buy.into_iter();
            let mut order_book_sell_iter = order_book_sell.into_iter();
            let mut t = tickers_iter.next();
            let mut e = executions_iter.next();
            let mut b = order_book_buy_iter.next();
            let mut s = order_book_sell_iter.next();
            loop {
                let buf = [
                    match &t {
                        Some(content) => content.received_at,
                        None => std::f64::MAX,
                    },
                    match &e {
                        Some(content) => content.received_at,
                        None => std::f64::MAX,
                    },
                    match &b {
                        Some(content) => content.received_at,
                        None => std::f64::MAX,
                    },
                    match &s {
                        Some(content) => content.received_at,
                        None => std::f64::MAX,
                    },
                ];
                let index = buf.iter().position_min_by(|x, y| x.partial_cmp(y).unwrap());
                match index {
                    Some(0) => {
                        if let Some(t) = t {
                            if let Err(result) = trader_trans.post_ticker(t).await {
                                log::error!(
                                    "failed to post to trader!\n-->\ndetails : {}\n<--",
                                    result
                                );
                                return Err(());
                            }
                        } else {
                            break;
                        }
                        t = tickers_iter.next();
                    }
                    Some(1) => {
                        if let Some(e) = e {
                            if let Err(result) = trader_trans.post_execution(e).await {
                                log::error!(
                                    "failed to post to trader!\n-->\ndetails : {}\n<--",
                                    result
                                );
                                return Err(());
                            }
                        }
                        e = executions_iter.next();
                    }
                    Some(2) => {
                        if let Some(b) = b {
                            if let Err(result) = trader_trans.post_order_book_buy(b).await {
                                log::error!(
                                    "failed to post to trader!\n-->\ndetails : {}\n<--",
                                    result
                                );
                                return Err(());
                            }
                        }
                        b = order_book_buy_iter.next();
                    }
                    Some(3) => {
                        if let Some(s) = s {
                            if let Err(result) = trader_trans.post_order_book_sell(s).await {
                                log::error!(
                                    "failed to post to trader!\n-->\ndetails : {}\n<--",
                                    result
                                );
                                return Err(());
                            }
                        }
                        s = order_book_sell_iter.next();
                    }
                    _ => break,
                }
            }
        }
        result => {
            log::error!(
                "failed to get old market data\n-->\ndetails : {:?}\n<--",
                result
            );
            return Err(());
        }
    };

    Ok(())
}
