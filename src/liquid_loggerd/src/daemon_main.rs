use super::*;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::time;

pub async fn daemon_main() {
    // load configurtation
    let env = match load_env("./env.toml").await {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
            return;
        }
    };
    /*let (_log_dir, db_path, etc_dir, _bin_dir, _keybox_path) = match misc::load_env() {
        Ok(result) => result,
        Err(result) => {
            log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
            return;
        }
    };*/

    // prevent same program running
    if let Err(result) = misc::lock_file(&env.general.etc_directory_path) {
        log::error!("failed to lock the file!\n-->\ndetails : {}\n<--", result);
        return;
    }

    // check wether systemd is running or not
    if !daemon::booted() {
        log::error!("systemd is not running!");
        return;
    };
    // check wether watchdog is enable or not and the duration
    let watchdog_duration = match daemon::watchdog_enabled(true) {
        Some(result) => result,
        None => {
            log::error!("watchdog is diabled! WatchdogSec must be set.");
            return;
        }
    };

    // initialize systemd journal
    if let Err(result) =
        systemd_journal_logger::init_with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
    {
        log::error!(
            "failed to initialize the journal!\n-->\ndetails : {:?}\n<--",
            result
        );
        return;
    };
    log::set_max_level(log::LevelFilter::Debug);

    // initiazlie signal handler
    let mut sig_term = match signal(SignalKind::terminate()) {
        Ok(result) => result,
        Err(result) => {
            log::error!(
                "failed to initialize SIGTERM!\n-->\ndetails : {:?}\n<--",
                result
            );
            return;
        }
    };
    let mut sig_hangup = match signal(SignalKind::hangup()) {
        Ok(result) => result,
        Err(result) => {
            log::error!(
                "failed to initialize SIGHUP!\n-->\ndetails : {:?}\n<--",
                result
            );
            return;
        }
    };

    // initialize watchdog timer
    let mut watchdog_timer = time::interval(watchdog_duration * 9 / 10);

    let listener = match ipc::ListenerAsync::open(
        &(env.general.etc_directory_path.clone()
            + env!("CARGO_PKG_NAME")
            + common_constants::SOCKET_EXTENSION),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            log::error!("failed to open the unix domai socket!");
            return;
        }
    };
    let (transmitter, mut receiver) =
        tokio::sync::mpsc::channel::<ipc::Command>(std::mem::size_of::<[ipc::Command; 16]>());
    let (task_transmitter, task_receiver) =
        mpsc::unbounded_channel::<tokio::task::JoinHandle<()>>();

    let task_handler = tokio::task::spawn(handler::task_handler(task_receiver));
    // initialize liquidn logger
    let (database, mut client) = match initialize::initialize(&env.general.database_url).await {
        Ok(result) => result,
        Err(_) => return,
    };

    log::info!("{} was initialized.", env!("CARGO_PKG_NAME"));

    // notify systemd that ready
    if let Err(result) = daemon::notify(false, &[daemon::NotifyState::Ready]) {
        log::error!(
            "failed to notify systemd!\n-->\ndetails : {:?}\n<--",
            result
        );
        return;
    }

    loop {
        tokio::select! {
            _ = sig_term.recv() => {
                return;
            }
            _ = sig_hangup.recv() => {}
            _ = watchdog_timer.tick() => {
                if let Err(result) = daemon::notify(false, &[daemon::NotifyState::Watchdog]) {
                    log::error!("failed to notify systemd that watchdog!\n-->\ndetails : {:?}\n<--", result);
                    return;
                }
            }
            Some(command) = receiver.recv() => {
                match command.command.as_ref() {
                    common_constants::DEFINED_COMMAND_SHUTDOWN => return,
                    _ => {}
                }
            }
            listened = listener.listen() => {
                match listened {
                    Ok(stream) => {
                        let transmitter_copied = transmitter.clone();
                        tokio::spawn(handler::ipc_handler(stream, transmitter_copied));
                    }
                    Err(result) => match result.kind() {
                        std::io::ErrorKind::WouldBlock => {}
                        _ => {
                            log::error!("cannot listen to ipc!\n-->\ndetails : {}\n<--", result);
                            return;
                        }
                    }
                }
            }

            Ok(checked) = time::timeout(watchdog_duration / 2, client.check()) => {
                match checked {
                    Ok(result) => {
                        channel_handler::channel_handler(&mut client, &result, database.clone(), &task_transmitter).await;
                    }
                    Err(result) => {
                        log::warn!(
                            "cannot check!\n-->\ndetails : {}\n<--", result
                        );
                        log::info!("reinitialize connection to liquid tap due to a previous error.");
                        client = match initialize::initialize_liquid_tap().await {
                            Ok(result) => result,
                            Err(_) => break,
                        };
                    }
                };
            }
            else => {
                log::error!("liquid tap was timeout! liquid_loggerd would shutdown!");
                break;
            }
        };
    }
    drop(task_transmitter);
    task_handler.await.expect("failed to run task!");
}
