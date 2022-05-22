use super::*;

/*pub async fn product_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    transmitter: &watch::Sender<database::data::Ticker>,
) {
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
                        return;
                    }
                };
                let ticker = database::data::Ticker {
                    _id: mongodb::bson::oid::ObjectId::new(),
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
                if !transmitter.is_closed() {
                    if let Err(result) = transmitter.send(ticker) {
                        log::error!("failed to send!\n-->\ndetails : {}\n<--", result);
                    }
                }
            }
            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}*/

pub async fn logging_response(response: &data_for_tap::ResponseValue) {
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
            }

            "created" => {
                log::debug!("{:?},{:?}", response.channel, response.data);
            }

            "updated" => {
                log::debug!("{:?},{:?}", response.channel, response.data);
            }
            "orders_updated" => {
                log::debug!("{:?},{:?}", response.channel, response.data);
            }

            _ => {
                log::warn!(
                    "{}",
                    error_message!("unknown event!\n-->\ndetails : {:?}\n<--", response)
                );
            }
        }
    }
}

/*pub fn generate_value(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<serde_json::Value, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<serde_json::Value> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}*/

/*pub async fn executions_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    database: &Database,
) {
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
                        return;
                    }
                };
                let execution = database::data::Execution {
                    _id: mongodb::bson::oid::ObjectId::new(),
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
                if let Err(result) = database
                    .create(
                        common_constants::DATABASE_COLLECTION_EXECUTIONS,
                        &[execution],
                        None,
                    )
                    .await
                {
                    log::error!("failed to create posts!\n-->\ndetails : {}\n<--", result);
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}

pub async fn order_book_buy(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    database: &Database,
) {
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
                        return;
                    }
                };
                let order_book_buy = database::data::OrderBook {
                    _id: mongodb::bson::oid::ObjectId::new(),
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    orders: data,
                };
                if let Err(result) = database
                    .create(
                        common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY,
                        &[order_book_buy],
                        None,
                    )
                    .await
                {
                    log::error!("failed to create posts!\n-->\ndetails : {}\n<--", result);
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}

pub async fn order_book_sell(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    database: &Database,
) {
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
                        return;
                    }
                };
                let order_book_sell = database::data::OrderBook {
                    _id: mongodb::bson::oid::ObjectId::new(),
                    received_at: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    orders: data,
                };
                if let Err(result) = database
                    .create(
                        common_constants::DATABASE_COLLECTION_ORDER_BOOK_SELL,
                        &[order_book_sell],
                        None,
                    )
                    .await
                {
                    log::error!("failed to create posts!\n-->\ndetails : {}\n<--", result);
                }
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}

pub fn executions_details_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    _database: &Database,
) {
    if response.event.as_ref().is_some() {
        match response.event.as_ref().unwrap().as_str() {
            "pusher_internal:subscription_succeeded" => {
                log::info!(
                    "subscribing {} succeeded.",
                    response.channel.as_ref().unwrap()
                );
            }

            "created" => {
                let _data = match liquid_tap::generate_execution_details(&response.data) {
                    Ok(result) => result,
                    Err(result) => {
                        log::warn!(
                            "failed to generate!\n-->\ndetails : {}\nresponse : {:?}<--",
                            result,
                            response
                        );
                        return;
                    }
                };
            }

            _ => {
                log::warn!("unknown event!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}*/
