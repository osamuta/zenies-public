use common::*;
use liquid::*;

use crate::event_handler;
use crate::trader::TraderEventSender;

pub async fn channel_handler(
    client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    trader_trans: &TraderEventSender,
) -> Result<(), ()> {
    if let Some(content) = &response.channel {
        match content.as_str() {
            //
            // btcjpy arms
            "product_cash_btcjpy_5" => {
                event_handler::product_handler(client, response, trader_trans).await?;
            }
            "executions_cash_btcjpy" => {
                event_handler::executions_handler(client, response, trader_trans).await?;
            }

            "price_ladders_cash_btcjpy_buy" => {
                event_handler::order_book_buy(client, response, trader_trans).await?;
            }

            "price_ladders_cash_btcjpy_sell" => {
                event_handler::order_book_sell(client, response, trader_trans).await?;
            }

            "execution_details_cash_btcjpy" => {
                //event_handler::executions_details_handler(client, response, database);
            }

            //
            // ethjpy arms
            "product_cash_ethjpy_29" => {
                event_handler::product_handler(client, response, trader_trans).await?;
            }
            "executions_cash_ethjpy" => {
                event_handler::executions_handler(client, response, trader_trans).await?;
            }

            "price_ladders_cash_ethjpy_buy" => {
                event_handler::order_book_buy(client, response, trader_trans).await?;
            }

            "price_ladders_cash_ethjpy_sell" => {
                event_handler::order_book_sell(client, response, trader_trans).await?;
            }

            "execution_details_cash_ethjpy" => {
                //event_handler::executions_details_handler(client, response, database);
            }

            //
            // private arms
            "user_account_jpy_orders" => {
                event_handler::private_orders(response, trader_trans).await?;
            }

            "user_account_jpy_trades" => {
                event_handler::private_trades(response, trader_trans).await?;
            }

            "user_executions_cash_btcjpy" => {
                event_handler::private_executions(response, trader_trans).await?;
            }
            "user_executions_cash_ethjpy" => {
                event_handler::private_executions(response, trader_trans).await?;
            }
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
                return Err(());
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
