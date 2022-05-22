use ipc::*;

const SOCKET_CLIENT_ASYNC_PATH: &str = "./zenies/etc/test2_async.sock";

#[tokio::main]
async fn main() {
    let msg = std::env::args().nth(1).expect("failed to get args");
    println!("{}", msg);

    let mut client = match StreamAsync::open(&String::from(SOCKET_CLIENT_ASYNC_PATH)).await {
        Ok(result) => result,
        Err(_) => {
            panic!();
        }
    };

    client.write_string(msg).await.expect("failed to write");
    let received = client.read_to_string().await.expect("failed to read");
    println!("{}", received);
}
