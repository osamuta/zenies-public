use super::*;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use tokio::io::{
    AsyncBufReadExt, AsyncWriteExt, BufReader as tokio_BufReader, BufWriter as tokio_BufWriter,
};

impl StreamBlocking {
    pub fn open(path: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let stream = match UnixStream::connect(&path) {
            Ok(stream) => stream,
            Err(stream) => {
                return Err(error_message!("failed to connect!\ndetails : {:?}", stream))
            }
        };

        if let Err(result) = stream.set_read_timeout(Some(std::time::Duration::from_secs(1))) {
            return Err(error_message!(
                "failed to set the read timeout!\ndetails : {:?}",
                result
            ));
        }

        if let Err(result) = stream.set_write_timeout(Some(std::time::Duration::from_secs(1))) {
            return Err(error_message!(
                "failed to set the read timeout!\ndetails : {:?}",
                result
            ));
        }

        Ok(StreamBlocking {
            _socket_path: String::from(path),
            _stream: stream,
        })
    }

    pub fn close(&self) -> Result<(), String> {
        match self._stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(_) => Err(error_message!("failed to shutdown a stream!")),
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<(), String> {
        match self._stream.set_nonblocking(nonblocking) {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to set the blocking configuraton!\ndetails : {:?}",
                result
            )),
        }
    }

    pub fn read(&self) -> Result<Vec<u8>, String> {
        let mut reader = BufReader::new(&self._stream);

        let mut received_data = Vec::new();
        match reader.read_until(DELIMITER, &mut received_data) {
            Ok(_) => {
                received_data.remove(received_data.len() - 1);
                Ok(received_data)
            }
            Err(result) => Err(error_message!(
                "failed to read from a stream!\ndetails : {:?}",
                result
            )),
        }
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        let mut writer = BufWriter::new(&self._stream);

        match writer.write_all(data) {
            Ok(_) => match writer.write_all(&[DELIMITER]) {
                Ok(_) => Ok(()),
                Err(result) => Err(error_message!(
                    "failed to write data to a stream!\ndetails : {:?}",
                    result
                )),
            },
            Err(result) => Err(error_message!(
                "failed to write data to a stream!\ndetails : {:?}",
                result
            )),
        }
    }

    pub fn read_to_string(&self) -> Result<String, String> {
        match self.read() {
            Ok(received_data) => Ok(match String::from_utf8(received_data) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to convert binary to string!\ndetails : {:?}",
                        result
                    ));
                }
            }),

            Err(received_data) => Err(error_message!(
                "failed to read string from a stream!\ndetails : {}",
                received_data
            )),
        }
    }

    pub fn write_string(&self, data: String) -> Result<(), String> {
        self.write(&data.into_bytes())
    }

    pub fn receive_command(&self) -> Result<Command, String> {
        let json_data = match self.read_to_string() {
            Ok(result) => result,
            Err(result) => return Err(result),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<Command> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub fn send_command(&self, command: Command) -> Result<(), String> {
        let json_data = match serde_json::to_string(&command) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\nserde_json message : {:?}\ncommand : {:?}",
                    result,
                    command
                ))
            }
        };

        match self.write_string(json_data) {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to send a command!\ndetails : {:?}\ncommand : {:?}",
                result,
                command
            )),
        }
    }
}

impl Drop for StreamBlocking {
    fn drop(&mut self) {
        self._stream
            .shutdown(std::net::Shutdown::Both)
            .expect("failed to shutdown a stream");
    }
}

impl StreamAsync {
    pub async fn open(path: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let stream = match tokio_UnixStream::connect(&path).await {
            Ok(stream) => stream,
            Err(stream) => {
                return Err(error_message!("failed to connect!\ndetails : {:?}", stream))
            }
        };

        /*if let Err(result) = stream.set_read_timeout(Some(std::time::Duration::from_secs(1))) {
            return Err(error_message!(
                "failed to set the read timeout!\ndetails : {:?}",
                result
            ));
        }

        if let Err(result) = stream.set_write_timeout(Some(std::time::Duration::from_secs(1))) {
            return Err(error_message!(
                "failed to set the read timeout!\ndetails : {:?}",
                result
            ));
        }*/

        //let (reader, writer) = stream.into_split();

        Ok(StreamAsync {
            _socket_path: String::from(path),
            _stream: stream,
        })
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, String> {
        let mut reader = tokio_BufReader::new(&mut self._stream);

        let mut received_data = Vec::new();
        match reader.read_until(DELIMITER, &mut received_data).await {
            Ok(_) => {
                received_data.remove(received_data.len() - 1);
                Ok(received_data)
            }
            Err(result) => Err(error_message!(
                "failed to read from a stream!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), String> {
        let mut writer = tokio_BufWriter::new(&mut self._stream);

        match writer.write_all(data).await {
            Ok(_) => match writer.write_all(&[DELIMITER]).await {
                Ok(_) => {
                    writer.flush().await.expect("failed to flush");
                    Ok(())
                }
                Err(result) => Err(error_message!(
                    "failed to write data to a stream!\ndetails : {:?}",
                    result
                )),
            },
            Err(result) => Err(error_message!(
                "failed to write data to a stream!\ndetails : {:?}",
                result
            )),
        }
    }

    pub async fn read_to_string(&mut self) -> Result<String, String> {
        match self.read().await {
            Ok(received_data) => Ok(match String::from_utf8(received_data) {
                Ok(result) => result,
                Err(result) => {
                    return Err(error_message!(
                        "failed to convert binary to string!\ndetails : {:?}",
                        result
                    ));
                }
            }),

            Err(received_data) => Err(error_message!(
                "failed to read string from a stream!\ndetails : {}",
                received_data
            )),
        }
    }

    pub async fn write_string(&mut self, data: String) -> Result<(), String> {
        self.write(&data.into_bytes()).await
    }

    pub async fn receive_command(&mut self) -> Result<Command, String> {
        let json_data = match self.read_to_string().await {
            Ok(result) => result,
            Err(result) => return Err(result),
        };

        match serde_json::from_str(&json_data) as serde_json::Result<Command> {
            Ok(result) => Ok(result),
            Err(result) => Err(error_message!(
                "failed to deserialize json!\nserde_json message : {:?}\njson : {}",
                result,
                json_data
            )),
        }
    }

    pub async fn send_command(&mut self, command: Command) -> Result<(), String> {
        let json_data = match serde_json::to_string(&command) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to deserialize json!\nserde_json message : {:?}\ncommand : {:?}",
                    result,
                    command
                ))
            }
        };

        match self.write_string(json_data).await {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to send a command!\ndetails : {:?}\ncommand : {:?}",
                result,
                command
            )),
        }
    }
}
