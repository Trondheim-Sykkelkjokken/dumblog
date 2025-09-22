use axum::http::StatusCode;
use axum::{Json, Router, extract::State, routing::post, serve};
use serde::Deserialize;
use std::{env, fs::OpenOptions, io::Write, sync::Arc};
use tokio::sync::Mutex;

const LOG_FILE: &str = "log.txt";

#[derive(Deserialize)]
struct LogRequest {
    message: String,
    secret: String,
}

#[derive(Clone)]
struct AppState {
    file_mutex: Arc<Mutex<()>>,
    secret: String,
}

#[tokio::main]
async fn main() {
    let secret = env::var("LOG_SECRET").expect("LOG_SECRET env variable must be set");
    let state = AppState {
        file_mutex: Arc::new(Mutex::new(())),
        secret,
    };
    let app = Router::new()
        .route("/log", post(log_handler))
        .with_state(state);
    println!("Listening on 127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn log_handler(State(state): State<AppState>, Json(payload): Json<LogRequest>) -> StatusCode {
    if payload.secret != state.secret {
        return StatusCode::UNAUTHORIZED;
    }
    let _lock = state.file_mutex.lock().await;
    let mut file = match OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        Ok(f) => f,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if let Err(_) = writeln!(file, "{}", payload.message) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}
