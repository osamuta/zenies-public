use super::*;

pub async fn initialize(
    key: &LiquidApiKey, /*db_path: &str*/
) -> Result<(LiquidTapClientAsync, LiquidClientAsync), ()> {
    //let (returned_database, returned_client) = tokio::join!(
    //    Database::new(db_path, common_constants::DATABASE_NAME),
    //    initialize_liquid_tap()
    //);

    //let database = match returned_database {
    //    Ok(result) => result,
    //    Err(result) => {
    //        log::error!(
    //            "failed to initialize database\n-->\ndetails : {}\n<--",
    //            result
    //        );
    //        return Err(());
    //    }
    //};
    //let database = std::rc::Rc::new(database);

    let client = match initialize_liquid_tap(key).await {
        Ok(result) => result,
        Err(_) => return Err(()),
    };

    Ok((client, LiquidClientAsync::new()))
}

pub async fn initialize_liquid_tap(key: &LiquidApiKey) -> Result<LiquidTapClientAsync, ()> {
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
        .subscribe(liquid_tap::private_channel_executions(CurrencyPair::BtcJpy))
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
        .subscribe(liquid_tap::private_channel_user_account("jpy"))
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
        .subscribe(liquid_tap::private_channel_trades_and_orders())
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

    //if let Err(result) = client
    //    .subscribe(liquid_tap::channel_executions(CurrencyPair::BtcJpy))
    //    .await
    //{
    //    log::error!(
    //        "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
    //        result
    //    );
    //    return Err(());
    //}

    //if let Err(result) = client
    //    .subscribe(liquid_tap::channel_order_book(
    //        CurrencyPair::BtcJpy,
    //        Side::Buy,
    //    ))
    //    .await
    //{
    //    log::error!(
    //        "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
    //        result
    //    );
    //    return Err(());
    //}

    //if let Err(result) = client
    //    .subscribe(liquid_tap::channel_order_book(
    //        CurrencyPair::BtcJpy,
    //        Side::Sell,
    //    ))
    //    .await
    //{
    //    log::error!(
    //        "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
    //        result
    //    );
    //    return Err(());
    //}

    //if let Err(result) = client
    //    .subscribe(liquid_tap::channel_executions_details(CurrencyPair::BtcJpy))
    //    .await
    //{
    //    log::error!(
    //        "failed to subscribe a channel!\n-->\ndetails : {}\n<--",
    //        result
    //    );
    //    return Err(());
    //}

    Ok(client)
}
