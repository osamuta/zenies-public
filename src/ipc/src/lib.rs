use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::net::{UnixListener, UnixStream};
use std::result::Result;
use tokio::net::{UnixListener as tokio_UnixListener, UnixStream as tokio_UnixStream};

mod client;
mod constants;
mod defined_commands;
mod server;

use common::{common_constants, error_message};
use constants::*;

#[derive(Debug)]
pub struct ListenerBlocking {
    _socket_path: String,
    _listener: UnixListener,
}

#[derive(Debug)]
pub struct StreamBlocking {
    _socket_path: String,
    _stream: UnixStream,
}

//
// async ver
#[derive(Debug)]
pub struct ListenerAsync {
    _socket_path: String,
    _listener: tokio_UnixListener,
}

#[derive(Debug)]
pub struct StreamAsync {
    _socket_path: String,
    _stream: tokio_UnixStream,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub command: String,
    pub messages: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOCKET_SERVER_PATH: &'static str = "../../zenies/etc/test.sock";
    const SOCKET_CLIENT_PATH: &'static str = "../../zenies/etc/test2.sock";
    const SOCKET_CLIENT_ASYNC_PATH: &'static str = "../../zenies/etc/test2_async.sock";

    #[test]
    fn check_server_initialize() {
        let _server = match ListenerBlocking::open(&String::from(SOCKET_SERVER_PATH)) {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };
    }

    #[test]
    fn check_funcation_through_the_pipline() {
        let server = match ListenerBlocking::open(&String::from(SOCKET_CLIENT_PATH)) {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        let client = match StreamBlocking::open(&String::from(SOCKET_CLIENT_PATH)) {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        let data = vec![1, 2, 3, 4];
        let message = String::from("hello");
        let command = Command::ping();

        let server = match server.listen() {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        assert_eq!(server.write(&data), Ok(()));
        assert_eq!(client.read(), Ok(data.clone()));

        assert_eq!(client.write(&data), Ok(()));
        assert_eq!(server.read(), Ok(data.clone()));

        assert_eq!(server.write_string(message.clone()), Ok(()));
        assert_eq!(client.read_to_string(), Ok(message.clone()));

        assert_eq!(client.write_string(message.clone()), Ok(()));
        assert_eq!(server.read_to_string(), Ok(message.clone()));

        assert_eq!(server.send_command(command.clone()), Ok(()));
        assert_eq!(client.receive_command().is_ok(), true);

        assert_eq!(client.send_command(command.clone()), Ok(()));
        assert_eq!(server.receive_command().is_ok(), true);

        // assert_eq!(client.close(), Ok(()));
        // assert_eq!(server.close(), Ok(()));
    }

    #[tokio::test]
    async fn check_funcation_through_the_pipline_async() {
        let server = match ListenerAsync::open(&String::from(SOCKET_CLIENT_ASYNC_PATH)).await {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        let mut client = match StreamAsync::open(&String::from(SOCKET_CLIENT_ASYNC_PATH)).await {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        let data = vec![1, 2, 3, 4];
        let message = String::from("hello");
        let command = Command::ping();

        let mut server = match server.listen().await {
            Ok(result) => result,
            Err(_) => {
                panic!();
            }
        };

        assert_eq!(server.write(&data).await, Ok(()));
        assert_eq!(client.read().await, Ok(data.clone()));

        assert_eq!(client.write(&data).await, Ok(()));
        assert_eq!(server.read().await, Ok(data.clone()));

        assert_eq!(server.write_string(message.clone()).await, Ok(()));
        assert_eq!(client.read_to_string().await, Ok(message.clone()));

        assert_eq!(client.write_string(message.clone()).await, Ok(()));
        assert_eq!(server.read_to_string().await, Ok(message.clone()));

        assert_eq!(server.send_command(command.clone()).await, Ok(()));
        assert_eq!(client.receive_command().await.is_ok(), true);

        assert_eq!(client.send_command(command).await, Ok(()));
        assert_eq!(server.receive_command().await.is_ok(), true);
    }
}
