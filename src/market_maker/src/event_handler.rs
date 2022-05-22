use common::*;
use liquid::*;

use crate::trader::TraderEventSender;

pub async fn product_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if let Some(content) = &response.event {
        match content.as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "updated" => {
                let data = match liquid_tap::generate_product(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                let ticker = database::data::Ticker {
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    timestamp: data.timestamp,
                    last_traded_price: data.last_traded_price,
                    last_traded_quantity: data.last_traded_quantity,
                    last_price_24h: data.last_price_24h,
                    average_price_24h: data.average_price,
                    volume_24h: data.volume_24h,
                    market_ask: data.market_ask,
                    market_bid: data.market_bid,
                    low_market_price_24h: data.low_market_bid,
                    high_market_price_24h: data.high_market_ask,
                };
                if let Err(result) = trader_trans.post_ticker(ticker).await {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                    return Err(());
                }
            }
            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
    Ok(())
}

pub async fn executions_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if response.event.as_ref().is_some() {
        match response.event.as_ref().unwrap().as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "created" => {
                let data = match liquid_tap::generate_execution(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                let execution = database::data::Execution {
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    timestamp: data.timestamp,
                    created_at: data.created_at as i32,
                    price: data.price,
                    quantity: data.quantity,
                    taker_side: data.taker_side,
                };
                if let Err(result) = trader_trans.post_execution(execution).await {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                    return Err(());
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
    Ok(())
}

pub async fn order_book_buy(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if response.event.as_ref().is_some() {
        match response.event.as_ref().unwrap().as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "updated" => {
                let data = match liquid_tap::generate_order_book(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                let order_book_buy_element = database::data::OrderBook {
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    orders: data,
                };
                if let Err(result) = trader_trans
                    .post_order_book_buy(order_book_buy_element)
                    .await
                {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                    return Err(());
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
    Ok(())
}

pub async fn order_book_sell(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if response.event.as_ref().is_some() {
        match response.event.as_ref().unwrap().as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "updated" => {
                let data = match liquid_tap::generate_order_book(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                let order_book_sell_element = database::data::OrderBook {
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    orders: data,
                };
                if let Err(result) = trader_trans
                    .post_order_book_sell(order_book_sell_element)
                    .await
                {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                    return Err(());
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
    Ok(())
}

pub async fn private_orders(
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if let Some(content) = &response.event {
        match content.as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "pusher_internal:subscription_failed" => {
                log::error!("subscribing {} failed!", response.channel.as_ref().unwrap());
                return Err(());
            }

            "updated" => {
                let data = match liquid_tap::generate_order_statuts(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                if let Err(result) = trader_trans.post_postion_from_order(data).await {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                    return Err(());
                }
            }
            _ => {
                log::warn!(
                    "{}",
                    error_message!("unknown event!\n-->\ndetails : {:?}\n<--", response)
                );
            }
        }
    }
    Ok(())
}

pub async fn private_trades(
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if let Some(content) = &response.event {
        match content.as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "pusher_internal:subscription_failed" => {
                log::error!("subscribing {} failed!", response.channel.as_ref().unwrap());
                return Err(());
            }

            "updated" => {
                let data = match liquid_tap::generate_trades_update(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                if let Err(result) = trader_trans.post_postion_from_trade(data).await {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                }
            }
            "pnl_updated" => {
                let data = match liquid_tap::generate_trades_panel_update(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return Err(());
                    }
                };
                if let Err(result) = trader_trans.post_postion_from_panel(data).await {
                    log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
                }
            }
            _ => {
                log::warn!(
                    "{}",
                    error_message!("unknown event!\n-->\ndetails : {:?}\n<--", response)
                );
            }
        }
    }
    Ok(())
}

pub async fn private_executions(
    response: &data_for_tap::ResponseValue,
    _trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if let Some(content) = &response.event {
        match content.as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "pusher_internal:subscription_failed" => {
                log::error!("subscribing {} failed!", response.channel.as_ref().unwrap());
                return Err(());
            }

            "created" => {}
            _ => {
                log::warn!(
                    "{}",
                    error_message!("unknown event!\n-->\ndetails : {:?}\n<--", response)
                );
            }
        }
    }
    Ok(())
}
