use super::constants::*;
use super::data::*;
use super::url_gen::*;
use super::*;

impl LiquidClientAsync {
    pub fn new() -> Self {
        Self {
            _client: reqwest::Client::new(),
            //_last_called: std::time::Instant::now(),
        }
    }

    //pub fn check_rate_limit(
    //    &self,
    //) -> std::result::Result<std::time::Duration, std::time::Duration> {
    //    let elapsed = self._last_called.elapsed();
    //    if elapsed > std::time::Duration::from_secs(1) {
    //        Ok(elapsed)
    //    } else {
    //        Err(elapsed)
    //    }
    //}

    //pub fn request_now(&mut self) {
    //    self._last_called = std::time::Instant::now();
    //}

    pub async fn get_product(
        &mut self,
        pair: CurrencyPair,
    ) -> std::result::Result<Product, String> {
        //self.request_now();

        let response = match self
            ._client
            .get(&generate_get_product_url(pair))
            .send()
            .await
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<ProductReceiver> {
            Ok(result) => Ok(Product::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn get_perpetual_product(
        &mut self,
        pair: CurrencyPair,
    ) -> std::result::Result<PerpetualProduct, String> {
        //self.request_now();

        let product_id = match pair {
            CurrencyPair::BtcJpy => 603,
            CurrencyPair::BtcUsd => 604,
            _ => {
                return Err(error_message!(
                    "invalid a argument! It is niether BtcJpy or BtcUsd\ndetails : {:?}",
                    pair
                ))
            }
        };

        let response = match self
            ._client
            .get(&generate_get_product_url(CurrencyPair::Custom(product_id)))
            .send()
            .await
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<PerpetualProductReceiver> {
            Ok(result) => Ok(PerpetualProduct::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn get_order_book(
        &mut self,
        pair: CurrencyPair,
        isfull: bool,
    ) -> std::result::Result<OrderBook, String> {
        //self.request_now();

        let response = match self
            ._client
            .get(&generate_get_order_book_url(pair))
            .query(&[("full", if isfull { "1" } else { "0" })])
            .send()
            .await
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<OrderBookReceiver> {
            Ok(result) => Ok(OrderBook::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn get_executions(
        &mut self,
        pair: CurrencyPair,
        items: u32,
    ) -> std::result::Result<Vec<Execution>, String> {
        //self.request_now();

        let response = match self
            ._client
            .get(GET_EXECUTIONS)
            .query(&[
                ("product_id", pair.generate_id().to_string()),
                ("limit", items.to_string()),
            ])
            .send()
            .await
        {
            Ok(result) => result,
            Err(result) => return Err(format!("cannnot get a data\ndetails : {:?}", result)),
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<Pagination<ExecutionReceiver>>
        {
            Ok(result) => {
                let mut generated = Vec::new();
                for i in result.models {
                    generated.push(Execution::generate_from_receiver(i));
                }
                Ok(generated)
            }
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                    result,
                    json_data
                ));
            }
        }
    }
}

impl Default for LiquidClientAsync {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidClientBlocking {
    pub fn new() -> Self {
        Self {
            _client: reqwest::blocking::Client::new(),
            //_last_called: std::time::Instant::now(),
        }
    }

    //pub fn check_rate_limit(
    //    &self,
    //) -> std::result::Result<std::time::Duration, std::time::Duration> {
    //    let elapsed = self._last_called.elapsed();
    //    if elapsed > std::time::Duration::from_secs(1) {
    //        Ok(elapsed)
    //    } else {
    //        Err(elapsed)
    //    }
    //}

    //pub fn request_now(&mut self) {
    //    self._last_called = std::time::Instant::now();
    //}

    pub fn get_product(&mut self, pair: CurrencyPair) -> std::result::Result<Product, String> {
        //self.request_now();

        let response = match self._client.get(&generate_get_product_url(pair)).send() {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "cannnot get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<ProductReceiver> {
            Ok(result) => Ok(Product::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub fn get_perpetual_product(
        &mut self,
        pair: CurrencyPair,
    ) -> std::result::Result<PerpetualProduct, String> {
        //self.request_now();

        let product_id = match pair {
            CurrencyPair::BtcJpy => 603,
            CurrencyPair::BtcUsd => 604,
            _ => {
                return Err(error_message!(
                    "invalid a argument! It is niether BtcJpy or BtcUsd\ndetails : {:?}",
                    pair
                ))
            }
        };

        let response = match self
            ._client
            .get(&generate_get_product_url(CurrencyPair::Custom(product_id)))
            .send()
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<PerpetualProductReceiver> {
            Ok(result) => Ok(PerpetualProduct::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub fn get_order_book(
        &mut self,
        pair: CurrencyPair,
        isfull: bool,
    ) -> std::result::Result<OrderBook, String> {
        //self.request_now();

        let response = match self
            ._client
            .get(&generate_get_order_book_url(pair))
            .query(&[("full", if isfull { "1" } else { "0" })])
            .send()
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ))
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<OrderBookReceiver> {
            Ok(result) => Ok(OrderBook::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub fn get_executions(
        &mut self,
        pair: CurrencyPair,
        items: u32,
    ) -> std::result::Result<Vec<Execution>, String> {
        //self.request_now();

        let response = match self
            ._client
            .get(GET_EXECUTIONS)
            .query(&[
                ("product_id", pair.generate_id().to_string()),
                ("limit", items.to_string()),
            ])
            .send()
        {
            Ok(result) => result,
            Err(result) => return Err(format!("failed to get a data\ndetails : {:?}", result)),
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {:?}",
                response
            ));
        }

        let json_data = match response.text() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<Pagination<ExecutionReceiver>>
        {
            Ok(result) => {
                let mut generated = Vec::new();
                for i in result.models {
                    generated.push(Execution::generate_from_receiver(i));
                }
                Ok(generated)
            }
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }
}

impl Default for LiquidClientBlocking {
    fn default() -> Self {
        Self::new()
    }
}
