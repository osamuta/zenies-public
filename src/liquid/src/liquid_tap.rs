use super::data::DataGenerater;
use super::data_for_tap::*;
use super::*;

impl LiquidTapClientBlocking {
    pub fn new() -> Self {
        LiquidTapClientBlocking::connect().expect("failed to new the liquidTapClientBlocking")
    }

    pub fn connect() -> std::result::Result<LiquidTapClientBlocking, String> {
        let mut socket = match tungstenite::connect(url::Url::parse(LIQUID_TAP_URL).unwrap()) {
            Ok(result) => result.0,
            Err(result) => {
                return Err(error_message!(
                    "failed to connect the liquid tap!\ndetails : {:?}",
                    result
                ))
            }
        };

        let first_message = match socket.read_message() {
            Ok(msg) => msg.into_text().unwrap(),
            Err(msg) => {
                return Err(error_message!(
                    "failed to confirm connection to the liquid tap sever!\ndetails : {:?}",
                    msg
                ))
            }
        };

        let first_response: Response<String> = match serde_json::from_str(&first_message) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\ndetails : {:?}",
                    result
                ))
            }
        };

        if first_response.event != Some(String::from("pusher:connection_established")) {
            return Err(error_message!(
                "invalid connection!\ndetails : {:?}",
                first_response
            ));
        }

        let data: Connection = match serde_json::from_str(&first_response.data.unwrap()) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\ndetails : {:?}",
                    result
                ))
            }
        };

        Ok(LiquidTapClientBlocking {
            _socket: socket,
            _activity_timeout: data.activity_timeout,
            _socket_id: data.socket_id.parse().unwrap(),
        })
    }

    pub fn subscribe(&mut self, channel: String) -> std::result::Result<(), String> {
        match self.write_message(format!(
            r#"{{"event":"pusher:subscribe","data":{{"channel":"{}"}}}}"#,
            channel
        )) {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!("failed to sunscribe!\ndetails : {}", result)),
        }
    }

    pub fn read_message(&mut self) -> std::result::Result<String, String> {
        match self._socket.read_message() {
            Ok(result) => match result.into_text() {
                Ok(text) => Ok(text),
                Err(text) => Err(error_message!("not text!\ndetails : {:?}", text)),
            },
            Err(result) => Err(error_message!(
                "failed to read messages!\ndetails : {:?}",
                result
            )),
        }
    }

    pub fn write_message(&mut self, message: String) -> std::result::Result<(), String> {
        match self
            ._socket
            .write_message(tungstenite::Message::Text(message))
        {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to write a message!\ndetails : {}",
                result
            )),
        }
    }

    pub fn check(&mut self) -> std::result::Result<Response<serde_json::Value>, String> {
        let message = match self.read_message() {
            Ok(result) => result,
            Err(result) => return Err(error_message!("failed to check!\ndetails : {}", result)),
        };

        let response: Response<serde_json::Value> = match serde_json::from_str(&message) {
            Ok(result) => result,
            Err(result) => {
                let datetime = match chrono::DateTime::parse_from_rfc3339(&message) {
                    Ok(result_datetime) => result_datetime,
                    Err(result_datetime) => {
                        return Err(error_message!("failed to deserialize json!\nserde_json message : {:?}\njson : {}\nchrono : {:?}", result, message, result_datetime));
                    }
                };
                Response {
                    channel: Some(String::from("time-signal")),
                    data: Some(serde_json::to_value(&datetime).unwrap()),
                    event: Some(String::from("ginggone")),
                }
            }
        };

        Ok(response)
    }
}

impl Default for LiquidTapClientBlocking {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidTapClientAsync {
    pub async fn new() -> Self {
        LiquidTapClientAsync::connect()
            .await
            .expect("failed to new the liquidTapClientAsync")
    }

