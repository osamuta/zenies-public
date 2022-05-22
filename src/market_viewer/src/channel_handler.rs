use super::event_handler;
use super::*;
//use liquid::*;

pub async fn channel_handler(
    _client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
) {
    if let Some(content) = &response.channel {
        match content.as_str() {
            //"product_cash_btcjpy_5" => {
            //    event_handler::product_handler(client, response, transmitter_market_info).await;
            //}
            "user_account_jpy_orders" => {
                event_handler::logging_response(response).await;
            }

            "user_account_jpy_trades" => {
                event_handler::logging_response(response).await;
            }

            "user_executions_cash_btcjpy" => {
                event_handler::logging_response(response).await;
            }

            "user_account_jpy" => {
                event_handler::logging_response(response).await;
            }

            "user" => {
                event_handler::logging_response(response).await;
            }

            //"executions_cash_btcjpy" => {
            //    event_handler::executions_handler(client, response, database).await;
            //}

            //"price_ladders_cash_btcjpy_buy" => {
            //    event_handler::order_book_buy(client, response, database).await;
            //}

            //"price_ladders_cash_btcjpy_sell" => {
            //    event_handler::order_book_sell(client, response, database).await;
            //}

            //"execution_details_cash_btcjpy" => {
            //    event_handler::executions_details_handler(client, response, database);
            //}
            "time-signal" => {}
            _ => {
                log::warn!("unknown channel!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    } else if let Some(content) = &response.event {
        match content.as_str() {
            "quoine:auth_success" => {
                log::info!("authentication succeeded.",);
            }

            "quoine:auth_failure" => {
                log::error!("authentication failed!");
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
