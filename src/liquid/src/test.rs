#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test() {
        let msg = format!("hello\nworld");
        println!("{}", msg);
    }

    #[test]
    fn check_runtime() {
        let runtime = tokio::runtime::Runtime::new().expect("Faild to initialize runtime");
        let task = async {
            let client = reqwest::Client::new();
            let result = client.get("https://api.liquid.com/products/5").send().await;
            println!("{}", result.unwrap().text().await.unwrap());
        };

        runtime.block_on(task);
    }

    //
    // https async liquid
    #[tokio::test]
    async fn check_get_product() {
        let mut client = LiquidClientAsync::new();
        let result = client
            .get_product(currency::CurrencyPair::BtcJpy)
            .await
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn check_perpatual_product() {
        let mut client = LiquidClientAsync::new();
        let result = client
            .get_perpetual_product(currency::CurrencyPair::BtcJpy)
            .await
            .expect("Failed to get a perpetual product.");
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn check_order_book() {
        let mut client = LiquidClientAsync::new();
        let result = client
            .get_order_book(currency::CurrencyPair::BtcJpy, true)
            .await
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn check_get_executions() {
        let mut client = LiquidClientAsync::new();
        let result = client
            .get_executions(currency::CurrencyPair::BtcJpy, 1)
            .await
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn check_rate_limit() {
        let mut client = LiquidClientAsync::new();
        let result = client
            .get_product(currency::CurrencyPair::BtcJpy)
            .await
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    //
    // https blocking liquid
    #[test]
    fn check_get_product_blocking() {
        let mut client = LiquidClientBlocking::new();
        let result = client
            .get_product(currency::CurrencyPair::BtcJpy)
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[test]
    fn check_perpatual_product_blocking() {
        let mut client = LiquidClientBlocking::new();
        let result = client
            .get_perpetual_product(currency::CurrencyPair::BtcJpy)
            .expect("Failed to get a perpetual product.");
        println!("{:?}", result);
    }

    #[test]
    fn check_order_book_blocking() {
        let mut client = LiquidClientBlocking::new();
        let result = client
            .get_order_book(currency::CurrencyPair::BtcJpy, true)
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[test]
    fn check_get_executions_blocking() {
        let mut client = LiquidClientBlocking::new();
        let result = client
            .get_executions(currency::CurrencyPair::BtcJpy, 1)
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[test]
    fn check_rate_limit_blocking() {
        let mut client = LiquidClientBlocking::new();
        let result = client
            .get_product(currency::CurrencyPair::BtcJpy)
            .expect("Failed to get a product.");
        println!("{:?}", result);
    }

    #[test]
    fn check_get_my_executions() {
        let key = LiquidApiKey {
            token_id: 0,
            secret_key: String::from(""),
        };

        let mut client = LiquidClientBlocking::new();
        let result = client.get_my_executions(&key, currency::CurrencyPair::BtcJpy, 1, 1000);
        println!("{:?}", result);
    }

    //
    // liquid tap
    #[test]
    fn check_connect_and_subscribe() {
        let mut client = LiquidTapClientBlocking::connect().unwrap();
        match client.subscribe(liquid_tap::channel_product(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client.subscribe(liquid_tap::channel_order_book(
            CurrencyPair::BtcJpy,
            Side::Buy,
        )) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client.subscribe(liquid_tap::channel_executions(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client.subscribe(liquid_tap::channel_executions_details(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
    }

    #[test]
    fn check_check() {
        let mut client = LiquidTapClientBlocking::connect().unwrap();
        match client.subscribe(String::from("product_cash_btcjpy_5")) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }

        match client.check() {
            Ok(result) => println!("{:?}", result),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn check_get_product_on_tap() {
        let mut client = LiquidTapClientBlocking::connect().expect("should be connected");
        match client.subscribe(liquid_tap::channel_product(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        let msg = match client.check() {
            Ok(result) => result,
            Err(_) => panic!(),
        };
        let result = liquid_tap::generate_product(&msg.data).unwrap();
        println!("Received: {:?}", result);
    }

    #[test]
    fn check_get_order_book_on_tap() {
        let mut client = LiquidTapClientBlocking::connect().expect("should be connected");
        match client.subscribe(liquid_tap::channel_order_book(
            CurrencyPair::BtcJpy,
            Side::Buy,
        )) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        let msg = match client.check() {
            Ok(result) => result,
            Err(_) => panic!(),
        };
        let result = liquid_tap::generate_order_book(&msg.data).unwrap();
        println!("Received: {:?}", result);
    }

    #[test]
    fn check_generate_execution_on_tap() {
        let mut client = LiquidTapClientBlocking::connect().expect("should be connected");
        match client.subscribe(liquid_tap::channel_executions(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        let msg = match client.check() {
            Ok(result) => result,
            Err(_) => panic!(),
        };
        let result = liquid_tap::generate_execution(&msg.data).unwrap();
        println!("Received: {:?}", result);
    }

    #[test]
    fn check_generate_execution_details_on_tap() {
        let mut client = LiquidTapClientBlocking::connect().expect("should be connected");
        match client.subscribe(liquid_tap::channel_executions_details(CurrencyPair::BtcJpy)) {
            Ok(()) => println!("{}", client.read_message().expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        let msg = match client.check() {
            Ok(result) => result,
            Err(_) => panic!(),
        };
        let result = liquid_tap::generate_execution_details(&msg.data).unwrap();
        println!("Received: {:?}", result);
    }

    //
    // liquid tap async
    #[tokio::test]
    async fn check_connect_subscribe_async() {
        let mut client = LiquidTapClientAsync::connect().await.unwrap();
        match client
            .subscribe(liquid_tap::channel_product(CurrencyPair::BtcJpy))
            .await
        {
            Ok(()) => println!("{}", client.read_message().await.expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client
            .subscribe(liquid_tap::channel_order_book(
                CurrencyPair::BtcJpy,
                Side::Buy,
            ))
            .await
        {
            Ok(()) => println!("{}", client.read_message().await.expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client
            .subscribe(liquid_tap::channel_executions(CurrencyPair::BtcJpy))
            .await
        {
            Ok(()) => println!("{}", client.read_message().await.expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
        match client
            .subscribe(liquid_tap::channel_executions_details(CurrencyPair::BtcJpy))
            .await
        {
            Ok(()) => println!("{}", client.read_message().await.expect("should be read")),
            Err(result) => assert!(false, "{}", result),
        }
    }

    //
    // liquid order check
    #[tokio::test]
    async fn check_post_order() {
        let key = LiquidApiKey {
            token_id: 2613257,
            secret_key: String::from("JVH9qnnNNCinkgvIFu6U18EqTt1tJzVTnfZyfg/c6aOWwPADIGwFCw+AjIgH1AU4FJdPOUf/VBGAGbxxk6KNwg==")
        };
        let client = LiquidClientAsync::new();
        assert_eq!(
            client
                .post_order(
                    &key,
                    &Order::limit(5, Side::Buy, 0.0001, 5_000_000).with_leverage_options(
                        2,
                        OrderDirection::NetOut,
                        None,
                        None
                    )
                )
                .await
                .is_ok(),
            true
        );
    }
}
