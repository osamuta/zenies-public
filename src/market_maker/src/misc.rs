use fs2::FileExt;
use serde::{Deserialize, Serialize};

use common::*;
use liquid::LiquidApiKey;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub identifier: String,
    pub quantity: f64,
    pub currency_pair: liquid::currency::CurrencyPair,
    pub evaluation_time: f64,
    pub limited_time: f64,
    pub offset_unit: i32,
    pub dry_trade: bool,
    pub key: LiquidApiKey,
}

impl Config {
    pub async fn load(path: &str) -> Result<Config, ()> {
        let content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(result) => {
                log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
                return Err(());
            }
        };
        let config = match toml::from_str::<Config>(&content) {
            Ok(content) => content,
            Err(result) => {
                log::error!(
                    "failed to deserializes config file!\n-->\ndetails : {}\n<--",
                    result
                );
                return Err(());
            }
        };
        Ok(config)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub offset: i32,
    pub succeeded_trade: usize,
    pub whole_trade: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: 0,
            succeeded_trade: 1,
            whole_trade: 1,
        }
    }
}

impl State {
    pub async fn load(path: &str) -> Result<State, ()> {
        let state = match tokio::fs::read_to_string(path).await {
            Ok(content) => match toml::from_str::<State>(&content) {
                Ok(content) => content,
                Err(result) => {
                    log::error!(
                        "failed to deserializes config file!\n-->\ndetails : {}\n<--",
                        result
                    );
                    return Err(());
                }
            },
            Err(result) => {
                log::warn!("failed to load!\n-->\ndetails : {}\n<--", result);
                log::warn!("loaded default!");
                State::default()
            }
        };
        if state.succeeded_trade < 1 && state.whole_trade < 1 {
            log::error!("invalid values!\n-->\ndetails : {:?}\n<--", state);
            return Err(());
        }
        Ok(state)
    }

    pub async fn save(
        &mut self,
        path: &str,
        offset: i32,
        succeeded_trade: usize,
        whole_trade: usize,
    ) -> Result<(), ()> {
        log::info!("Saved states");
        self.offset = offset;
        self.succeeded_trade = succeeded_trade;
        self.whole_trade = whole_trade;
        let string = match toml::to_string(self) {
            Ok(content) => content,
            Err(result) => {
                log::error!(
                    "failed to serializes config file!\n-->\ndetails : {}\n<--",
                    result
                );
                return Err(());
            }
        };
        match tokio::fs::write(path, string.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(result) => {
                log::error!("failed to save!\n-->\ndetails : {}\n<--", result);
                Err(())
            }
        }
    }
}

pub fn lock_file(etc_dir: &str, identifier: &str) -> Result<(), String> {
    let lock_file = match std::fs::File::create(
        String::from(etc_dir)
            + env!("CARGO_PKG_NAME")
            + "_"
            + identifier
            + common_constants::LOCK_EXTENSION,
    ) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to open a lock file!\ndetails : {:?}",
                result
            ));
        }
    };
    if let Err(result) = lock_file.try_lock_exclusive() {
        return Err(error_message!(
            "failed to lock a lock file!\ndetails :{:?}",
            result
        ));
    }
    Ok(())
}
