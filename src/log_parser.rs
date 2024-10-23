use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::NaiveDateTime;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub ip_address: String,
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub status: String,
    pub response_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogType {
    AccessLog(AccessLogEntry),
    ErrorLog(LogEntry),
}

// Parse a log line and detect the log level
pub fn parse_log_line(line: &str, file_name: &str) -> LogType {
    if file_name.contains("access") {
        LogType::AccessLog(parse_access_log_line(line))
    } else {
        LogType::ErrorLog(parse_error_log_line(line))
    }
    
}

pub fn parse_access_log_line(line: &str) -> AccessLogEntry {
    let access_log_pattern = Regex::new(r#"(\S+) - - \[(.*?)\] "(.*?)" (\d+) (\d+)"#).unwrap();

    if let Some(captured) = access_log_pattern.captures(line) {
        let ip_address = captured.get(1).map_or("", |m| m.as_str()).to_string();
        let raw_timestamp = captured.get(2).map_or("", |m| m.as_str()).to_string();
        let request = captured.get(3).map_or("", |m| m.as_str()).to_string();

        let (request_method, resource, http_version) = parse_request(&request);

        let status = captured.get(4).map_or("", |m| m.as_str()).to_string();
        let response_size = captured.get(5).map_or("", |m| m.as_str()).to_string();

        AccessLogEntry {
            ip_address: ip_address,
            timestamp: format_timestamp(&raw_timestamp),
            method: request_method,
            path: resource,
            http_version: http_version,
            status: status,
            response_size: response_size,
        }

    } else {
        AccessLogEntry {
            ip_address: "Unknown".to_string(),
            timestamp: "Unknown".to_string(),
            method: "Unknown".to_string(),
            path: "Unknown".to_string(),
            http_version: "Unknown".to_string(),
            status: "Unknown".to_string(),
            response_size: "Unknown".to_string(),
        }
    }
}

pub fn parse_error_log_line(line: &str) -> LogEntry {
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

fn parse_request( request: &str) -> (String, String, String) {
    let parts: Vec<&str> = request.split_whitespace().collect();
    if parts.len() == 3 {
        (
            parts[0].to_string(),  // Method (GET, POST, etc.)
            parts[1].to_string(),  // Resource (/index.html)
            parts[2].to_string(),  // HTTP Version (HTTP/1.1)
        )
    } else {
        ("".to_string(), "".to_string(), "".to_string())
    }
}

fn format_timestamp(raw_timestamp: &str) -> String {
    // Two formats: access logs and error logs
    let formats = [
        "%a %b %d %H:%M:%S%.f %Y",  // Error log format
        "%d/%b/%Y:%H:%M:%S %z",     // Access log format
    ];

    for format in formats.iter() {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(raw_timestamp, format) {
            return naive_dt.format("%a %b %d %I:%M:%S %p %Y").to_string();
        }
    }

    "Invalid Timestamp".to_string()  // Fallback if parsing fails
}