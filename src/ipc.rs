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

#[cfg(test)]
use std::os::unix::net::UnixListener as StdUnixListener;

const SOCKET_NAME: &str = "cosmical.sock";
const IPC_SUBSCRIPTION_ID: &str = "cosmical-ipc-listener";

pub enum StartupAction {
    StartPrimary,
    ForwardedToPrimary,
}

pub fn socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join(SOCKET_NAME);
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    std::env::temp_dir().join(format!("cosmical-{user}.sock"))
}

pub fn prepare_startup(commands: &[AppCommand]) -> Result<StartupAction, String> {
    let path = socket_path();
    prepare_startup_at_path(&path, commands)
}

pub fn forward_commands(commands: &[AppCommand]) -> Result<(), String> {
    let path = socket_path();
    forward_commands_at_path(&path, commands)
}

fn prepare_startup_at_path(path: &Path, commands: &[AppCommand]) -> Result<StartupAction, String> {
    if !path.exists() {
        return Ok(StartupAction::StartPrimary);
    }

    match StdUnixStream::connect(&path) {
        Ok(mut stream) => {
            write_commands(&mut stream, commands)?;
            Ok(StartupAction::ForwardedToPrimary)
        }
        Err(err) => {
            remove_socket_file(&path).map_err(|remove_err| {
                format!(
                    "failed to remove stale socket {} after connect error ({err}): {remove_err}",
                    path.display()
                )
            })?;
            Ok(StartupAction::StartPrimary)
        }
    }
}

fn forward_commands_at_path(path: &Path, commands: &[AppCommand]) -> Result<(), String> {
    let mut stream = StdUnixStream::connect(path).map_err(|e| e.to_string())?;
    write_commands(&mut stream, commands)
}

fn write_commands(stream: &mut StdUnixStream, commands: &[AppCommand]) -> Result<(), String> {
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
                let (listener, _socket_guard) = match bind_listener(&path) {
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

fn bind_listener(path: &Path) -> Result<(UnixListener, SocketGuard), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if path.exists() {
        match StdUnixStream::connect(path) {
            Ok(_) => {
                return Err(format!(
                    "socket {} is already in use by another instance",
                    path.display()
                ));
            }
            Err(_) => remove_socket_file(path).map_err(|err| {
                format!("failed to remove stale socket {}: {err}", path.display())
            })?,
        }
    }

    let listener = UnixListener::bind(path).map_err(|err| err.to_string())?;
    Ok((listener, SocketGuard::new(path.to_path_buf())))
}

fn remove_socket_file(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

struct SocketGuard {
    path: PathBuf,
}

impl SocketGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        if let Err(err) = remove_socket_file(&self.path) {
            eprintln!(
                "failed to remove IPC socket {} during shutdown: {err}",
                self.path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prepare_startup_forwards_when_socket_is_live() {
        let path = unique_test_socket_path("live");
        let _listener = StdUnixListener::bind(&path).expect("bind live socket");

        let action = prepare_startup_at_path(&path, &[AppCommand::SummonToggle])
            .expect("forward to existing instance");

        assert!(matches!(action, StartupAction::ForwardedToPrimary));
        assert!(path.exists(), "live socket should not be removed");

        cleanup_test_socket(&path);
    }

    #[test]
    fn prepare_startup_removes_stale_socket() {
        let path = unique_test_socket_path("stale");
        {
            let listener = StdUnixListener::bind(&path).expect("bind stale socket");
            drop(listener);
        }
        assert!(
            path.exists(),
            "stale socket file should remain before recovery"
        );

        let action = prepare_startup_at_path(&path, &[AppCommand::SummonToggle])
            .expect("recover from stale socket");

        assert!(matches!(action, StartupAction::StartPrimary));
        assert!(
            !path.exists(),
            "stale socket file should be removed before primary startup"
        );

        cleanup_test_socket(&path);
    }

    #[test]
    fn forward_commands_writes_to_live_socket() {
        let path = unique_test_socket_path("forward");
        let listener = StdUnixListener::bind(&path).expect("bind forward socket");

        forward_commands_at_path(&path, &[AppCommand::ShowSurface, AppCommand::FocusScratchpad])
            .expect("forward commands");

        let (mut stream, _) = listener.accept().expect("accept forwarded connection");
        let mut text = String::new();
        use std::io::Read;
        stream
            .read_to_string(&mut text)
            .expect("read forwarded commands");

        assert_eq!(text, "show_surface\nfocus_scratchpad\n");

        cleanup_test_socket(&path);
    }

    fn unique_test_socket_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("cosmical-{label}-{nanos}.sock"))
    }

    fn cleanup_test_socket(path: &Path) {
        let _ = fs::remove_file(path);
    }
}