    pub async fn connect() -> std::result::Result<LiquidTapClientAsync, String> {
        let mut socket = match tokio_tungstenite::connect_async(
            url::Url::parse(LIQUID_TAP_URL).unwrap(),
        )
        .await
        {
            Ok(result) => result.0,
            Err(result) => {
                return Err(error_message!(
                    "failed to connect the liquid tap!\ndetails : {:?}",
                    result
                ))
            }
        };

        let first_message = match socket
            .next()
            .await
            .expect("failed to receive some messages!")
        {
            Ok(msg) => msg.into_text().unwrap(),
            Err(msg) => {
                return Err(error_message!(
                    "failed to confirm connection to the liquid tap sever!\ndetails : {:?}",
                    msg
                ))
            }
        };

        let first_response: Response<String> = match serde_json::from_str(&first_message) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\ndetails : {:?}",
                    result
                ))
            }
        };

        if first_response.event != Some(String::from("pusher:connection_established")) {
            return Err(error_message!(
                "invalid connection!\ndetails : {:?}",
                first_response
            ));
        }

        let data: Connection = match serde_json::from_str(&first_response.data.unwrap()) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\ndetails : {:?}",
                    result
                ))
            }
        };

        Ok(LiquidTapClientAsync {
            _socket: socket,
            _activity_timeout: data.activity_timeout,
            _socket_id: data.socket_id.parse().unwrap(),
        })
    }

    pub async fn subscribe(&mut self, channel: String) -> std::result::Result<(), String> {
        match self
            .write_message(format!(
                r#"{{"event":"pusher:subscribe","data":{{"channel":"{}"}}}}"#,
                channel
            ))
            .await
        {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!("failed to sunscribe!\ndetails : {}", result)),
        }
    }

    pub async fn read_message(&mut self) -> std::result::Result<String, String> {
        match self
            ._socket
            .next()
            .await
            .expect("failed to receive some messages!")
        {
            Ok(result) => match result.into_text() {
                Ok(text) => Ok(text),
                Err(text) => Err(error_message!("not text!\ndetails : {:?}", text)),
            },
            Err(result) => Err(error_message!(
                "failed to read messages!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn write_message(&mut self, message: String) -> std::result::Result<(), String> {
        match self._socket.send(tungstenite::Message::Text(message)).await {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to write a message!\ndetails : {}",
                result
            )),
        }
    }

    pub async fn check(&mut self) -> std::result::Result<Response<serde_json::Value>, String> {
        let message = match self.read_message().await {
            Ok(result) => result,
            Err(result) => return Err(error_message!("failed to check!\ndetails : {}", result)),
        };

        let response: Response<serde_json::Value> = match serde_json::from_str(&message) {
            Ok(result) => result,
            Err(result) => {
                let datetime = match chrono::DateTime::parse_from_rfc3339(&message) {
                    Ok(result_datetime) => result_datetime,
                    Err(result_datetime) => {
                        return Err(error_message!("failed to deserialize json!\nserde_json message : {:?}\njson : {}\nchrono : {:?}", result, message, result_datetime));
                    }
                };
                Response {
                    channel: Some(String::from("time-signal")),
                    data: Some(serde_json::to_value(&datetime).unwrap()),
                    event: Some(String::from("ginggone")),
                }
            }
        };

        Ok(response)
    }

    pub async fn authenticate(&mut self, key: &LiquidApiKey) -> std::result::Result<(), String> {
        let encoded = match authorizer(String::from("/realtime"), key.token_id, &key.secret_key) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to construct a payload!\ndetails : {}",
                    result
                ))
            }
        };

        let authentication = format!(
            r#"{{"event":"quoine:auth_request","data": {{ "path": "/realtime", "headers": {{ "X-Quoine-Auth": "{}" }} }} }}"#,
            encoded
        );

        match self.write_message(authentication).await {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to authenticate!\ndetails : {}",
                result
            )),
        }
    }
}

pub fn channel_product(pair: CurrencyPair) -> String {
    format!(
        "product_cash_{}_{}",
        pair.generate_pair_code(),
        pair.generate_id()
    )
}

pub fn channel_order_book(pair: CurrencyPair, side: Side) -> String {
    format!(
        "price_ladders_cash_{}_{}",
        pair.generate_pair_code(),
        side.generate_side_string()
    )
}

pub fn channel_executions(pair: CurrencyPair) -> String {
    format!("executions_cash_{}", pair.generate_pair_code())
}

pub fn channel_executions_details(pair: CurrencyPair) -> String {
    format!("execution_details_cash_{}", pair.generate_pair_code())
}

pub fn private_channel_order_book(pair: CurrencyPair, side: Side) -> String {
    format!(
        "price_ladders_{}_{}",
        pair.generate_pair_code(),
        side.generate_side_string()
    )
}

pub fn private_channel_orders(currency: &str) -> String {
    format!("user_account_{}_orders", currency)
}

pub fn private_channel_trades(currency: &str) -> String {
    format!("user_account_{}_trades", currency)
}

pub fn private_channel_executions(pair: CurrencyPair) -> String {
    format!("user_executions_cash_{}", pair.generate_pair_code())
}

pub fn private_channel_user_account(currency: &str) -> String {
    format!("user_account_{}", currency)
}

pub fn private_channel_trades_and_orders() -> String {
    format!("user")
}

pub fn generate_product(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<Product, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
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

pub fn generate_order_book(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<Vec<data::Order>, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<Vec<Vec<String>>> {
        Ok(result) => {
            let mut temp = Vec::new();
            for i in result {
                temp.push(data::Order {
                    price: i[0].parse::<f64>().expect("failed to parse") as i32,
                    amount: i[1].parse().unwrap(),
                });
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

pub fn generate_execution(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<data::Execution, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<data::ExecutionReceiver> {
        Ok(result) => Ok(data::Execution::generate_from_receiver(result)),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}

pub fn generate_execution_details(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<data_for_tap::ExecutionDetails, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data)
        as serde_json::Result<data_for_tap::ExecutionDetailsReceiver>
    {
        Ok(result) => Ok(data_for_tap::ExecutionDetails::generate_from_receiver(
            result,
        )),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}

pub fn generate_order_statuts(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<data_for_tap::OrderStatus, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<data_for_tap::OrderStatus> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}

pub fn generate_trades_panel_update(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<data_for_tap::TradesPanelUpdate, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<data_for_tap::TradesPanelUpdate> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}

pub fn generate_trades_update(
    res_data: &Option<serde_json::Value>,
) -> std::result::Result<data_for_tap::TradesUpdate, String> {
    let json_data = match res_data {
        Some(data) => {
            if data.is_string() {
                String::from(data.as_str().unwrap())
            } else {
                return Err(error_message!("not string!\ndetails : {:?}", data));
            }
        }
        None => return Err(error_message!("no data!\ndetails : {:?}", res_data)),
    };
    match serde_json::from_str(&json_data) as serde_json::Result<data_for_tap::TradesUpdate> {
        Ok(result) => Ok(result),
        Err(result) => Err(error_message!(
            "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
            result,
            json_data
        )),
    }
}
