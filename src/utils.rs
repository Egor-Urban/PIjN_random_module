use chrono::Local;
use serde::Deserialize;
use serde_json;
use std::fs;
use tracing::{error, info, warn};
use tracing_subscriber;
use reqwest;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub ip: String,
    pub port_manager_ip: String,
    pub port_manager_port: String,
    pub port_manager_endpoint: String,
    pub name_for_port_manager: String,
    pub logs_dir: String,
    pub workers_count: usize
}

#[derive(serde::Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

pub fn load_config() -> Config {
    let config_path = "config.json";
    let config_data = fs::read_to_string(config_path).expect("Can't read config.json");
    serde_json::from_str(&config_data).expect("Can't parse config.json")
}

pub fn init_tracing(logs_dir: &str, log_name: &str) {
    let date = Local::now().format("%d_%m_%Y").to_string();
    let log_dir = if logs_dir.trim().is_empty() {
        "./logs"
    } else {
        logs_dir
    };

    fs::create_dir_all(log_dir).expect("Can't create logs directory");

    let log_path = format!("{}/{}_{}.log", log_dir, log_name, date);

    tracing_subscriber::fmt()
        .with_target(true)
        .with_writer(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Can't open log file"),
        )
        .with_thread_names(true)
        .with_ansi(false)
        .init();
}

pub async fn fetch_port(config: &Config) -> Option<u16> {
    let url = format!(
        "http://{}:{}/{}/{}",
        config.port_manager_ip,
        config.port_manager_port,
        config.port_manager_endpoint,
        config.name_for_port_manager
    );

    for attempt in 1..=3 {
        info!(target: "port_resolver", "Attempt {}: Requesting port from {}", attempt, url);

        match reqwest::get(&url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<ApiResponse<serde_json::Value>>().await {
                        Ok(json) => {
                            if json.success {
                                if let Some(port_val) = json.data.as_u64() {
                                    let port = port_val as u16;
                                    info!(target: "port_resolver", "Received port: {}", port);
                                    return Some(port);
                                } else {
                                    error!(target: "port_resolver", "No port found in response data");
                                }
                            } else {
                                warn!(target: "port_resolver", "Server returned error: {:?}", json.data);
                            }
                        }
                        Err(e) => error!(target: "port_resolver", "JSON parse error: {}", e),
                    }
                } else {
                    warn!(target: "port_resolver", "Response status: {}", resp.status());
                }
            }
            Err(e) => {
                warn!(target: "port_resolver", "Attempt {} failed: {}", attempt, e);
                if attempt == 3 {
                    error!(target: "port_resolver", "All attempts to fetch port failed");
                    return None;
                }
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    None
}