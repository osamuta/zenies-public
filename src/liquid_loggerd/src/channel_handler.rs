use super::event_handler;
use super::*;
//use liquid::*;

pub async fn channel_handler(
    client: &mut LiquidTapClientAsync,
    response: &data_for_tap::ResponseValue,
    database: std::sync::Arc<Database>,
    transmitter: &tokio::sync::mpsc::UnboundedSender<tokio::task::JoinHandle<()>>,
) {
    if let Some(content) = &response.channel {
        match content.as_str() {
            "product_cash_btcjpy_5" => {
                event_handler::product_handler(client, response, database, transmitter).await;
            }

            "executions_cash_btcjpy" => {
                event_handler::executions_handler(client, response, database, transmitter).await;
            }

            "price_ladders_cash_btcjpy_buy" => {
                event_handler::order_book_buy(client, response, database, transmitter).await;
            }

            "price_ladders_cash_btcjpy_sell" => {
                event_handler::order_book_sell(client, response, database, transmitter).await;
            }

            "execution_details_cash_btcjpy" => {
                event_handler::executions_details_handler(client, response, database, transmitter);
            }

            "time-signal" => {}
            _ => {
                log::warn!("unknown channel!\n-->\ndetails : {:?}\n<--", response);
            }
        }
    }
}
