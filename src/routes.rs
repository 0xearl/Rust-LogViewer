use actix_web::{web, HttpResponse};
use serde_json::json;
use std::fs;
use tokio::sync::broadcast;
use crate::log_parser::LogEntry;
use crate::log_watcher::watch_log_file;
use actix_web::web::Bytes;
use std::path::PathBuf;
use async_stream::stream;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/logs/{file_name}", web::get().to(stream_logs))
        .route("/list_logs", web::get().to(list_logs));
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("index.html"))
}

async fn list_logs(log_folder: web::Data<String>) -> HttpResponse {
    let log_folder = log_folder.get_ref();
    let mut log_files = Vec::new();

    if let Ok(entries) = fs::read_dir(log_folder) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("log") {
                    log_files.push(path.file_name().unwrap().to_string_lossy().to_string());
                }
            }
        }
    }

    HttpResponse::Ok().json(json!({ "log_files": log_files }))

}

async fn stream_logs(
    log_folder: web::Data<String>,
    file_name: web::Path<String>,
    log_channel: web::Data<broadcast::Sender<LogEntry>>,
) -> HttpResponse {
    let log_folder = log_folder.get_ref();
    let log_file_path = PathBuf::from(format!("{}/{}", log_folder, file_name)); 

    let log_channel_clone = log_channel.get_ref().clone();  // Dereference and clone Sender<LogEntry>

    // Dynamically watch the requested log file
    tokio::spawn(async move {

        watch_log_file(&log_file_path, log_channel_clone).await.unwrap();
    });

    // Subscribe to the broadcast channel to stream logs to the client via SSE
    let mut rx = log_channel.subscribe();
    let log_stream = stream! {
        while let Ok(log_entry) = rx.recv().await {
            let log_json = serde_json::to_string(&log_entry).unwrap();
            yield Ok::<_, actix_web::Error>(Bytes::from(format!("data: {}\n\n", log_json)));
        }
    };

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(log_stream)
}