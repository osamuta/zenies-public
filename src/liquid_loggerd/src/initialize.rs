use super::*;

pub async fn initialize(
    db_path: &str,
) -> Result<(std::sync::Arc<database::Database>, LiquidTapClientAsync), ()> {
    let (returned_database, returned_client) = tokio::join!(
        Database::new(db_path, common_constants::DATABASE_NAME),
        initialize_liquid_tap()
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
    let database = std::sync::Arc::new(database);

    let client = match returned_client {
        Ok(result) => result,
        Err(_) => return Err(()),
    };

    Ok((database, client))
}

pub async fn initialize_liquid_tap() -> Result<LiquidTapClientAsync, ()> {
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
        .subscribe(liquid_tap::channel_product(CurrencyPair::BtcJpy))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_executions(CurrencyPair::BtcJpy))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_order_book(
            CurrencyPair::BtcJpy,
            Side::Buy,
        ))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_order_book(
            CurrencyPair::BtcJpy,
            Side::Sell,
        ))
        .await
    {
        log::error!(
            "failed to subscribe a channel! \n-->\ndetails : {}\n<--",
            result
        );
        return Err(());
    }

    if let Err(result) = client
        .subscribe(liquid_tap::channel_executions_details(CurrencyPair::BtcJpy))
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
