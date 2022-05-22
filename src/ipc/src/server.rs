use super::*;

impl ListenerBlocking {
    pub fn listen(&self) -> Result<StreamBlocking, std::io::Error> {
        match self._listener.accept() {
            Ok((stream, _)) => Ok(StreamBlocking {
                _socket_path: self._socket_path.clone(),
                _stream: stream,
            }),
            Err(result) => Err(result),
        }
    }

    pub fn open(path: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let sockfile = std::path::Path::new(path);
        if sockfile.exists() {
            match std::fs::remove_file(&sockfile) {
                Ok(_) => {}
                Err(result) => {
                    return Err(error_message!(
                        "failed to delete a socket!\ndetails : {:?}",
                        result
                    ));
                }
            }
        }

        let listener = match UnixListener::bind(path) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to bind a socket!\ndetails : {:?}",
                    result
                ));
            }
        };

        Ok(ListenerBlocking {
            _socket_path: String::from(path),
            _listener: listener,
        })
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<(), String> {
        match self._listener.set_nonblocking(nonblocking) {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!(
                "failed to set the blocking configuraton!\ndetails : {:?}",
                result
            )),
        }
    }
}

impl Drop for ListenerBlocking {
    fn drop(&mut self) {
        std::fs::remove_file(&self._socket_path).expect("failed to close!\ndetails : {:?}");
    }
}

impl ListenerAsync {
    pub async fn listen(&self) -> Result<StreamAsync, std::io::Error> {
        match self._listener.accept().await {
            Ok((stream, _)) => Ok(StreamAsync {
                _socket_path: self._socket_path.clone(),
                _stream: stream,
            }),
            Err(result) => Err(result),
        }
    }

    pub async fn open(path: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let sockfile = std::path::Path::new(path);
        if sockfile.exists() {
            match tokio::fs::remove_file(&sockfile).await {
                Ok(_) => {}
                Err(result) => {
                    return Err(error_message!(
                        "failed to delete a socket!\ndetails : {:?}",
                        result
                    ));
                }
            }
        }

        let listener = match tokio_UnixListener::bind(path) {
            Ok(result) => result,
            Err(result) => {
                return Err(error_message!(
                    "failed to bind a socket!\ndetails : {:?}",
                    result
                ));
            }
        };

        Ok(ListenerAsync {
            _socket_path: String::from(path),
            _listener: listener,
        })
    }

    pub async fn close(&self) -> Result<(), String> {
        match tokio::fs::remove_file(&self._socket_path).await {
            Ok(_) => Ok(()),
            Err(result) => Err(error_message!("failed to close!\ndetails : {:?}", result)),
        }
    }
}

impl Drop for ListenerAsync {
    fn drop(&mut self) {
        std::fs::remove_file(&self._socket_path).expect("failed to close!\ndetails : {:?}");
    }
}
