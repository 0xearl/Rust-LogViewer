use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::NaiveDateTime;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

// Parse a log line and detect the log level
pub fn parse_log_line(line: &str) -> LogEntry {
    let re = Regex::new(r"^\[(?P<timestamp>.*?)\] \[(?P<level>.*?)\] \[.*?\] \[.*?\] (?P<message>.*)").unwrap();

    if let Some(captured) = re.captures(line) {
        let raw_timestamp = captured["timestamp"].to_string();
        let mut level = captured["level"].to_string();
        level = level.replace("php:", "");

        let timestamp = format_timestamp(&raw_timestamp);

        let message = captured["message"].to_string();
        LogEntry {
            timestamp: timestamp,
            level: level,
            message: message,
        }
    } else {
        LogEntry {
            timestamp: "Unknown".to_string(),
            level: "Unknown".to_string(),
            message: line.to_string(),
        }
    }
}

fn format_timestamp(raw_timestamp: &str) -> String {

    let format = "%a %b %d %H:%M:%S%.f %Y";
    
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(raw_timestamp, format) {
        
        naive_dt.format("%a %b %d %I:%M:%S %p %Y").to_string()
    } else {
        "Invalid Timestamp".to_string()  // Fallback if parsing fails
    }
}