use std::{convert::Infallible, path::PathBuf, process::Stdio, time::Duration};

use futures::{SinkExt, channel::mpsc::{UnboundedSender}, stream::{Stream, StreamExt}};
use serde::Serialize;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{ChildStderr, ChildStdout, Command},
};
use warp::sse::Event;

type StreamItem = Result<Event, Infallible>;

pub fn write_nfc(set: &str, name: &str) -> impl Stream<Item = StreamItem> {
    let public = PathBuf::from("public");
    let path = public
        .join("amiibo")
        .join(set)
        .join(name.replace("%20", " "))
        .with_extension("bin");
    let timeout = if let Ok(timeout) = std::env::var("FAMIIBO_TIMEOUT") {
        timeout.parse().unwrap_or(120)
    } else {
        120
    };
    log::info!("Attempting to exec {} with timeout {}", path.display(), timeout);
    let (mut tx, rx) = futures::channel::mpsc::unbounded();
    tokio::task::spawn(async move {
        
        let ev = if tokio::time::timeout(Duration::from_secs(timeout),
        handle_stream(path, tx.clone())).await.is_err() {
            Event::default().data("timeout")
        } else{
            Event::default()
        };
        tx.send(Ok(ev.event("complete"))).await.map_err(|e| {
            log::error!("Failed to send close: {}", e);
        }).ok();
    });
    rx
}

async fn handle_stream(path: PathBuf, mut tx: UnboundedSender<StreamItem>) {
    if !path.exists() {
        tx.send(
            Ok(Message::Error(format!(
                "Path not found: {}",
                path.display()
            )).as_event()),
        ).await.map_err(|e| {
            log::error!("failed to send path error on tx: {}", e);
            e
        }).expect("path");
        return;
    }
    let mut child = match Command::new("pimiibo")
        .arg("public/key_retail.bin")
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            tx.send(
                Ok(Message::Error(format!("Error executing pimiibo: {}", e)).as_event()),
            ).await.expect("spawn");
            return;
        }
    };
    let stderr = child.stderr.take().ok_or_else(|| {
        log::error!("no stderr on child");
    }).expect("stderr");
    let stderr = BufReader::new(stderr);
    let stderr = stderr.lines();
    let stdout = child.stdout.take().ok_or_else(|| {
        log::error!("no stdout on child");
    }).expect("stdout");
    let stdout = BufReader::new(stdout);
    let stdout = stdout.lines();
    let mut stream = Box::pin(stream_from_io(stderr, stdout));
    loop {
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                log::debug!("exited: {}", exit_status);
                let msg = if exit_status.success() {
                    log::info!("exit status was success");
                    Message::Success
                } else {
                    log::error!("exit status was error");
                    Message::Error(format!("non-success exit status: {}", exit_status))
                };
                let msg = Ok(msg.as_event());
                tx.send(msg).await.map_err(|e| {
                    log::error!("send exit error: {}",  e);
                }).ok();
                return;
            },
            Ok(None) => {
                log::debug!("no exit status yet, trying stream");
                if let Some(ev) = stream.next().await {
                    tx.send(Ok(ev)).await.map_err(|e| {
                        log::error!("io send: {}", e);
                    }).ok();
                }
            },
            Err(e) => {
                let msg = format!("error while waiting: {}", e);
                log::error!("{}", msg);
                let msg = Message::Error(msg);
                let msg = Ok(msg.as_event());
                tx.send(msg).await.map_err(|e| {
                    log::error!("send error: {}", e);
                }).ok();
                return;
            }
        }
    }
}

fn stream_from_io(
    mut stderr: Lines<BufReader<ChildStderr>>,
    mut stdout: Lines<BufReader<ChildStdout>>,
) -> impl Stream<Item = Event> {
    async_stream::stream! {
        loop {
            tokio::select! {
                line = stdout.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log::debug!("stdout: {}", line);
                            yield Message::StdOut(line).as_event()
                        },
                        Ok(None) => return,
                        Err(e) => {
                            yield Message::Error(format!("Error reading line from stdout: {}", e)).as_event();
                            return;
                        }
                    }
                },
                line = stderr.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log::debug!("stderr: {}", line);
                            yield Message::StdErr(line).as_event()
                        },
                        Ok(None) => return,
                        Err(e) => {
                            yield Message::Error(format!("Error reading line from stderr: {}", e)).as_event();
                            return;
                        }
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    log::debug!("timeout waiting for line");
                    yield Message::Ping.as_event();
                }
            }
        }
    }
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "data")]
pub enum Message {
    Error(String),
    Success,
    StdOut(String),
    StdErr(String),
    Ping,
}

impl Message {
    fn as_event(&self) -> Event {
        let mut ev = Event::default();
        if matches!(self, Self::Error(_)) {
            log::error!("sending error message: {:?}", self);
            ev = ev.event("error");
        }
        ev.json_data(self).map_err(|e| {
            log::error!("json error: {}", e);
            e
        }).expect("into json event")
    }
}
