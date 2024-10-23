mod log_watcher;
mod log_parser;
mod routes;
mod config;

use actix_web::{App, HttpServer, web};
use log_parser::LogType;
use tokio::sync::broadcast;
use crate::config::Config;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    env_logger::init();

    let config = Config::from_file("./config.json");

    // Create the broadcast channel for LogEntry
    let (tx, _) = broadcast::channel::<LogType>(100);
    let log_channel = tx.clone();

    let log_folder = config.log_folder.clone();

    println!("web server started at http://127.0.0.1:8080");
    
    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(log_channel.clone()))
            .app_data(web::Data::new(log_folder.clone()))
            .configure(routes::init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
    
}
