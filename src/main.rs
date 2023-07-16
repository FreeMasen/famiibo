use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::AtomicU64;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use warp::hyper::StatusCode;
use warp::sse::Event;
use warp::{Filter, Reply};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let files = warp::fs::dir("public");
    let execute = warp::get()
        .and(warp::path!("write" / String / String))
        .then(|set: String, bin: String| write_nfc(set, bin));
    warp::serve(files.or(execute).with(warp::log("amiibo")))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

async fn write_nfc(set: String, name: String) -> Box<dyn Reply> {
    log::trace!("write_nfc {set} - {name}",);
    let cmd = match generate_command(&set, &name).await {
        Ok(cmd) => cmd,
        Err(_e) => return Box::new(response(&Response::NotFound, StatusCode::NOT_FOUND))
    };
    let rx = spawn_write_sse(cmd).await;
    let stream = rx.map(|st| Event::default()
        .id(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string())
        .json_data(&st).map_err(|e| Error::Internal(e.to_string())));
    
    Box::new(warp::sse::reply(warp::sse::keep_alive().stream(stream)))
}

async fn generate_command(set: &str, name: &str) -> Result<Command, Error> {
    log::trace!("generate_command {set} - {name}");
    let public = PathBuf::from("public");
    let path = public
        .join("amiibo")
        .join(set)
        .join(name.replace("%20", " "))
        .with_extension("bin");
    if !path.exists() {
        log::warn!("{} doesn't exist!", path.display());
        return Err(Error::NotFound(format!("Amiibo not found: {}", path.display())));
    }
    let mut cmd = Command::new("pimiibo");
    cmd.arg("public/key_retail.bin").arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(cmd)
}

async fn spawn_write_sse(cmd: Command) -> UnboundedReceiver<CmdStatus> {
    let (tx, rx) = futures::channel::mpsc::unbounded();
    spawn_sse(cmd, tx);
    rx
}

fn spawn_sse(mut cmd: Command, mut tx: UnboundedSender<CmdStatus>) {
    tokio::task::spawn(async move {
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                tx.send(
                    CmdStatus::Failed(format!("Spawn Failure: {e}")),
                )
                .await
                .ok();
                return;
            }
        };
        tx.send(
            CmdStatus::Started,
        )
        .await
        .ok();
        let msg = if let Some(stdout) = child.stdout.take() {
            if let Err(e) = watch_stdout(stdout, tx.clone()).await {
                tx.send(
                    CmdStatus::Failed(format!("Exited unsuccessfully: {e}"))
                )
                .await
                .ok();
            }
            wait_for_child(child)
        } else {
            wait_for_child(child)
        }
        .await;
        tx.send(
            msg,
        )
        .await
        .ok();
    });
}

async fn wait_for_child(mut child: Child) -> CmdStatus {
    if let Some(mut stderr) = child.stderr.take() {
        let status = match child.wait().await {
            Ok(status) => if status.success() {
                CmdStatus::Success
            } else {
                let mut s = Vec::new();
                stderr.read_to_end(&mut s).await.ok();
                CmdStatus::Failed(format!("Error exit status: {status}: {}", String::from_utf8_lossy(&s)))
            }
            Err(e) => {
                log::error!("Error from wait: {e}");
                CmdStatus::Failed(e.to_string())
            }
        };
        Ok(status)
    } else {
        child
            .wait_with_output()
            .await
            .map(|o| {
                if !o.status.success() {
                    let info = String::from_utf8_lossy(&o.stderr);
                    log::error!("Error from wait_with_output {info}");
                    CmdStatus::Failed(format!("Exited unsuccessfully: {} - {}", o.status, info.trim()))
                } else {
                    CmdStatus::Success
                }
            })
    }
    .unwrap_or_else(|e| CmdStatus::Failed(format!("{e}")))
}

async fn watch_stdout(
    stdout: impl AsyncRead + Unpin,
    mut tx: UnboundedSender<CmdStatus>,
) -> Result<(), Error> {
    const TOTAL_LINES: f32 = 159.0;
    let stdout = tokio::io::BufReader::new(stdout);
    let mut lines = stdout.lines();
    let mut ct = 0.0;
    while let Ok(Some(line)) = lines.next_line().await {
        ct += 1.0;
        if line.ends_with("...Failed") {
            return Err(Error::Internal(line));
        }
        if line == "Finished writing tag" {
            tx.send(
                CmdStatus::Progress(100.0),
            )
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
            ct = TOTAL_LINES;
            break;
        }
        let percent = ((ct / TOTAL_LINES) * 100.0).floor();
        tx.send(
            CmdStatus::Progress(percent),
        )
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    }
    if ct < TOTAL_LINES {
        return Err(Error::Internal(format!(
            "Exited early expected {TOTAL_LINES} found {ct}"
        )));
    }
    Ok(())
}

fn response(r: &Response, status: warp::http::StatusCode) -> impl Reply {
    let inner = warp::reply::json(r);
    warp::reply::with_status(inner, status)
}

#[derive(Debug, Serialize)]
enum Response {
    NotFound,
}

#[derive(Debug, Serialize)]
struct CmdFailed {
    std_err: String,
    std_out: String,
    status: Option<i32>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum CmdStatus {
    Started,
    Success,
    Progress(f32),
    Failed(String),
}

#[derive(Debug, Serialize, thiserror::Error)]
pub enum Error {
    #[error("Error: {0}")]
    Internal(String),
    #[error("{0} was not found")]
    NotFound(String),
}


#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn exec_happy_path() {
        pretty_env_logger::formatted_builder().is_test(true).try_init().ok();
        let cmd = generate_command("test", "success").await.unwrap();
        let mut rx = spawn_write_sse(cmd).await;
        let first = rx.next().await.unwrap();
        assert_eq!(first, CmdStatus::Started);
        while let Some(msg) = rx.next().await {
            if let CmdStatus::Progress(progress) = &msg {
                assert!(*progress <= 100.0, "progress > 100% {}", progress);
                if *progress == 100.0 {
                    break;
                }
            } else {
                panic!("expected progress found {:?}", msg);
            }
        }
        let last = rx.next().await.unwrap();
        assert_eq!(last, CmdStatus::Success);
        assert_eq!(rx.next().await, None);
    }

    #[tokio::test]
    async fn exec_failure() {
        pretty_env_logger::formatted_builder().is_test(true).try_init().ok();
        let cmd = generate_command("test", "failure").await.unwrap();
        let mut rx = spawn_write_sse(cmd).await;
        let first = rx.next().await.unwrap();
        assert_eq!(first, CmdStatus::Started);
        let mut expected_info = "Exited unsuccessfully: Error: Writing to 4: aa aa aa aa...Failed";
        while let Some(msg) = rx.next().await {
            match msg {
                CmdStatus::Progress(progress) => {
                    assert!(progress <= 100.0, "progress > 100% {}", progress);
                    if progress == 100.0 {
                        break;
                    }
                },
                CmdStatus::Failed(info) => {
                    assert_eq!(info, expected_info);
                    expected_info = "Exited unsuccessfully: exit status: 1 - Expected error";
                }
                msg => panic!("expected progress or failure found {:?}", msg),
            }
        }
        assert_eq!(rx.next().await, None);
    }
}
