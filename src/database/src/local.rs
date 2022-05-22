use super::*;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalTicker {
    pub collection: String,
    pub downloaded_at: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub size: i32,
    pub ticker: Vec<data::Ticker>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExecutions {
    pub collection: String,
    pub downloaded_at: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub size: i32,
    pub executions: Vec<data::Execution>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalOrderBook {
    pub collection: String,
    pub downloaded_at: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub size: i32,
    pub order_book: Vec<data::OrderBook>,
}

impl LocalTicker {
    pub fn new() -> Self {
        Self {
            collection: String::from("none"),
            downloaded_at: Utc::now(),
            start: Utc::now(),
            end: Utc::now(),
            size: 0,
            ticker: Vec::new(),
        }
    }

    pub async fn read(path: &str) -> Result<Self, String> {
        let path_copied = String::from(path);
        tokio::task::spawn_blocking(move || {
            let reader = match std::fs::File::open(path_copied) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            let data = match serde_json::from_reader(reader) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            Ok(data)
        })
        .await
        .expect("failed to run blocking task!")
    }

    pub async fn write(&self, path: &str) -> Result<(), String> {
        let path_copied = String::from(path);
        let task = tokio::spawn(File::create(path_copied));
        let json = match serde_json::to_string(self) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to serialize!\ndetails : {:?}",
                    result
                ))
            }
        };
        let mut file = match task.await.expect("failed to excute a task!") {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!("failed to create!\ndetails : {:?}", result));
            }
        };
        if let Err(result) = file.write_all(json.as_bytes()).await {
            return Err(error_message!("failed to write!\ndetails : {:?}", result));
        }
        Ok(())
    }
}

impl Default for local::LocalTicker {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalExecutions {
    pub fn new() -> Self {
        Self {
            collection: String::from("none"),
            downloaded_at: Utc::now(),
            start: Utc::now(),
            end: Utc::now(),
            size: 0,
            executions: Vec::new(),
        }
    }

    pub async fn read(path: &str) -> Result<Self, String> {
        let path_copied = String::from(path);
        tokio::task::spawn_blocking(move || {
            let reader = match std::fs::File::open(path_copied) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            let data = match serde_json::from_reader(reader) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            Ok(data)
        })
        .await
        .expect("failed to run blocking task!")
    }

    pub async fn write(&self, path: &str) -> Result<(), String> {
        let path_copied = String::from(path);
        let task = tokio::spawn(File::create(path_copied));
        let json = match serde_json::to_string(self) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to serialize!\ndetails : {:?}",
                    result
                ))
            }
        };
        let mut file = match task.await.expect("failed to excute a task!") {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!("failed to create!\ndetails : {:?}", result));
            }
        };
        if let Err(result) = file.write_all(json.as_bytes()).await {
            return Err(error_message!("failed to write!\ndetails : {:?}", result));
        }
        Ok(())
    }
}

impl Default for local::LocalExecutions {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalOrderBook {
    pub fn new() -> Self {
        Self {
            collection: String::from("none"),
            downloaded_at: Utc::now(),
            start: Utc::now(),
            end: Utc::now(),
            size: 0,
            order_book: Vec::new(),
        }
    }

    pub async fn read(path: &str) -> Result<Self, String> {
        let path_copied = String::from(path);
        tokio::task::spawn_blocking(move || {
            let reader = match std::fs::File::open(path_copied) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            let data = match serde_json::from_reader(reader) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!("failed to read!\ndetails : {:?}", result));
                }
            };
            Ok(data)
        })
        .await
        .expect("failed to run blocking task!")
    }

    pub async fn write(&self, path: &str) -> Result<(), String> {
        let path_copied = String::from(path);
        let task = tokio::spawn(File::create(path_copied));
        let json = match serde_json::to_string(self) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to serialize!\ndetails : {:?}",
                    result
                ))
            }
        };
        let mut file = match task.await.expect("failed to excute a task!") {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!("failed to create!\ndetails : {:?}", result));
            }
        };
        if let Err(result) = file.write_all(json.as_bytes()).await {
            return Err(error_message!("failed to write!\ndetails : {:?}", result));
        }
        Ok(())
    }
}

impl Default for local::LocalOrderBook {
    fn default() -> Self {
        Self::new()
    }
}
