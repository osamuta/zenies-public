use super::constants::*;
use super::data;
use super::data::DataGenerater;
use super::*;
use std::result::Result;

impl Order {
    pub fn new() -> Self {
        Order {
            order_type: String::new(),
            product_id: 0,
            side: String::new(),
            quantity: 0.0,
            price: None,
            price_range: None,
            trailing_stop_type: None,
            trailing_stop_value: None,
            client_order_id: Some(String::from(
                uuid::Uuid::new_v4()
                    .to_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer()),
            )),
            leverage_level: Some(1),
            order_direction: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn limit(product_id: i32, side: Side, quantity: f64, price: i32) -> Self {
        Order {
            order_type: String::from("limit"),
            product_id,
            side: String::from(side.generate_side_string()),
            quantity,
            price: Some(price),
            price_range: None,
            trailing_stop_type: None,
            trailing_stop_value: None,
            client_order_id: Some(String::from(
                uuid::Uuid::new_v4()
                    .to_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer()),
            )),
            leverage_level: Some(1),
            order_direction: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn market(product_id: i32, side: Side, quantity: f64, price_range: Option<i32>) -> Self {
        Order {
            order_type: String::from("market"),
            product_id,
            side: String::from(side.generate_side_string()),
            quantity,
            price: None,
            price_range,
            trailing_stop_type: None,
            trailing_stop_value: None,
            client_order_id: Some(String::from(
                uuid::Uuid::new_v4()
                    .to_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer()),
            )),
            leverage_level: Some(1),
            order_direction: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn stop(product_id: i32, side: Side, quantity: f64, price: i32) -> Self {
        Order {
            order_type: String::from("stop"),
            product_id,
            side: String::from(side.generate_side_string()),
            quantity,
            price: Some(price),
            price_range: None,
            trailing_stop_type: None,
            trailing_stop_value: None,
            client_order_id: Some(String::from(
                uuid::Uuid::new_v4()
                    .to_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer()),
            )),
            leverage_level: Some(1),
            order_direction: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn trailing(
        product_id: i32,
        side: Side,
        quantity: f64,
        trailing_stop_type: TrailType,
        trailing_stop_value: f64,
    ) -> Self {
        Order {
            order_type: String::from("trailing_stop"),
            product_id,
            side: String::from(side.generate_side_string()),
            quantity,
            price: None,
            price_range: None,
            trailing_stop_type: match trailing_stop_type {
                TrailType::Fiat => Some(String::from("fiat")),
                TrailType::Percentage => Some(String::from("percentage")),
            },
            trailing_stop_value: Some(trailing_stop_value),
            client_order_id: Some(String::from(
                uuid::Uuid::new_v4()
                    .to_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer()),
            )),
            leverage_level: Some(1),
            order_direction: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn with_leverage_options(
        self,
        leverage_level: i32,
        order_direction: OrderDirection,
        take_profit: Option<i32>,
        stop_loss: Option<i32>,
    ) -> Self {
        Self {
            leverage_level: Some(leverage_level),
            order_direction: Some(match order_direction {
                OrderDirection::OneDirection => String::from("one_direction"),
                OrderDirection::TwoDirection => String::from("two_direction"),
                OrderDirection::NetOut => String::from("netout"),
            }),
            take_profit,
            stop_loss,
            ..self
        }
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidClientBlocking {
    pub fn get_my_executions(
        &mut self,
        key: &LiquidApiKey,
        pair: CurrencyPair,
        page: u32,
        max_pages: u32,
    ) -> std::result::Result<data::Pagination<data::MyExecution>, String> {
        //self.request_now();

        let encoded: String = match authorizer(
            format!(
                "/executions/me?product_id={}&page={}&limit={}",
                pair.generate_id(),
                page,
                max_pages
            ),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .get(GET_PRIVATE_EXECUTIONS)
            .query(&[
                ("product_id", pair.generate_id().to_string()),
                ("page", page.to_string()),
                ("limit", max_pages.to_string()),
            ])
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().unwrap()
            ));
        }

        let json_data = match response.text() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data)
            as serde_json::Result<data::Pagination<data::MyExecutionReceiver>>
        {
            Ok(result) => {
                let mut temp = data::Pagination {
                    models: Vec::new(),
                    current_page: result.current_page,
                    total_pages: result.total_pages,
                };
                for i in result.models {
                    temp.models
                        .push(data::MyExecution::generate_from_receiver(i));
                }
                Ok(temp)
            }
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }
}

impl LiquidClientAsync {
    pub async fn get_my_executions(
        &mut self,
        key: &LiquidApiKey,
        pair: CurrencyPair,
        page: u32,
        max_pages: u32,
    ) -> std::result::Result<data::Pagination<data::MyExecution>, String> {
        //self.request_now();

        let encoded: String = match authorizer(
            format!(
                "/executions/me?product_id={}&page={}&limit={}",
                pair.generate_id(),
                page,
                max_pages
            ),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .get(GET_PRIVATE_EXECUTIONS)
            .query(&[
                ("product_id", pair.generate_id().to_string()),
                ("page", page.to_string()),
                ("limit", max_pages.to_string()),
            ])
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data)
            as serde_json::Result<data::Pagination<data::MyExecutionReceiver>>
        {
            Ok(result) => {
                let mut temp = data::Pagination {
                    models: Vec::new(),
                    current_page: result.current_page,
                    total_pages: result.total_pages,
                };
                for i in result.models {
                    temp.models
                        .push(data::MyExecution::generate_from_receiver(i));
                }
                Ok(temp)
            }
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn post_order(
        &self,
        key: &LiquidApiKey,
        order: &Order,
    ) -> Result<data::PostOrderResponse, String> {
        //self.request_now();

        let encoded: String = match authorizer_without_nonce(
            String::from("/orders"),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .post("https://api.liquid.com/orders")
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
            .json(order)
            .send()
            .await
        {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to get a data!\ndetails : {:?}",
                    result
                ));
            }
        };

        if response.status().is_client_error() {
            return Err(error_message!(
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data)
            as serde_json::Result<data::PostOrderResponseReceiver>
        {
            Ok(result) => Ok(data::PostOrderResponse::generate_from_receiver(result)),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn is_order_excuted(&self, key: &LiquidApiKey, id: i64) -> Result<bool, String> {
        //self.request_now();

        let encoded: String = match authorizer(
            String::from("/orders?status=filled"),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .get("https://api.liquid.com/orders")
            .query(&[("status", "filled")])
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        let json_data = match response.text().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("not text!\ndetails : {:?}", result)),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<serde_json::Value> {
            Ok(result) => {
                for data in result["models"]
                    .as_array()
                    .expect("failed to convert to array!")
                {
                    if data["id"].as_i64().expect("failed to get a id!") == id {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn cancel_order(&self, key: &LiquidApiKey, id: u64) -> Result<(), String> {
        //self.request_now();

        let encoded: String = match authorizer(
            format!("/orders/{}/cancel", id),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .put(format!("https://api.liquid.com/orders/{}/cancel", id))
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        Ok(())
    }

    pub async fn update_position(
        &self,
        key: &LiquidApiKey,
        trade_id: u64,
        take_profit: i32,
        stop_loss: i32,
    ) -> Result<(), String> {
        //self.request_now();

        #[derive(Serialize)]
        struct Query {
            take_profit: i32,
            stop_loss: i32,
        }

        let encoded: String = match authorizer(
            format!("/trades/{}", trade_id),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .put(format!("https://api.liquid.com/trades/{}", trade_id))
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
            .json(&Query {
                take_profit,
                stop_loss,
            })
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        Ok(())
    }

    pub async fn close_positon(
        &self,
        key: &LiquidApiKey,
        trade_id: u64,
        quantity: f64,
    ) -> Result<(), String> {
        //self.request_now();

        let encoded: String = match authorizer(
            format!("/trades/{}/close?closed_quantity={}", trade_id, quantity),
            key.token_id,
            &key.secret_key,
        ) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let response = match self
            ._client
            .put(format!("https://api.liquid.com/trades/{}/close", trade_id))
            .query(&[("closed_quantity", quantity.to_string())])
            .header("X-Quoine-API-Version", "2")
            .header("X-Quoine-Auth", encoded)
            .header("Content-Type", "application/json")
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
                "status code was not 200 OK!\ndetails : {}",
                response.text().await.expect("failed to get texts!")
            ));
        }

        Ok(())
    }
}
