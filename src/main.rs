use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::AtomicU64;
use tera::{Context, Tera};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use urlencoding::decode;
use warp::hyper::StatusCode;
use warp::sse::Event;
use warp::{Filter, Reply};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let tera = Tera::new("templates/**/*.html").expect("tera");
    let port = std::env::var("FAMIIBO_PORT")
        .map_err(|_| ())
        .and_then(|v| v.parse::<u16>().map_err(|_| ()))
        .unwrap_or(80u16);
    let files = warp::fs::dir("public");
    let execute = warp::get()
        .and(warp::path!("write"))
        .and(warp::query())
        .then(|q: ExecuteQuery| write_nfc(q.path));
    let games_tera = tera.clone();
    let games = warp::get()
        .and(warp::path!("games"))
        .then(move || all_games(games_tera.clone()));
    let game = warp::get()
        .and(warp::path!("game" / String))
        .then(move |name: String| single_game(tera.clone(), name));
    warp::serve(
        execute
            .or(games)
            .or(game)
            .or(files)
            .with(warp::log("amiibo")),
    )
    .run(([0, 0, 0, 0], port))
    .await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteQuery {
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    name: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAmiibo {
    game: String,
    basic: Vec<Character>,
    categories: HashMap<String, Vec<Character>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    name: String,
    url: String,
}

async fn single_game(tera: Tera, name: String) -> Box<dyn Reply> {
    let mut context = Context::new();
    let name = decode(&name).unwrap();

    let amiibos = collect_game_amiibo(name.to_string()).await;
    println!("{}: {:#?}", name, amiibos);
    context.insert("game", &amiibos);
    let text = tera.render("game.html", &context).unwrap();
    Box::new(warp::reply::html(text))
}

async fn all_games(tera: Tera) -> Box<dyn Reply> {
    let mut context = Context::new();
    let mut read_dir = tokio::fs::read_dir("public/amiibo").await.unwrap();
    let mut games = Vec::new();
    while let Ok(Some(dir)) = read_dir.next_entry().await {
        println!("{}", dir.path().display());
        let is_dir = dir.metadata().await.map(|m| m.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        let dir_path = dir.path();
        let Some(file_stem) = dir_path.file_stem() else {
            continue;
        };
        let Some(file_stem) = file_stem.to_str() else {
            continue;
        };

        games.push(Game {
            name: file_stem.replace("Amiibo", "").trim().to_string(),
            url: format!("/game/{file_stem}"),
        });
    }
    context.insert("games", &games);
    let text = tera.render("games.html", &context).unwrap();
    Box::new(warp::reply::html(text))
}

async fn collect_game_amiibo(game: String) -> GameAmiibo {
    let mut ret = GameAmiibo {
        game: game.clone(),
        basic: Vec::new(),
        categories: HashMap::new(),
    };
    let dir = PathBuf::from("public/amiibo").join(game);
    println!("{}", dir.display());
    let Ok(mut read_dir) = tokio::fs::read_dir(&dir).await else {
        return ret;
    };

    while let Ok(Some(dir)) = read_dir.next_entry().await {
        let Ok(meta) = dir.metadata().await else {
            continue;
        };
        let path = dir.path();
        if meta.is_dir() {
            collect_sub_directory(&path, &mut ret.categories).await;
        } else {
            let Some(name) = path.file_stem() else {
                continue;
            };
            let Some(name) = name.to_str().map(|s| s.to_string()) else {
                continue;
            };
            let name = name_replacer(&name);
            let param = format!("{}", path.display());
            let param = urlencoding::encode(&param);
            let url = format!("/write?path={}", param);
            ret.basic.push(Character { name, url })
        }
    }
    ret
}

async fn collect_sub_directory(
    path: impl AsRef<Path>,
    comps: &mut HashMap<String, Vec<Character>>,
) {
    let Ok(category_name) = path.as_ref().strip_prefix("public/amiibo") else {
        return;
    };
    println!("{}", path.as_ref().display());
    let category_name: String = category_name
        .components()
        .filter_map(|c| {
            let s = c.as_os_str().to_str()?;
            Some(format!("{} ", s))
        })
        .collect();
    let Ok(mut read_dir) = tokio::fs::read_dir(path.as_ref()).await else {
        return;
    };
    let mut category = Vec::new();
    while let Ok(Some(dir)) = read_dir.next_entry().await {
        let Ok(meta) = dir.metadata().await else {
            continue;
        };
        if meta.is_dir() {
            collect_sub_directory2(dir.path(), comps).await;
            continue;
        }
        let amiibo_path = dir.path();
        let Some(name) = amiibo_path.file_stem() else {
            continue;
        };
        let Some(name) = name.to_str().map(|s| s.to_string()) else {
            continue;
        };
        let name = name_replacer(&name);
        let param = format!("{}", amiibo_path.display());
        let param = urlencoding::encode(&param);
        let url = format!("/write?path={}", param);
        category.push(Character { name, url })
    }
    if !category.is_empty() {
        comps.insert(category_name, category);
    }
}

fn name_replacer(name: &str) -> String {
    if name.trim().starts_with("[AC]") {
        let Some(dash_idx) = name.find("-") else {
            return name.to_string();
        };
        let Some(from_dash) = name.get(dash_idx+1..) else {
            return name.to_string();
        };
        return from_dash.to_string()
    }
    name
        .replace("[AC]", "")

        .trim().to_string()
}

async fn collect_sub_directory2(
    path: impl AsRef<Path>,
    comps: &mut HashMap<String, Vec<Character>>,
) {
    let Ok(category_name) = path.as_ref().strip_prefix("public/amiibo") else {
        return;
    };
    println!("{}", path.as_ref().display());
    let category_name: String = category_name
        .components()
        .filter_map(|c| {
            let s = c.as_os_str().to_str()?;
            Some(format!("{} ", s))
        })
        .collect();
    let Ok(mut read_dir) = tokio::fs::read_dir(path.as_ref()).await else {
        return;
    };
    let mut category = Vec::new();
    while let Ok(Some(dir)) = read_dir.next_entry().await {
        let Ok(meta) = dir.metadata().await else {
            continue;
        };
        if meta.is_dir() {
            log::warn!("Skipping {}", dir.path().display());
            // collect_sub_directory(dir.path(), comps).await;
            continue;
        }
        let amiibo_path = dir.path();
        let Some(name) = amiibo_path.file_stem() else {
            continue;
        };
        let Some(name) = name.to_str().map(|s| s.to_string()) else {
            continue;
        };
        let name = name_replacer(&name);
        let param = format!("{}", amiibo_path.display());
        let param = urlencoding::encode(&param);
        let url = format!("/write?path={}", param);
        category.push(Character { name, url })
    }
    if !category.is_empty() {
        comps.insert(category_name, category);
    }
}

async fn write_nfc(name: String) -> Box<dyn Reply> {
    log::trace!("write_nfc {name}",);
    let cmd = match generate_command(&name).await {
        Ok(cmd) => cmd,
        Err(_e) => return Box::new(response(&Response::NotFound, StatusCode::NOT_FOUND)),
    };
    let rx = spawn_write_sse(cmd).await;
    let stream = rx.map(|st| {
        Event::default()
            .id(COUNTER
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string())
            .json_data(&st)
            .map_err(|e| Error::Internal(e.to_string()))
    });

    Box::new(warp::sse::reply(warp::sse::keep_alive().stream(stream)))
}

async fn generate_command(path: &str) -> Result<Command, Error> {
    println!("generate_command {path}");
    let path = urlencoding::decode(path).map_err(|e| Error::NotFound(format!("{path}: {e}")))?;
    let path = PathBuf::from(path.as_ref());
    if !path.exists() {
        log::warn!("{} doesn't exist!", path.display());
        return Err(Error::NotFound(format!(
            "Amiibo not found: {}",
            path.display()
        )));
    }
    let mut cmd = Command::new("pimiibo");
    cmd.arg("public/key_retail.bin")
        .arg(path)
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
                tx.send(CmdStatus::Failed(format!("Spawn Failure: {e}")))
                    .await
                    .ok();
                return;
            }
        };
        tx.send(CmdStatus::Started).await.ok();
        let msg = if let Some(stdout) = child.stdout.take() {
            if let Err(e) = watch_stdout(stdout, tx.clone()).await {
                tx.send(CmdStatus::Failed(format!("Exited unsuccessfully: {e}")))
                    .await
                    .ok();
            }
            wait_for_child(child)
        } else {
            wait_for_child(child)
        }
        .await;
        tx.send(msg).await.ok();
    });
}

