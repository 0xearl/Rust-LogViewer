use crate::log_parser::LogEntry;
use crate::log_parser;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::channel;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};

pub async fn watch_log_file(log_file_path: &std::path::Path, log_channel: broadcast::Sender<LogEntry>) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(move |res: Result<Event>| {
        tx.send(res).unwrap();
    }, Config::default())?;

    watcher.watch(log_file_path, RecursiveMode::NonRecursive)?;

    let file = File::open(log_file_path)?;
    let mut reader = BufReader::new(file);
    let mut interval = time::interval(Duration::from_secs(1));

    let chunk_size = 50;

    // Step 1: Read the log file in chunks
    loop {
        let mut log_entries = read_chunk(&mut reader, chunk_size);

        // If no more log entries were read, stop chunking and start watching for real-time updates
        if log_entries.is_empty() {
            break;
        }

        send_log_entries(&log_entries, &log_channel).await;

        // Small pause between chunks to avoid overwhelming the client
        time::sleep(Duration::from_millis(100)).await;
    }

    // Step 2: Start watching for real-time log file changes
    loop {
        if let Ok(Ok(event)) = rx.try_recv() {
            handle_event(event, &log_channel, &mut reader);
        }

        interval.tick().await;
        read_new_lines(&mut reader, &log_channel).await;
    }
}

fn handle_event(
    event: Event,
    log_channel: &broadcast::Sender<LogEntry>,
    reader: &mut BufReader<File>,
) {
   if let Event { kind: notify::EventKind::Modify(_), .. } = event {
        read_new_lines(reader, log_channel);
   }
}

fn read_chunk(reader: &mut BufReader<File>, chunk_size: usize) -> Vec<LogEntry> {
    let mut log_entries = Vec::new();
    let mut new_log = String::new();

    for _ in 0..chunk_size {
        if reader.read_line(&mut new_log).unwrap() > 0 {
            let log_entry = log_parser::parse_log_line(&new_log);
            log_entries.push(log_entry);
            new_log.clear();
        } else {
            break;
        }
    }

    log_entries
}

async fn send_log_entries(log_entries: &[LogEntry], log_channel: &broadcast::Sender<LogEntry>) {
    for log_entry in log_entries {
        if log_channel.receiver_count() > 0 {
            if let Err(err) = log_channel.send(log_entry.clone()) {
                eprintln!("Error sending log entry: {:?}", err);
            }
        } else {
            println!("No active subscribers, skipping log entry");
        }
    }
}

async fn read_new_lines(reader: &mut BufReader<File>, log_channel: &broadcast::Sender<LogEntry>) {
    let log_entries = read_chunk(reader, 50);
    send_log_entries(&log_entries, log_channel).await;
}