use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub log_folder: String,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let contents = fs::read_to_string(path).expect("Could not read config file");
        serde_json::from_str(&contents).expect("Could not parse config file")
    }
}