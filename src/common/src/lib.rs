use serde::{Deserialize, Serialize};
use std::result::Result;

pub mod common_constants {
    ///
    /// defined envoirnmental variables name
    pub const ENV_NAME_ZENIES_DIR: &str = "ZENIES_DIR";
    pub const ENV_NAME_ZENIES_BIN_DIR: &str = "ZENIES_BIN_DIR";
    pub const ENV_NAME_ZENIES_DATABASE_PATH: &str = "ZENIES_DATABASE_PATH";
    pub const ENV_NAME_ZENIES_LOG_DIR: &str = "ZENIES_LOG_DIR";
    pub const ENV_NAME_ZENIES_ETC_DIR: &str = "ZENIES_ETC_DIR";
    pub const ENV_NAME_ZENIES_KEY_BOX_PATH: &str = "ZENIES_KEY_BOX_PATH";

    pub const LOG_EXTENSION: &str = ".log";
    pub const SOCKET_EXTENSION: &str = ".sock";
    pub const LOCK_EXTENSION: &str = ".lock";
    pub const STDOUT_EXTENSION: &str = ".stdout";
    pub const STDERR_EXTENSION: &str = ".stderr";
    pub const PID_EXTENSION: &str = ".pid";

    ///
    /// defined commands definition
    pub const DEFINED_COMMAND_PING: &str = "ping";
    pub const DEFINED_COMMAND_GREET: &str = "greet";
    pub const DEFINED_COMMAND_SHUTDOWN: &str = "shutdown";
    pub const DEFINED_COMMAND_RESPONSE: &str = "response";
    pub const DEFINED_COMMAND_GET_STATUS: &str = "status";

    ///
    /// defined commands option definition
    pub const DEFINED_COMMAND_OPTION_SUCCESS: &str = "success";
    pub const DEFINED_COMMAND_OPTION_ERROR: &str = "error";

    ///
    /// database name
    pub const DATABASE_NAME: &str = "BtcJpy_In_Liquid";

    ///
    /// database collection name
    pub const DATABASE_COLLECTION_TICKER: &str = "ticker";
    pub const DATABASE_COLLECTION_EXECUTIONS: &str = "executions";
    pub const DATABASE_COLLECTION_ORDER_BOOK_BUY: &str = "order_book_buy";
    pub const DATABASE_COLLECTION_ORDER_BOOK_SELL: &str = "order_book_sell";
}

#[macro_export]
macro_rules! error_message_colored {
    ($($arg:tt)*) => {{
        let res: String = format!("[{}] {}, line:{}, column:{}, ", console::style(" ERROR ").red(), file!(), line!(), column!()) + &std::fmt::format(std::format_args!($($arg)*));
        res
    }}
}

#[macro_export]
macro_rules! ok_message_colored {
    ($($arg:tt)*) => {{
        let res: String = format!("[{}] {}, line:{}, column:{}, ", console::style(" OK    ").green(), file!(), line!(), column!()) + &std::fmt::format(std::format_args!($($arg)*));
        res
    }}
}

#[macro_export]
macro_rules! info_message_colored {
    ($($arg:tt)*) => {{
        let res: String = format!("[{}] {}, line:{}, column:{}, ", console::style(" INFO  ").blue(), file!(), line!(), column!()) + &std::fmt::format(std::format_args!($($arg)*));
        res
    }}
}

#[macro_export]
macro_rules! error_message {
    ($($arg:tt)*) => {{
        let res: String = format!("{}, line:{}, column:{}, ", file!(), line!(), column!()) + &std::fmt::format(std::format_args!($($arg)*));
        res
    }}
}

#[macro_export]
macro_rules! concat_vectors {
    ($($x:expr),*) => {
        {
            let mut buffer = Vec::new();
            $(
                buffer.append(&mut $x);
            )*
            buffer
        }
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enviornment {
    pub general: General,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct General {
    pub working_directory_path: String,
    pub bin_directory_path: String,
    pub etc_directory_path: String,
    pub log_directory_path: String,
    pub database_url: String,
}

pub async fn load_env(path: &str) -> Result<Enviornment, String> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "failed to load environment file!\ndetails : {:?}",
                result
            ));
        }
    };
    let config = match toml::from_str::<Enviornment>(&content) {
        Ok(content) => content,
        Err(result) => {
            return Err(error_message!(
                "faild deserialize toml file!\ndetails : {:?}",
                result
            ));
        }
    };
    Ok(config)
}

#[macro_export]
macro_rules! breakable_block {
    ($block:block) => {
        loop { $block break; }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!(
            "{:?}",
            error_message!(
                "cannot deserialize json!\nserde_json message : {:?}\njson : {}",
                "result",
                "json_data"
            )
        );
        println!("{:?}", concat_vectors!(vec![1, 2, 3], vec![4, 5, 6]));
    }
}
