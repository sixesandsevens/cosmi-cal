// SPDX-License-Identifier: MPL-2.0

use crate::commands::AppCommand;
use cosmic::iced::Subscription;
use cosmic::iced_futures;
use futures_util::SinkExt;
use std::io::{ErrorKind, Write};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_NAME: &str = "cosmical.sock";
const IPC_SUBSCRIPTION_ID: &str = "cosmical-ipc-listener";

pub fn socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join(SOCKET_NAME);
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    std::env::temp_dir().join(format!("cosmical-{user}.sock"))
}

pub fn send_commands(commands: &[AppCommand]) -> Result<(), String> {
    let path = socket_path();
    let mut stream = StdUnixStream::connect(&path)
        .map_err(|e| format!("failed to connect to {}: {e}", path.display()))?;

    for command in commands {
        writeln!(stream, "{}", command.as_wire()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn subscription() -> Subscription<AppCommand> {
    Subscription::run_with(IPC_SUBSCRIPTION_ID, |_| {
        let path = socket_path();

        iced_futures::stream::channel(
            32,
            move |mut tx: iced_futures::futures::channel::mpsc::Sender<AppCommand>| async move {
                let listener = match bind_listener(&path) {
                    Ok(listener) => listener,
                    Err(err) => {
                        eprintln!("IPC listener unavailable: {err}");
                        return;
                    }
                };

                loop {
                    let (stream, _) = match listener.accept().await {
                        Ok(pair) => pair,
                        Err(err) => {
                            eprintln!("IPC accept failed: {err}");
                            continue;
                        }
                    };

                    let mut lines = BufReader::new(stream).lines();
                    loop {
                        match lines.next_line().await {
                            Ok(Some(line)) => {
                                if let Some(command) = AppCommand::from_wire(&line) {
                                    let _ = tx.send(command).await;
                                }
                            }
                            Ok(None) => break,
                            Err(err) => {
                                eprintln!("IPC read failed: {err}");
                                break;
                            }
                        }
                    }
                }
            },
        )
    })
}

fn bind_listener(path: &Path) -> Result<UnixListener, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err.to_string()),
    }

    UnixListener::bind(path).map_err(|err| err.to_string())
}
