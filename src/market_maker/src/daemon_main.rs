use libsystemd::*;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::*;
use tokio::time;

use common::*;

use crate::channel_handler;
use crate::handler;
use crate::initialize;
use crate::misc;
use crate::trader;

pub async fn daemon_main(ident: &str) {
    //
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

    //
    // check wether systemd is running or not
    if !daemon::booted() {
        log::error!("systemd is reuqired!");
        return;
    };

    //
    // check wether watchdog is enable or not and the duration
    let watchdog_duration = match daemon::watchdog_enabled(true) {
        Some(result) => result,
        None => {
            log::error!("watchdog is diabled! WatchdogSec must be set.");
            return;
        }
    };

    //
    // set logging level
    log::set_max_level(log::LevelFilter::Info);

    //
    // load configurtation
    let env = match load_env("./env.toml").await {
        Ok(result) => Arc::new(result),
        Err(result) => {
            log::error!("failed to load!\n-->\ndetails : {}\n<--", result);
            return;
        }
    };
    let config = match misc::Config::load(&(String::from(ident) + ".toml")).await {
        Ok(mut content) => {
            content.identifier = String::from(ident);
            Arc::new(content)
        }
        Err(_) => {
            return;
        }
    };

    //
    // prevent same program running
    if let Err(result) = misc::lock_file(&env.general.etc_directory_path, &config.identifier) {
        log::error!("failed to lock the file!\n-->\ndetails : {}\n<--", result);
        return;
    }

    //
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
    //let mut kill_timer = time::interval(time::Duration::from_secs(config.maximum_running_time));
    //kill_timer.tick().await;

    //
    // initialize unix socket
    let listener = match ipc::ListenerAsync::open(
        &(env.general.etc_directory_path.clone()
            + env!("CARGO_PKG_NAME")
            + "_"
            + &config.identifier
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
        mpsc::channel::<ipc::Command>(std::mem::size_of::<[ipc::Command; 16]>());

    //
    // initialize liquid
    let (mut client, database) =
        match initialize::initialize(&config.key, config.currency_pair, &env.general.database_url)
            .await
        {
            Ok(result) => result,
            Err(_) => return,
        };
    //let _database = Arc::new(database);

    //
    // intialize trader
    let (trader_trans, trader_recv) = trader::new_trader_event_channel();

    log::info!("{} was initialized.", env!("CARGO_PKG_NAME"));

    // notify systemd that ready
    if let Err(result) = daemon::notify(false, &[daemon::NotifyState::Ready]) {
        log::error!(
            "failed to notify systemd!\n-->\ndetails : {:?}\n<--",
            result
        );
        return;
    }

    //
    // start main trading tasks
    {
        tokio::spawn(trader::trader(trader_recv, config.clone(), env.clone()));
        if let Err(_) =
            trader::get_old_market(database.clone(), trader_trans.clone(), config.clone()).await
        {
            return;
        }
        if let Err(result) = trader_trans.post_booted().await {
            log::error!("failed to shutdown!\n-->\ndetails : {}\n<--", result);
        }
    }

    //
    // main loop
    'main: loop {
        tokio::select! {
            _ = sig_term.recv() => {
                break;
            }
            _ = sig_hangup.recv() => {}
            _ = watchdog_timer.tick() => {
                if let Err(result) = daemon::notify(false, &[daemon::NotifyState::Watchdog]) {
                    log::error!("failed to notify systemd that watchdog!\n-->\ndetails : {:?}\n<--", result);
                    break;
                }
            }
            Some(command) = receiver.recv() => {
                match command.command.as_ref() {
                    common_constants::DEFINED_COMMAND_SHUTDOWN => break,
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
                            break;
                        }
                    }
                }
            }

            Ok(checked) = time::timeout(time::Duration::from_secs(60 * 5), client.check()) => {
                match checked {
                    Ok(result) => {
                        if let Err(()) = channel_handler::channel_handler(&mut client, &result, &trader_trans).await{
                            break;
                        }
                    }
                    Err(result) => {
                        'trials: for i in 1..=3 {
                            log::warn!(
                                "cannot check!\n-->\ndetails : {}\n<--", result
                            );
                            log::info!("reinitialize connection to liquid tap due to a previous error. {} / 3 trials", i);
                            match initialize::initialize_liquid_tap(&config.key, config.currency_pair).await {
                                Ok(result) => {
                                    client = result;
                                    break 'trials;
                                },
                                Err(_) => if i >= 3 {
                                    log::error!("failed to reinitialize connection to liquid tap due to exceeded trial limits!");
                                    break 'main;
                                },
                            };
                        }
                    }
                };
            }
            else => {
                log::error!("liquid tap was timeout! {} would shutdown!", env!("CARGO_PKG_NAME"));
                break;
            }
        };
    }

    if let Err(result) = trader_trans.post_shudown().await {
        log::error!("failed to shutdown!\n-->\ndetails : {}\n<--", result);
    }
    tokio::task::yield_now().await;
}
