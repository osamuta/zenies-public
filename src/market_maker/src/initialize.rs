use std::sync::Arc;

use common::*;
use database::*;
use liquid::*;

pub async fn initialize(
    key: &LiquidApiKey,
    currency_pair: CurrencyPair,
    db_path: &str,
) -> Result<(LiquidTapClientAsync, Arc<database::Database>), ()> {
    let (returned_database, returned_client) = tokio::join!(
        Database::new(db_path, common_constants::DATABASE_NAME),
        initialize_liquid_tap(key, currency_pair)
    );

    let database = match returned_database {
        Ok(result) => result,
        Err(result) => {
            log::error!(
                "failed to initialize database\n-->\ndetails : {}\n<--",
                result
            );
            return Err(());
        }
    };

    let client = match returned_client {
        Ok(result) => result,
        Err(_) => return Err(()),
    };

    Ok((client, Arc::new(database)))
}

pub async fn initialize_liquid_tap(
    key: &LiquidApiKey,
    currency_pair: CurrencyPair,
) -> Result<LiquidTapClientAsync, ()> {
    let mut client = match LiquidTapClientAsync::connect().await {
        Ok(result) => result,
        Err(result) => {
            log::error!(
                "fatal error! cannnot connect liquid. \n-->\ndetails : {}\n<--",
                result
            );
            return Err(());
        }
    };

    if let Err(result) = client
        .subscribe(liquid_tap::channel_product(currency_pair))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    match client.authenticate(&key).await {
        Ok(()) => {
            log::info!("initialized LiquidTap : authentication");
        }
        Err(result) => {
            log::error!(
                "{}",
                error_message!("failed to authenticate! \n-->\ndetails : {}\n<--", result)
            );
            return Err(());
        }
    }

    match client
        .subscribe(liquid_tap::private_channel_orders("jpy"))
        .await
    {
        Ok(()) => {
            log::info!("initialized LiquidTap : sending subscribing signal succeeded.");
        }
        Err(result) => {
            log::error!(
                "{}",
                error_message!(
                    "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
                    result
                )
            );
            return Err(());
        }
    }

    match client
        .subscribe(liquid_tap::private_channel_trades("jpy"))
        .await
    {
        Ok(()) => {
            log::info!("initialized LiquidTap : sending subscribing signal succeeded.");
        }
        Err(result) => {
            log::error!(
                "{}",
                error_message!(
                    "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
                    result
                )
            );
            return Err(());
        }
    }

    match client
        .subscribe(liquid_tap::private_channel_executions(currency_pair))
        .await
    {
        Ok(()) => {
            log::info!("initialized LiquidTap : sending subscribing signal succeeded.");
        }
        Err(result) => {
            log::error!(
                "{}",
                error_message!(
                    "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
                    result
                )
            );
            return Err(());
        }
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_executions(currency_pair))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_order_book(currency_pair, Side::Buy))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_order_book(currency_pair, Side::Sell))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_executions_details(currency_pair))
        .await
    {
        log::error!(
            "failed to subscribe a channel!\n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    Ok(client)
}
