use std::sync::Arc;
use tokio::sync::mpsc;

use liquid::*;

use crate::misc::Config;
use crate::trader;
use crate::trader::*;

#[allow(dead_code)]
pub async fn post_close_order(position: trader::Position, config: Arc<Config>) -> Result<(), ()> {
    let client = LiquidClientAsync::new();

    let trade_id = match position.trade_id {
        Some(content) => content,
        None => return Err(()),
    };
    let _order_response = match client
        .close_positon(&config.key, trade_id, position.quantity)
        .await
    {
        Ok(result) => {
            log::info!(
                "succeeded to close positions!\n-->\ndetails : {:?}\n<--",
                result
            );
            result
        }
        Err(result) => {
            log::error!("failed to close positions!\n-->\ndetails : {}\n<--", result);
            return Err(());
        }
    };
    Ok(())
}

pub async fn post_detailed_order(
    buy_order: Order,
    sell_order: Order,
    config: Arc<Config>,
    positions_checker: PositionsChecker,
    positions_sender: mpsc::UnboundedSender<
        Result<
            (
                Result<data::PostOrderResponse, data::PostOrderResponse>,
                Result<data::PostOrderResponse, data::PostOrderResponse>,
            ),
            (),
        >,
    >,
) {
    let client = LiquidClientAsync::new();

    let (task_buy, task_sell) = tokio::join!(
        client.post_order(&config.key, &buy_order),
        client.post_order(&config.key, &sell_order)
    );

    let buy_order_response = match task_buy {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to post orders!\n-->\ndetails : {}\n<--", result);
            if let Err(result) = positions_sender.send(Err(())) {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
            return;
        }
    };

    let sell_order_response = match task_sell {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to post orders!\n-->\ndetails : {}\n<--", result);
            if let Err(result) = positions_sender.send(Err(())) {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
            return;
        }
    };

    let (task_wait_buy, task_wait_sell) = tokio::join!(
        positions_checker.wait_until_with_timeout(
            &buy_order_response,
            Some(PositionStatus::Filled),
            std::time::Duration::from_nanos((config.limited_time * 1_000_000_000.0) as u64),
        ),
        positions_checker.wait_until_with_timeout(
            &sell_order_response,
            Some(PositionStatus::Filled),
            std::time::Duration::from_nanos((config.limited_time * 1_000_000_000.0) as u64),
        )
    );

    match (task_wait_buy, task_wait_sell) {
        (Ok(()), Ok(())) => {
            if let Err(result) =
                positions_sender.send(Ok((Ok(buy_order_response), Ok(sell_order_response))))
            {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
        }
        (Ok(()), Err(_)) => {
            if let Err(result) =
                positions_sender.send(Ok((Ok(buy_order_response), Err(sell_order_response))))
            {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
        }
        (Err(_), Ok(())) => {
            if let Err(result) =
                positions_sender.send(Ok((Err(buy_order_response), Ok(sell_order_response))))
            {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
        }
        (Err(_), Err(_)) => {
            log::warn!(
                "failed to excute the both of orders in limited_time therefore, keep old order."
            );
            let (task_cancel_buy, task_cancel_sell) = tokio::join!(
                client.cancel_order(&config.key, buy_order_response.id),
                client.cancel_order(&config.key, sell_order_response.id)
            );
            match task_cancel_buy {
                Ok(result) => {
                    log::info!(
                        "succeeded to cancel orders!\n-->\ndetails : {:?}\n<--",
                        result
                    );
                }
                Err(result) => {
                    log::error!("failed to cancel orders!\n-->\ndetails : {}\n<--", result);
                }
            }
            match task_cancel_sell {
                Ok(result) => {
                    log::info!(
                        "succeeded to cancel orders!\n-->\ndetails : {:?}\n<--",
                        result
                    );
                }
                Err(result) => {
                    log::error!("failed to cancel orders!\n-->\ndetails : {}\n<--", result);
                }
            }
            if let Err(result) = positions_sender.send(Err(())) {
                log::error!("failed to post to trader!\n-->\ndetails : {}\n<--", result);
            }
        }
    }
}