async fn wait_for_child(mut child: Child) -> CmdStatus {
    if let Some(mut stderr) = child.stderr.take() {
        let status = match child.wait().await {
            Ok(status) => {
                if status.success() {
                    CmdStatus::Success
                } else {
                    let mut s = Vec::new();
                    stderr.read_to_end(&mut s).await.ok();
                    CmdStatus::Failed(format!(
                        "Error exit status: {status}: {}",
                        String::from_utf8_lossy(&s)
                    ))
                }
            }
            Err(e) => {
                log::error!("Error from wait: {e}");
                CmdStatus::Failed(e.to_string())
            }
        };
        Ok(status)
    } else {
        child.wait_with_output().await.map(|o| {
            if !o.status.success() {
                let info = String::from_utf8_lossy(&o.stderr);
                log::error!("Error from wait_with_output {info}");
                CmdStatus::Failed(format!(
                    "Exited unsuccessfully: {} - {}",
                    o.status,
                    info.trim()
                ))
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
            tx.send(CmdStatus::Progress(100.0))
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;
            ct = TOTAL_LINES;
            break;
        }
        let percent = ((ct / TOTAL_LINES) * 100.0).floor();
        tx.send(CmdStatus::Progress(percent))
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
        pretty_env_logger::formatted_builder()
            .is_test(true)
            .try_init()
            .ok();
        let cmd = generate_command("test success").await.unwrap();
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
        pretty_env_logger::formatted_builder()
            .is_test(true)
            .try_init()
            .ok();
        let cmd = generate_command("test failure").await.unwrap();
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
                }
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
