use super::*;

pub async fn status_handler() {
    // load configurtation
    let env = match load_env("./env.toml").await {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
            return;
        }
    };

    let mut stream = match ipc::StreamAsync::open(
        &(env.general.etc_directory_path
            + env!("CARGO_PKG_NAME")
            + common_constants::SOCKET_EXTENSION),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            eprintln!(
                "{}",
                error_message_colored!("failed connect to {}!", env!("CARGO_PKG_NAME"))
            );
            return;
        }
    };

    if let Err(result) = stream.send_command(ipc::Command::status()).await {
        println!(
            "{}",
            error_message_colored!("failed to send a command!\n-->\ndetails : {}\n<--", result)
        );
        return;
    }

    let result: ipc::Command = match stream.receive_command().await {
        Ok(result) => result,
        Err(result) => {
            eprintln!(
                "{}",
                error_message_colored!(
                    "failed to receive a command!\n-->\ndetails : {}\n<--",
                    result
                )
            );
            return;
        }
    };
    let data = result.messages.expect("should contain!");
    println!(
        "name : {}\nversion : {}\ndescription : {}\nstatus : {}",
        data["name"], data["version"], data["description"], data["status"]
    );
}

pub async fn start_handler(config_path: &str) {
    daemon_main::daemon_main(config_path).await;
}

pub async fn shutdown_handler() {
    // load configurtation
    let env = match load_env("./env.toml").await {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
            return;
        }
    };

    let mut stream = match ipc::StreamAsync::open(
        &(env.general.etc_directory_path
            + env!("CARGO_PKG_NAME")
            + common_constants::SOCKET_EXTENSION),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            eprintln!(
                "{}",
                error_message_colored!("failed connect to {}!", env!("CARGO_PKG_NAME"))
            );
            return;
        }
    };

    if let Err(result) = stream
        .send_command(ipc::Command::gracefull_shutdown())
        .await
    {
        eprintln!(
            "{}",
            error_message_colored!("failed to send a command!\n-->\ndetails : {}\n<--", result)
        );
        return;
    }
}

pub async fn ipc_handler(
    mut stream: ipc::StreamAsync,
    transmitter: tokio::sync::mpsc::Sender<ipc::Command>,
) {
    let result: ipc::Command = match stream.receive_command().await {
        Ok(result) => result,
        Err(result) => {
            log::error!(
                "failed to receive a command!\n-->\ndetails : {}\n<--",
                result
            );
            return;
        }
    };

    match result.command.as_ref() {
        common_constants::DEFINED_COMMAND_GET_STATUS => {
            if let Err(result) = stream
                .send_command(ipc::Command::response(
                    [
                        (String::from("name"), String::from(env!("CARGO_PKG_NAME"))),
                        (
                            String::from("version"),
                            String::from(env!("CARGO_PKG_VERSION")),
                        ),
                        (
                            String::from("description"),
                            String::from(env!("CARGO_PKG_DESCRIPTION")),
                        ),
                        (
                            String::from("status"),
                            format!("{}", console::style("active(running)").green()),
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                ))
                .await
            {
                log::error!("failed to send a command!\n-->\ndetails : {}\n<--", result);
                return;
            }
        }
        _ => {
            if let Err(result) = transmitter.send(result).await {
                log::error!(
                    "failed to send to the main thread!\n-->\ndetails : {}\n<--",
                    result
                );
            }
        }
    }
}
