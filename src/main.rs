use serde::Serialize;
use std::path::PathBuf;
use std::process::{Command, Output};
use warp::{Filter, Reply};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let files = warp::fs::dir("public");
    let execute = warp::post()
        .and(warp::path!("write" / String / String))
        .map(|set: String, bin: String| write_nfc(&set, &bin));
    warp::serve(files.or(execute).with(warp::log("amiibo")))
        .run(([0, 0, 0, 0], 80))
        .await;
}

fn write_nfc(set: &str, name: &str) -> impl Reply {
    use warp::http::StatusCode;
    let public = PathBuf::from("public");
    let path = public
        .join("amiibo")
        .join(set)
        .join(name.replace("%20", " "))
        .with_extension("bin");
    println!("Attempting to write {}", path.display());
    if !path.exists() {
        return response(&Response::NotFound, StatusCode::NOT_FOUND);
    }
    let mut cmd = Command::new("pimiibo");
    cmd.arg("public/key_retail.bin").arg(path);

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return response(
                &Response::Spawn(format!("error executing pimiibo: {}", e)),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };
    match child.wait_with_output() {
        Ok(o) => {
            if !o.status.success() {
                let r = cmd_failed(&o);
                response(&Response::Cmd(r), StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                response(&Response::Success, StatusCode::OK)
            }
        }
        Err(e) => {
            eprintln!("error waiting for cmd {}", e);
            let msg = format!("{}", e);
            response(&Response::Wait(msg), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn response(r: &Response, status: warp::http::StatusCode) -> impl Reply {
    let inner = warp::reply::json(r);
    warp::reply::with_status(inner, status)
}

fn cmd_failed(o: &Output) -> CmdFailed {
    CmdFailed {
        std_err: String::from_utf8_lossy(&o.stderr).to_string(),
        std_out: String::from_utf8_lossy(&o.stdout).to_string(),
        status: o.status.code(),
    }
}

#[derive(Debug, Serialize)]
enum Response {
    Success,
    Cmd(CmdFailed),
    Wait(String),
    Spawn(String),
    NotFound,
}

#[derive(Debug, Serialize)]
struct CmdFailed {
    std_err: String,
    std_out: String,
    status: Option<i32>,
}
