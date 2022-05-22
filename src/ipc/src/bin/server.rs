use ipc::*;

const SOCKET_CLIENT_ASYNC_PATH: &str = "./zenies/etc/test2_async.sock";

#[tokio::main]
async fn main() {
    let server = match ListenerAsync::open(&String::from(SOCKET_CLIENT_ASYNC_PATH)).await {
        Ok(result) => result,
        Err(_) => {
            panic!();
        }
    };

    while let Ok(mut stream) = server.listen().await {
        tokio::task::spawn(async move {
            let msg = stream.read_to_string().await.expect("failed to send");
            println!("{}", msg);
            stream
                .write_string(msg.to_uppercase())
                .await
                .expect("failed to write");
        })
        .await
        .expect("failed to spawn");
    }
}
